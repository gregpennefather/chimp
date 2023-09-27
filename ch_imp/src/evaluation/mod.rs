use log::trace;

use crate::{
    board::{
        board_rep::BoardRep, king_position_analysis::ThreatRaycastCollision, see::piece_safety, attack_and_defend_lookups::AttackAndDefendTable,
    },
    evaluation::pawn_structure::get_pawn_structure_eval,
    shared::piece_type::PieceType,
};

use self::{eval_precomputed_data::PHASE_MATERIAL_VALUES, utils::piece_aggregate_score};

mod endgame;
mod eval_precomputed_data;
mod opening;
pub mod pawn_structure;
mod utils;

const MAX_PHASE_MATERIAL_SCORE: i16 = 24;

#[derive(Copy, Clone)]
pub struct PieceSafetyInfo {
    pub index: u8,
    pub score: i8,
    pub is_black: bool,
    pub piece_type: PieceType,
}

pub fn calculate(
    board: BoardRep,
    black_pins: Vec<ThreatRaycastCollision>,
    white_pins: Vec<ThreatRaycastCollision>,
) -> i16 {
    let phase = phase(board) as i32;
    let mut ad_table = AttackAndDefendTable::new();

    let piece_safety_results = generate_piece_safety(&mut ad_table, board);

    let pawn_structure_eval = get_pawn_structure_eval(
        board.king_pawn_zorb,
        board.white_occupancy & board.pawn_bitboard,
        board.black_occupancy & board.pawn_bitboard,
        board.white_king_position,
        board.black_king_position,
    );
    let opening = opening::calculate(
        board,
        &white_pins,
        &black_pins,
        pawn_structure_eval.opening,
        &piece_safety_results,
        &mut ad_table
    ) as i32;
    let endgame = endgame::calculate(
        board,
        &white_pins,
        &black_pins,
        pawn_structure_eval.endgame,
        &piece_safety_results,
    ) as i32;
    let result = ((opening * (256 - phase)) + (endgame * phase)) / 256;

    trace!(
        "Evaluate: {}\nPhase:{phase}\tOpening:{opening},Endgame:{endgame} => {result}",
        board.to_fen()
    );

    result as i16
}

fn phase(board: BoardRep) -> i16 {
    let material_score = MAX_PHASE_MATERIAL_SCORE
        - piece_aggregate_score(board, board.occupancy, PHASE_MATERIAL_VALUES);
    return (material_score * 256 + (MAX_PHASE_MATERIAL_SCORE / 2)) / MAX_PHASE_MATERIAL_SCORE;
}

fn generate_piece_safety(ad_table: &mut  AttackAndDefendTable, board: BoardRep) -> Vec<PieceSafetyInfo> {
    let mut r = Vec::new();
    let mut w_o = board.white_occupancy;
    while w_o != 0 {
        let lsb = w_o.trailing_zeros();
        r.push(get_piece_safety(ad_table, board, lsb as u8, false));
        w_o ^= 1 << lsb;
    }
    let mut b_o = board.black_occupancy;
    while b_o != 0 {
        let lsb = b_o.trailing_zeros();
        r.push(get_piece_safety(ad_table, board, lsb as u8, true));
        b_o ^= 1 << lsb;
    }
    r
}

fn get_piece_safety(ad_table: &mut AttackAndDefendTable, board: BoardRep, index: u8, is_black: bool) -> PieceSafetyInfo {
    let attacked_by = ad_table.get_attacked_by(index, board, !is_black);
    let defended_by = ad_table.get_attacked_by(index, board, is_black);
    let piece_type = board.get_piece_type_at_index(index);
    let score = piece_safety(piece_type, false, attacked_by, defended_by);

    PieceSafetyInfo {
        index,
        score,
        is_black,
        piece_type,
    }
}
