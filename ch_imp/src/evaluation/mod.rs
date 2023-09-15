use log::trace;

use crate::{
    board::board_rep::BoardRep, evaluation::pawn_structure::get_pawn_structure_eval, move_generation::MoveGenerationEvalMetrics};

use self::{eval_precomputed_data::PHASE_MATERIAL_VALUES, utils::piece_aggregate_score};

mod endgame;
mod eval_precomputed_data;
mod opening;
pub mod pawn_structure;
mod utils;

const MAX_PHASE_MATERIAL_SCORE: i32 = 24;

pub fn calculate(board: BoardRep, move_gen_metrics: MoveGenerationEvalMetrics) -> i32 {
    let phase = phase(board);
    let pawn_structure_eval = get_pawn_structure_eval(
        board.king_pawn_zorb,
        board.white_occupancy & board.pawn_bitboard,
        board.black_occupancy & board.pawn_bitboard,
        board.white_king_position,
        board.black_king_position,
    );
    let opening = opening::calculate(
        board,
        &move_gen_metrics.white_pinned,
        &move_gen_metrics.black_pinned,
        move_gen_metrics.white_threatboard,
        move_gen_metrics.black_threatboard,
        pawn_structure_eval.opening,
    );
    let endgame = endgame::calculate(
        board,
        &move_gen_metrics.white_pinned,
        &move_gen_metrics.black_pinned,
        pawn_structure_eval.endgame,
    );
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
