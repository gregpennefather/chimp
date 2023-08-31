use crate::{board::position::Position, r#move::Move};

use super::{eval_precomputed_data::{PieceValues, PieceValueBoard}, utils::{piece_aggregate_score, piece_square_score}};

static MATERIAL_VALUES: PieceValues = [
    100, // Pawn
    300, // Knight
    300, // Bishop
    500, // Rook
    900, // Queen
    0, // King
];

static PAWN_SQUARE_SCORE: PieceValueBoard =[0,0,0,0,0,0,0,0,0,0,0,-1,-1,0,0,0,0,0,1,0,0,1,0,0,0,0,0,3,3,0,0,0,0,0,0,3,3,0,0,0,0,0,1,-1,-1,1,0,0,0,0,0,-1,-1,0,0,0,0,0,0,0,0,0,0,0];
static PAWN_SQUARE_FACTOR: i32 = 5;
static KNIGHT_SQUARE_SCORE: PieceValueBoard = [-1,-1,-1,-1,-1,-1,-1,-1,-1,0,0,0,0,0,0,-1,-1,0,0,0,0,0,0,-1,-1,0,0,0,0,0,0,-1,-1,0,0,0,0,0,0,-1,-1,0,0,0,0,0,0,-1,-1,0,0,0,0,0,0,-1,-1,-1,-1,-1,-1,-1,-1,-1];
static KNIGHT_SQUARE_FACTOR: i32 = 10;



static MOBILITY_VALUE : i32 = 1;

pub fn calculate(p: Position, white_moves: &Vec<Move>, black_moves: &Vec<Move>) -> i32 {
    let mut r = 0;
    r += piece_aggregate_score(p, p.white_bitboard, MATERIAL_VALUES);
    r -= piece_aggregate_score(p, p.black_bitboard, MATERIAL_VALUES);

    r += white_moves.len() as i32 * MOBILITY_VALUE;
    r -= black_moves.len() as i32 * MOBILITY_VALUE;

    r += piece_square_score(p.white_bitboard & p.pawn_bitboard, PAWN_SQUARE_SCORE) * PAWN_SQUARE_FACTOR;
    r -= piece_square_score(p.black_bitboard & p.pawn_bitboard, PAWN_SQUARE_SCORE) * PAWN_SQUARE_FACTOR;

    r += piece_square_score(p.white_bitboard & p.knight_bitboard, KNIGHT_SQUARE_SCORE) * KNIGHT_SQUARE_FACTOR;
    r -= piece_square_score(p.black_bitboard & p.knight_bitboard, KNIGHT_SQUARE_SCORE) * KNIGHT_SQUARE_FACTOR;

    r
}