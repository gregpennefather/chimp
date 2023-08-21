use super::{
    bitboard::{BitboardExtensions, Bitboard},
    board_utils::{get_rank, get_file, get_file_i8, file_and_rank_to_index, file_from_char},
};

pub fn get_available_slide_pos(
    bitboard: Bitboard,
    pos: u8,
    rank_delta: i8,
    file_delta: i8,
    max_depth: i32,
) -> Vec<u8> {
    let mut results = Vec::new();
    let delta = (rank_delta * 8) + (-1 * file_delta);
    let mut check_pos = pos as i8 + delta;
    let mut check_rank = get_rank(pos);
    let check_file = get_file(pos);
    while check_pos > -1 && check_pos < 64 {
        let cur_file = get_file_i8(check_pos);

        if (file_delta > 0 && cur_file < check_file) || (file_delta < 0 && cur_file > check_file) {
            break;
        }

        results.push(check_pos.try_into().unwrap());
        if bitboard.occupied_i8(check_pos) {
            break;
        }
        check_pos += delta;
        if rank_delta == 0 && check_rank != get_rank(check_pos as u8) {
            break;
        }

        check_rank = get_rank(check_pos as u8);

        if max_depth == 1 {
            break;
        }
    }
    results
}

pub fn standard_notation_to_move(std_notation: &str) -> u16 {
    let capture = std_notation.chars().nth(2).unwrap() == 'x';

    let mut result: u16 = 0;

    let from_file_char = std_notation.chars().nth(0).unwrap();
    let from_file = file_from_char(from_file_char);
    let from_rank: u8 = std_notation.chars().nth(1).unwrap().to_digit(8).unwrap() as u8;

    let from_index = file_and_rank_to_index(from_file, from_rank - 1) as u16;
    result = result | (from_index << 10);

    let start_pos = if capture { 3 } else { 2 };
    let to_file_char = std_notation.chars().nth(start_pos).unwrap();
    let to_file = file_from_char(to_file_char);
    let to_rank: u8 = std_notation
        .chars()
        .nth(start_pos + 1)
        .unwrap()
        .to_digit(8)
        .unwrap() as u8;

    let to_index = file_and_rank_to_index(to_file, to_rank - 1) as u16;
    result = result | (to_index << 4);

    if capture {
        result = result | 0b100;
    }

    result
}

#[cfg(test)]
mod test {
    use crate::board::state::BoardState;

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
    pub fn get_available_slide_pos_e4_diag_down_right() {
        let bitboard = Bitboard::default();
        let result = get_available_slide_pos(bitboard, file_and_rank_to_index(4, 3), -1, 1, 8);
        assert_eq!(result.len(), 3);
        assert_eq!(result.get(0).unwrap(), &18);
        assert_eq!(result.get(1).unwrap(), &9);
        assert_eq!(result.get(2).unwrap(), &0);
    }

    #[test]
    pub fn get_available_slide_pos_c1_diag_up_left() {
        let bitboard = Bitboard::default();
        let result = get_available_slide_pos(bitboard, file_and_rank_to_index(2, 0), 1, -1, 8);
        assert_eq!(result.len(), 2);
        assert_eq!(
            result.get(0).unwrap(),
            &file_and_rank_to_index(1, 1),
            "1,1 issue"
        );
        assert_eq!(
            result.get(1).unwrap(),
            &file_and_rank_to_index(0, 2),
            "0,2 issue"
        );
    }

    #[test]
    pub fn get_available_slide_pos_a3_diag_up_right_blocked_at_d6() {
        let bitboard = Bitboard::default().flip(file_and_rank_to_index(3, 5));
        let result = get_available_slide_pos(bitboard, file_and_rank_to_index(0, 2), 1, 1, 8);
        assert_eq!(result.len(), 3);
        assert_eq!(result.get(0).unwrap(), &file_and_rank_to_index(1, 3));
        assert_eq!(result.get(1).unwrap(), &file_and_rank_to_index(2, 4));
        assert_eq!(result.get(2).unwrap(), &file_and_rank_to_index(3, 5));
    }

    #[test]
    pub fn get_available_slide_pos_rook_d7_left_unblocked() {
        let bitboard = Bitboard::default();
        let result = get_available_slide_pos(bitboard, file_and_rank_to_index(3, 6), 0, -1, 8);
        assert_eq!(result.len(), 3);
        assert_eq!(result.get(0).unwrap(), &file_and_rank_to_index(2, 6));
        assert_eq!(result.get(1).unwrap(), &file_and_rank_to_index(1, 6));
        assert_eq!(result.get(2).unwrap(), &file_and_rank_to_index(0, 6));
    }

    #[test]
    pub fn get_available_slide_pos_rook_b3_right_unblocked() {
        let bitboard = Bitboard::default();
        let result = get_available_slide_pos(bitboard, file_and_rank_to_index(1, 2), 0, 1, 8);
        assert_eq!(result.len(), 6);
    }

    #[test]
    pub fn get_available_slide_pos_rook_h1_blocked_in() {
        let bitboard = Bitboard::new(0b1111111110u64);
        let result = get_available_slide_pos(bitboard, file_and_rank_to_index(7, 0), 1, 0, 8);
        assert_eq!(result.len(), 1); // blocked in at h2
    }

    #[test]
    pub fn get_available_slide_pos_bishop_moves_d2_pawn_opening() {
        let board = BoardState::from_fen(
            &"rnbqkb1r/pppppppp/5n2/8/8/3P4/PPP1PPPP/RNBQKBNR w KQkq - 0 1".into(),
        );
        let r = get_available_slide_pos(board.bitboard, 5, 1, 1, 8);
        assert_eq!(r.len(), 5, "{r:?}");
        assert_eq!(r.get(0).unwrap(), &file_and_rank_to_index(3, 1));
        assert_eq!(r.get(1).unwrap(), &file_and_rank_to_index(4, 2));
        assert_eq!(r.get(2).unwrap(), &file_and_rank_to_index(5, 3));
        assert_eq!(r.get(3).unwrap(), &file_and_rank_to_index(6, 4));
        assert_eq!(r.get(4).unwrap(), &file_and_rank_to_index(7, 5));
    }
}
