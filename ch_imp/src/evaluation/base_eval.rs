use crate::{
    board::{bitboard::Bitboard, position::Position},
    shared::piece_type::PieceType,
};

use super::{
    eval_precomputed_data::*,
    utils::{board_occupancy_score, piece_aggregate_score},
};

pub fn base_eval(p: Position) -> f32 {
    let mut eval = 0.0;

    // eval += piece_aggregate_score(p, p.white_bitboard, MATERIAL_VALUES);
    // eval -= piece_aggregate_score(p, p.black_bitboard, MATERIAL_VALUES);

    // // Reward for threatening an enemy piece
    // eval += piece_aggregate_score(p, p.black_bitboard & p.white_threatboard, THREATENED_PIECE_VALUE);
    // eval -= piece_aggregate_score(p, p.white_bitboard & p.black_threatboard, THREATENED_PIECE_VALUE);

    // // General amount of mobility and threat area
    // eval += p.white_threatboard.count_ones() as f32 * AGGREGATE_THREAT_AREA_REWARD;
    // eval += p.white_mobility_board.count_ones() as f32 * AGGREGATE_MOBILITY_AREA_REWARD;
    // eval -= p.black_threatboard.count_ones() as f32 * AGGREGATE_THREAT_AREA_REWARD;
    // eval -= p.black_mobility_board.count_ones() as f32 * AGGREGATE_MOBILITY_AREA_REWARD;

    // eval += board_occupancy_score(
    //     p.white_bitboard & p.knight_bitboard,
    //     EDGE_SQUARES,
    //     KNIGHT_ON_EDGE,
    // );
    // eval -= board_occupancy_score(
    //     p.black_bitboard & p.knight_bitboard,
    //     EDGE_SQUARES,
    //     KNIGHT_ON_EDGE,
    // );

    // let white_hanging = p.white_bitboard & !p.white_threatboard & p.black_threatboard;
    // eval -= piece_aggregate_score(p, white_hanging, HANGING_PIECE_VALUE);
    // let black_hanging = p.black_bitboard & !p.black_threatboard & p.white_threatboard;
    // eval += piece_aggregate_score(p, black_hanging, HANGING_PIECE_VALUE);

    eval
}
