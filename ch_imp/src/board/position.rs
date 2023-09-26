use crate::{
    evaluation::{self},
    r#move::move_segment::MoveSegment,
    search::zorb_set_precomputed::ZORB_SET,
};
use std::fmt::Debug;

use super::board_rep::BoardRep;

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
        let white_king_analysis = board.get_white_king_analysis();
        let black_king_analysis = board.get_black_king_analysis();
        let mut eval = 0;

        eval = evaluation::calculate(board, black_king_analysis.pins, white_king_analysis.pins);

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

    pub fn current_in_check(&self) -> bool {
        return (self.board.black_turn && self.black_in_check)
            || (!self.board.black_turn && self.white_in_check);
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
        let eval = evaluation::calculate(board, Vec::new(), Vec::new());

        Self {
            board,
            black_in_check: false,
            white_in_check: false,
            double_check: false,
            eval,
        }
    }
}
