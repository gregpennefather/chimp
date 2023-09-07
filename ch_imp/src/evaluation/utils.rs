use crate::board::{bitboard::Bitboard, board_rep::BoardRep, position::Position};

use super::eval_precomputed_data::{PieceValueBoard, PieceValues};

const CENTER_DISTANCE_BIT_0: u64 = 0xFF81BDA5A5BD81FF;
const CENTER_DISTANCE_BIT_1: u64 = 0xFFFFC3C3C3C3FFFF;

pub(super) fn half_board_occupancy_score(pov_occupancy: u64, pov_board: u32, factor: i32) -> i32 {
    (pov_occupancy & pov_board as u64).count_ones() as i32 * factor
}

pub(super) fn board_occupancy_score(occupancy: u64, board: u64, factor: i32) -> i32 {
    (occupancy & board as u64).count_ones() as i32 * factor
}

pub(super) fn piece_positional_reward(occupancy: u64, index: u8, factor: i32) -> i32 {
    if occupancy.occupied(index) {
        factor
    } else {
        0
    }
}

pub(super) fn piece_aggregate_score(board: BoardRep, occ: u64, piece_value: PieceValues) -> i32 {
    let mut r = 0;

    r += board_occupancy_score(board.pawn_bitboard, occ, piece_value[0]);
    r += board_occupancy_score(board.knight_bitboard, occ, piece_value[1]);
    r += board_occupancy_score(board.bishop_bitboard, occ, piece_value[2]);
    r += board_occupancy_score(board.rook_bitboard, occ, piece_value[3]);
    r += board_occupancy_score(board.queen_bitboard, occ, piece_value[4]);

    r
}

pub(super) fn piece_square_score(piece_bitboard: u64, piece_value_board: PieceValueBoard) -> i32 {
    let mut bb = piece_bitboard;
    let mut r = 0;
    while bb != 0 {
        let lsb = bb.trailing_zeros() as usize;
        r += piece_value_board[lsb];
        bb ^= 1 << lsb;
    }
    r
}

// https://www.chessprogramming.org/Center_Distance
pub(super) fn distance_to_center(square : u8) -> i32 {
    (2 * ((CENTER_DISTANCE_BIT_1 >> square) & 1) + ((CENTER_DISTANCE_BIT_0 >> square) & 1)) as i32
}

pub(super) fn chebyshev_distance(a:i8, b:i8) -> u8 {
    let a_file = a & 7;
    let a_rank = a >> 3;

    let b_file = b & 7;
    let b_rank = b >> 3;

    u8::max(i8::abs(b_rank - a_rank) as u8, i8::abs(b_file - a_file) as u8)
}

// https://www.chessprogramming.org/Center_Manhattan-Distance
pub(super)  fn manhattan_distance_to_center(square: u8) -> i32 {
   let mut file  = (square as i32)  & 7;
   let mut rank  = (square as i32) >> 3;
   file ^= (file-4) >> 8;
   rank ^= (rank-4) >> 8;
   (file + rank) & 7
}

pub(super)  fn manhattan_distance(a: i8, b:i8) -> u8 {
    let a_file = a & 7;
    let a_rank = a >> 3;

    let b_file = b & 7;
    let b_rank = b >> 3;

    (i8::abs(b_rank - a_rank) + i8::abs(b_file - a_file)) as u8
}