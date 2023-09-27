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

pub fn calculate(
    board: BoardRep,
    white_pinned: &Vec<ThreatRaycastCollision>,
    black_pinned: &Vec<ThreatRaycastCollision>,
    pawn_structure: i16,
    piece_safety_results: &Vec<PieceSafetyInfo>,
) -> i16 {
    let mut eval = pawn_structure;
    eval += piece_aggregate_score(board, board.white_occupancy, MATERIAL_VALUES);
    eval -= piece_aggregate_score(board, board.black_occupancy, MATERIAL_VALUES);

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

    eval += mop_up_score(board.white_king_position, board.black_king_position);
    eval -= mop_up_score(board.black_king_position, board.white_king_position);

    for white_pin in white_pinned {
        if white_pin.reveal_attack == false {
            let piece: PieceType = board.get_piece_type_at_index(white_pin.at);
            eval -= MATERIAL_VALUES[piece as usize] / 4 * 3
        } else {
            eval -= 25; // Todo improve this - currently a flat penalty for opponent having a possible reveal attack
        }
    }

    for black_pin in black_pinned {
        if black_pin.reveal_attack == false {
            let piece = board.get_piece_type_at_index(black_pin.at);
            eval += MATERIAL_VALUES[piece as usize] / 4 * 3
        } else {
            eval += 25; // Todo improve this - currently a flat penalty for opponent having a possible reveal attack
        }
    }

    eval += sum_piece_safety_penalties(piece_safety_results, MATERIAL_VALUES, board.black_turn);

    eval
}

fn mop_up_score(king_pos: u8, b_king_pos: u8) -> i16 {
    let cmd = manhattan_distance_to_center(king_pos);
    let md = manhattan_distance(king_pos as i8, b_king_pos as i8) as i16;
    (4 * cmd) + (14 - md)
}
