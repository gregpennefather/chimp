use crate::{
    evaluation::{self},
    move_generation::{generate_threat_board, MoveGenerationEvalMetrics},
    r#move::{move_segment::MoveSegment, Move},
    search::zorb_set_precomputed::ZORB_SET,
};
use std::fmt::Debug;

use super::{board_rep::BoardRep, king_position_analysis::*};

pub type MoveSegmentArray = [MoveSegment; 6];

#[derive(Copy, Clone, PartialEq)]
pub struct Position {
    pub board: BoardRep,
    pub white_in_check: bool,
    pub black_in_check: bool,
    pub double_check: bool,
    pub eval: i32,
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
            panic!(
                "Invalid king position {} / {}",
                board.black_king_position, board.white_king_position
            )
        }
        let white_king_analysis = analyze_king_position(
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
            board.black_turn,
        );
        let black_king_analysis = analyze_king_position(
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
            !board.black_turn,
        );
        let mut eval = 0;

        let legal = !((board.black_turn && white_king_analysis.check)
            || (!board.black_turn && black_king_analysis.check));

        if legal {
            let move_gen_metrics = MoveGenerationEvalMetrics {
                white_threatboard: generate_threat_board(false, board.white_occupancy, board),
                black_threatboard: generate_threat_board(true, board.black_occupancy, board),
                white_pinned: white_king_analysis.pins,
                black_pinned: black_king_analysis.pins,
            };
            eval = evaluation::calculate(board, move_gen_metrics);
        }
        Self {
            board,
            white_in_check: white_king_analysis.check,
            black_in_check: black_king_analysis.check,
            double_check: if board.black_turn {
                black_king_analysis.double_check
            } else {
                white_king_analysis.double_check
            },
            eval,
        }
    }
}

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

        let eval_metrics = MoveGenerationEvalMetrics {
            white_threatboard: generate_threat_board(false, board.white_occupancy, board),
            black_threatboard: generate_threat_board(true, board.black_occupancy, board),
            white_pinned: vec![],
            black_pinned: vec![],
        };
        let eval = evaluation::calculate(board, eval_metrics);

        Self {
            board,
            black_in_check: false,
            white_in_check: false,
            double_check: false,
            eval,
        }
    }
}
