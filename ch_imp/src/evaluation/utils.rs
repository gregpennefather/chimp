use log::trace;

use crate::{
    board::{bitboard::Bitboard, board_rep::BoardRep, position::Position},
    shared::board_utils::get_coords_from_index,
};

use super::{
    eval_precomputed_data::{PieceValueBoard, PieceValues},
    PieceSafetyInfo,
};

const CENTER_DISTANCE_BIT_0: u64 = 0xFF81BDA5A5BD81FF;
const CENTER_DISTANCE_BIT_1: u64 = 0xFFFFC3C3C3C3FFFF;

pub(super) fn half_board_occupancy_score(pov_occupancy: u64, pov_board: u32, factor: i16) -> i16 {
    (pov_occupancy & pov_board as u64).count_ones() as i16 * factor
}

pub(super) fn board_occupancy_score(occupancy: u64, board: u64, factor: i16) -> i16 {
    (occupancy & board as u64).count_ones() as i16 * factor
}

pub(super) fn piece_positional_reward(occupancy: u64, index: u8, factor: i16) -> i16 {
    if occupancy.occupied(index) {
        factor
    } else {
        0
    }
}

pub(super) fn piece_aggregate_score(board: BoardRep, occ: u64, piece_value: PieceValues) -> i16 {
    let mut r = 0;

    r += board_occupancy_score(board.pawn_bitboard, occ, piece_value[0]);
    r += board_occupancy_score(board.knight_bitboard, occ, piece_value[1]);
    r += board_occupancy_score(board.bishop_bitboard, occ, piece_value[2]);
    r += board_occupancy_score(board.rook_bitboard, occ, piece_value[3]);
    r += board_occupancy_score(board.queen_bitboard, occ, piece_value[4]);

    r
}

pub(super) fn piece_square_score(piece_bitboard: u64, piece_value_board: PieceValueBoard) -> i16 {
    let mut bb = piece_bitboard;
    let mut r = 0;
    while bb != 0 {
        let lsb = bb.trailing_zeros() as usize;
        r += piece_value_board[lsb];
        bb ^= 1 << lsb;
    }
    r
}

// Calculates the score penalty for hanging pieces
// For active players turn the penalty is:
// - 75% of second least valuable piece
// - + 5 per unsafe piece  > 2
// For inactive player the penalty is:
// - 75% of most valuable piece
// - +10 per unsafe piece > 1
pub fn get_piece_safety_penalty(
    piece_safety_results: &Vec<PieceSafetyInfo>,
    piece_values: PieceValues,
    black_turn: bool,
) -> i16 {
    let mut r = 0;

    let mut unsafe_active: Vec<PieceSafetyInfo> = piece_safety_results
        .clone()
        .into_iter()
        .filter(|r| r.is_black == black_turn && r.score < 0)
        .collect();

    if unsafe_active.len() > 1 {
        unsafe_active.sort_by(|a, b| b.piece_type.cmp(&a.piece_type));
        let second_lvp = unsafe_active[1].piece_type;
        if black_turn {
            r += piece_values[second_lvp as usize - 1] / 4 * 3;
            r += (unsafe_active.len() as i16 - 2) * 5;
        } else {
            r -= piece_values[second_lvp as usize - 1] / 4 * 3;
            r -= (unsafe_active.len() as i16 - 2) * 5;
        }

        trace!("active {r}: {second_lvp:?} {:?}", unsafe_active);
    }

    let mut unsafe_inactive: Vec<PieceSafetyInfo> = piece_safety_results
        .clone()
        .into_iter()
        .filter(|r| r.is_black != black_turn && r.score < 0)
        .collect();

    if unsafe_inactive.len() > 0 {
        unsafe_inactive.sort_by(|a, b| b.piece_type.cmp(&a.piece_type));
        let second_lvp = unsafe_inactive[0].piece_type;
        if black_turn {
            r -= piece_values[second_lvp as usize - 1] / 8 * 7;
            r -= (unsafe_inactive.len() as i16 - 1) * 10;
        } else {
            r += piece_values[second_lvp as usize - 1] / 8 * 7;
            r += (unsafe_inactive.len() as i16 - 1) * 10;
        }

        trace!("inactive {r}: {second_lvp:?} {:?}", unsafe_active);
    }

    trace!("piece safety: {r}");

    r
}

// https://www.chessprogramming.org/Center_Distance
pub(super) fn distance_to_center(square: u8) -> i16 {
    (2 * ((CENTER_DISTANCE_BIT_1 >> square) & 1) + ((CENTER_DISTANCE_BIT_0 >> square) & 1)) as i16
}

// https://www.chessprogramming.org/Center_Manhattan-Distance
pub(super) fn manhattan_distance_to_center(square: u8) -> i16 {
    let mut file = (square as i16) & 7;
    let mut rank = (square as i16) >> 3;
    file ^= (file - 4) >> 8;
    rank ^= (rank - 4) >> 8;
    (file + rank) & 7
}

pub(super) fn manhattan_distance(a: i8, b: i8) -> u8 {
    let a_file = a & 7;
    let a_rank = a >> 3;

    let b_file = b & 7;
    let b_rank = b >> 3;

    (i8::abs(b_rank - a_rank) + i8::abs(b_file - a_file)) as u8
}
