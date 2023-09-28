use std::i8;

use log::trace;

use crate::{
    board::{
        board_rep::BoardRep, king_position_analysis::ThreatRaycastCollision, position::Position,
    },
    r#move::Move,
    shared::piece_type::PieceType,
};

use super::{
    eval_precomputed_data::{PieceValueBoard, PieceValues},
    utils::{
        distance_to_center, manhattan_distance, manhattan_distance_to_center,
        piece_aggregate_score, piece_square_score, sum_piece_safety_penalties,
    }, PieceSafetyInfo,
};

const MATERIAL_VALUES: PieceValues = [
    200, // Pawn
    300, // Knight
    300, // Bishop
    500, // Rook
    900, // Queen
    0,   // King
];

const WHITE_PAWN_SQUARE_SCORE: PieceValueBoard = [
    0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 2, 2, 2, 2, 2, 2, 2, 2,
    4, 4, 4, 4, 4, 4, 4, 4, 6, 6, 6, 6, 6, 6, 6, 6, 8, 8, 8, 8, 8, 8, 8, 8, 0, 0, 0, 0, 0, 0, 0, 0,
];
const BLACK_PAWN_SQUARE_SCORE: PieceValueBoard = [
    0, 0, 0, 0, 0, 0, 0, 0, 8, 8, 8, 8, 8, 8, 8, 8, 6, 6, 6, 6, 6, 6, 6, 6, 4, 4, 4, 4, 4, 4, 4, 4,
    2, 2, 2, 2, 2, 2, 2, 2, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0,
];
const PAWN_SQUARE_FACTOR: i16 = 5;

const KNIGHT_SQUARE_SCORE: PieceValueBoard = [
    -1, -1, -1, -1, -1, -1, -1, -1, -1, 0, 0, 0, 0, 0, 0, -1, -1, 0, 0, 0, 0, 0, 0, -1, -1, 0, 0,
    0, 0, 0, 0, -1, -1, 0, 0, 0, 0, 0, 0, -1, -1, 0, 0, 0, 0, 0, 0, -1, -1, 0, 0, 0, 0, 0, 0, -1,
    -1, -1, -1, -1, -1, -1, -1, -1,
];
const KNIGHT_SQUARE_FACTOR: i16 = 2;

const DOUBLE_BISHOP_REWARD: i16 = MATERIAL_VALUES[0] / 2;
const DOUBLE_KNIGHT_PENALTY: i16 = MATERIAL_VALUES[0] / 4;
const DOUBLE_ROOK_PENALTY: i16 = MATERIAL_VALUES[0] / 4;

pub fn calculate(
    board: BoardRep,
    white_pinned: &Vec<ThreatRaycastCollision>,
    black_pinned: &Vec<ThreatRaycastCollision>,
    pawn_structure: i16,
    piece_safety_results: &Vec<PieceSafetyInfo>,
) -> i16 {
    let mut eval = pawn_structure;

    eval += material_score(board);

    trace!("after mat+ps: {eval}");

    eval += piece_positioning_score(board);

    eval += king_positioning_analysis(board);

    eval += turn_order_advantage(board, &white_pinned, &black_pinned);

    eval += sum_piece_safety_penalties(piece_safety_results, MATERIAL_VALUES, board.black_turn);

    eval
}

fn material_score(board: BoardRep) -> i16 {
    let mut score = 0;
    score += piece_aggregate_score(board, board.white_occupancy, MATERIAL_VALUES);
    score -= piece_aggregate_score(board, board.black_occupancy, MATERIAL_VALUES);

    // Double Bishop reward
    score += if (board.white_occupancy & board.bishop_bitboard).count_ones() == 2 {
        DOUBLE_BISHOP_REWARD
    } else {
        0
    };
    score -= if (board.black_occupancy & board.bishop_bitboard).count_ones() == 2 {
        DOUBLE_BISHOP_REWARD
    } else {
        0
    };

    // Double Knight penalty
    score += if (board.white_occupancy & board.knight_bitboard).count_ones() == 2 {
        DOUBLE_KNIGHT_PENALTY
    } else {
        0
    };
    score -= if (board.black_occupancy & board.knight_bitboard).count_ones() == 2 {
        DOUBLE_KNIGHT_PENALTY
    } else {
        0
    };

    // Double Rook penalty
    score += if (board.white_occupancy & board.rook_bitboard).count_ones() == 2 {
        DOUBLE_ROOK_PENALTY
    } else {
        0
    };
    score -= if (board.black_occupancy & board.rook_bitboard).count_ones() == 2 {
        DOUBLE_ROOK_PENALTY
    } else {
        0
    };
    score
}

fn piece_positioning_score(board: BoardRep) -> i16 {
    let mut score = 0;
    score += piece_square_score(
        board.white_occupancy & board.pawn_bitboard,
        WHITE_PAWN_SQUARE_SCORE,
    ) * PAWN_SQUARE_FACTOR;
    score -= piece_square_score(
        board.black_occupancy & board.pawn_bitboard,
        BLACK_PAWN_SQUARE_SCORE,
    ) * PAWN_SQUARE_FACTOR;

    score += piece_square_score(
        board.white_occupancy & board.knight_bitboard,
        KNIGHT_SQUARE_SCORE,
    ) * KNIGHT_SQUARE_FACTOR;
    score -= piece_square_score(
        board.black_occupancy & board.knight_bitboard,
        KNIGHT_SQUARE_SCORE,
    ) * KNIGHT_SQUARE_FACTOR;
    trace!("piece_positioning_score: {score}");
    score
}

fn king_positioning_analysis(board: BoardRep) -> i16 {
    let mut score = 0;


    score += match orthogonal_piece_difference(board) {
        1..=i8::MAX => {
            mop_up_score(board.black_king_position, board.white_king_position)
        },
        i8::MIN..=-1 => {
            -mop_up_score(board.white_king_position, board.black_king_position)
        },
        _ => 0
    };

    trace!("king_positioning_analysis: {score}");
    score
}

fn turn_order_advantage(
    board: BoardRep,
    white_pinned: &[ThreatRaycastCollision],
    black_pinned: &[ThreatRaycastCollision],
) -> i16 {
    let mut score = 0;
    for white_pin in white_pinned {
        if white_pin.reveal_attack == false {
            let piece: PieceType = board.get_piece_type_at_index(white_pin.at);
            score -= MATERIAL_VALUES[piece as usize] / 4 * 3
        } else {
            score -= 25; // Todo improve this - currently a flat penalty for opponent having a possible reveal attack
        }
    }

    for black_pin in black_pinned {
        if black_pin.reveal_attack == false {
            let piece = board.get_piece_type_at_index(black_pin.at);
            score += MATERIAL_VALUES[piece as usize] / 4 * 3
        } else {
            score += 25; // Todo improve this - currently a flat penalty for opponent having a possible reveal attack
        }
    }
    trace!("turn_order_advantage: {score}");
    score
}

fn mop_up_score(king_pos: u8, b_king_pos: u8) -> i16 {
    let cmd = manhattan_distance_to_center(king_pos);
    let md = manhattan_distance(king_pos as i8, b_king_pos as i8) as i16;
    let r = ((4.7 * cmd as f32) + 1.6 * (14.0 - md as f32)) as i16;
    r
}

fn orthogonal_piece_difference(board: BoardRep) -> i8 {
    let orthog_pieces = board.rook_bitboard | board.queen_bitboard;
    let w = (orthog_pieces & board.white_occupancy).count_ones() as i8;
    let b = (orthog_pieces & board.black_occupancy).count_ones() as i8;
    w-b
}