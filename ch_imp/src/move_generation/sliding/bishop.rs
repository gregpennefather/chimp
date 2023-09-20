use crate::{board::{board_rep::BoardRep, king_position_analysis::{ThreatSource, ThreatRaycastCollision}, bitboard::Bitboard}, r#move::Move, MOVE_DATA, move_generation::moveboard_to_moves, shared::piece_type::PieceType};

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
    if MOVE_DATA.is_slide_legal(m.from(), m.to()).1 {
        board.occupancy & MOVE_DATA.get_slide_inbetween(m.from(), m.to()) == 0
    } else {
        false
    }
}

#[cfg(test)]
mod test {
    use crate::{board::board_rep::BoardRep, shared::{board_utils::index_from_coords, constants::MF_CAPTURE, piece_type::PieceType}, r#move::Move, move_generation::sliding::bishop::is_legal_bishop_move};

    #[test]
    fn is_legal_bishop_move_not_diagonal() {
        let board = BoardRep::from_fen("8/2p5/3p2b1/KP5r/1R2Pp1k/8/6P1/8 b - e3 0 1".into());
        let m = Move::new(index_from_coords("g6"), index_from_coords("g2"), MF_CAPTURE, PieceType::Bishop, true, 0);

        assert!(!is_legal_bishop_move(m, board))
    }

    #[test]
    fn is_legal_bishop_move_is_diagonal_and_legal() {
        let board = BoardRep::from_fen("8/2p5/3p2b1/KP5r/1R2Pp1k/8/6P1/8 b - e3 0 1".into());
        let m = Move::new(index_from_coords("g6"), index_from_coords("e4"), MF_CAPTURE, PieceType::Bishop, true, 0);

        assert!(is_legal_bishop_move(m, board))
    }
}