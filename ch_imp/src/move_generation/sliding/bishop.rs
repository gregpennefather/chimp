use crate::{
    board::{
        attack_and_defend_lookups::AttackAndDefendTable,
        bitboard::Bitboard,
        board_rep::BoardRep,
        king_position_analysis::{ThreatRaycastCollision, ThreatSource},
    },
    move_generation::moveboard_to_moves,
    r#move::Move,
    shared::piece_type::PieceType,
    MOVE_DATA,
};

pub fn generate_bishop_moves(
    index: u8,
    board: BoardRep,
    ad_table: &mut AttackAndDefendTable,
    opponent_occupancy: u64,
    occupancy: u64,
    king_threat: Option<ThreatSource>,
    pin: Option<ThreatRaycastCollision>,
    reveal_attack: Option<ThreatRaycastCollision>,
    phase: i16,
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
        ad_table,
        reveal_attack,
        phase,
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
    use crate::{
        board::{attack_and_defend_lookups::AttackAndDefendTable, board_rep::BoardRep},
        move_generation::sliding::bishop::{generate_bishop_moves, is_legal_bishop_move},
        r#move::Move,
        shared::{board_utils::index_from_coords, constants::MF_CAPTURE, piece_type::PieceType},
    };

    #[test]
    fn is_legal_bishop_move_not_diagonal() {
        let board = BoardRep::from_fen("8/2p5/3p2b1/KP5r/1R2Pp1k/8/6P1/8 b - e3 0 1".into());
        let m = Move::new(
            index_from_coords("g6"),
            index_from_coords("g2"),
            MF_CAPTURE,
            PieceType::Bishop,
            true,
            0,
            0,
        );

        assert!(!is_legal_bishop_move(m, board))
    }

    #[test]
    fn is_legal_bishop_move_is_diagonal_and_legal() {
        let board = BoardRep::from_fen("8/2p5/3p2b1/KP5r/1R2Pp1k/8/6P1/8 b - e3 0 1".into());
        let m = Move::new(
            index_from_coords("g6"),
            index_from_coords("e4"),
            MF_CAPTURE,
            PieceType::Bishop,
            true,
            0,
            0,
        );

        assert!(is_legal_bishop_move(m, board))
    }

    #[test]
    fn generate_bishop_moves_should_have_two_moves_with_see_of_zero_as_they_are_unthreatened() {
        let board = BoardRep::from_fen(
            "1r1n1rk1/3qp2p/P2p2p1/1p6/5pP1/1p3P1P/5PB1/R1QR2K1 w - - 0 1".into(),
        );

        let mut ad_table = AttackAndDefendTable::new();

        let moves = generate_bishop_moves(
            index_from_coords("g2"),
            board,
            &mut ad_table,
            board.get_opponent_occupancy(),
            board.occupancy,
            None,
            None,
            None,
            0,
        );

        assert_eq!(moves[0].see(), 0, "{}", moves[0]);
        assert_eq!(moves[1].see(), 0, "{}", moves[0]);
    }
}
