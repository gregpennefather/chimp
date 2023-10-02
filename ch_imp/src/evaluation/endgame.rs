use std::i8;

use log::trace;

use crate::{
    board::{
        bitboard, board_rep::BoardRep, king_position_analysis::ThreatRaycastCollision,
        position::Position,
    },
    evaluation::{
        shared::count_knight_outposts,
        subcategories::{
            pawn::forts::get_forts,
            rook::{forts_on_rook_open_file::{get_forts_on_rook_open_file, self}, on_open_file::count_rooks_on_open_file},
        },
    },
    r#move::Move,
    shared::piece_type::PieceType,
};

use super::{
    eval_precomputed_data::{PieceValueBoard, PieceValues},
    subcategories::mobility::get_mobility,
    utils::{
        distance_to_center, get_piece_safety_penalty, manhattan_distance,
        manhattan_distance_to_center, piece_aggregate_score, piece_square_score,
    },
    PieceSafetyInfo,
};

const MATERIAL_VALUES: PieceValues = [
    195,  // Pawn
    420,  // Knight
    500,  // Bishop
    1300, // Rook
    2300, // Queen
    0,    // King
];

const WHITE_PAWN_SQUARE_SCORE: PieceValueBoard = [
    0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 2, 2, 2, 2, 2, 2, 2, 2,
    4, 4, 4, 4, 4, 4, 4, 4, 6, 6, 6, 6, 6, 6, 6, 6, 8, 8, 8, 8, 8, 8, 8, 8, 0, 0, 0, 0, 0, 0, 0, 0,
];
const BLACK_PAWN_SQUARE_SCORE: PieceValueBoard = [
    0, 0, 0, 0, 0, 0, 0, 0, 8, 8, 8, 8, 8, 8, 8, 8, 6, 6, 6, 6, 6, 6, 6, 6, 4, 4, 4, 4, 4, 4, 4, 4,
    2, 2, 2, 2, 2, 2, 2, 2, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0,
];
const PAWN_SQUARE_FACTOR: i16 = 10;

const KNIGHT_SQUARE_SCORE: PieceValueBoard = [
    -1, -1, -1, -1, -1, -1, -1, -1, -1, 0, 0, 0, 0, 0, 0, -1, -1, 0, 0, 0, 0, 0, 0, -1, -1, 0, 0,
    0, 0, 0, 0, -1, -1, 0, 0, 0, 0, 0, 0, -1, -1, 0, 0, 0, 0, 0, 0, -1, -1, 0, 0, 0, 0, 0, 0, -1,
    -1, -1, -1, -1, -1, -1, -1, -1,
];
const KNIGHT_SQUARE_FACTOR: i16 = 2;

const DOUBLE_BISHOP_REWARD: i16 = 240;
const ROOK_ON_OPEN_FILE_REWARD: i16 = 120;
const KNIGHT_OUTPOST_REWARD: i16 = 50;

const PAWN_DIFFERENCE_SCORE: [i16; 8] = [0, 18, 36, 56, 78, 102, 130, 155];

pub fn calculate(
    board: BoardRep,
    white_pinned: &Vec<ThreatRaycastCollision>,
    black_pinned: &Vec<ThreatRaycastCollision>,
    pawn_structure: i16,
    open_files: u64,
    piece_safety_results: &Vec<PieceSafetyInfo>,
) -> i16 {
    let mut eval = pawn_structure;

    eval += material_score(board);

    trace!("after mat+ps: {eval}");

    eval += piece_positioning_score(board, open_files);

    eval += king_positioning_analysis(board);

    eval += turn_order_advantage(board, &white_pinned, &black_pinned);

    eval += get_piece_safety_penalty(piece_safety_results, MATERIAL_VALUES, board.black_turn);

    // Mobility
    eval += mobility_score(board);

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

    // Pawn advantage
    let pawn_difference = (board.pawn_bitboard & board.white_occupancy).count_ones() as i16
        - (board.pawn_bitboard & board.black_occupancy).count_ones() as i16;
    let difference_score =
        i16::signum(pawn_difference) * PAWN_DIFFERENCE_SCORE[i16::abs(pawn_difference) as usize];
    score += difference_score;

    score
}

fn piece_positioning_score(board: BoardRep, open_files: u64) -> i16 {
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

    // Knight Outpost
    score += count_knight_outposts(
        false,
        board.white_occupancy & board.knight_bitboard,
        board.white_occupancy & board.pawn_bitboard,
        board.black_occupancy & board.pawn_bitboard,
    ) * KNIGHT_OUTPOST_REWARD;
    score -= count_knight_outposts(
        false,
        board.white_occupancy & board.knight_bitboard,
        board.white_occupancy & board.pawn_bitboard,
        board.black_occupancy & board.pawn_bitboard,
    ) * KNIGHT_OUTPOST_REWARD;

    // Rook on open file
    score += count_rooks_on_open_file(board.rook_bitboard & board.white_occupancy, open_files)
        * ROOK_ON_OPEN_FILE_REWARD;
    score -= count_rooks_on_open_file(board.rook_bitboard & board.black_occupancy, open_files)
        * ROOK_ON_OPEN_FILE_REWARD;

    // Blocking open file rook with minor piece fort
    score += get_forts_on_rook_open_file(
        get_forts(
            false,
            (board.knight_bitboard | board.bishop_bitboard) & board.white_occupancy,
            board.pawn_bitboard & board.white_occupancy,
        ),
        board.rook_bitboard & board.black_occupancy,
        open_files,
    ) * (ROOK_ON_OPEN_FILE_REWARD / 2);
    score -= get_forts_on_rook_open_file(
        get_forts(
            true,
            (board.knight_bitboard | board.bishop_bitboard) & board.black_occupancy,
            board.pawn_bitboard & board.black_occupancy,
        ),
        board.rook_bitboard & board.white_occupancy,
        open_files,
    ) * (ROOK_ON_OPEN_FILE_REWARD / 2);

    trace!("piece_positioning_score: {score}");
    score
}

fn king_positioning_analysis(board: BoardRep) -> i16 {
    let mut score = 0;

    score += match orthogonal_piece_difference(board) {
        1..=i8::MAX => mop_up_score(board.black_king_position, board.white_king_position),
        i8::MIN..=-1 => -mop_up_score(board.white_king_position, board.black_king_position),
        _ => 0,
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

fn mobility_score(board: BoardRep) -> i16 {
    let w = get_mobility(false, board) as i16 - 50;
    let b = get_mobility(true, board) as i16 - 50;
    (w - b) * 3
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
    w - b
}
