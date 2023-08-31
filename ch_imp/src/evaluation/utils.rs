use crate::{
    board::{bitboard::Bitboard, position::Position},
    shared::piece_type::{self, PieceType},
};

use super::eval_precomputed_data::{PieceValues, PieceValueBoard};

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

pub(super) fn piece_aggregate_score(
    p: Position,
    white_hanging: u64,
    piece_value: PieceValues,
) -> i32 {
    let mut r = 0;

    r += board_occupancy_score(p.pawn_bitboard, white_hanging, piece_value[0]);
    r += board_occupancy_score(p.knight_bitboard, white_hanging, piece_value[1]);
    r += board_occupancy_score(p.bishop_bitboard, white_hanging, piece_value[2]);
    r += board_occupancy_score(p.rook_bitboard, white_hanging, piece_value[3]);
    r += board_occupancy_score(p.queen_bitboard, white_hanging, piece_value[4]);
    r += board_occupancy_score(p.pawn_bitboard, white_hanging, piece_value[5]);

    r
}

pub(super) fn piece_square_score(
    piece_bitboard: u64,
    piece_value_board: PieceValueBoard
) -> i32 {
    let mut bb = piece_bitboard;
    let mut r = 0;
    let mut position = 0;
    while bb != 0 {
        let lsb =  bb.trailing_zeros() as usize;
        position += lsb;
        r += piece_value_board[position];
        position += 1;
        bb >>= lsb + 1;
    }
    r
}