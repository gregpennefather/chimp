use rand::Rng;

use crate::{
    board::position::Position,
    r#move::move_segment::{MoveSegment, MoveSegmentType},
    shared::piece_type::PieceType,
};

const WHITE_PAWN_ID: usize = 0;
const BLACK_PAWN_ID: usize = 1;
const WHITE_KNIGHT_ID: usize = 2;
const BLACK_KNIGHT_ID: usize = 3;
const BLACK_BISHOP_ID: usize = 4;
const WHITE_BISHOP_ID: usize = 5;
const WHITE_ROOK_ID: usize = 6;
const BLACK_ROOK_ID: usize = 7;
const WHITE_QUEEN_ID: usize = 8;
const BLACK_QUEEN_ID: usize = 9;
const WHITE_KING_ID: usize = 10;
const BLACK_KING_ID: usize = 11;

#[derive(Clone, Copy)]
pub struct ZorbSet {
    table: [[u64; 12]; 64],
    black_turn: u64,
}

// https://en.wikipedia.org/wiki/Zobrist_hashing
impl ZorbSet {
    pub fn new() -> Self {
        let mut arr: [[u64; 12]; 64] = [[0u64; 12]; 64];
        let mut rng = rand::thread_rng();

        for i in 0..64 {
            for j in 0..12 {
                arr[i][j] = rng.gen();
            }
        }
        let black_turn = rng.gen();
        Self {
            table: arr,
            black_turn,
        }
    }

    pub fn hash(&self, position: Position, is_black_turn: bool) -> u64 {
        let mut r = 0;
        if is_black_turn {
            r ^= self.black_turn;
        }
        for position_index in 0..64 {
            if !position.occupancy.occupied(position_index as u8) {
                continue;
            }
            let piece_type = position.get_piece_type_at_index(position_index as u8);
            let is_black = position.black_bitboard.occupied(position_index as u8);
            r ^= match (piece_type, is_black) {
                (PieceType::Pawn, false) => self.table[position_index][WHITE_PAWN_ID],
                (PieceType::Knight, false) => self.table[position_index][WHITE_KNIGHT_ID],
                (PieceType::Bishop, false) => self.table[position_index][WHITE_BISHOP_ID],
                (PieceType::Rook, false) => self.table[position_index][WHITE_ROOK_ID],
                (PieceType::Queen, false) => self.table[position_index][WHITE_QUEEN_ID],
                (PieceType::King, false) => self.table[position_index][WHITE_KING_ID],
                (PieceType::Pawn, true) => self.table[position_index][BLACK_PAWN_ID],
                (PieceType::Knight, true) => self.table[position_index][BLACK_KNIGHT_ID],
                (PieceType::Bishop, true) => self.table[position_index][BLACK_BISHOP_ID],
                (PieceType::Rook, true) => self.table[position_index][BLACK_ROOK_ID],
                (PieceType::Queen, true) => self.table[position_index][BLACK_QUEEN_ID],
                (PieceType::King, true) => self.table[position_index][BLACK_KING_ID],
                _ => panic!("Unknown piece"),
            }
        }
        r
    }

    pub fn shift(&self, zorb: u64, move_segment: MoveSegment) -> u64 {
        if move_segment.segment_type == MoveSegmentType::Pickup
            || move_segment.segment_type == MoveSegmentType::Place
        {
            let piece_zorb_id = (((move_segment.piece_type as usize)-1) * 2)
                + if move_segment.black_piece { 1 } else { 0 };
            zorb ^ self.table[move_segment.index as usize][piece_zorb_id]
        } else {
            zorb
        }
    }

    pub fn colour_shift(&self, zorb: u64) -> u64 {
        zorb ^ self.black_turn
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    pub fn shift_by_move_segment() {
        let zorb_set = ZorbSet::new();
        let origin_position =
            Position::new("rnbq1rk1/ppp2pbp/3p1np1/4p3/2PPP3/2N2N2/PP2BPPP/R1BQ1RK1".into());
        let origin_hash = zorb_set.hash(origin_position, true);

        // Pickup
        let mut result_hash = zorb_set.shift(
            origin_hash,
            MoveSegment {
                segment_type: MoveSegmentType::Pickup,
                index: 48,
                piece_type: PieceType::Pawn,
                black_piece: true,
            },
        );
        // Place
        result_hash = zorb_set.shift(
            result_hash,
            MoveSegment {
                segment_type: MoveSegmentType::Place,
                index: 40,
                piece_type: PieceType::Pawn,
                black_piece: true,
            },
        );
        // Switch active player
        result_hash = zorb_set.colour_shift(result_hash);

        let expected_position =
            Position::new("rnbq1rk1/ppp2pb1/3p1npp/4p3/2PPP3/2N2N2/PP2BPPP/R1BQ1RK1".into());
        let expected_hash = zorb_set.hash(expected_position, false);

        assert_eq!(result_hash, expected_hash);
    }

    #[test]
    pub fn shift_by_move_segment_opening_and_retreating_knights() {
        let zorb_set = ZorbSet::new();
        let origin_position = Position::default();
        let origin_hash = zorb_set.hash(origin_position, false);

        // Pickup Knight
        let mut result_hash = zorb_set.shift(
            origin_hash,
            MoveSegment {
                segment_type: MoveSegmentType::Pickup,
                index: 1,
                piece_type: PieceType::Knight,
                black_piece: false,
            },
        );
        // Place
        result_hash = zorb_set.shift(
            result_hash,
            MoveSegment {
                segment_type: MoveSegmentType::Place,
                index: 16,
                piece_type: PieceType::Knight,
                black_piece: false,
            },
        );
        // Switch active player
        result_hash = zorb_set.colour_shift(result_hash);
        // Pickup Knight
        let mut result_hash = zorb_set.shift(
            result_hash,
            MoveSegment {
                segment_type: MoveSegmentType::Pickup,
                index: 62,
                piece_type: PieceType::Knight,
                black_piece: true,
            },
        );
        // Place
        result_hash = zorb_set.shift(
            result_hash,
            MoveSegment {
                segment_type: MoveSegmentType::Place,
                index: 47,
                piece_type: PieceType::Knight,
                black_piece: true,
            },
        );
        // Switch active player
        result_hash = zorb_set.colour_shift(result_hash);
        // Pickup Knight
        let mut result_hash = zorb_set.shift(
            result_hash,
            MoveSegment {
                segment_type: MoveSegmentType::Pickup,
                index: 16,
                piece_type: PieceType::Knight,
                black_piece: false,
            },
        );
        // Place
        result_hash = zorb_set.shift(
            result_hash,
            MoveSegment {
                segment_type: MoveSegmentType::Place,
                index: 1,
                piece_type: PieceType::Knight,
                black_piece: false,
            },
        );
        // Switch active player
        result_hash = zorb_set.colour_shift(result_hash);
        // Pickup Knight
        let mut result_hash = zorb_set.shift(
            result_hash,
            MoveSegment {
                segment_type: MoveSegmentType::Pickup,
                index: 47,
                piece_type: PieceType::Knight,
                black_piece: true,
            },
        );
        // Place
        result_hash = zorb_set.shift(
            result_hash,
            MoveSegment {
                segment_type: MoveSegmentType::Place,
                index: 62,
                piece_type: PieceType::Knight,
                black_piece: true,
            },
        );
        // Switch active player
        result_hash = zorb_set.colour_shift(result_hash);

        let expected_hash = zorb_set.hash(origin_position, false);

        assert_eq!(result_hash, expected_hash);
    }
}
