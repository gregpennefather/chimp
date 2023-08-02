use super::{
    super::constants::*,
    r#move::Move,
    utils::{check_board_position, valid_position, is_piece_type},
};
use crate::chess::board::position::*;
use std::fmt;

#[derive(Default, Copy, Clone)]
pub struct Piece {
    pub pos: Position,
    pub code: u8,
}

impl fmt::Debug for Piece {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let colour = if (&self.code >> 3) > 0 {
            "Black"
        } else {
            "White"
        };
        let piece_code = &self.code & PIECE_MASK;
        let piece_type = match piece_code {
            PAWN_INDEX => "Pawn",
            KNIGHT_INDEX => "Knight",
            BISHOP_INDEX => "Bishop",
            ROOK_INDEX => "Rook",
            QUEEN_INDEX => "Queen",
            KING_INDEX => "King",
            _ => "Unknown",
        };

        f.debug_struct("Piece")
            .field("pos", &self.pos)
            .field("colour", &colour)
            .field("type", &piece_type)
            .finish()
    }
}

impl Piece {
    pub fn empty(&self) -> bool {
        return &self.code <= &0;
    }
}

fn get_pawn_moves(
    piece_code: u8,
    pos: Position,
    last_opponent_move: Move,
    white_orientated: bool,
    friendly_bitboard: u64,
    opponent_bitboard: u64,
) -> Vec<Move> {
    let orientation: i8 = if white_orientated { 1 } else { -1 };
    let end_file = if white_orientated { 7 } else { 0 };
    let mut moves = Vec::new();

    // Move Forward
    let forward_file = pos.file + orientation;
    if valid_position(pos.rank, forward_file)
        && !check_board_position(friendly_bitboard, pos.rank, forward_file)
        && !check_board_position(opponent_bitboard, pos.rank, forward_file)
    {
        if forward_file == end_file {
            moves.push(Move::new(
                pos,
                Position {
                    rank: pos.rank,
                    file: forward_file,
                },
                piece_code,
                false,
                KNIGHT_INDEX
            ));
            moves.push(Move::new(
                pos,
                Position {
                    rank: pos.rank,
                    file: forward_file,
                },
                piece_code,
                false,
                BISHOP_INDEX
            ));
            moves.push(Move::new(
                pos,
                Position {
                    rank: pos.rank,
                    file: forward_file,
                },
                piece_code,
                false,
                ROOK_INDEX
            ));
            moves.push(Move::new(
                pos,
                Position {
                    rank: pos.rank,
                    file: forward_file,
                },
                piece_code,
                false,
                QUEEN_INDEX
            ));
        } else {
            moves.push(Move::new(
                pos,
                Position {
                    rank: pos.rank,
                    file: forward_file,
                },
                piece_code,
                false,
                0
            ));
        }


        // Double move Forward
        if (white_orientated && pos.file == 1) || (!white_orientated && pos.file == 6) {
            let double_forward_file = pos.file + orientation + orientation;
            if valid_position(pos.rank, double_forward_file)
                && !check_board_position(friendly_bitboard, pos.rank, double_forward_file)
                && !check_board_position(opponent_bitboard, pos.rank, double_forward_file)
            {
                moves.push(Move::new(
                    pos,
                    Position {
                        rank: pos.rank,
                        file: double_forward_file,
                    },
                    piece_code,
                    false,
                    0
                ));
            }
        }
    }

    // Capture - Left
    let left_rank = pos.rank - 1;
    if valid_position(left_rank, forward_file)
        && check_board_position(opponent_bitboard, left_rank, forward_file)
    {
        if forward_file == end_file {
            moves.push(Move::new(
                pos,
                Position {
                    rank: left_rank,
                    file: forward_file,
                },
                piece_code,
                true,
                KNIGHT_INDEX
            ));
            moves.push(Move::new(
                pos,
                Position {
                    rank: left_rank,
                    file: forward_file,
                },
                piece_code,
                true,
                BISHOP_INDEX
            ));
            moves.push(Move::new(
                pos,
                Position {
                    rank: left_rank,
                    file: forward_file,
                },
                piece_code,
                true,
                ROOK_INDEX
            ));
            moves.push(Move::new(
                pos,
                Position {
                    rank: left_rank,
                    file: forward_file,
                },
                piece_code,
                true,
                QUEEN_INDEX
            ));
        } else {
            moves.push(Move::new(
                pos,
                Position {
                    rank: left_rank,
                    file: forward_file,
                },
                piece_code,
                true,
                0
            ));
        }
    }

    // Capture - Right
    let right_rank = pos.rank + 1;
    if valid_position(right_rank, forward_file)
        && check_board_position(opponent_bitboard, right_rank, forward_file)
    {
        if forward_file == end_file {
            moves.push(Move::new(
                pos,
                Position {
                    rank: right_rank,
                    file: forward_file,
                },
                piece_code,
                true,
                KNIGHT_INDEX
            ));
            moves.push(Move::new(
                pos,
                Position {
                    rank: right_rank,
                    file: forward_file,
                },
                piece_code,
                true,
                BISHOP_INDEX
            ));
            moves.push(Move::new(
                pos,
                Position {
                    rank: right_rank,
                    file: forward_file,
                },
                piece_code,
                true,
                ROOK_INDEX
            ));
            moves.push(Move::new(
                pos,
                Position {
                    rank: right_rank,
                    file: forward_file,
                },
                piece_code,
                true,
                QUEEN_INDEX
            ));
        } else {
            moves.push(Move::new(
                pos,
                Position {
                    rank: right_rank,
                    file: forward_file,
                },
                piece_code,
                true,
                0
            ));
        }
    }

    // En Passant
    let opponent_double_start_file = if white_orientated { 6 } else { 1 };
    let opponent_double_end_file = if white_orientated { 4 } else { 3 };
    if is_piece_type(last_opponent_move.piece, PAWN_INDEX)
        && last_opponent_move.from.file == opponent_double_start_file
        && last_opponent_move.to.file == opponent_double_end_file
    {
        if last_opponent_move.to.rank == left_rank {
            moves.push(Move::new(
                pos,
                Position {
                    rank: left_rank,
                    file: forward_file,
                },
                piece_code,
                true,
                0
            ));
        }

        if last_opponent_move.to.rank == right_rank {
            moves.push(Move::new(
                pos,
                Position {
                    rank: right_rank,
                    file: forward_file,
                },
                piece_code,
                true,
                0
            ));
        }
    }

    return moves;
}

#[cfg(test)]
mod tests {
    use crate::chess::board::utils::build_bitboard;

    use super::*;

    #[test]
    fn get_pawn_moves_white_forward_unblocked() {
        // Act
        let moves = get_pawn_moves(
            0,
            Position { rank: 0, file: 1 },
            Move::default(),
            true,
            0,
            0,
        );
        // Assert
        let m = moves.first().unwrap();
        assert_eq!(m.to.file, 2);
        assert_eq!(m.to.rank, 0);
    }

    #[test]
    fn get_pawn_moves_black_forward_unblocked() {
        // Act
        let moves = get_pawn_moves(
            0,
            Position { rank: 4, file: 4 },
            Move::default(),
            false,
            0,
            0,
        );
        // Assert
        assert_eq!(moves.len(), 1);
        let m = moves.first().unwrap();
        assert_eq!(m.to.file, 3);
        assert_eq!(m.to.rank, 4);
    }

    #[test]
    fn get_pawn_moves_forward_blocked_by_opponent() {
        // Act
        let moves: Vec<Move> = get_pawn_moves(
            0,
            Position { rank: 0, file: 1 },
            Move::default(),
            true,
            0,
            build_bitboard(&[Position { rank: 0, file: 2 }]),
        );
        // Assert
        assert_eq!(moves.len(), 0);
    }

    #[test]
    fn get_pawn_moves_forward_blocked_by_friendly() {
        // Act
        let moves: Vec<Move> = get_pawn_moves(
            0,
            Position { rank: 0, file: 3 },
            Move::default(),
            false,
            build_bitboard(&[Position { rank: 0, file: 2 }]),
            0,
        );
        // Assert
        assert_eq!(moves.len(), 0);
    }

    #[test]
    fn get_pawn_double_moves_forward_blocked_by_friendly_immediately() {
        // Act
        let moves: Vec<Move> = get_pawn_moves(
            0,
            Position { rank: 0, file: 1 },
            Move::default(),
            true,
            build_bitboard(&[Position { rank: 0, file: 2 }]),
            0,
        );
        // Assert
        assert_eq!(moves.len(), 0);
    }

    #[test]
    fn get_pawn_double_moves_forward_blocked_by_enemy_in_second_position() {
        // Act
        let moves: Vec<Move> = get_pawn_moves(
            0,
            Position { rank: 0, file: 1 },
            Move::default(),
            true,
            0,
            build_bitboard(&[Position { rank: 0, file: 3 }]),
        );
        // Assert
        assert_eq!(moves.len(), 1);
    }

    #[test]
    fn get_pawn_capture_right() {
        // Act
        let moves: Vec<Move> = get_pawn_moves(
            0,
            Position { rank: 4, file: 2 },
            Move::default(),
            true,
            build_bitboard(&[Position { rank: 4, file: 3 }]),
            build_bitboard(&[Position { rank: 5, file: 3 }]),
        );
        // Assert
        assert_eq!(moves.len(), 1);
        let m = moves.first().unwrap();
        assert_eq!(m.to.rank, 5);
        assert_eq!(m.to.file, 3);
        assert!(m.capture);
    }

    #[test]
    fn get_pawn_capture_left() {
        // Act
        let moves: Vec<Move> = get_pawn_moves(
            0,
            Position { rank: 4, file: 2 },
            Move::default(),
            true,
            build_bitboard(&[Position { rank: 4, file: 3 }]),
            build_bitboard(&[Position { rank: 3, file: 3 }]),
        );
        // Assert
        assert_eq!(moves.len(), 1);
        let m = moves.first().unwrap();
        assert_eq!(m.to.rank, 3);
        assert_eq!(m.to.file, 3);
        assert!(m.capture);
    }

    #[test]
    fn get_pawn_capture_en_passant_left() {
        // Act
        let moves: Vec<Move> = get_pawn_moves(
            0,
            Position { rank: 4, file: 2 },
            Move::default(),
            true,
            build_bitboard(&[Position { rank: 4, file: 3 }]),
            build_bitboard(&[Position { rank: 3, file: 3 }]),
        );
        // Assert
        assert_eq!(moves.len(), 1);
        let m = moves.first().unwrap();
        assert_eq!(m.to.rank, 3);
        assert_eq!(m.to.file, 3);
        assert!(m.capture);
    }

    #[test]
    fn get_pawn_capture_en_passant_right() {
        // Act
        let moves: Vec<Move> = get_pawn_moves(
            0,
            Position { rank: 4, file: 4 },
            Move::new(Position { rank: 5, file: 6 }, Position { rank: 5, file: 4 }, BLACK_MASK | PAWN_INDEX, false, 0),
            true,
            build_bitboard(&[Position { rank: 4, file: 5 }]),
            build_bitboard(&[Position { rank: 5, file: 4 }]),
        );
        // Assert
        assert_eq!(moves.len(), 1);
        let m = moves.first().unwrap();
        assert_eq!(m.to.rank, 5);
        assert_eq!(m.to.file, 5);
        assert!(m.capture);
    }

    #[test]
    fn get_pawn_no_capture_opponent_third_rank_but_not_double_move() {
        // Act
        let moves: Vec<Move> = get_pawn_moves(
            0,
            Position { rank: 7, file: 3 },
            Move::new(Position { rank: 6, file: 2 }, Position { rank: 6, file: 3 }, PAWN_INDEX, false, 0),
            false,
            0,
            build_bitboard(&[Position { rank: 6, file: 3 }]),
        );
        // Assert
        assert_eq!(moves.len(), 1);
        let m = moves.first().unwrap();
        assert!(!m.capture); // The forward move
    }

    #[test]
    fn get_pawn_promotion_moves() {
        // Act
        let moves: Vec<Move> = get_pawn_moves(
            0,
            Position { rank: 2, file: 6 },
            Move::default(),
            true,
            0,
            0,
        );
        // Assert
        assert_eq!(moves.len(), 4);
        assert_eq!(moves[0].promote, KNIGHT_INDEX);
        assert_eq!(moves[1].promote, BISHOP_INDEX);
        assert_eq!(moves[2].promote, ROOK_INDEX);
        assert_eq!(moves[3].promote, QUEEN_INDEX);
    }

    #[test]
    fn get_pawn_promotion_capture_moves() {
        // Act
        let moves: Vec<Move> = get_pawn_moves(
            0,
            Position { rank: 2, file: 6 },
            Move::default(),
            true,
            0,
            build_bitboard(&[Position{rank: 2, file: 7}, Position { rank: 3, file: 7}]),
        );
        // Assert
        assert_eq!(moves.len(), 4);
        assert_eq!(moves[0].promote, KNIGHT_INDEX);
        assert!(moves[0].capture);
        assert_eq!(moves[1].promote, BISHOP_INDEX);
        assert!(moves[1].capture);
        assert_eq!(moves[2].promote, ROOK_INDEX);
        assert!(moves[2].capture);
        assert_eq!(moves[3].promote, QUEEN_INDEX);
        assert!(moves[3].capture);
    }
}
