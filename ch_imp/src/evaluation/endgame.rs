use crate::{board::{position::Position, board_rep::BoardRep}, r#move::Move};

use super::{
    eval_precomputed_data::{PieceValueBoard, PieceValues},
    utils::{piece_aggregate_score, piece_square_score, distance_to_center, manhattan_distance_to_center, manhattan_distance},
};

const MATERIAL_VALUES: PieceValues = [
    200, // Pawn
    300, // Knight
    300, // Bishop
    500, // Rook
    900, // Queen
    0,   // King
];

const HANGING_PIECE_VALUE: PieceValues = [
    MATERIAL_VALUES[0] / 2, // Pawn
    MATERIAL_VALUES[1] / 2, // Knight
    MATERIAL_VALUES[2] / 2, // Bishop
    MATERIAL_VALUES[3] / 2, // Rook
    MATERIAL_VALUES[4] / 2, // Queen
    0,                      // King
];

const WHITE_PAWN_SQUARE_SCORE: PieceValueBoard = [
    0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 2, 2, 2, 2, 2, 2, 2, 2,
    4, 4, 4, 4, 4, 4, 4, 4, 6, 6, 6, 6, 6, 6, 6, 6, 8, 8, 8, 8, 8, 8, 8, 8, 0, 0, 0, 0, 0, 0, 0, 0,
];
const BLACK_PAWN_SQUARE_SCORE: PieceValueBoard = [
    0, 0, 0, 0, 0, 0, 0, 0, 8, 8, 8, 8, 8, 8, 8, 8, 6, 6, 6, 6, 6, 6, 6, 6, 4, 4, 4, 4, 4, 4, 4, 4,
    2, 2, 2, 2, 2, 2, 2, 2, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0,
];
const PAWN_SQUARE_FACTOR: i32 = 5;

const KNIGHT_SQUARE_SCORE: PieceValueBoard = [
    -1, -1, -1, -1, -1, -1, -1, -1, -1, 0, 0, 0, 0, 0, 0, -1, -1, 0, 0, 0, 0, 0, 0, -1, -1, 0, 0,
    0, 0, 0, 0, -1, -1, 0, 0, 0, 0, 0, 0, -1, -1, 0, 0, 0, 0, 0, 0, -1, -1, 0, 0, 0, 0, 0, 0, -1,
    -1, -1, -1, -1, -1, -1, -1, -1,
];
const KNIGHT_SQUARE_FACTOR: i32 = 2;

const DOUBLE_BISHOP_REWARD: i32 = MATERIAL_VALUES[0] / 2;

pub fn calculate(board: BoardRep) -> i32 {
    let mut eval = 0;
    eval += piece_aggregate_score(board, board.white_occupancy, MATERIAL_VALUES);
    eval -= piece_aggregate_score(board, board.black_occupancy, MATERIAL_VALUES);

    // Double Bishop reward
    eval += if (board.white_occupancy & board.bishop_bitboard).count_ones() == 2 { DOUBLE_BISHOP_REWARD } else { 0 };
    eval -= if (board.black_occupancy & board.bishop_bitboard).count_ones() == 2 { DOUBLE_BISHOP_REWARD } else { 0 };

    eval += piece_square_score(board.white_occupancy & board.pawn_bitboard, WHITE_PAWN_SQUARE_SCORE)
        * PAWN_SQUARE_FACTOR;
    eval -= piece_square_score(board.black_occupancy & board.pawn_bitboard, BLACK_PAWN_SQUARE_SCORE)
        * PAWN_SQUARE_FACTOR;

    eval += piece_square_score(board.white_occupancy & board.knight_bitboard, KNIGHT_SQUARE_SCORE)
        * KNIGHT_SQUARE_FACTOR;
    eval -= piece_square_score(board.black_occupancy & board.knight_bitboard, KNIGHT_SQUARE_SCORE)
        * KNIGHT_SQUARE_FACTOR;

    eval += mop_up_score(board.white_king_position, board.black_king_position);
    eval -= mop_up_score(board.black_king_position, board.white_king_position);

    eval
}

fn mop_up_score(king_pos: u8, b_king_pos: u8) -> i32 {
    let cmd = manhattan_distance_to_center(king_pos);
    let md = manhattan_distance(king_pos as i8, b_king_pos as i8) as i32;
    (4 * cmd) + (14 - md)
}