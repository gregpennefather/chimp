use crate::{board::position::Position, r#move::Move};

use super::{eval_precomputed_data::PieceValues, utils::piece_aggregate_score};

static MATERIAL_VALUES: PieceValues = [
    100, // Pawn
    300, // Knight
    300, // Bishop
    500, // Rook
    900, // Queen
    0, // King
];

static MOBILITY_VALUE : i32 = 1;

pub fn calculate(p: Position, white_moves: &Vec<Move>, black_moves: &Vec<Move>) -> i32 {
    let mut r = 0;
    r += piece_aggregate_score(p, p.white_bitboard, MATERIAL_VALUES);
    r -= piece_aggregate_score(p, p.black_bitboard, MATERIAL_VALUES);

    r += white_moves.len() as i32 * MOBILITY_VALUE;
    r *= black_moves.len() as i32 * MOBILITY_VALUE;

    r
}