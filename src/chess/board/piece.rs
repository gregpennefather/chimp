use super::{super::constants::*, r#move::Move, utils::check_board_position};
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
    pos: Position,
    white_orientated: bool,
    friendly_bitboard: u64,
    opponent_bitboard: u64,
) -> Vec<Move> {

    let mut moves = Vec::new();

    // Move Forward
    let check_rank = pos.rank;
    let check_file = pos.file + (if white_orientated { 1 } else { if pos.file > 0 { -1 } else { 0 }}) as u8;
    if !check_board_position(friendly_bitboard, check_rank, check_file)
        && !check_board_position(opponent_bitboard, check_rank, check_file)
    {
        moves.push(Move::new(
            pos,
            Position {
                rank: check_rank,
                file: check_file,
            },
            false,
        ));

        // Double move Forward
        if !check_board_position(friendly_bitboard, check_rank, check_file)
            && !check_board_position(opponent_bitboard, check_rank, check_file)
        {
            moves.push(Move::new(
                pos,
                Position {
                    rank: check_rank,
                    file: check_file,
                },
                false,
            ));
        }
    }

    return moves;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_pawn_moves_white_forward_unblocked() {
        // Act
        let moves = get_pawn_moves(Position { rank: 0, file: 1 }, true, 0, 0);
        // Assert
        assert_eq!(moves.len(), 1);
        let m = moves.first().unwrap();
        assert_eq!(m.to.file, 2 as u8);
        assert_eq!(m.to.rank, 0 as u8);
    }

    #[test]
    fn get_pawn_moves_black_forward_unblocked() {
        // Act
        let moves = get_pawn_moves(Position { rank: 4, file: 4 }, false, 0, 0);
        // Assert
        assert_eq!(moves.len(), 1);
        let m = moves.first().unwrap();
        assert_eq!(m.to.file, 3 as u8);
        assert_eq!(m.to.rank, 4 as u8);
    }

    #[test]
    fn get_pawn_moves_forward_blocked_by_opponent() {
        // Act
        let moves: Vec<Move> = get_pawn_moves(Position { rank: 0, file: 2 }, false, 0, 256);
        // Assert
        assert_eq!(moves.len(), 0);
    }

    #[test]
    fn get_pawn_moves_forward_blocked_by_friendly() {
        // Act
        let moves: Vec<Move> = get_pawn_moves(Position { rank: 0, file: 2 }, false, 256, 0);
        // Assert
        assert_eq!(moves.len(), 0);
    }
}
