use crate::{
    board::{
        attack_and_defend_lookups::AttackAndDefendTable,
        bitboard::Bitboard,
        board_rep::BoardRep,
        king_position_analysis::{ThreatRaycastCollision, ThreatSource},
    },
    r#move::Move,
    shared::piece_type::PieceType,
    MOVE_DATA,
};

use super::moveboard_to_moves;

pub(super) fn generate_knight_moves(
    index: u8,
    opponent_occupancy: u64,
    occupancy: u64,
    king_threat: Option<ThreatSource>,
    board: BoardRep,
    ad_table: &mut AttackAndDefendTable,
    reveal_attack: Option<ThreatRaycastCollision>,
    phase: i16,
) -> Vec<Move> {
    let mut moveboard = MOVE_DATA.knight_moves[index as usize];

    if king_threat != None {
        let threat = king_threat.unwrap();
        moveboard &= threat.threat_ray_mask | (1 << threat.from);
    }

    moveboard_to_moves(
        index,
        PieceType::Knight,
        moveboard,
        opponent_occupancy,
        occupancy,
        board,
        ad_table,
        reveal_attack,
        phase,
    )
}

pub(super) fn is_legal_knight_move(m: Move, board: BoardRep) -> bool {
    let moveboard = MOVE_DATA.knight_moves[m.from() as usize];

    if !moveboard.occupied(m.to()) {
        return false;
    }
    if m.is_capture() {
        let opponent_occupancy = board.get_opponent_occupancy();
        return opponent_occupancy.occupied(m.to());
    } else {
        return !board.occupancy.occupied(m.to());
    }
}
