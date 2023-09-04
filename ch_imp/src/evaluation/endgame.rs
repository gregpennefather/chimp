use crate::{board::{position::Position, board_rep::BoardRep}, r#move::Move};

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
    MATERIAL_VALUES[0] / 2, // Pawn
    MATERIAL_VALUES[1] / 2, // Knight
    MATERIAL_VALUES[2] / 2, // Bishop
    MATERIAL_VALUES[3] / 2, // Rook
    MATERIAL_VALUES[4] / 2, // Queen
    0,                      // King
];

static WHITE_PAWN_SQUARE_SCORE: PieceValueBoard = [
    0, 0, 0, 0, 0, 0, 0, 0, 8, 8, 8, 8, 8, 8, 8, 8, 6, 6, 6, 6, 6, 6, 6, 6, 4, 4, 4, 4, 4, 4, 4, 4,
    2, 2, 2, 2, 2, 2, 2, 2, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0,
];
static BLACK_PAWN_SQUARE_SCORE: PieceValueBoard = [
    0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 2, 2, 2, 2, 2, 2, 2, 2,
    4, 4, 4, 4, 4, 4, 4, 4, 6, 6, 6, 6, 6, 6, 6, 6, 8, 8, 8, 8, 8, 8, 8, 8, 0, 0, 0, 0, 0, 0, 0, 0,
];
static PAWN_SQUARE_FACTOR: i32 = 5;

static KNIGHT_SQUARE_SCORE: PieceValueBoard = [
    -1, -1, -1, -1, -1, -1, -1, -1, -1, 0, 0, 0, 0, 0, 0, -1, -1, 0, 0, 0, 0, 0, 0, -1, -1, 0, 0,
    0, 0, 0, 0, -1, -1, 0, 0, 0, 0, 0, 0, -1, -1, 0, 0, 0, 0, 0, 0, -1, -1, 0, 0, 0, 0, 0, 0, -1,
    -1, -1, -1, -1, -1, -1, -1, -1,
];
static KNIGHT_SQUARE_FACTOR: i32 = 2;

static MOBILITY_VALUE: i32 = 1;

pub fn calculate(board: BoardRep) -> i32 {
    let mut eval = 0;
    eval += piece_aggregate_score(board, board.white_occupancy, MATERIAL_VALUES);
    eval -= piece_aggregate_score(board, board.black_occupancy, MATERIAL_VALUES);

    // eval += piece_square_score(p.white_bitboard & p.pawn_bitboard, WHITE_PAWN_SQUARE_SCORE)
    //     * PAWN_SQUARE_FACTOR;
    // eval -= piece_square_score(p.black_bitboard & p.pawn_bitboard, BLACK_PAWN_SQUARE_SCORE)
    //     * PAWN_SQUARE_FACTOR;

    // eval += piece_square_score(p.white_bitboard & p.knight_bitboard, KNIGHT_SQUARE_SCORE)
    //     * KNIGHT_SQUARE_FACTOR;
    // eval -= piece_square_score(p.black_bitboard & p.knight_bitboard, KNIGHT_SQUARE_SCORE)
    //     * KNIGHT_SQUARE_FACTOR;

    // // Did you leave anything hanging?
    // let white_hanging = p.white_bitboard & !p.white_threatboard & p.black_threatboard;
    // eval -= piece_aggregate_score(p, white_hanging, HANGING_PIECE_VALUE);
    // let black_hanging = p.black_bitboard & !p.black_threatboard & p.white_threatboard;
    // eval += piece_aggregate_score(p, black_hanging, HANGING_PIECE_VALUE);
    eval
}
