use crate::shared::{BLACK_KING, KING_INDEX, PIECE_MASK, bitboard_to_string, PAWN_INDEX, KNIGHT_INDEX, BISHOP_INDEX, ROOK_INDEX, QUEEN_INDEX};

use super::{
    bitboard::BitboardExtensions,
    board_utils::{get_piece_from_position_index, get_position_index_from_piece_index, get_rank},
    move_utils::get_available_slide_pos,
    piece_utils::is_piece_black,
    state::BoardState,
};

#[derive(Clone)]
pub struct BoardMetrics {
    pub white_threat_board: u64,
    pub black_threat_board: u64,
    pub white_mobility_board: u64,
    pub black_mobility_board: u64,
    pub white_king_position: u8,
    pub black_king_position: u8,
    pub white_in_check: bool,
    pub black_in_check: bool,
}

impl BoardState {
    pub fn generate_metrics(&self) -> BoardMetrics {
        let mut white_threat_board = 0;
        let mut black_threat_board = 0;
        let mut white_mobility_board = 0;
        let mut black_mobility_board = 0;
        let mut white_king_position = u8::MAX;
        let mut black_king_position = u8::MAX;

        for piece_index in 0..self.piece_count {
            let position_index =
                get_position_index_from_piece_index(self.bitboard, 0, 0, piece_index);
            let piece = get_piece_from_position_index(self.bitboard, self.pieces, position_index);
            let is_black = is_piece_black(piece);

            if !is_black {
                if piece == KING_INDEX {
                    white_king_position = position_index;
                }
                white_threat_board = white_threat_board
                    | generate_threat_board(self.bitboard, position_index, piece, is_black);
                white_mobility_board = white_mobility_board
                    | generate_mobility_board(self.bitboard, position_index, piece, is_black);
            } else {
                if piece == BLACK_KING {
                    black_king_position = position_index;
                }
                black_threat_board = black_threat_board
                    | generate_threat_board(self.bitboard, position_index, piece, is_black);
                black_mobility_board = black_mobility_board
                    | generate_mobility_board(self.bitboard, position_index, piece, is_black);
            }
        }

        let white_in_check = black_threat_board.occupied(white_king_position);
        let black_in_check = white_threat_board.occupied(black_king_position);

        BoardMetrics {
            white_threat_board,
            black_threat_board,
            white_mobility_board,
            black_mobility_board,
            white_king_position,
            black_king_position,
            white_in_check,
            black_in_check,
        }
    }
}

fn generate_threat_board(bitboard: u64, position_index: u8, piece: u8, is_black: bool) -> u64 {
    let piece_code = piece & PIECE_MASK;
    match piece_code {
        PAWN_INDEX => generate_pawn_threat_board(is_black, position_index),
        KNIGHT_INDEX => generate_knight_threat_board(position_index),
        BISHOP_INDEX => generate_bishop_threat_board(bitboard, position_index),
        ROOK_INDEX => generate_rook_threat_board(bitboard, position_index),
        QUEEN_INDEX => generate_queen_threat_board(bitboard, position_index),
        KING_INDEX => generate_king_threat_board(bitboard, position_index),
        _ => 0,
    }
}

fn generate_mobility_board(bitboard: u64, position_index: u8, piece: u8, is_black: bool) -> u64 {
    0
}

fn generate_pawn_threat_board(is_black: bool, position_index: u8) -> u64 {
    let mut threat_bitboard: u64 = 0;
    let rank = get_rank(position_index);
    if !is_black {
        if rank != 7 && position_index <= 56 {
            threat_bitboard |= 1 << (position_index + 7);
        }

        if rank != 0 && position_index <= 54 {
            threat_bitboard |= 1 << (position_index + 9);
        }
    } else {
        if rank != 7 && position_index >= 9 {
            threat_bitboard |= 1 << (position_index - 9);
        }

        if rank != 0 && position_index >= 7 {
            threat_bitboard |= 1 << (position_index - 7);
        }
    }
    threat_bitboard
}

fn generate_knight_threat_board(position_index: u8) -> u64 {
    let mut threat_bitboard: u64 = 0;
    let rank = get_rank(position_index);

    // U2R1 = +16-1 = 15
    if position_index <= 48 && rank != 7 {
        let tar = position_index + 15;
        threat_bitboard = threat_bitboard | (1 << tar);
    }
    // U1R2 = +8-2 = 6
    if position_index <= 55 && rank < 6 {
        let tar = position_index + 6;
        threat_bitboard = threat_bitboard | (1 << tar);
    }
    // D1R2 = -8-2 = -10
    if position_index >= 10 && rank < 6 {
        let tar = position_index - 10;
        threat_bitboard = threat_bitboard | (1 << tar);
    }
    // D2R1 = -16-1 = -17
    if position_index >= 17 && rank != 7 {
        let tar = position_index - 17;
        threat_bitboard = threat_bitboard | (1 << tar);
    }
    // D2L1 = -16+1 = -15
    if position_index >= 15 && rank != 0 {
        let tar = position_index - 15;
        threat_bitboard = threat_bitboard | (1 << tar);
    }
    // D1L2 = -8+2 = -6
    if position_index >= 6 && rank > 1 {
        let tar = position_index - 6;
        threat_bitboard = threat_bitboard | (1 << tar);
    }
    // U1L2 = 8+2 = 10
    if position_index <= 53 && rank > 1 {
        let tar = position_index + 10;
        threat_bitboard = threat_bitboard | (1 << tar);
    }
    // U2L1 = 16+1 = 17
    if position_index <= 46 && rank != 0 {
        let tar = position_index + 17;
        threat_bitboard = threat_bitboard | (1 << tar);
    }
    threat_bitboard
}

fn generate_bishop_threat_board(bitboard: u64, position_index: u8) -> u64 {
    sliding_threat_generator(bitboard, position_index, true, false, false)
}

fn generate_rook_threat_board(bitboard: u64, position_index: u8) -> u64 {
    sliding_threat_generator(bitboard, position_index, false, true, false)
}

fn generate_queen_threat_board(bitboard: u64, position_index: u8) -> u64 {
    sliding_threat_generator(bitboard, position_index, true, true, false)
}

fn generate_king_threat_board(bitboard: u64, position_index: u8) -> u64 {
    sliding_threat_generator(bitboard, position_index, true, true, true)
}

fn sliding_threat_generator(
    bitboard: u64,
    position_index: u8,
    diag: bool,
    straight: bool,
    king: bool,
) -> u64 {
    let mut threat_bitboard = 0;
    let depth = if king { 1 } else { 8 };
    if diag {
        let positions_d_l = get_available_slide_pos(bitboard, position_index, -1, -1, depth);

        for i in 0..positions_d_l.len() {
            threat_bitboard |= 1 << positions_d_l[i];
        }

        let positions_u_l = get_available_slide_pos(bitboard, position_index, 1, -1, depth);

        for i in 0..positions_u_l.len() {
            threat_bitboard |= 1 << positions_u_l[i];
        }

        let positions_u_r = get_available_slide_pos(bitboard, position_index, 1, 1, depth);
        for i in 0..positions_u_r.len() {
            threat_bitboard |= 1 << positions_u_r[i];
        }

        let positions_d_r = get_available_slide_pos(bitboard, position_index, -1, 1, depth);

        for i in 0..positions_d_r.len() {
            threat_bitboard |= 1 << positions_d_r[i];
        }
    }

    if straight {
        let positions_r = get_available_slide_pos(bitboard, position_index, 0, 1, depth);

        for i in 0..positions_r.len() {
            threat_bitboard |= 1 << positions_r[i];
        }

        let positions_l = get_available_slide_pos(bitboard, position_index, 0, -1, depth);

        for i in 0..positions_l.len() {
            threat_bitboard |= 1 << positions_l[i];
        }

        let positions_u = get_available_slide_pos(bitboard, position_index, 1, 0, depth);

        for i in 0..positions_u.len() {
            threat_bitboard |= 1 << positions_u[i];
        }

        let positions_d = get_available_slide_pos(bitboard, position_index, -1, 0, depth);

        for i in 0..positions_d.len() {
            threat_bitboard |= 1 << positions_d[i];
        }
    }

    threat_bitboard
}

#[cfg(test)]
mod test {
    use super::*;

    // #[test]
    // pub fn is_white_check_starting_pos() {
    //     assert!(!BoardState::default().generate_metrics().is_white_check());
    // }

    // #[test]
    // pub fn is_black_check_starting_pos() {
    //     assert!(!BoardState::default().is_black_check());
    // }

    // #[test]
    // pub fn is_black_check_early_b5_bishop() {
    //     assert!(BoardState::from_fen(
    //         &"rnbqkbnr/ppp1pppp/8/1B1p4/4P3/8/PPPP1PPP/RNBQK1NR b KQkq - 0 1".into()
    //     )
    //     .is_black_check())
    // }

    // #[test]
    // pub fn is_white_check_early_c2_knight() {
    //     assert!(BoardState::from_fen(
    //         &"r1bqkbnr/pppppppp/8/8/4P3/3P4/PPnB1PPP/RN1QKBNR w KQkq - 0 1".into()
    //     )
    //     .is_white_check())
    // }
}
