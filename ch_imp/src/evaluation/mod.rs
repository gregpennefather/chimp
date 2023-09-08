use log::trace;

use crate::{board::board_rep::BoardRep, r#move::move_generation::MoveGenerationEvalMetrics};

use self::{eval_precomputed_data::PHASE_MATERIAL_VALUES, utils::piece_aggregate_score};

mod endgame;
mod eval_precomputed_data;
mod opening;
mod utils;
pub mod pawn_structure;

const MAX_PHASE_MATERIAL_SCORE: i32 = 24;

pub fn calculate(board: BoardRep, move_gen_metrics: MoveGenerationEvalMetrics) -> i32 {
    let phase = phase(board);
    let opening = opening::calculate(board, move_gen_metrics.white_threatboard, move_gen_metrics.black_threatboard);
    let endgame = endgame::calculate(board);
    let result = ((opening * (256 - phase)) + (endgame * phase)) / 256;

    trace!(
        "Evaluate: {}\nPhase:{phase}\tOpening:{opening},Endgame:{endgame} => {result}",
        board.to_fen()
    );

    result
}

fn phase(board: BoardRep) -> i32 {
    let material_score = MAX_PHASE_MATERIAL_SCORE
        - piece_aggregate_score(board, board.occupancy, PHASE_MATERIAL_VALUES);
    return (material_score * 256 + (MAX_PHASE_MATERIAL_SCORE / 2)) / MAX_PHASE_MATERIAL_SCORE;
}
