use crate::{board::position::Position, r#move::Move};

use self::{eval_precomputed_data::PHASE_MATERIAL_VALUES, utils::piece_aggregate_score};

pub mod base_eval;
pub mod early_eval;
mod endgame;
mod eval_precomputed_data;
mod opening;
mod utils;

const MAX_PHASE_MATERIAL_SCORE: i32 = 24;

pub fn calculate(p: Position, white_moves: &Vec<Move>, black_moves: &Vec<Move>) -> i32 {
    let phase = phase(p);

    let opening = opening::calculate(p, &white_moves, &black_moves);
    let endgame = endgame::calculate(p, &white_moves, &black_moves);
    (endgame * (256 - phase)) + (opening * phase) / 256
}

fn phase(p: Position) -> i32 {
    let material_score = piece_aggregate_score(p, p.occupancy, PHASE_MATERIAL_VALUES);
    return (material_score * 256 + (MAX_PHASE_MATERIAL_SCORE / 2)) / MAX_PHASE_MATERIAL_SCORE;
}
