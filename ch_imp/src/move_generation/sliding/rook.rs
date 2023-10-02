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

pub fn generate_rook_moves(
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
        ad_table,
        reveal_attack,
        phase,
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
    use crate::{
        board::{attack_and_defend_lookups::AttackAndDefendTable, board_rep::BoardRep},
        move_generation::sliding::rook::{generate_rook_moves, is_legal_rook_move},
        r#move::Move,
        shared::{board_utils::index_from_coords, constants::MF_CAPTURE, piece_type::PieceType},
    };

    #[test]
    fn generate_rook_moves_ignore_opponent_when_calculating_see_when_reveal_check_move() {
        let board = BoardRep::from_fen("8/3p3k/3pb3/8/4R3/8/2B5/1K6 w - - 0 1".into());
        let reveal_attack = board.get_black_king_analysis().pins[0];

        let mut moves = generate_rook_moves(
            index_from_coords("e4"),
            board,
            &mut AttackAndDefendTable::new(),
            board.black_occupancy,
            board.occupancy,
            None,
            None,
            Some(reveal_attack),0
        );

        moves.sort();

        println!("{moves:?}");

        assert_eq!(moves[0].to(), index_from_coords("e6"));
        assert_eq!(moves[0].see(), 3);
    }

    #[test]
    fn is_legal_rook_move_not_orthog() {
        let board = BoardRep::from_fen(
            "rnbqk2r/pppp1ppp/4pn2/8/2PP4/P1P5/4PPPP/R1BQKBNR b KQkq - 0 1".into(),
        );
        let m = Move::new(
            index_from_coords("a1"),
            index_from_coords("b2"),
            0b0,
            PieceType::Rook,
            false,
            0,
            0,
        );

        assert!(!is_legal_rook_move(m, board))
    }

    #[test]
    fn is_legal_rook_move_is_orthog_and_legal() {
        let board = BoardRep::from_fen(
            "rnbqk2r/pppp1ppp/4pn2/8/2PP4/P1P5/4PPPP/R1BQKBNR b KQkq - 0 1".into(),
        );
        let m = Move::new(
            index_from_coords("a1"),
            index_from_coords("b1"),
            0b0,
            PieceType::Rook,
            false,
            0,
            0,
        );

        assert!(is_legal_rook_move(m, board))
    }

    #[test]
    fn is_legal_rook_move_scenario_0() {
        let board =
            BoardRep::from_fen("rn3b1r/ppp2kpp/3p4/8/P3P1nP/4q3/4K3/4R1NR w - - 6 21".into());
        let m = Move::new(
            index_from_coords("e1"),
            index_from_coords("e3"),
            MF_CAPTURE,
            PieceType::Rook,
            false,
            0,
            0,
        );

        assert!(!is_legal_rook_move(m, board))
    }
}
