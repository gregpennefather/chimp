use crate::{
    board::board_rep::BoardRep,
    move_generation::pawn::{ep_leads_to_orthogonal_check, generate_pawn_moves},
    shared::board_utils::index_from_coords,
};

#[test]
pub fn pawn_moves_scenario_0() {
    let board = BoardRep::from_fen(
        "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1".into(),
    );
    let moves = generate_pawn_moves(board, 9, board.black_occupancy, None, None);
    assert_eq!(moves.len(), 3);
}

#[test]
pub fn pawn_moves_scenario_1() {
    let board = BoardRep::from_fen(
        "r3k2r/p1ppqpb1/bn2pnp1/1N1PN3/1p2P3/5Q2/PPPBBPpP/R3K2R b KQkq - 0 2".into(),
    );
    let moves = generate_pawn_moves(board, 9, board.white_occupancy, None, None);
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