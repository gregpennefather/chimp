use super::{
    super::constants::*,
    r#move::Move,
    utils::{check_board_position, is_piece_type, valid_position},
};
use crate::chess::board::position::*;
use std::fmt;

const KNIGHT_MOVE_FILE_DELTA: [i8; 8] = [2, 1, -1, -2, -2, -1, 1, 2];
const KNIGHT_MOVE_RANK_DELTA: [i8; 8] = [1, 2, 2, 1, -1, -2, -2, -1];

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

pub fn get_pawn_moves(
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
                KNIGHT_INDEX,
            ));
            moves.push(Move::new(
                pos,
                Position {
                    rank: pos.rank,
                    file: forward_file,
                },
                piece_code,
                false,
                BISHOP_INDEX,
            ));
            moves.push(Move::new(
                pos,
                Position {
                    rank: pos.rank,
                    file: forward_file,
                },
                piece_code,
                false,
                ROOK_INDEX,
            ));
            moves.push(Move::new(
                pos,
                Position {
                    rank: pos.rank,
                    file: forward_file,
                },
                piece_code,
                false,
                QUEEN_INDEX,
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
                0,
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
                    0,
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
                KNIGHT_INDEX,
            ));
            moves.push(Move::new(
                pos,
                Position {
                    rank: left_rank,
                    file: forward_file,
                },
                piece_code,
                true,
                BISHOP_INDEX,
            ));
            moves.push(Move::new(
                pos,
                Position {
                    rank: left_rank,
                    file: forward_file,
                },
                piece_code,
                true,
                ROOK_INDEX,
            ));
            moves.push(Move::new(
                pos,
                Position {
                    rank: left_rank,
                    file: forward_file,
                },
                piece_code,
                true,
                QUEEN_INDEX,
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
                0,
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
                KNIGHT_INDEX,
            ));
            moves.push(Move::new(
                pos,
                Position {
                    rank: right_rank,
                    file: forward_file,
                },
                piece_code,
                true,
                BISHOP_INDEX,
            ));
            moves.push(Move::new(
                pos,
                Position {
                    rank: right_rank,
                    file: forward_file,
                },
                piece_code,
                true,
                ROOK_INDEX,
            ));
            moves.push(Move::new(
                pos,
                Position {
                    rank: right_rank,
                    file: forward_file,
                },
                piece_code,
                true,
                QUEEN_INDEX,
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
                0,
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
                0,
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
                0,
            ));
        }
    }

    return moves;
}

pub fn get_knight_moves(
    piece_code: u8,
    pos: Position,
    friendly_bitboard: u64,
    opponent_bitboard: u64,
) -> Vec<Move> {
    let mut moves = Vec::new();

    for index in 0..8 {
        let tar_rank = pos.rank + KNIGHT_MOVE_RANK_DELTA[index];
        if (tar_rank > 7 || tar_rank < 0) {
            continue;
        }
        let tar_file = pos.file + KNIGHT_MOVE_FILE_DELTA[index];
        if (tar_file > 7 || tar_file < 0) {
            continue;
        }
        if check_board_position(friendly_bitboard, tar_rank, tar_file) {
            continue;
        }
        if check_board_position(opponent_bitboard, tar_rank, tar_file) {
            moves.push(Move::new(
                pos,
                Position {
                    rank: tar_rank,
                    file: tar_file,
                },
                piece_code,
                true,
                0,
            ));
        } else {
            moves.push(Move::new(
                pos,
                Position {
                    rank: tar_rank,
                    file: tar_file,
                },
                piece_code,
                false,
                0,
            ));
        }
    }
    moves
}

pub fn get_bishop_moves(
    piece_code: u8,
    pos: Position,
    friendly_bitboard: u64,
    opponent_bitboard: u64,
) -> Vec<Move> {
    let mut moves = Vec::new();

    moves.extend(generate_slide_moves(piece_code, pos, 1, 1, friendly_bitboard, opponent_bitboard));
    moves.extend(generate_slide_moves(piece_code, pos, 1, -1, friendly_bitboard, opponent_bitboard));
    moves.extend(generate_slide_moves(piece_code, pos, -1, 1, friendly_bitboard, opponent_bitboard));
    moves.extend(generate_slide_moves(piece_code, pos, -1, -1, friendly_bitboard, opponent_bitboard));

    moves
}

pub fn get_rook_moves(
    piece_code: u8,
    pos: Position,
    friendly_bitboard: u64,
    opponent_bitboard: u64,
) -> Vec<Move> {
    let mut moves = Vec::new();

    moves.extend(generate_slide_moves(piece_code, pos, 1, 0, friendly_bitboard, opponent_bitboard));
    moves.extend(generate_slide_moves(piece_code, pos, 0, 1, friendly_bitboard, opponent_bitboard));
    moves.extend(generate_slide_moves(piece_code, pos, -1, 0, friendly_bitboard, opponent_bitboard));
    moves.extend(generate_slide_moves(piece_code, pos, 0, -1, friendly_bitboard, opponent_bitboard));

    moves
}

pub fn get_queen_moves(
    piece_code: u8,
    pos: Position,
    friendly_bitboard: u64,
    opponent_bitboard: u64,
) -> Vec<Move> {
    let mut moves = Vec::new();

    moves.extend(generate_slide_moves(piece_code, pos, 1, 1, friendly_bitboard, opponent_bitboard));
    moves.extend(generate_slide_moves(piece_code, pos, 1, -1, friendly_bitboard, opponent_bitboard));
    moves.extend(generate_slide_moves(piece_code, pos, -1, 1, friendly_bitboard, opponent_bitboard));
    moves.extend(generate_slide_moves(piece_code, pos, -1, -1, friendly_bitboard, opponent_bitboard));

    moves.extend(generate_slide_moves(piece_code, pos, 1, 0, friendly_bitboard, opponent_bitboard));
    moves.extend(generate_slide_moves(piece_code, pos, 0, 1, friendly_bitboard, opponent_bitboard));
    moves.extend(generate_slide_moves(piece_code, pos, -1, 0, friendly_bitboard, opponent_bitboard));
    moves.extend(generate_slide_moves(piece_code, pos, 0, -1, friendly_bitboard, opponent_bitboard));

    moves
}

pub fn get_king_moves(
    piece_code: u8,
    pos: Position,
    friendly_bitboard: u64,
    opponent_bitboard: u64,
) -> Vec<Move> {
    let mut moves = Vec::new();

    let m1 = create_move(piece_code, pos, pos.rank + 1, pos.file, friendly_bitboard, opponent_bitboard);
    if (!m1.empty()) {
        moves.push(m1);
    }

    let m2 = create_move(piece_code, pos, pos.rank + 1, pos.file + 1, friendly_bitboard, opponent_bitboard);
    if (!m2.empty()) {
        moves.push(m2);
    }

    let m3 = create_move(piece_code, pos, pos.rank, pos.file + 1, friendly_bitboard, opponent_bitboard);
    if (!m3.empty()) {
        moves.push(m3);
    }

    let m4 = create_move(piece_code, pos, pos.rank -1 , pos.file + 1, friendly_bitboard, opponent_bitboard);
    if (!m4.empty()) {
        moves.push(m4);
    }

    let m5 = create_move(piece_code, pos, pos.rank -1 , pos.file, friendly_bitboard, opponent_bitboard);
    if (!m5.empty()) {
        moves.push(m5);
    }


    let m6 = create_move(piece_code, pos, pos.rank -1 , pos.file -1, friendly_bitboard, opponent_bitboard);
    if (!m6.empty()) {
        moves.push(m6);
    }

    let m7 = create_move(piece_code, pos, pos.rank , pos.file -1, friendly_bitboard, opponent_bitboard);
    if (!m7.empty()) {
        moves.push(m7);
    }

    let m8 = create_move(piece_code, pos, pos.rank + 1 , pos.file -1, friendly_bitboard, opponent_bitboard);
    if (!m8.empty()) {
        moves.push(m8);
    }


    moves
}


fn generate_slide_moves(
    piece_code: u8,
    pos: Position,
    rank_offset: i8,
    file_offset: i8,
    friendly_bitboard: u64,
    opponent_bitboard: u64
) -> Vec<Move> {
    let mut moves = Vec::new();
    let mut index = 1;
    loop {
        let tar_rank = pos.rank + (rank_offset * index);
        let tar_file = pos.file + (file_offset * index);
        if valid_position(tar_rank, tar_file) {
            if check_board_position(opponent_bitboard, tar_rank, tar_file) {
                // Add opponent capture

                moves.push(Move::new(
                    pos,
                    Position {
                        rank: tar_rank,
                        file: tar_file,
                    },
                    piece_code,
                    true,
                    0,
                ));
                return moves;
            } else if check_board_position(friendly_bitboard, tar_rank, tar_file) {
                return moves;
            } else {
                // Add move
                moves.push(Move::new(
                    pos,
                    Position {
                        rank: tar_rank,
                        file: tar_file,
                    },
                    piece_code,
                    false,
                    0,
                ));
                index += 1;
            }
        } else {
            break;
        }
    }
    moves
}

fn create_move(piece_code: u8, from: Position, rank: i8, file: i8, friendly_bitboard: u64, opponent_bitboard: u64) -> Move {
    match move_result(rank, file, friendly_bitboard, opponent_bitboard) {
        1 => Move::new(
            from,
            Position {
                rank: rank,
                file: file,
            },
            piece_code,
            true,
            0,
        ),
        0 => Move::new(
            from,
            Position {
                rank: rank,
                file: file,
            },
            piece_code,
            false,
            0,
        ),
        _ => Move::default()
    }
}

// -1 = invalid
// 0 = move
// 1 = capture
fn move_result(rank: i8, file: i8, friendly_bitboard: u64, opponent_bitboard: u64) -> i8 {
    if valid_position(rank, file) {
        if check_board_position(friendly_bitboard, rank, file) {
            return -1;
        }
        if check_board_position(opponent_bitboard, rank, file) {
            return 1;
        }
        return 0;
    }
    -1
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
            Move::new(
                Position { rank: 5, file: 6 },
                Position { rank: 5, file: 4 },
                BLACK_MASK | PAWN_INDEX,
                false,
                0,
            ),
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
            Move::new(
                Position { rank: 6, file: 2 },
                Position { rank: 6, file: 3 },
                PAWN_INDEX,
                false,
                0,
            ),
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
            build_bitboard(&[Position { rank: 2, file: 7 }, Position { rank: 3, file: 7 }]),
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

    #[test]
    fn get_knight_moves_all_moves_valid() {
        // Arrange
        let moves: Vec<Move> = get_knight_moves(0, Position { rank: 4, file: 4 }, 0, 0);
        // Act
        assert_eq!(moves.len(), 8)
    }

    #[test]
    fn get_knight_moves_left_edge_center() {
        // Arrange
        let moves: Vec<Move> = get_knight_moves(0, Position { rank: 0, file: 4 }, 0, 0);
        // Act
        assert_eq!(moves.len(), 4)
    }

    #[test]
    fn get_bishop_moves_all_moves_valid() {
        // Arrange
        let moves: Vec<Move> = get_bishop_moves(0, Position { rank: 4, file: 4 }, 0, 0);
        // Act
        assert_eq!(moves.len(), 13)
    }

    #[test]
    fn get_bishop_moves_starting_position() {
        // Arrange
        let moves: Vec<Move> = get_bishop_moves(0, Position { rank: 0, file: 0 }, 65535, 0);
        // Act
        assert_eq!(moves.len(), 0)
    }

    #[test]
    fn get_rook_moves_all_moves_valid() {
        // Arrange
        let moves: Vec<Move> = get_rook_moves(0, Position { rank: 1, file: 1 }, 0, 0);
        // Act
        assert_eq!(moves.len(), 14)
    }

    #[test]
    fn get_rook_moves_starting_position() {
        // Arrange
        let moves: Vec<Move> = get_rook_moves(0, Position { rank: 0, file: 0 }, 65535, 0);
        // Act
        assert_eq!(moves.len(), 0)
    }

    #[test]
    fn get_queen_moves_all_moves_valid() {
        // Arrange
        let moves: Vec<Move> = get_queen_moves(0, Position { rank: 4, file: 4 }, 0, 0);
        // Act
        assert_eq!(moves.len(), 27)
    }

    #[test]
    fn get_queen_moves_starting_position() {
        // Arrange
        let moves: Vec<Move> = get_queen_moves(0, Position { rank: 3, file: 0 }, 65535, 0);
        // Act
        assert_eq!(moves.len(), 0)
    }

    #[test]
    fn get_king_moves_all_moves_valid() {
        // Arrange
        let moves: Vec<Move> = get_king_moves(0, Position { rank: 1, file: 1 }, 0, 0);
        // Act
        assert_eq!(moves.len(), 8)
    }

    #[test]
    fn get_king_moves_top_corner() {
        // Arrange
        let moves: Vec<Move> = get_king_moves(0, Position { rank: 7, file: 7 }, 0, 0);
        // Act
        assert_eq!(moves.len(), 3)
    }
}
