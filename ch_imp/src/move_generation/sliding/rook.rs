use crate::{board::{board_rep::BoardRep, king_position_analysis::{ThreatSource, ThreatRaycastCollision}, bitboard::Bitboard}, r#move::Move, MOVE_DATA, move_generation::moveboard_to_moves, shared::piece_type::PieceType};

pub fn generate_rook_moves(
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
            .get_rook_attacks(index as usize, board.occupancy.into()),
    };

    if king_threat != None {
        let threat = king_threat.unwrap();
        moveboard &= threat.threat_ray_mask | (1 << threat.from);
    }

    moveboard_to_moves(
        index,
        PieceType::Rook,
        moveboard,
        opponent_occupancy,
        occupancy,
        board,
    )
}


pub fn is_legal_rook_move(m: Move, board: BoardRep) -> bool {
    if MOVE_DATA.is_slide_legal(m.from(), m.to()).0 {
        board.occupancy & MOVE_DATA.get_slide_inbetween(m.from(), m.to()) == 0
    } else {
        false
    }
}

#[cfg(test)]
mod test {
    use crate::{board::board_rep::BoardRep, shared::{board_utils::index_from_coords, piece_type::PieceType, constants::MF_CAPTURE}, r#move::Move, move_generation::sliding::rook::is_legal_rook_move};

    #[test]
    fn is_legal_rook_move_not_orthog() {
        let board = BoardRep::from_fen("rnbqk2r/pppp1ppp/4pn2/8/2PP4/P1P5/4PPPP/R1BQKBNR b KQkq - 0 1".into());
        let m = Move::new(index_from_coords("a1"), index_from_coords("b2"), 0b0, PieceType::Rook, false, 0);

        assert!(!is_legal_rook_move(m, board))
    }

    #[test]
    fn is_legal_rook_move_is_orthog_and_legal() {
        let board = BoardRep::from_fen("rnbqk2r/pppp1ppp/4pn2/8/2PP4/P1P5/4PPPP/R1BQKBNR b KQkq - 0 1".into());
        let m = Move::new(index_from_coords("a1"), index_from_coords("b1"), 0b0, PieceType::Rook, false, 0);

        assert!(is_legal_rook_move(m, board))
    }

    #[test]
    fn is_legal_rook_move_scenario_0() {
        let board = BoardRep::from_fen("rn3b1r/ppp2kpp/3p4/8/P3P1nP/4q3/4K3/4R1NR w - - 6 21".into());
        let m = Move::new(index_from_coords("e1"), index_from_coords("e3"), MF_CAPTURE, PieceType::Rook, false, 0);

        assert!(!is_legal_rook_move(m, board))
    }
}