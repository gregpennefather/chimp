use super::{bitboard::BitboardExtensions, state::BoardState};
use crate::{
    board::piece::*,
    shared::{
        binary_utils::BinaryUtils, BISHOP_INDEX, BLACK_MASK, KING_INDEX, KNIGHT_INDEX, PAWN_INDEX,
        PIECE_MASK, QUEEN_INDEX, ROOK_INDEX,
    },
};

impl BoardState {
    pub fn apply_move(&self, m: u16) -> BoardState {
        let mut bitboard: u64 = self.bitboard;
        let mut white_bitboard: u64 = self.white_bitboard;
        let mut black_bitboard: u64 = self.black_bitboard;
        let mut pieces: u128 = self.pieces;
        let mut flags: u8 = self.flags;
        let mut half_moves: u8 = self.half_moves;

        let from_index: u8 = (m >> 10).try_into().unwrap();
        let to_index: u8 = (m >> 4 & 0b111111).try_into().unwrap();

        let capture = is_capture(m);

        let (picked_up_piece, mut new_pieces) = pickup_piece(pieces, bitboard, from_index);
        let black_move = (picked_up_piece as u8) & BLACK_MASK > 0;
        bitboard = bitboard ^ (1 << from_index);
        if black_move {
            black_bitboard = black_bitboard ^ (1 << from_index);
        } else {
            white_bitboard = white_bitboard ^ (1 << from_index);
        }

        if capture {
            new_pieces = remove_piece(new_pieces, bitboard, to_index);
        }

        pieces = place_piece(new_pieces, bitboard, to_index, picked_up_piece);
        bitboard = bitboard | (1 << to_index);
        if black_move {
            black_bitboard = black_bitboard | (1 << to_index);
        } else {
            white_bitboard = white_bitboard | (1 << to_index);
        }

        // Turn
        flags = flags ^ 0b1;

        // Double Pawn Push
        let piece_u8: u8 = picked_up_piece.try_into().unwrap();
        if is_double_pawn_push(piece_u8, from_index, to_index) {
            flags = flags & 0b00011111;
            flags = flags ^ (((from_index % 8 as u8) + 1) << 5);
        }

        // Half moves
        if (piece_u8 & PIECE_MASK) == PAWN_INDEX {
            half_moves = 0;
        } else {
            half_moves = half_moves + 1;
        }

        // Full moves
        let full_moves: u32 = self.full_moves + if (flags & 1) == 1 { 1 } else { 0 };

        // Piece Count
        let piece_count = bitboard.count_ones() as u8;

        BoardState {
            bitboard,
            white_bitboard,
            black_bitboard,
            pieces,
            flags,
            half_moves,
            full_moves,
            piece_count,
        }
    }

    pub fn generate_moves(&self) -> Vec<u16> {
        let mut moves = Vec::new();
        let white_turn = self.flags & 0b1 > 0;
        let mut piece_index = 0;
        let mut next_start_pos = 0;
        let mut start_count = 0;
        println!("gen-move-start --------------------------------------");
        while piece_index < self.piece_count {
            let piece = get_piece_code(&self.pieces, piece_index);
            if is_white(piece) == white_turn {
                let position_index =
                    get_position_index_from_piece_index(self.bitboard, 0, 0, piece_index);
                next_start_pos = position_index + 1;
                start_count = piece_index;
                moves.extend(self.generate_piece_moves(position_index, piece));
            }
            piece_index += 1;
        }
        println!("gen-move-end {}:{moves:?} --------------------------------------", moves.len());

        moves
    }

    pub fn generate_piece_moves(&self, position_index: u8, piece: u8) -> Vec<u16> {
        let piece_code = piece & PIECE_MASK;
        match piece_code {
            PAWN_INDEX => generate_pawn_moves(
                self.bitboard,
                self.white_bitboard,
                self.black_bitboard,
                position_index,
                piece,
            ),
            KNIGHT_INDEX => generate_knight_moves(self.bitboard, position_index, piece),
            BISHOP_INDEX => generate_bishop_moves(self.bitboard, position_index, piece),
            ROOK_INDEX => generate_rook_moves(self.bitboard, position_index, piece),
            QUEEN_INDEX => generate_queen_moves(self.bitboard, position_index, piece),
            KING_INDEX => generate_king_moves(self.bitboard, position_index, piece),
            _ => vec![],
        }
    }
}

fn generate_knight_moves(bitboard: u64, pos: u8, piece: u8) -> Vec<u16> {
    let mut vec: Vec<_> = Vec::new();
    let rank = pos % 8;
    // U2R1 = +16-1 = 15
    if pos <= 48 && rank != 0 && !bitboard.occupied(pos + 15) {
        vec.push(build_move(pos, pos + 15, 0b0));
    }
    // U1R2 = +8-2 = 6
    if pos <= 55 && rank > 1 && !bitboard.occupied(pos + 6) {
        vec.push(build_move(pos, pos + 6, 0b0));
    }
    // D1R2 = -8-2 = -10
    if pos >= 10 && rank > 1 && !bitboard.occupied(pos - 10) {
        vec.push(build_move(pos, pos - 10, 0b0));
    }
    // D2R1 = -16-1 = -17
    if pos >= 17 && rank != 0 && !bitboard.occupied(pos - 17) {
        vec.push(build_move(pos, pos - 17, 0b0));
    }
    // D2L1 = -16+1 = -15
    if pos >= 15 && rank != 7 && !bitboard.occupied(pos - 15) {
        vec.push(build_move(pos, pos - 15, 0b0));
    }
    // D1L2 = -8+2 = -6
    if pos >= 6 && rank < 6 && !bitboard.occupied(pos - 6) {
        vec.push(build_move(pos, pos - 6, 0b0));
    }
    // U1L2 = 8+2 = 10
    if pos <= 53 && rank < 6 && !bitboard.occupied(pos + 10) {
        vec.push(build_move(pos, pos + 10, 0b0));
    }
    // U2L1 = 16+1 = 17
    if pos <= 46 && rank != 7 && !bitboard.occupied(pos + 17) {
        vec.push(build_move(pos, pos + 17, 0b0));
    }
    vec
}

fn generate_pawn_moves(
    bitboard: u64,
    white_bitboard: u64,
    black_bitboard: u64,
    position_index: u8,
    piece: u8,
) -> Vec<u16> {
    let mut vec: Vec<_> = Vec::new();
    let is_white = piece & BLACK_MASK == 0;
    let file = position_index / 8;
    let rank = position_index % 8;
    let opponent_bitboard = if is_white {
        black_bitboard
    } else {
        white_bitboard
    };

    if is_white {
        if !bitboard.occupied(position_index + 8) {
            vec.push(build_move(position_index, position_index + 8, 0b0));

            if file == 1 {
                if !bitboard.occupied(position_index + 16) {
                    vec.push(build_move(position_index, position_index + 16, 0b0));
                }
            }
        }

        if rank != 0 && opponent_bitboard.occupied(position_index + 7) {
            vec.push(build_move(position_index, position_index + 7, 0b0));
        }

        if rank != 7 && opponent_bitboard.occupied(position_index + 9) {
            vec.push(build_move(position_index, position_index + 9, 0b0));
        }
    } else {
        if !bitboard.occupied(position_index - 8) {
            vec.push(build_move(position_index, position_index - 8, 0b0));

            if file == 6 {
                if !bitboard.occupied(position_index - 16) {
                    vec.push(build_move(position_index, position_index - 16, 0b0));
                }
            }

            if rank != 0 && opponent_bitboard.occupied(position_index - 9) {
                vec.push(build_move(position_index, position_index - 9, 0b0));
            }

            if rank != 7 && opponent_bitboard.occupied(position_index - 7) {
                vec.push(build_move(position_index, position_index - 7, 0b0));
            }
        }
    }
    vec
}

fn generate_bishop_moves(bitboard: u64, position_index: u8, piece: u8) -> Vec<u16> {
    sliding_move_generator(bitboard, position_index, true, false, false)
}

fn generate_rook_moves(bitboard: u64, position_index: u8, piece: u8) -> Vec<u16> {
    sliding_move_generator(bitboard, position_index, false, true, false)
}

fn generate_queen_moves(bitboard: u64, position_index: u8, piece: u8) -> Vec<u16> {
    sliding_move_generator(bitboard, position_index, true, true, false)
}

fn generate_king_moves(bitboard: u64, position_index: u8, piece: u8) -> Vec<u16> {
    sliding_move_generator(bitboard, position_index, true, true, true)
}

fn sliding_move_generator(
    bitboard: u64,
    pos: u8,
    diag: bool,
    straight: bool,
    king: bool,
) -> Vec<u16> {
    let mut moves: Vec<u16> = Vec::new();

    let depth = if king { 1 } else { 8 };

    if diag {
        let positions_d_l = get_available_slide_pos(bitboard, pos, -1, -1, depth);

        for i in 0..positions_d_l.len() {
            if (i == positions_d_l.len() - 1) && bitboard.occupied(positions_d_l[i]) {
                continue;
            }
            moves.push(build_move(pos, positions_d_l[i], 0b0));
        }

        let positions_u_l = get_available_slide_pos(bitboard, pos, 1, -1, depth);

        for i in 0..positions_u_l.len() {
            if (i == positions_u_l.len() - 1) && bitboard.occupied(positions_u_l[i]) {
                continue;
            }
            moves.push(build_move(pos, positions_u_l[i], 0b0));
        }

        let positions_u_r = get_available_slide_pos(bitboard, pos, 1, 1, depth);
        println!("{pos} ur {positions_u_r:?}");
        for i in 0..positions_u_r.len() {
            if (i == positions_u_r.len() - 1) && bitboard.occupied(positions_u_r[i]) {
                println!("Not adding {}", get_friendly_name_for_index(positions_u_r[i]));
                continue;
            }
            println!("Adding {} {}", get_friendly_name_for_index(positions_u_r[i]), get_move_uci(build_move(pos, positions_u_r[i], 0b0)));
            moves.push(build_move(pos, positions_u_r[i], 0b0));
        }

        let positions_d_r = get_available_slide_pos(bitboard, pos, -1, 1, depth);

        for i in 0..positions_d_r.len() {
            if (i == positions_d_r.len() - 1) && bitboard.occupied(positions_d_r[i]) {
                continue;
            }
            moves.push(build_move(pos, positions_d_r[i], 0b0));
        }
    }

    if straight {
        let positions_r = get_available_slide_pos(bitboard, pos, 0, 1, depth);

        for i in 0..positions_r.len() {
            if (i == positions_r.len() - 1) && bitboard.occupied(positions_r[i]) {
                continue;
            }
            moves.push(build_move(pos, positions_r[i], 0b0));
        }

        let positions_l = get_available_slide_pos(bitboard, pos, 0, -1, depth);

        for i in 0..positions_l.len() {
            if (i == positions_l.len() - 1) && bitboard.occupied(positions_l[i]) {
                continue;
            }
            moves.push(build_move(pos, positions_l[i], 0b0));
        }

        let positions_u = get_available_slide_pos(bitboard, pos, 1, 0, depth);

        for i in 0..positions_u.len() {
            if (i == positions_u.len() - 1) && bitboard.occupied(positions_u[i]) {
                continue;
            }
            moves.push(build_move(pos, positions_u[i], 0b0));
        }

        let positions_d = get_available_slide_pos(bitboard, pos, -1, 0, depth);

        for i in 0..positions_d.len() {
            if (i == positions_d.len() - 1) && bitboard.occupied(positions_d[i]) {
                continue;
            }
            moves.push(build_move(pos, positions_d[i], 0b0));
        }
    }

    println!("moves : {moves:?}");

    moves
}

fn get_available_slide_pos(
    bitboard: u64,
    pos: u8,
    file_delta: i8,
    rank_delta: i8,
    max_depth: i32,
) -> Vec<u8> {
    let mut results = Vec::new();
    let delta = (file_delta * 8) + (-1 * rank_delta);
    let mut check_pos = pos as i8 + delta;
    let mut check_file = get_file(pos);
    let check_rank = get_rank(pos);
    while check_pos > -1 && check_pos < 64 {
        let cur_rank = get_rank_i8(check_pos);

        if (rank_delta > 0 && cur_rank < check_rank) || (rank_delta < 0 && cur_rank > check_rank) {
            break;
        }

        results.push(check_pos.try_into().unwrap());
        if bitboard.occupied_i8(check_pos) {
            break;
        }
        check_pos += delta;
        if file_delta == 0 && check_file != get_file(check_pos as u8) {
            break;
        }

        check_file = get_file(check_pos as u8);

        if max_depth == 1 {
            break;
        }
    }
    results
}

fn build_move(from_index: u8, to_index: u8, flags: u16) -> u16 {
    let f: u16 = from_index.into();
    let t: u16 = to_index.into();
    let m: u16 = f << 10 | t << 4 | flags;
    m
}

fn is_white(piece: u8) -> bool {
    piece & BLACK_MASK == 0
}

fn get_position_index_from_piece_index(
    bitboard: u64,
    start_index: u8,
    start_count: u8,
    search_index: u8,
) -> u8 {
    let mut pos: u32 = start_index as u32;
    let mut count = start_count;

    while pos < 64 {
        if bitboard & u64::pow(2, pos) > 0 {
            count += 1;
            if count > search_index.into() {
                break;
            }
        }
        pos += 1;
    }
    pos.try_into().unwrap()
}

fn is_double_pawn_push(picked_up_piece: u8, from_index: u8, to_index: u8) -> bool {
    if (picked_up_piece & PIECE_MASK) != PAWN_INDEX {
        return false;
    }

    if picked_up_piece & BLACK_MASK > 0 {
        return from_index >= 48 && from_index <= 55 && to_index >= 32 && to_index <= 39;
    }

    return from_index >= 8 && from_index <= 15 && to_index >= 24 && to_index <= 31;
}

fn is_capture(m: u16) -> bool {
    m >> 2 & 0b1 > 0
}

fn pickup_piece(pieces: u128, bitboard: u64, index: u8) -> (u128, u128) {
    let bitboard_relevant = bitboard & (u64::pow(2, index.into()) - 1);
    let bitboard_pos: usize = bitboard_relevant.count_ones() as usize;
    let piece = pieces.copy_b(bitboard_pos * 4, 4);
    let board = pieces.remove_b(bitboard_pos * 4, 4);
    (piece, board)
}

fn remove_piece(pieces: u128, bitboard: u64, index: u8) -> u128 {
    let bitboard_relevant = bitboard & (u64::pow(2, index.into()) - 1);
    let bitboard_pos: usize = bitboard_relevant.count_ones() as usize;
    let board = pieces.remove_b(bitboard_pos * 4, 4);
    board
}

fn place_piece(pieces: u128, bitboard: u64, index: u8, piece: u128) -> u128 {
    let bitboard_relevant = bitboard & (u64::pow(2, index.into()) - 1);
    let bitboard_pos = (bitboard_relevant.count_ones()) as usize;
    pieces.insert_b(bitboard_pos * 4, piece, 4)
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

fn rank_and_file_to_index(rank: u8, file: u8) -> u8 {
    ((file) * 8) + (7 - rank)
}

pub fn get_move_uci(m: u16) -> String {
    let from = (m >> 10) as u8;
    let to = (m >> 4 & 0b111111) as u8;
    format!(
        "{}{}",
        get_friendly_name_for_index(from),
        get_friendly_name_for_index(to)
    )
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

    #[test]
    pub fn get_available_slide_pos_e4_diag_down_right() {
        let bitboard = 0b0u64;
        let result = get_available_slide_pos(bitboard, rank_and_file_to_index(4, 3), -1, 1, 8);
        assert_eq!(result.len(), 3);
        assert_eq!(result.get(0).unwrap(), &18);
        assert_eq!(result.get(1).unwrap(), &9);
        assert_eq!(result.get(2).unwrap(), &0);
    }

    #[test]
    pub fn get_available_slide_pos_c1_diag_up_left() {
        let bitboard = 0b0u64;
        let result = get_available_slide_pos(bitboard, rank_and_file_to_index(2, 0), 1, -1, 8);
        assert_eq!(result.len(), 2);
        assert_eq!(result.get(0).unwrap(), &rank_and_file_to_index(1, 1), "1,1 issue");
        assert_eq!(result.get(1).unwrap(), &rank_and_file_to_index(0, 2), "0,2 issue");
    }


    #[test]
    pub fn get_available_slide_pos_a3_diag_up_right_blocked_at_d6() {
        let bitboard = 0b0u64.flip(rank_and_file_to_index(3, 5));
        let result = get_available_slide_pos(bitboard, rank_and_file_to_index(0, 2), 1, 1, 8);
        assert_eq!(result.len(), 3);
        assert_eq!(result.get(0).unwrap(), &rank_and_file_to_index(1, 3));
        assert_eq!(result.get(1).unwrap(), &rank_and_file_to_index(2, 4));
        assert_eq!(result.get(2).unwrap(), &rank_and_file_to_index(3, 5));
    }

    #[test]
    pub fn get_available_slide_pos_rook_d7_left_unblocked() {
        let bitboard = 0b0u64;
        let result = get_available_slide_pos(bitboard, rank_and_file_to_index(3, 6), 0, -1, 8);
        assert_eq!(result.len(), 3);
        assert_eq!(result.get(0).unwrap(), &rank_and_file_to_index(2, 6));
        assert_eq!(result.get(1).unwrap(), &rank_and_file_to_index(1, 6));
        assert_eq!(result.get(2).unwrap(), &rank_and_file_to_index(0, 6));
    }

    #[test]
    pub fn get_available_slide_pos_rook_b3_right_unblocked() {
        let bitboard = 0b0u64;
        let result = get_available_slide_pos(bitboard, rank_and_file_to_index(1, 2), 0, 1, 8);
        assert_eq!(result.len(), 6);
    }

    #[test]
    pub fn get_available_slide_pos_rook_h1_blocked_in() {
        let bitboard = 0b1111111110u64;
        let result = get_available_slide_pos(bitboard, rank_and_file_to_index(7, 0), 1, 0, 8);
        assert_eq!(result.len(), 1); // blocked in at h2
    }

    #[test]
    pub fn sliding_move_generator_rook_with_one_move() {
        let board = BoardState::from_fen(
            &"rnbqkbnr/ppppppp1/7p/8/8/7P/PPPPPPP1/RNBQKBNR w KQkq - 0 2".into(),
        );
        let r = sliding_move_generator(board.bitboard, 0, false, true, false);
        assert_eq!(r.len(), 1);

        let rook_moves = generate_rook_moves(board.bitboard, 0, 0b0);
        assert_eq!(rook_moves.len(), 1);
    }

    #[test]
    pub fn sliding_move_generator_rook_with_zero_moves() {
        let board = BoardState::from_fen(
            &"rnbqkbnr/ppppppp1/7p/8/8/7P/PPPPPPP1/RNBQKBNR w KQkq - 0 2".into(),
        );
        let r = sliding_move_generator(board.bitboard, 7, false, true, false);
        assert_eq!(r.len(), 0);

        let rook_moves = generate_rook_moves(board.bitboard, 7, 0b0);
        assert_eq!(rook_moves.len(), 0);
    }

    #[test]
    pub fn generate_bishop_moves_b2_pawn_opening() {
        let board = BoardState::from_fen(
            &"r1bqkbnr/pppppppp/n7/8/1P6/8/P1PPPPPP/RNBQKBNR w KQkq - 0 1".into(),
        );
        let r = generate_bishop_moves(board.bitboard, 5, 0b0);
        assert_eq!(r.len(), 2, "{r:?}");
    }

    #[test]
    pub fn generate_bishop_moves_d2_pawn_opening() {
        let board = BoardState::from_fen(
            &"rnbqkb1r/pppppppp/5n2/8/8/3P4/PPP1PPPP/RNBQKBNR w KQkq - 0 1".into(),
        );
        let r = get_available_slide_pos(board.bitboard, 5, 1,1,8);
        assert_eq!(r.len(), 5, "{r:?}");
        assert_eq!(r.get(0).unwrap(), &rank_and_file_to_index(3, 1));
        assert_eq!(r.get(1).unwrap(), &rank_and_file_to_index(4, 2));
        assert_eq!(r.get(2).unwrap(), &rank_and_file_to_index(5, 3));
        assert_eq!(r.get(3).unwrap(), &rank_and_file_to_index(6, 4));
        assert_eq!(r.get(4).unwrap(), &rank_and_file_to_index(7, 5));

    }
}
