use log::trace;

use crate::{
    board::{board_rep::BoardRep, see::piece_safety}, evaluation::pawn_structure::get_pawn_structure_eval, move_generation::MoveGenerationEvalMetrics, shared::{board_utils::get_coords_from_index, piece_type::PieceType}};

use self::{eval_precomputed_data::PHASE_MATERIAL_VALUES, utils::piece_aggregate_score};

mod endgame;
mod eval_precomputed_data;
mod opening;
pub mod pawn_structure;
mod utils;

const MAX_PHASE_MATERIAL_SCORE: i32 = 24;

#[derive(Copy,Clone)]
pub struct PieceSafetyInfo {
    pub index: u8,
    pub score: i8,
    pub is_black: bool,
    pub piece_type: PieceType
}

pub fn calculate(board: BoardRep, move_gen_metrics: MoveGenerationEvalMetrics) -> i32 {
    let phase = phase(board);

    let piece_safety_results = generate_piece_safety(board);

    let pawn_structure_eval = get_pawn_structure_eval(
        board.king_pawn_zorb,
        board.white_occupancy & board.pawn_bitboard,
        board.black_occupancy & board.pawn_bitboard,
        board.white_king_position,
        board.black_king_position
    );
    let opening = opening::calculate(
        board,
        &move_gen_metrics.white_pinned,
        &move_gen_metrics.black_pinned,
        move_gen_metrics.white_threatboard,
        move_gen_metrics.black_threatboard,
        pawn_structure_eval.opening,
        &piece_safety_results
    );
    let endgame = endgame::calculate(
        board,
        &move_gen_metrics.white_pinned,
        &move_gen_metrics.black_pinned,
        pawn_structure_eval.endgame,
        &piece_safety_results
    );
    let result = ((opening * (256 - phase)) + (endgame * phase)) / 256;

    trace!(
        "Evaluate: {}\nPhase:{phase}\tOpening:{opening},Endgame:{endgame} => {result}",
        board.to_fen()
    );

    result
}

fn phase(board: BoardRep) -> i32 {
    let material_score = MAX_PHASE_MATERIAL_SCORE
        - piece_aggregate_score(board, board.occupancy, PHASE_MATERIAL_VALUES);
    return (material_score * 256 + (MAX_PHASE_MATERIAL_SCORE / 2)) / MAX_PHASE_MATERIAL_SCORE;
}

fn generate_piece_safety(board: BoardRep) -> Vec<PieceSafetyInfo> {
    let mut r = Vec::new();
    let mut w_o = board.white_occupancy;
    while w_o != 0 {
        let lsb = w_o.trailing_zeros();
        r.push(get_piece_safety(board, lsb as u8, false));
        w_o ^= 1<<lsb;
    }
    let mut b_o = board.black_occupancy;
    while b_o != 0 {
        let lsb = b_o.trailing_zeros();
        r.push(get_piece_safety(board, lsb as u8, true));
        b_o ^= 1<<lsb;
    }
    r
}

fn get_piece_safety(board: BoardRep, index: u8, is_black: bool) -> PieceSafetyInfo {
    let attacked_by = board.get_attacked_by(index, !is_black);
    let defended_by = board.get_attacked_by(index, is_black);
    let piece_type = board.get_piece_type_at_index(index);
    let score = piece_safety(piece_type, attacked_by, defended_by);

    PieceSafetyInfo { index, score, is_black, piece_type }
}