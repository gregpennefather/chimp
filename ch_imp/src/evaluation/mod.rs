use crate::{board::position::Position, r#move::Move};

use self::{utils::piece_aggregate_score, eval_precomputed_data::PHASE_MATERIAL_VALUES};

pub mod base_eval;
pub mod early_eval;
mod eval_precomputed_data;
pub mod late_eval;
pub mod mid_eval;
mod utils;
mod opening;
mod endgame;

const MAX_PHASE_MATERIAL_SCORE: i32 = 32;

pub fn calculate(p: Position, moves: Vec<Move>) -> i32 {
    let phase = phase(p);
    let (white_moves, black_moves): (Vec<_>, Vec<_>) = moves.into_iter().partition(|m| !m.is_black());
    let opening = opening::calculate(p, &white_moves, &black_moves);
    let endgame = endgame::calculate(p, &white_moves, &black_moves);
    (opening * (256 - phase)) + (endgame * phase) / 256
}

fn phase(p: Position) -> i32 {
    let material_score =
        piece_aggregate_score(p, p.occupancy ^ p.pawn_bitboard, PHASE_MATERIAL_VALUES);
    return (material_score * 256 + (MAX_PHASE_MATERIAL_SCORE / 2)) / MAX_PHASE_MATERIAL_SCORE;
}
