use crate::{
    board::{bitboard::Bitboard, board_rep::BoardRep, position::Position},
    r#move::Move,
    shared::piece_type::PieceType,
};

use super::{
    eval_precomputed_data::{PieceValueBoard, PieceValues},
    utils::{distance_to_center, piece_aggregate_score, piece_square_score},
};

static MATERIAL_VALUES: PieceValues = [
    100, // Pawn
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
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 2, 2, 2, 2, 2, 2, 2,
    2, 2, 2, 2, 2, 2, 2, 2, 2, 4, 4, 4, 4, 4, 4, 4, 4, 5, 5, 5, 5, 5, 5, 5, 5, 0, 0, 0, 0, 0, 0, 0,
    0,
];
static BLACK_PAWN_SQUARE_SCORE: PieceValueBoard = [
    0, 0, 0, 0, 0, 0, 0, 0, 5, 5, 5, 5, 5, 5, 5, 5, 4, 4, 4, 4, 4, 4, 4, 4, 2, 2, 2, 2, 2, 2, 2, 2,
    2, 2, 2, 2, 2, 2, 2, 2, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0,
];
static PAWN_SQUARE_FACTOR: i32 = 2;

static KNIGHT_SQUARE_SCORE: PieceValueBoard = [
    -1, -1, -1, -1, -1, -1, -1, -1, -1, 0, 0, 0, 0, 0, 0, -1, -1, 0, 1, 0, 0, 1, 0, -1, -1, 0, 0,
    0, 0, 0, 0, -1, -1, 0, 0, 0, 0, 0, 0, -1, -1, 0, 1, 0, 0, 1, 0, -1, -1, 0, 0, 0, 0, 0, 0, -1,
    -1, -1, -1, -1, -1, -1, -1, -1,
];
static KNIGHT_SQUARE_FACTOR: i32 = 2;

static BISHOP_SQUARE_SCORE: PieceValueBoard = [
    -1, -1, -1, -1, -1, -1, -1, -1, -1, 3, 0, 0, 0, 0, 3, -1, -1, 0, 0, 0, 0, 0, 0, -1, -1, 0, 0,
    0, 0, 0, 0, -1, -1, 0, 0, 0, 0, 0, 0, -1, -1, 0, 0, 0, 0, 0, 0, -1, -1, 3, 0, 0, 0, 0, 3, -1,
    -1, -1, -1, -1, -1, -1, -1, -1,
];
static BISHOP_SQUARE_FACTOR: i32 = 2;

const BOARD_CONTROL_SQUARE_REWARD: PieceValueBoard = [
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 2, 2, 2, 2, 1, 1, 1, 1, 2, 3, 3, 2, 1, 1,
    1, 1, 2, 3, 3, 2, 1, 1, 1, 1, 2, 2, 2, 2, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
];

const BOARD_CONTROL_SQUARES_PER_POINT: i32 = 4;

const UNDER_DEVELOPED_PENALTY_POSITIONS: [(PieceType, u8); 4] = [
    (PieceType::Knight, 1),
    (PieceType::Bishop, 2),
    (PieceType::Bishop, 5),
    (PieceType::Knight, 6),
];
static UNDER_DEVELOPED_PENALTY_FACTOR: i32 = 3;

pub fn calculate(board: BoardRep, white_threatboard: u64, black_threatboard: u64) -> i32 {
    let mut eval = 0;
    eval += piece_aggregate_score(board, board.white_occupancy, MATERIAL_VALUES);
    eval -= piece_aggregate_score(board, board.black_occupancy, MATERIAL_VALUES);

    eval += piece_square_score(
        board.white_occupancy & board.pawn_bitboard,
        WHITE_PAWN_SQUARE_SCORE,
    ) * PAWN_SQUARE_FACTOR;
    eval -= piece_square_score(
        board.black_occupancy & board.pawn_bitboard,
        BLACK_PAWN_SQUARE_SCORE,
    ) * PAWN_SQUARE_FACTOR;

    eval += piece_square_score(
        board.white_occupancy & board.knight_bitboard,
        KNIGHT_SQUARE_SCORE,
    ) * KNIGHT_SQUARE_FACTOR;
    eval -= piece_square_score(
        board.black_occupancy & board.knight_bitboard,
        KNIGHT_SQUARE_SCORE,
    ) * KNIGHT_SQUARE_FACTOR;

    eval += piece_square_score(
        board.white_occupancy & board.bishop_bitboard,
        BISHOP_SQUARE_SCORE,
    ) * BISHOP_SQUARE_FACTOR;
    eval -= piece_square_score(
        board.black_occupancy & board.bishop_bitboard,
        BISHOP_SQUARE_SCORE,
    ) * BISHOP_SQUARE_FACTOR;

    eval += piece_centralization_score(
        board.white_occupancy & board.pawn_bitboard & board.knight_bitboard,
    );
    eval -= piece_centralization_score(
        board.black_occupancy & board.pawn_bitboard & board.knight_bitboard,
    );

    eval += under_developed_penalty(board, board.white_occupancy);
    eval -= under_developed_penalty(board, board.black_occupancy.reverse_bits());

    eval += piece_square_score(
        white_threatboard | board.white_occupancy,
        BOARD_CONTROL_SQUARE_REWARD,
    ) / BOARD_CONTROL_SQUARES_PER_POINT;
    eval -= piece_square_score(
        black_threatboard | board.black_occupancy,
        BOARD_CONTROL_SQUARE_REWARD,
    ) / BOARD_CONTROL_SQUARES_PER_POINT;

    eval
}

fn piece_centralization_score(side_occupancy: u64) -> i32 {
    let mut occ = side_occupancy;
    let mut score = 0;
    while occ != 0 {
        let pos = occ.trailing_zeros();
        score += 3 - distance_to_center(pos as u8);
        occ ^= 1 << pos;
    }
    score
}

fn under_developed_penalty(board: BoardRep, orientated_side_occupancy: u64) -> i32 {
    let mut score = 0;

    for penalty in UNDER_DEVELOPED_PENALTY_POSITIONS {
        if orientated_side_occupancy.occupied(penalty.1)
            && board.get_piece_type_at_index(penalty.1) == penalty.0
        {
            score += 1;
        }
    }

    score * UNDER_DEVELOPED_PENALTY_FACTOR
}
