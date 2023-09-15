use crate::{board::{king_position_analysis::ThreatSource, board_rep::BoardRep}, r#move::Move, MOVE_DATA, shared::piece_type::PieceType};

use super::moveboard_to_moves;

pub(super) fn generate_knight_moves(
    index: u8,
    opponent_occupancy: u64,
    occupancy: u64,
    king_threat: Option<ThreatSource>,
    board: BoardRep,
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
    )
}
