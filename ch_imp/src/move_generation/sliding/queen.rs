use crate::{
    board::{
        bitboard::Bitboard,
        board_rep::BoardRep,
        king_position_analysis::{ThreatRaycastCollision, ThreatSource}, attack_and_defend_lookups::AttackAndDefendTable,
    },
    move_generation::moveboard_to_moves,
    r#move::Move,
    shared::piece_type::PieceType,
    MOVE_DATA,
};

pub(crate) fn generate_queen_moves(
    index: u8,
    board: BoardRep,
    ad_table: &mut AttackAndDefendTable,
    opponent_occupancy: u64,
    occupancy: u64,
    king_threat: Option<ThreatSource>,
    pin: Option<ThreatRaycastCollision>,
) -> Vec<Move> {
    let mut moveboard = match pin {
        Some(p) => p.threat_ray_mask | (1 << p.from),
        None => {
            MOVE_DATA
                .magic_bitboard_table
                .get_bishop_attacks(index as usize, board.occupancy.into())
                | MOVE_DATA
                    .magic_bitboard_table
                    .get_rook_attacks(index as usize, board.occupancy.into())
        }
    };

    if king_threat != None {
        let threat = king_threat.unwrap();
        moveboard &= threat.threat_ray_mask | (1 << threat.from);
    }

    moveboard_to_moves(
        index,
        PieceType::Queen,
        moveboard,
        opponent_occupancy,
        occupancy,
        board,
        ad_table
    )
}

pub fn is_legal_queen_move(m: Move, board: BoardRep) -> bool {
    let legal = MOVE_DATA.is_slide_legal(m.from(), m.to());
    if legal.0 || legal.1 {
        board.occupancy & MOVE_DATA.get_slide_inbetween(m.from(), m.to()) == 0
    } else {
        false
    }
}

#[cfg(test)]
mod test {
    use crate::{
        board::board_rep::BoardRep,
        move_generation::sliding::queen::is_legal_queen_move,
        r#move::Move,
        shared::{board_utils::index_from_coords, piece_type::PieceType},
    };

    #[test]
    fn is_legal_queen_move_not_diagonal_or_orthog() {
        let board = BoardRep::from_fen(
            "r1bqkbnr/pp1npppp/2p5/8/3PN3/8/PPP2PPP/R1BQKBNR w KQkq - 0 1".into(),
        );
        let m = Move::new(
            index_from_coords("d1"),
            index_from_coords("c3"),
            0b0,
            PieceType::Queen,
            false,
            0,
        );

        assert!(!is_legal_queen_move(m, board))
    }

    #[test]
    fn is_legal_queen_move_is_diagonal_and_legal() {
        let board = BoardRep::from_fen(
            "r1bqkbnr/pp1npppp/2p5/8/3PN3/8/PPP2PPP/R1BQKBNR w KQkq - 0 1".into(),
        );
        let m = Move::new(
            index_from_coords("d1"),
            index_from_coords("f3"),
            0b0,
            PieceType::Queen,
            true,
            0,
        );

        assert!(is_legal_queen_move(m, board))
    }
}
