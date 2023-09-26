use crate::{board::{board_rep::BoardRep, bitboard::Bitboard, attack_and_defend_lookups::AttackAndDefendTable}, r#move::Move, shared::{board_utils::index_from_coords, piece_type::PieceType, constants::MF_QUEEN_CASTLING}, move_generation::{king::{is_legal_king_move, is_legal_castling}}};

use super::generate_king_moves;

#[test]
fn is_legal_king_move_scenario_0() {
    let board = BoardRep::from_fen("r1b1k1nr/pp1p1pp1/n2Bp3/7p/1PpPP3/2N5/PP3PPP/R3K1NR b KQkq - 3 10".into());

    let m = Move::new(index_from_coords("e8"), index_from_coords("f8"), 0b0, PieceType::King, true, 0);

    assert!(!is_legal_king_move(m, board, &board.get_black_king_analysis()))
}

#[test]
fn is_legal_castling_scenario_0() {
    let board = BoardRep::from_fen("rn3bnr/ppp3k1/1q1p4/1b4Pp/3PPB2/P4N2/2P3P1/R2NK2R w KQ h6 0 16".into());

    let m = Move::new(index_from_coords("e1"), index_from_coords("c1"), MF_QUEEN_CASTLING, PieceType::King, false, 0);

    assert!(!is_legal_king_move(m, board, &board.get_black_king_analysis()))
}

#[test]
fn generate_moves_illegal_move_scenario_0() {
    let board = BoardRep::from_fen("8/P1k5/1p1pr1p1/4p2p/2r4P/5P1B/3R3K/8 b - - 0 3".into());

    let king_analysis = board.get_black_king_analysis();

    let mut ad_table = AttackAndDefendTable::new();

    let moves = generate_king_moves(
        board.black_king_position,
        board.white_occupancy,
        board.occupancy,
        &king_analysis,
        board.black_turn,
        board.black_king_side_castling,
        board.black_queen_side_castling,
        board,
        &mut ad_table,
    );

    let illegal_move = Move::new(board.black_king_position, index_from_coords("b8"), 0b0, PieceType::King, true, 0);

    assert!(!moves.contains(&illegal_move))
}