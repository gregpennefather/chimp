use crate::{board::position::Position, r#move::Move};

use super::{
    eval_precomputed_data::{PieceValueBoard, PieceValues},
    utils::{piece_aggregate_score, piece_square_score},
};

static MATERIAL_VALUES: PieceValues = [
    200, // Pawn
    300, // Knight
    300, // Bishop
    500, // Rook
    900, // Queen
    0,   // King
];

static HANGING_PIECE_VALUE: PieceValues = [
    MATERIAL_VALUES[0] / 4, // Pawn
    MATERIAL_VALUES[1] / 2, // Knight
    MATERIAL_VALUES[2] / 2, // Bishop
    MATERIAL_VALUES[3] / 2, // Rook
    MATERIAL_VALUES[4] / 2, // Queen
    0,                      // King
];

static WHITE_PAWN_SQUARE_SCORE: PieceValueBoard = [
    10, 10, 10, 10, 10, 10, 10, 10, 5, 5, 5, 5, 5, 5, 5, 5, 4, 4, 4, 4, 4, 4, 4, 4, 3, 3, 3, 3, 3,
    3, 3, 3, 2, 2, 2, 2, 2, 2, 2, 2, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0,
];
static BLACK_PAWN_SQUARE_SCORE: PieceValueBoard = [
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 2, 2, 2, 2, 2, 2, 2, 2,
    3, 3, 3, 3, 3, 3, 3, 3, 4, 4, 4, 4, 4, 4, 4, 4, 5, 5, 5, 5, 5, 5, 5, 5, 10, 10, 10, 10, 10, 10,
    10, 10,
];
static PAWN_SQUARE_FACTOR: i32 = 10;

static KNIGHT_SQUARE_SCORE: PieceValueBoard = [
    -1, -1, -1, -1, -1, -1, -1, -1, -1, 0, 0, 0, 0, 0, 0, -1, -1, 0, 0, 0, 0, 0, 0, -1, -1, 0, 0,
    0, 0, 0, 0, -1, -1, 0, 0, 0, 0, 0, 0, -1, -1, 0, 0, 0, 0, 0, 0, -1, -1, 0, 0, 0, 0, 0, 0, -1,
    -1, -1, -1, -1, -1, -1, -1, -1,
];
static KNIGHT_SQUARE_FACTOR: i32 = 5;

static MOBILITY_VALUE: i32 = 0;

pub fn calculate(p: Position, white_moves: &Vec<Move>, black_moves: &Vec<Move>) -> i32 {
    let mut eval = 0;
    eval += piece_aggregate_score(p, p.white_bitboard, MATERIAL_VALUES);
    eval -= piece_aggregate_score(p, p.black_bitboard, MATERIAL_VALUES);

    eval += white_moves.len() as i32 * MOBILITY_VALUE;
    eval *= black_moves.len() as i32 * MOBILITY_VALUE;

    eval += piece_square_score(p.white_bitboard & p.pawn_bitboard, WHITE_PAWN_SQUARE_SCORE)
        * PAWN_SQUARE_FACTOR;
    eval -= piece_square_score(p.black_bitboard & p.pawn_bitboard, BLACK_PAWN_SQUARE_SCORE)
        * PAWN_SQUARE_FACTOR;

    eval += piece_square_score(p.white_bitboard & p.knight_bitboard, KNIGHT_SQUARE_SCORE)
        * KNIGHT_SQUARE_FACTOR;
    eval -= piece_square_score(p.black_bitboard & p.knight_bitboard, KNIGHT_SQUARE_SCORE)
        * KNIGHT_SQUARE_FACTOR;

    // Did you leave anything hanging?
    // if p.black_turn {
    //     let white_hanging = p.white_bitboard & !p.white_threatboard & p.black_threatboard;
    //     eval -= piece_aggregate_score(p, white_hanging, HANGING_PIECE_VALUE);
    // // } else {
    //     let black_hanging = p.black_bitboard & !p.black_threatboard & p.white_threatboard;
    //     eval += piece_aggregate_score(p, black_hanging, HANGING_PIECE_VALUE);
    // }
    eval
}
