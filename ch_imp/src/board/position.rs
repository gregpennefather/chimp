use crate::{
    evaluation::{self},
    r#move::{move_generation::generate_moves, move_segment::MoveSegment, Move},
    search::zorb_set_precomputed::ZORB_SET,
};
use std::fmt::Debug;

use super::{board_rep::BoardRep, king_position_analysis::*};

pub type MoveSegmentArray = [MoveSegment; 6];

pub type OrderedMoveList = [Move; 64];

fn default_ordered_move_list() -> OrderedMoveList {
    [Move::default(); 64]
}

#[derive(Copy, Clone, PartialEq)]
pub struct Position {
    pub board: BoardRep,
    pub white_in_check: bool,
    pub black_in_check: bool,
    pub legal: bool,
    pub eval: i32,
    pub moves: [Move; 128],
}

impl Position {
    pub fn new(
        position_segment: String,
        turn_segment: String,
        castling_segment: String,
        ep_segment: String,
    ) -> Self {
        let mut board = BoardRep::new(position_segment, turn_segment, castling_segment, ep_segment);
        board.zorb_key = ZORB_SET.hash(board);

        Self::build(board)
    }

    pub fn from_fen(fen: String) -> Self {
        let mut fen_segments = fen.split_whitespace();

        let position_segment = fen_segments.nth(0).unwrap().to_string();
        let turn_segment = fen_segments.nth(0).unwrap().to_string();
        let castling_segment = fen_segments.nth(0).unwrap().to_string();
        let ep_segment = fen_segments.nth(0).unwrap().to_string();
        Position::new(position_segment, turn_segment, castling_segment, ep_segment)
    }

    pub(crate) fn apply_segments(
        &self,
        move_segments: [MoveSegment; 6],
        new_zorb: u64,
    ) -> Position {
        let board = self.board.apply_segments(move_segments, new_zorb);

        Self::build(board)
    }

    fn build(board: BoardRep) -> Self {
        if board.black_king_position == 255 || board.white_king_position == 255 {
            return Self {
                board,
                white_in_check: false,
                black_in_check: false,
                legal: false,
                eval: 0,
                moves: [Move::default();128],
            };
        }
        let white_king_analysis = if board.black_turn {
            analyze_king_position_shallow(
                board.white_king_position,
                false,
                board.occupancy,
                board.black_occupancy,
                board.pawn_bitboard,
                board.knight_bitboard,
                board.bishop_bitboard,
                board.rook_bitboard,
                board.queen_bitboard,
            )
        } else {
            analyze_king_position(
                board.white_king_position,
                false,
                board.occupancy,
                board.white_occupancy,
                board.black_occupancy,
                board.pawn_bitboard,
                board.knight_bitboard,
                board.bishop_bitboard,
                board.rook_bitboard,
                board.queen_bitboard,
            )
        };
        let black_king_analysis = if board.black_turn {
            analyze_king_position(
                board.black_king_position,
                true,
                board.occupancy,
                board.black_occupancy,
                board.white_occupancy,
                board.pawn_bitboard,
                board.knight_bitboard,
                board.bishop_bitboard,
                board.rook_bitboard,
                board.queen_bitboard,
            )
        } else {
            analyze_king_position_shallow(
                board.black_king_position,
                true,
                board.occupancy,
                board.white_occupancy,
                board.pawn_bitboard,
                board.knight_bitboard,
                board.bishop_bitboard,
                board.rook_bitboard,
                board.queen_bitboard,
            )
        };
        let mut eval = 0;
        let mut moves = [Move::default(); 128];

        let legal = !((board.black_turn && white_king_analysis.check)
            || (!board.black_turn && black_king_analysis.check));

        if legal {
            moves = if board.black_turn {
                generate_moves(&black_king_analysis, board)
            } else {
                generate_moves(&white_king_analysis, board)
            };

            eval = evaluation::calculate(board);
        }
        Self {
            board,
            white_in_check: white_king_analysis.check,
            black_in_check: black_king_analysis.check,
            legal,
            eval,
            moves,
        }
    }
}

// fn set_position_moves_and_meta(mut position: Position) -> (Position, Vec<Move>) {
//     let (
//         white_moves,
//         black_moves,
//         white_threatboard,
//         black_threatboard,
//         white_mobility_board,
//         black_mobility_board,
//     ) = MOVE_DATA.generate_moves_old(position);

//     // Sort moves
//     let mut active_colour_moves = if position.black_turn {
//         black_moves.clone()
//     } else {
//         white_moves.clone()
//     };
//     active_colour_moves.sort();

//     position.white_threatboard = white_threatboard;
//     position.black_threatboard = black_threatboard;
//     position.white_mobility_board = white_mobility_board;
//     position.black_mobility_board = black_mobility_board;

//     position.white_in_check = black_threatboard.occupied(position.white_king_position);
//     position.black_in_check = white_threatboard.occupied(position.black_king_position);

//     position.eval = evaluation::calculate(position, &white_moves, &black_moves);

//     (position, active_colour_moves)
// }

impl Debug for Position {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Position")
            .field(&self.board.to_fen())
            .field(&self.board.zorb_key)
            .finish()
    }
}

impl Default for Position {
    fn default() -> Self {
        let board = BoardRep::default();

        let king_analysis = KingPositionAnalysis {
            check: false,
            double_check: false,
            threat_source: None,
            pins: Vec::new(),
        };
        let moves = generate_moves(&king_analysis, board);

        let eval = evaluation::calculate(board);

        Self {
            board,
            black_in_check: false,
            white_in_check: false,
            legal: true,
            eval,
            moves,
        }
    }
}
