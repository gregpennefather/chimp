use rand::Rng;

use crate::{
    board::{position::{MoveSegmentArray, Position}, bitboard::Bitboard, board_rep::BoardRep},
    r#move::move_segment::{MoveSegment, MoveSegmentType},
    shared::{board_utils::get_file, piece_type::PieceType},
};

const WHITE_PAWN_ID: usize = 0;
const BLACK_PAWN_ID: usize = 1;
const WHITE_KNIGHT_ID: usize = 2;
const BLACK_KNIGHT_ID: usize = 3;
const WHITE_BISHOP_ID: usize = 4;
const BLACK_BISHOP_ID: usize = 5;
const WHITE_ROOK_ID: usize = 6;
const BLACK_ROOK_ID: usize = 7;
const WHITE_QUEEN_ID: usize = 8;
const BLACK_QUEEN_ID: usize = 9;
const WHITE_KING_ID: usize = 10;
const BLACK_KING_ID: usize = 11;

#[derive(Clone, Copy, Debug)]
pub struct ZorbSet {
    pub table: [[u64; 12]; 64],
    pub ep_table: [u64; 8],
    pub black_turn: u64,
    pub wkc: u64,
    pub wqc: u64,
    pub bkc: u64,
    pub bqc: u64,
}

// https://en.wikipedia.org/wiki/Zobrist_hashing
impl ZorbSet {
    pub fn new() -> Self {
        let mut rng = rand::thread_rng();

        let mut arr = [[0; 12]; 64];
        for i in 0..64 {
            for j in 0..12 {
                arr[i][j] = rng.gen();
            }
        }

        let mut ep_table: [u64; 8] = [rng.gen(); 8];
        for i in 0..8 {
            ep_table[i] = rng.gen();
        }
        let black_turn = rng.gen();
        let wkc = rng.gen();
        let wqc = rng.gen();
        let bkc = rng.gen();
        let bqc = rng.gen();
        Self {
            table: arr,
            black_turn,
            ep_table,
            wkc,
            wqc,
            bkc,
            bqc,
        }
    }

    pub fn hash(&self, board: BoardRep) -> u64 {
        let mut r = 0;
        if board.black_turn {
            r ^= self.black_turn;
        }
        for position_index in 0..64 {
            if !board.occupancy.occupied(position_index as u8) {
                continue;
            }
            let piece_type = board.get_piece_type_at_index(position_index as u8);
            let is_black = board.black_occupancy.occupied(position_index as u8);

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
                _ => panic!("Unknown piece {:?} at {} is black {}", piece_type, position_index, is_black),
            }
        }
        if board.white_king_side_castling {
            r ^= self.wkc;
        }
        if board.white_queen_side_castling {
            r ^= self.wqc;
        }
        if board.black_king_side_castling {
            r ^= self.bkc;
        }
        if board.black_queen_side_castling {
            r ^= self.bqc;
        }
        if board.ep_index != u8::MAX {
            r ^= self.ep_table[get_file(board.ep_index) as usize]
        }
        r
    }

    pub fn shift(&self, zorb: u64, move_segment: MoveSegment) -> u64 {
        match move_segment.segment_type {
            MoveSegmentType::Pickup | MoveSegmentType::Place => {
                let piece_zorb_id = (((move_segment.piece_type as usize) - 1) * 2)
                    + if move_segment.black_piece { 1 } else { 0 };
                zorb ^ self.table[move_segment.index as usize][piece_zorb_id]
            }
            MoveSegmentType::ClearCastling => {
                if move_segment.black_piece {
                    match move_segment.index {
                        56 => zorb ^ self.bkc,
                        63 => zorb ^ self.bqc,
                        59 => zorb ^ self.bkc ^ self.bqc,
                        _ => zorb,
                    }
                } else {
                    match move_segment.index {
                        0 => zorb ^ self.wkc,
                        7 => zorb ^ self.wqc,
                        3 => zorb ^ self.wkc ^ self.wqc,
                        _ => zorb,
                    }
                }
            }
            MoveSegmentType::DoublePawnPush => {
                zorb ^ self.ep_table[get_file(move_segment.index) as usize]
            }
            MoveSegmentType::ClearEP => zorb ^ self.ep_table[move_segment.index as usize],
            _ => zorb,
        }
    }

    pub fn apply_segments(&self, zorb: u64, move_segments: MoveSegmentArray) -> u64 {
        let mut output = zorb;
        for segment in move_segments {
            if segment.segment_type != MoveSegmentType::None {
                output = self.shift(output, segment);
            } else {
                break;
            }
        }

        output ^= self.black_turn;

        output
    }

    pub fn colour_shift(&self, zorb: u64) -> u64 {
        zorb ^ self.black_turn
    }
}

#[cfg(test)]
mod test {
    use crate::{
        r#move::Move, search::zorb_set_precomputed::ZORB_SET,
        shared::board_utils::index_from_coords,
    };

    use super::*;

    // TODO: Update these

    #[test]
    pub fn hash_should_differ_based_on_ep_position() {
        let board_rep = BoardRep::default();
        let board_rep_ep_0 =
        BoardRep::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq h3".into());
        let board_rep_ep_5 =
        BoardRep::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq c3".into());

        assert_ne!(
            board_rep.zorb_key, board_rep_ep_0.zorb_key,
            "No ep and ep=0 should not match"
        );
        assert_ne!(
            board_rep_ep_5.zorb_key, board_rep_ep_0.zorb_key,
            "Ep=5 and ep=0 should not match"
        );
        assert_ne!(
            board_rep.zorb_key, board_rep_ep_5.zorb_key,
            "No ep and ep=5 should not match"
        );
    }

    #[test]
    pub fn shift_by_move_segment() {
        let zorb_set = ZorbSet::new();
        let origin_board_rep = BoardRep::from_fen(
            "rnbq1rk1/ppp2pbp/3p1np1/4p3/2PPP3/2N2N2/PP2BPPP/R1BQ1RK1 b KQkq -".into(),
        );
        let origin_hash = zorb_set.hash(origin_board_rep);

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

        let expected_board_rep = BoardRep::from_fen(
            "rnbq1rk1/ppp2pb1/3p1npp/4p3/2PPP3/2N2N2/PP2BPPP/R1BQ1RK1 w KQkq -".into(),
        );
        let expected_hash = zorb_set.hash(expected_board_rep);

        assert_eq!(result_hash, expected_hash);
    }

    #[test]
    pub fn shift_by_move_segment_opening_and_retreating_knights() {
        let zorb_set = ZorbSet::new();
        let origin_board_rep = BoardRep::default();
        let origin_hash = zorb_set.hash(origin_board_rep);

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

        let expected_hash = zorb_set.hash(origin_board_rep);

        assert_eq!(result_hash, expected_hash);
    }

    // #[test]
    // pub fn zorb_sequence_test_case_0() {
    //     // In this black will move their king-side rook out and then back in - this should result in two different codes
    //     let original_position =
    //         Position::from_fen("rnbqkbnr/ppppppp1/8/7p/4P3/8/PPPP1PPP/RNBQKBNR w KQkq -".into());
    //     let moves = [
    //         Move::new(
    //             index_from_coords("f1"),
    //             index_from_coords("e2"),
    //             0b0,
    //             PieceType::Bishop,
    //             false,
    //         ),
    //         Move::new(
    //             index_from_coords("h8"),
    //             index_from_coords("h7"),
    //             0b0,
    //             PieceType::Rook,
    //             true,
    //         ),
    //         Move::new(
    //             index_from_coords("e2"),
    //             index_from_coords("f1"),
    //             0b0,
    //             PieceType::Bishop,
    //             false,
    //         ),
    //         Move::new(
    //             index_from_coords("h7"),
    //             index_from_coords("h8"),
    //             0b0,
    //             PieceType::Rook,
    //             true,
    //         ),
    //     ];
    //     let mut position = original_position;
    //     for m in moves {
    //         position = position.make(m);
    //     }
    //     assert_ne!(
    //         position.zorb_key, original_position.zorb_key,
    //         "New zorb should not match original"
    //     );

    //     let new_position =
    //         Position::from_fen("rnbqkbnr/ppppppp1/8/7p/4P3/8/PPPP1PPP/RNBQKBNR w KQq -".into());
    //     assert_eq!(position.zorb_key, new_position.zorb_key)
    // }

    // #[test]
    // pub fn zorb_sequence_test_case_1() {
    //     // In this white has double pawn pushed leaving themself open to a EP capture, black does not capture, instead pushing forward a knight, white pushes their own knight, then both retreat

    //     let original_position = Position::from_fen("1k6/8/8/5n2/6pP/8/8/1K2N3 b - h3".into());
    //     let moves = [
    //         Move::new(
    //             index_from_coords("f5"),
    //             index_from_coords("e3"),
    //             0b0,
    //             PieceType::Knight,
    //             true,
    //         ),
    //         Move::new(
    //             index_from_coords("e1"),
    //             index_from_coords("c2"),
    //             0b0,
    //             PieceType::Knight,
    //             false,
    //         ),
    //         Move::new(
    //             index_from_coords("e3"),
    //             index_from_coords("f5"),
    //             0b0,
    //             PieceType::Knight,
    //             true,
    //         ),
    //         Move::new(
    //             index_from_coords("c2"),
    //             index_from_coords("e1"),
    //             0b0,
    //             PieceType::Knight,
    //             false,
    //         ),
    //     ];

    //     let mut position = original_position;
    //     for m in moves {
    //         position = position.make(m);
    //     }
    //     assert_ne!(position, original_position);
    //     assert_ne!(
    //         position.zorb_key, original_position.zorb_key,
    //         "New zorb should not match original"
    //     );
    //     let new_position = Position::from_fen("1k6/8/8/5n2/6pP/8/8/1K2N3 b - -".into());
    //     assert_eq!(position.zorb_key, new_position.zorb_key)
    // }
}
