use crate::{
    board::board_utils::get_friendly_name_for_index,
    shared::{
        BISHOP_CAPTURE_PROMOTION, BISHOP_PROMOTION, BLACK_PAWN, DOUBLE_PAWN_FLAG,
        KNIGHT_CAPTURE_PROMOTION, KNIGHT_PROMOTION, PAWN_INDEX, QUEEN_CAPTURE_PROMOTION,
        QUEEN_PROMOTION, ROOK_CAPTURE_PROMOTION, ROOK_PROMOTION,
    },
};

use super::{
    board_metrics::BoardMetrics,
    board_utils::{
        char_from_rank, get_piece_from_position_index, get_rank, rank_and_file_to_index,
        rank_from_char,
    },
    piece_utils::get_piece_char,
    state::BoardState,
};

pub fn build_move(from_index: u8, to_index: u8, flags: u16) -> u16 {
    let f: u16 = from_index.into();
    let t: u16 = to_index.into();
    let m: u16 = f << 10 | t << 4 | flags;
    m
}

pub fn standard_notation_to_move(std_notation: &str) -> u16 {
    let capture = std_notation.chars().nth(2).unwrap() == 'x';

    let mut result: u16 = 0;

    let from_rank_char = std_notation.chars().nth(0).unwrap();
    let from_rank = rank_from_char(from_rank_char);
    let from_file: u8 = std_notation.chars().nth(1).unwrap().to_digit(8).unwrap() as u8;

    let from_index = rank_and_file_to_index(from_rank, from_file - 1) as u16;
    result = result | (from_index << 10);

    let start_pos = if capture { 3 } else { 2 };
    let to_rank_char = std_notation.chars().nth(start_pos).unwrap();
    let to_rank = rank_from_char(to_rank_char);
    let to_file: u8 = std_notation
        .chars()
        .nth(start_pos + 1)
        .unwrap()
        .to_digit(8)
        .unwrap() as u8;

    let to_index = rank_and_file_to_index(to_rank, to_file - 1) as u16;
    result = result | (to_index << 4);

    if capture {
        result = result | 0b100;
    }

    result
}

pub fn get_move_uci(m: u16) -> String {
    let from = (m >> 10) as u8;
    let to = (m >> 4 & 0b111111) as u8;
    let flags = m & 0b1111;
    let promotion = match flags {
        KNIGHT_PROMOTION => "n",
        KNIGHT_CAPTURE_PROMOTION => "n",
        BISHOP_PROMOTION => "b",
        BISHOP_CAPTURE_PROMOTION => "b",
        ROOK_PROMOTION => "r",
        ROOK_CAPTURE_PROMOTION => "r",
        QUEEN_PROMOTION => "q",
        QUEEN_CAPTURE_PROMOTION => "q",
        _ => "",
    };
    format!(
        "{}{}{}",
        get_friendly_name_for_index(from),
        get_friendly_name_for_index(to),
        promotion
    )
}

pub fn get_move_san(board_state: BoardState, board_metrics: BoardMetrics, m: u16) -> String {
    let to = (m >> 4 & 0b111111) as u8;
    let from = (m >> 10) as u8;
    let piece = get_piece_from_position_index(board_state.bitboard, board_state.pieces, from);
    let piece_letter = get_piece_char(piece).to_ascii_uppercase();
    let flags = (m & 0b1111) as u8;

    let mut r = if piece_letter != 'P' {
        format!("{}", piece_letter)
    } else {
        "".into()
    };

    if is_castling(flags) {
        if is_king_castling(flags) {
            return "O-O".into();
        } else {
            return "O-O-O".into();
        }
    }

    let mut moves_targeting_square = Vec::new();
    for c_m in board_metrics.psudolegal_moves {
        let cm_to = (c_m >> 4 & 0b111111) as u8;
        let cm_from = (c_m >> 10) as u8;
        let cm_piece =
            get_piece_from_position_index(board_state.bitboard, board_state.pieces, cm_from);
        if cm_to == to && (cm_piece == piece || piece == PAWN_INDEX || piece == BLACK_PAWN) {
            moves_targeting_square.push(c_m);
        }
    }

    if moves_targeting_square.len() >= 1 {
        let from_rank = char_from_rank(get_rank(from));
        r = format!("{r}{from_rank}");
    }

    if is_capture(flags) {
        r = format!("{r}x");
    }

    format!("{r}{}", get_friendly_name_for_index(to))
}

pub fn is_capture(m: u8) -> bool {
    m & 0b100 > 0
}

pub fn is_castling(m: u8) -> bool {
    m == 2 || m == 3
}

pub fn is_king_castling(m: u8) -> bool {
    m == 2
}

pub fn is_promotion(m: u8) -> bool {
    m & 0b1000 > 0
}

pub fn is_ep_capture(m: u8) -> bool {
    m == 5
}

pub fn is_double_pawn_push(move_flags: u8) -> bool {
    move_flags == DOUBLE_PAWN_FLAG as u8
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    pub fn standard_notation_to_move_b1b2() {
        let r = standard_notation_to_move(&"b1b2".to_string());
        // b1 = 6th pos aka 0b000110
        // b2 = 14th pos ob001110
        //  promotion 0 capture 0 specials 0 0 = 0000
        //  => 0001100011100000
        println!("{r:#018b}");
        assert_eq!(r, 0b0001100011100000);
    }

    #[test]
    pub fn standard_notation_to_move_e2e4() {
        let r = standard_notation_to_move(&"e2e4".to_string());
        // e2 = 11th pos aka 001011
        // e4 = 27th pos aka 011011
        //  promotion 0 capture 0 specials 1 0 = 0000
        //  => 0010110110110000
        println!("{r:#018b}");
        assert_eq!(r, 0b0010110110110000);
    }

    #[test]
    pub fn build_move_e1_e2_pawn_push() {
        let from_index = 11; // 001011
        let to_index = 19; // 010011
        let r = build_move(from_index, to_index, 0b0u16);
        println!("{r:#018b}");
        assert_eq!(r, 0b0010110100110000);
    }

    #[test]
    pub fn build_move_a7_a8_pawn_push() {
        let from_index = 63; // 111111
        let to_index = 55; // 110111
        let r = build_move(from_index, to_index, 0b0u16);
        println!("{r:#018b}");
        assert_eq!(r, 0b1111111101110000);
    }
}
