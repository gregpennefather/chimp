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
    850, // Rook
    1500, // Queen
    0,    // King
];

const WHITE_PAWN_SQUARE_SCORE: PieceValueBoard = [0,0,0,0,0,0,0,0,13,8,8,10,13,0,2,-7,4,7,-6,1,0,-5,-1,-8,13,9,-3,-7,-7,-8,3,-1,32,24,13,5,-2,4,17,17,94,100,85,67,56,53,82,84,178,173,158,134,147,132,165,187,0,0,0,0,0,0,0,0];
const BLACK_PAWN_SQUARE_SCORE: PieceValueBoard = [0,0,0,0,0,0,0,0,178,173,158,134,147,132,165,187,94,100,85,67,56,53,82,84,32,24,13,5,-2,4,17,17,13,9,-3,-7,-7,-8,3,-1,4,7,-6,1,0,-5,-1,-8,13,8,8,10,13,0,2,-7,0,0,0,0,0,0,0,0];

const WHITE_KNIGHT_SQUARE_SCORE: PieceValueBoard = [-29,-51,-23,-15,-22,-18,-50,-64,-42,-20,-10,-5,-2,-20,-23,-44,-23,-3,-1,15,10,-3,-20,-22,-18,-6,16,25,16,17,4,-18,-17,3,22,22,22,11,8,-18,-24,-20,10,9,-1,-9,-19,-41,-25,-8,-25,-2,-9,-25,-24,-52,-58,-38,-13,-28,-31,-27,-63,-99];
const BLACK_KNIGHT_SQUARE_SCORE: PieceValueBoard = [-58,-38,-13,-28,-31,-27,-63,-99,-25,-8,-25,-2,-9,-25,-24,-52,-24,-20,10,9,-1,-9,-19,-41,-17,3,22,22,22,11,8,-18,-18,-6,16,25,16,17,4,-18,-23,-3,-1,15,10,-3,-20,-22,-42,-20,-10,-5,-2,-20,-23,-44,-29,-51,-23,-15,-22,-18,-50,-64];

const WHITE_BISHOP_SQUARE_SCORE: PieceValueBoard = [-23,-9,-23,-5,-9,-16,-5,-17,-14,-18,-7,-1,4,-9,-15,-27,-12,-3,8,10,13,3,-7,-15,-6,3,13,19,7,10,-3,-9,-3,9,12,9,14,10,3,2,2,-8,0,-1,-2,6,0,4,-8,-4,7,-12,-3,-13,-4,-14,-14,-21,-11,-8,-7,-9,-17,-24];
const BLACK_BISHOP_SQUARE_SCORE: PieceValueBoard = [-14,-21,-11,-8,-7,-9,-17,-24,-8,-4,7,-12,-3,-13,-4,-14,2,-8,0,-1,-2,6,0,4,-3,9,12,9,14,10,3,2,-6,3,13,19,7,10,-3,-9,-12,-3,8,10,13,3,-7,-15,-14,-18,-7,-1,4,-9,-15,-27,-23,-9,-23,-5,-9,-16,-5,-17];

const WHITE_ROOK_SQUARE_SCORE: PieceValueBoard = [-9,2,3,-1,-5,-13,4,-20,-6,-6,0,2,-9,-9,-11,-3,-4,0,-5,-1,-7,-12,-8,-16,3,5,8,4,-5,-6,-8,-11,4,3,13,1,2,1,-1,2,7,7,7,5,4,-3,-5,-3,11,13,13,11,-3,3,8,3,13,10,18,15,12,12,8,5];
const BLACK_ROOK_SQUARE_SCORE: PieceValueBoard = [13,10,18,15,12,12,8,5,11,13,13,11,-3,3,8,3,7,7,7,5,4,-3,-5,-3,4,3,13,1,2,1,-1,2,3,5,8,4,-5,-6,-8,-11,-4,0,-5,-1,-7,-12,-8,-16,-6,-6,0,2,-9,-9,-11,-3,-9,2,3,-1,-5,-13,4,-20];

const WHITE_QUEEN_SQUARE_SCORE: PieceValueBoard = [-33,-28,-22,-43,-5,-32,-20,-41,-22,-23,-30,-16,-16,-23,-36,-32,-16,-27,15,6,9,17,10,5,-18,28,19,47,31,34,39,23,3,22,24,45,57,40,57,36,-20,6,9,49,47,35,19,9,-17,20,32,41,58,25,30,0,-9,22,22,27,27,19,10,20];
const BLACK_QUEEN_SQUARE_SCORE: PieceValueBoard = [-9,22,22,27,27,19,10,20,-17,20,32,41,58,25,30,0,-20,6,9,49,47,35,19,9,3,22,24,45,57,40,57,36,-18,28,19,47,31,34,39,23,-16,-27,15,6,9,17,10,5,-22,-23,-30,-16,-16,-23,-36,-32,-33,-28,-22,-43,-5,-32,-20,-41];

const WHITE_KING_SQUARE_SCORE: PieceValueBoard = [-53,-34,-21,-11,-28,-14,-24,-43,-27,-11,4,13,14,4,-5,-17,-19,-3,11,21,23,16,7,-9,-18,-4,21,24,27,23,9,-11,-8,22,24,27,26,33,26,3,10,17,23,15,20,45,44,13,-12,17,14,17,17,38,23,11,-74,-35,-18,-18,-11,15,4,-17];
const BLACK_KING_SQUARE_SCORE: PieceValueBoard = [-74,-35,-18,-18,-11,15,4,-17,-12,17,14,17,17,38,23,11,10,17,23,15,20,45,44,13,-8,22,24,27,26,33,26,3,-18,-4,21,24,27,23,9,-11,-19,-3,11,21,23,16,7,-9,-27,-11,4,13,14,4,-5,-17,-53,-34,-21,-11,-28,-14,-24,-43];

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
    );
    score -= piece_square_score(
        board.black_occupancy & board.pawn_bitboard,
        BLACK_PAWN_SQUARE_SCORE,
    );

    score += piece_square_score(
        board.white_occupancy & board.knight_bitboard,
        WHITE_KNIGHT_SQUARE_SCORE,
    );
    score -= piece_square_score(
        board.black_occupancy & board.knight_bitboard,
        BLACK_KNIGHT_SQUARE_SCORE,
    );

    score += piece_square_score(
        board.white_occupancy & board.bishop_bitboard,
        WHITE_BISHOP_SQUARE_SCORE,
    );
    score -= piece_square_score(
        board.black_occupancy & board.bishop_bitboard,
        BLACK_BISHOP_SQUARE_SCORE,
    );

    score += piece_square_score(
        board.white_occupancy & board.rook_bitboard,
        WHITE_ROOK_SQUARE_SCORE,
    );
    score -= piece_square_score(
        board.black_occupancy & board.rook_bitboard,
        BLACK_ROOK_SQUARE_SCORE,
    );

    score += piece_square_score(
        board.white_occupancy & board.queen_bitboard,
        WHITE_QUEEN_SQUARE_SCORE,
    );
    score -= piece_square_score(
        board.black_occupancy & board.queen_bitboard,
        BLACK_QUEEN_SQUARE_SCORE,
    );
    score += WHITE_KING_SQUARE_SCORE[board.white_king_position as usize];
    score -= BLACK_KING_SQUARE_SCORE[board.black_king_position as usize];

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
