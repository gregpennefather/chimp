use crate::shared::*;
use super::{
    bitboard::{Bitboard, BitboardExtensions},
    board_utils::get_file,
    move_utils::get_available_slide_pos,
    state::BoardState, piece::{PieceType, Piece},
};

#[derive(Clone)]
pub struct BoardMetrics {
    pub white_threat_board: Bitboard,
    pub black_threat_board: Bitboard,
    pub white_mobility_board: Bitboard,
    pub black_mobility_board: Bitboard,
    pub white_king_position: u8,
    pub black_king_position: u8,
    pub white_in_check: bool,
    pub black_in_check: bool,
}

impl BoardState {
    pub fn generate_metrics(&self) -> BoardMetrics {
        let mut white_threat_board = Bitboard::default();
        let mut black_threat_board = Bitboard::default();
        let mut white_mobility_board = Bitboard::default();
        let mut black_mobility_board = Bitboard::default();
        let mut white_king_position = u8::MAX;
        let mut black_king_position = u8::MAX;

        for piece_index in 0..self.piece_count {
            let position_index = self.bitboard.get_nth_piece_position_index(piece_index);
            let piece = self.pieces.get_by_position_index(self.bitboard, position_index);
            let is_black = piece.is_black();

            if !is_black {
                if piece.is(PieceType::King) {
                    white_king_position = position_index;
                }
                white_threat_board = white_threat_board
                    | generate_threat_board(self.bitboard, position_index, piece, is_black);
                white_mobility_board = white_mobility_board
                    | generate_mobility_board(self.bitboard, position_index, piece, is_black);
            } else {
                if piece.is(PieceType::King) {
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

fn generate_threat_board(
    bitboard: Bitboard,
    position_index: u8,
    piece: Piece,
    is_black: bool,
) -> Bitboard {
    match piece.without_colour() {
        PieceType::Pawn => generate_pawn_threat_board(is_black, position_index),
        PieceType::Knight => generate_knight_threat_board(position_index),
        PieceType::Bishop => generate_bishop_threat_board(bitboard, position_index),
        PieceType::Rook => generate_rook_threat_board(bitboard, position_index),
        PieceType::Queen => generate_queen_threat_board(bitboard, position_index),
        PieceType::King => generate_king_threat_board(bitboard, position_index)
    }
}

fn generate_mobility_board(
    bitboard: Bitboard,
    position_index: u8,
    piece: Piece,
    is_black: bool,
) -> Bitboard {
    Bitboard::default()
}

fn generate_pawn_threat_board(is_black: bool, position_index: u8) -> Bitboard {
    let mut threat_bitboard = Bitboard::default();
    let file = get_file(position_index);
    if !is_black {
        if file != 7 && position_index <= 56 {
            threat_bitboard = threat_bitboard.set(position_index + 7);
        }

        if file != 0 && position_index <= 54 {
            threat_bitboard = threat_bitboard.set(position_index + 9);
        }
    } else {
        if file != 7 && position_index >= 9 {
            threat_bitboard = threat_bitboard.set(position_index - 9);
        }

        if file != 0 && position_index >= 7 {
            threat_bitboard = threat_bitboard.set(position_index - 7);
        }
    }
    threat_bitboard
}

fn generate_knight_threat_board(position_index: u8) -> Bitboard {
    let mut threat_bitboard = Bitboard::default();
    let file = get_file(position_index);

    // U2R1 = +16-1 = 15
    if position_index <= 48 && file != 7 {
        let tar = position_index + 15;
        threat_bitboard = threat_bitboard.set(tar);
    }
    // U1R2 = +8-2 = 6
    if position_index <= 55 && file < 6 {
        let tar = position_index + 6;
        threat_bitboard = threat_bitboard.set(tar);
    }
    // D1R2 = -8-2 = -10
    if position_index >= 10 && file < 6 {
        let tar = position_index - 10;
        threat_bitboard = threat_bitboard.set(tar);
    }
    // D2R1 = -16-1 = -17
    if position_index >= 17 && file != 7 {
        let tar = position_index - 17;
        threat_bitboard = threat_bitboard.set(tar);
    }
    // D2L1 = -16+1 = -15
    if position_index >= 15 && file != 0 {
        let tar = position_index - 15;
        threat_bitboard = threat_bitboard.set(tar);
    }
    // D1L2 = -8+2 = -6
    if position_index >= 6 && file > 1 {
        let tar = position_index - 6;
        threat_bitboard = threat_bitboard.set(tar);
    }
    // U1L2 = 8+2 = 10
    if position_index <= 53 && file > 1 {
        let tar = position_index + 10;
        threat_bitboard = threat_bitboard.set(tar);
    }
    // U2L1 = 16+1 = 17
    if position_index <= 46 && file != 0 {
        let tar = position_index + 17;
        threat_bitboard = threat_bitboard.set(tar);
    }
    threat_bitboard
}

fn generate_bishop_threat_board(bitboard: Bitboard, position_index: u8) -> Bitboard {
    sliding_threat_generator(bitboard, position_index, true, false, false)
}

fn generate_rook_threat_board(bitboard: Bitboard, position_index: u8) -> Bitboard {
    sliding_threat_generator(bitboard, position_index, false, true, false)
}

fn generate_queen_threat_board(bitboard: Bitboard, position_index: u8) -> Bitboard {
    sliding_threat_generator(bitboard, position_index, true, true, false)
}

fn generate_king_threat_board(bitboard: Bitboard, position_index: u8) -> Bitboard {
    sliding_threat_generator(bitboard, position_index, true, true, true)
}

fn sliding_threat_generator(
    bitboard: Bitboard,
    position_index: u8,
    diag: bool,
    straight: bool,
    king: bool,
) -> Bitboard {
    let mut threat_bitboard = Bitboard::default();
    let depth = if king { 1 } else { 8 };
    if diag {
        let positions_d_l = get_available_slide_pos(bitboard, position_index, -1, -1, depth);

        for i in 0..positions_d_l.len() {
            threat_bitboard = threat_bitboard.set(positions_d_l[i]);
        }

        let positions_u_l = get_available_slide_pos(bitboard, position_index, 1, -1, depth);

        for i in 0..positions_u_l.len() {
            threat_bitboard = threat_bitboard.set(positions_u_l[i]);
        }

        let positions_u_r = get_available_slide_pos(bitboard, position_index, 1, 1, depth);
        for i in 0..positions_u_r.len() {
            threat_bitboard = threat_bitboard.set(positions_u_r[i]);
        }

        let positions_d_r = get_available_slide_pos(bitboard, position_index, -1, 1, depth);

        for i in 0..positions_d_r.len() {
            threat_bitboard = threat_bitboard.set(positions_d_r[i]);
        }
    }

    if straight {
        let positions_r = get_available_slide_pos(bitboard, position_index, 0, 1, depth);

        for i in 0..positions_r.len() {
            threat_bitboard = threat_bitboard.set(positions_r[i]);
        }

        let positions_l = get_available_slide_pos(bitboard, position_index, 0, -1, depth);

        for i in 0..positions_l.len() {
            threat_bitboard = threat_bitboard.set(positions_l[i]);
        }

        let positions_u = get_available_slide_pos(bitboard, position_index, 1, 0, depth);

        for i in 0..positions_u.len() {
            threat_bitboard = threat_bitboard.set(positions_u[i]);
        }

        let positions_d = get_available_slide_pos(bitboard, position_index, -1, 0, depth);

        for i in 0..positions_d.len() {
            threat_bitboard = threat_bitboard.set(positions_d[i]);
        }
    }

    threat_bitboard
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn white_threat_board_at_initial_position() {
        let board = BoardState::default();
        let metrics = board.generate_metrics();
        let expected_threat_board = Bitboard::new(0b11111111_11111111_01111110);
        assert_eq!(metrics.white_threat_board, expected_threat_board);
    }

    #[test]
    fn black_threat_board_at_initial_position() {
        let board = BoardState::default();
        let metrics = board.generate_metrics();
        let expected_threat_board = Bitboard::new(0b11111111_11111111_01111110u64.reverse_bits());
        assert_eq!(metrics.black_threat_board, expected_threat_board);
    }
}
