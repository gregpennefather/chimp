use crate::board::position::Position;

use super::{
    eval_precomputed_data::*,
    utils::{board_occupancy_score, half_board_occupancy_score, piece_positional_reward},
};

pub fn early_eval(position: Position) -> f32 {
    let mut r = 0.0;

    // // Holding the center with pawns
    // r += board_occupancy_score(
    //     position.white_bitboard & position.pawn_bitboard,
    //     CENTER_FOUR,
    //     PAWN_HOLDING_CENTER_REWARD,
    // );
    // r -= board_occupancy_score(
    //     position.black_bitboard & position.pawn_bitboard,
    //     CENTER_FOUR,
    //     PAWN_HOLDING_CENTER_REWARD,
    // );

    // // Threatening the center
    // r += board_occupancy_score(
    //     position.white_threatboard,
    //     CENTER_FOUR,
    //     THREATENING_CENTER_REWARD,
    // );
    // r -= board_occupancy_score(
    //     position.black_threatboard,
    //     CENTER_FOUR,
    //     THREATENING_CENTER_REWARD,
    // );

    // // Castling
    // r += castling_reward_king_side(
    //     position.white_bitboard & position.king_bitboard,
    //     position.white_bitboard & position.rook_bitboard,
    // );
    // r += castling_reward_queen_side(
    //     position.white_bitboard & position.king_bitboard,
    //     position.white_bitboard & position.rook_bitboard,
    // );

    // r -= castling_reward_king_side(
    //     (position.black_bitboard & position.king_bitboard).reverse_bits(),
    //     (position.black_bitboard & position.rook_bitboard).reverse_bits(),
    // );
    // r -= castling_reward_queen_side(
    //     (position.black_bitboard & position.king_bitboard).reverse_bits(),
    //     (position.black_bitboard & position.rook_bitboard).reverse_bits(),
    // );

    r
}

// fn castling_reward_king_side(king_occupancy: u64, rook_occupancy: u64) -> f32 {
//     piece_positional_reward(king_occupancy, 1, 1.0)
//         * piece_positional_reward(rook_occupancy, 2, 1.0)
//         * CASTLING_REWARD
// }

// fn castling_reward_queen_side(king_occupancy: u64, rook_occupancy: u64) -> f32 {
//     piece_positional_reward(king_occupancy, 5, 1.0)
//         * piece_positional_reward(rook_occupancy, 4, 1.0)
//         * CASTLING_REWARD
// }
