use crate::{board::{board_rep::BoardRep, king_position_analysis::{ThreatSource, ThreatRaycastCollision}}, r#move::Move, MOVE_DATA, move_generation::moveboard_to_moves, shared::piece_type::PieceType};

pub fn generate_bishop_moves(
    index: u8,
    board: BoardRep,
    opponent_occupancy: u64,
    occupancy: u64,
    king_threat: Option<ThreatSource>,
    pin: Option<ThreatRaycastCollision>,
) -> Vec<Move> {
    let mut moveboard = match pin {
        Some(p) => p.threat_ray_mask | (1 << p.from),
        None => MOVE_DATA
            .magic_bitboard_table
            .get_bishop_attacks(index as usize, board.occupancy.into()),
    };

    if king_threat != None {
        let threat = king_threat.unwrap();
        moveboard &= threat.threat_ray_mask | (1 << threat.from);
    }

    moveboard_to_moves(
        index,
        PieceType::Bishop,
        moveboard,
        opponent_occupancy,
        occupancy,
        board,
    )
}



pub fn is_legal_bishop_move(m: Move, board: BoardRep) -> bool {
    return false;
}