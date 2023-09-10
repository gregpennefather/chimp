use crate::{
    board::{bitboard::Bitboard, board_rep::BoardRep},
    r#move::move_generation::generate_queen_moves,
    shared::{
        board_utils::{get_file, get_rank},
        piece_type::PieceType,
    },
};

use super::{
    eval_precomputed_data::{PieceValueBoard, PieceValues},
    utils::*,
};

const MATERIAL_VALUES: PieceValues = [
    100, // Pawn
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
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 2, 2, 2, 2, 2, 2, 2, 2,
    2, 2, 2, 2, 2, 2, 2, 2, 4, 4, 4, 4, 4, 4, 4, 4, 5, 5, 5, 5, 5, 5, 5, 5, 0, 0, 0, 0, 0, 0, 0, 0,
];
const BLACK_PAWN_SQUARE_SCORE: PieceValueBoard = [
    0, 0, 0, 0, 0, 0, 0, 0, 5, 5, 5, 5, 5, 5, 5, 5, 4, 4, 4, 4, 4, 4, 4, 4, 2, 2, 2, 2, 2, 2, 2, 2,
    2, 2, 2, 2, 2, 2, 2, 2, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
];
const PAWN_SQUARE_FACTOR: i32 = 2;

const KNIGHT_SQUARE_SCORE: PieceValueBoard = [
    -1, -1, -1, -1, -1, -1, -1, -1, -1, 0, 0, 0, 0, 0, 0, -1, -1, 0, 1, 0, 0, 1, 0, -1, -1, 0, 0,
    0, 0, 0, 0, -1, -1, 0, 0, 0, 0, 0, 0, -1, -1, 0, 1, 0, 0, 1, 0, -1, -1, 0, 0, 0, 0, 0, 0, -1,
    -1, -1, -1, -1, -1, -1, -1, -1,
];
const KNIGHT_SQUARE_FACTOR: i32 = 2;

const BISHOP_SQUARE_SCORE: PieceValueBoard = [
    -1, -1, -1, -1, -1, -1, -1, -1, -1, 3, 0, 0, 0, 0, 3, -1, -1, 0, 0, 0, 0, 0, 0, -1, -1, 0, 0,
    0, 0, 0, 0, -1, -1, 0, 0, 0, 0, 0, 0, -1, -1, 0, 0, 0, 0, 0, 0, -1, -1, 3, 0, 0, 0, 0, 3, -1,
    -1, -1, -1, -1, -1, -1, -1, -1,
];
const BISHOP_SQUARE_FACTOR: i32 = 2;

const BOARD_CONTROL_SQUARE_REWARD: PieceValueBoard = [1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,2,2,2,2,1,1,1,1,2,4,4,2,1,1,1,1,2,4,4,2,1,1,1,1,2,2,2,2,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1];

const BOARD_CONTROL_SQUARES_PER_POINT: i32 = 4;

const UNDER_DEVELOPED_PENALTY_POSITIONS: [(PieceType, u8); 4] = [
    (PieceType::Knight, 1),
    (PieceType::Bishop, 2),
    (PieceType::Bishop, 5),
    (PieceType::Knight, 6),
];
const UNDER_DEVELOPED_PENALTY_FACTOR: i32 = 5;

const DOUBLE_BISHOP_REWARD: i32 = MATERIAL_VALUES[0] / 2;

const CAN_NOT_CASTLE_PENALTY: i32 = 10;

pub fn calculate(
    board: BoardRep,
    white_threatboard: u64,
    black_threatboard: u64,
    pawn_structure_eval: i16,
) -> i32 {
    let mut eval = pawn_structure_eval as i32;
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

    // Double Bishop reward
    eval += if (board.white_occupancy & board.bishop_bitboard).count_ones() == 2 {
        DOUBLE_BISHOP_REWARD
    } else {
        0
    };
    eval -= if (board.black_occupancy & board.bishop_bitboard).count_ones() == 2 {
        DOUBLE_BISHOP_REWARD
    } else {
        0
    };

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

    eval += king_tropism(board.white_king_position, board.black_occupancy, board);
    eval -= king_tropism(board.black_king_position, board.white_occupancy, board);

    eval -= king_openness(board.white_king_position, board);
    eval += king_openness(board.black_king_position, board);

    if !(board.white_king_side_castling || board.white_queen_side_castling) {
        eval -= CAN_NOT_CASTLE_PENALTY;
    }
    if !(board.black_king_side_castling || board.black_queen_side_castling) {
        eval += CAN_NOT_CASTLE_PENALTY;
    }

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

// https://www.chessprogramming.org/King_Safety
// The higher the safer the king is
fn king_tropism(king_pos: u8, opponent_occupancy: u64, board: BoardRep) -> i32 {
    let mut occ = opponent_occupancy;
    let mut score = 0;
    while occ != 0 {
        let pos = occ.trailing_zeros() as u8;
        let sc = match board.get_piece_type_at_index(pos) {
            PieceType::Knight => chebyshev_distance(pos as i8, king_pos as i8),
            PieceType::Bishop | PieceType::Rook => chebyshev_distance(pos as i8, king_pos as i8),
            PieceType::Queen => chebyshev_distance(pos as i8, king_pos as i8) * 2,
            PieceType::Pawn | PieceType::King => 0, // Should never happen
            PieceType::None => panic!("Unknown piece type"),
        } as i32;
        score += sc;
        occ ^= 1 << pos;
    }
    score
}

// King openness is a penalty for each square the king could reach if they were a queen
fn king_openness(king_pos: u8, board: BoardRep) -> i32 {
    let possible_queen_moves =
        generate_queen_moves(king_pos, board, 0, board.occupancy, false, None);
    possible_queen_moves.len() as i32
}

fn simple_pawn_shield_score(is_black: bool, king_position: u8, pawn_occupancy: u64) -> i32 {
    // Give every king position a score from 0-9
    // 1-2 points for the king position
    // 2 points for pawns 1 rank ahead of the king
    // 1 point for pawns 2 ranks ahead of the king
    // Give a portion of the KING_PAWN_SHIELD_REWARD according to the portion of the
    // max score achieved

    let king_file = get_file(king_position);
    let king_rank = get_rank(king_position);
    if king_rank != 0 && king_rank != 7 {
        return 0;
    }

    let mut score = 0;
    score += match king_rank {
        0 | 1 | 7 => 1,
        2 | 6 => 2,
        _ => 0,
    };

    //let rank_1_shield_mask = if is_black + king_position +
    0
}
