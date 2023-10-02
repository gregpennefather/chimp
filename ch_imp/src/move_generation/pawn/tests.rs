use crate::{
    board::{board_rep::BoardRep, attack_and_defend_lookups::AttackAndDefendTable, bitboard::Bitboard},
    move_generation::pawn::{ep_leads_to_orthogonal_check, generate_pawn_moves, legal_move::is_legal_pawn_move, get_pawn_threat_positions},
    shared::{board_utils::index_from_coords, constants::MF_ROOK_CAPTURE_PROMOTION, piece_type::PieceType}, r#move::Move,
};

#[test]
pub fn pawn_moves_scenario_0() {
    let board = BoardRep::from_fen(
        "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1".into(),
    );
    let moves = generate_pawn_moves(board, &mut AttackAndDefendTable::new(), 9, board.black_occupancy, None, None, None, 0);
    assert_eq!(moves.len(), 3);
}

#[test]
pub fn pawn_moves_scenario_1() {
    let board = BoardRep::from_fen(
        "r3k2r/p1ppqpb1/bn2pnp1/1N1PN3/1p2P3/5Q2/PPPBBPpP/R3K2R b KQkq - 0 2".into(),
    );
    let moves = generate_pawn_moves(board, &mut AttackAndDefendTable::new(), 9, board.white_occupancy, None, None, None, 0);
    assert_eq!(moves.len(), 8);
}

#[test]
pub fn ep_leads_to_orthogonal_check_right_true() {
    let board = BoardRep::from_fen("8/2p5/3p4/KP5r/1R3pPk/8/4P3/8 b - g3 0 1".into());
    let leads_to_check = ep_leads_to_orthogonal_check(
        board,
        index_from_coords("f4"),
        index_from_coords("f4") - 1,
        board.white_occupancy,
    );
    assert!(leads_to_check);
}

#[test]
pub fn ep_leads_to_orthogonal_check_left_true() {
    let board = BoardRep::from_fen("8/2p5/3p4/KP5r/1R2Pp1k/8/6P1/8 b - e3 0 1".into());
    let leads_to_check = ep_leads_to_orthogonal_check(
        board,
        index_from_coords("f4"),
        index_from_coords("f4") + 1,
        board.white_occupancy,
    );
    assert!(leads_to_check);
}

#[test]
fn is_legal_capture_promotion_scenario_0() {
    let board = BoardRep::from_fen("3n2k1/P4r1p/3qp1p1/1r1p4/1p3pP1/1Q3P1P/R4P2/2R2BK1 w - - 1 1".into());
    let m = Move::new(index_from_coords("a7"), index_from_coords("b8"), MF_ROOK_CAPTURE_PROMOTION, PieceType::Pawn, false, 3, 0);
    assert!(!is_legal_pawn_move(m, board));
}

#[test]
fn get_pawn_threat_positions_a_file() {
    assert_eq!(get_pawn_threat_positions(index_from_coords("a3"), false), 1 << index_from_coords("b4"));
    assert_eq!(get_pawn_threat_positions(index_from_coords("a6"), true), 1 << index_from_coords("b5"));
}

#[test]
fn get_pawn_threat_positions_h_file() {
    assert_eq!(get_pawn_threat_positions(index_from_coords("h6"), false), 1 << index_from_coords("g7"));
    assert_eq!(get_pawn_threat_positions(index_from_coords("h4"), true), 1 << index_from_coords("g3"));
}