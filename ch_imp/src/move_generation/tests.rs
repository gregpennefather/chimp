use rand::seq::index;

use crate::{
    board::king_position_analysis::analyze_king_position, shared::{board_utils::index_from_coords, piece_type::PieceType},
};

use super::*;

#[test]
pub fn startpos_move_generation() {
    let board = BoardRep::default();
    let white_king_analysis = analyze_king_position(
        board.white_king_position,
        false,
        board.occupancy,
        board.white_occupancy,
        board.black_occupancy,
        board.pawn_bitboard,
        board.knight_bitboard,
        board.bishop_bitboard,
        board.rook_bitboard,
        board.queen_bitboard,
        board.black_turn,
    );

    let black_king_analysis = analyze_king_position(
        board.black_king_position,
        true,
        board.occupancy,
        board.black_occupancy,
        board.white_occupancy,
        board.pawn_bitboard,
        board.knight_bitboard,
        board.bishop_bitboard,
        board.rook_bitboard,
        board.queen_bitboard,
        !board.black_turn,
    );
    let moves = generate_moves(&white_king_analysis, &black_king_analysis, board);
    assert_eq!(moves.0.len(), 20);
}

#[test]
pub fn king_double_checked() {
    let board =
        BoardRep::from_fen("rnbqk1nr/pppp1pNp/2Pb4/8/1B6/4Q3/PP1PPPPP/RN2KB1R b KQkq - 0 1".into());
    let white_king_analysis = analyze_king_position(
        board.white_king_position,
        false,
        board.occupancy,
        board.white_occupancy,
        board.black_occupancy,
        board.pawn_bitboard,
        board.knight_bitboard,
        board.bishop_bitboard,
        board.rook_bitboard,
        board.queen_bitboard,
        board.black_turn,
    );

    let black_king_analysis = analyze_king_position(
        board.black_king_position,
        true,
        board.occupancy,
        board.black_occupancy,
        board.white_occupancy,
        board.pawn_bitboard,
        board.knight_bitboard,
        board.bishop_bitboard,
        board.rook_bitboard,
        board.queen_bitboard,
        !board.black_turn,
    );
    let moves = generate_moves(&black_king_analysis, &white_king_analysis, board);
    assert!(moves.0.len() <= 2);
}

#[test]
pub fn move_generation_capture_the_threat_with_knight_or_move_the_king() {
    let board = BoardRep::from_fen(
        "r3k2r/p1Np1pb1/b3pnpq/1n1PN3/1p2P3/5Q1p/PPPBBPPP/R3K2R b KQkq - 0 2".into(),
    );
    let white_king_analysis = analyze_king_position(
        board.white_king_position,
        false,
        board.occupancy,
        board.white_occupancy,
        board.black_occupancy,
        board.pawn_bitboard,
        board.knight_bitboard,
        board.bishop_bitboard,
        board.rook_bitboard,
        board.queen_bitboard,
        board.black_turn,
    );

    let black_king_analysis = analyze_king_position(
        board.black_king_position,
        true,
        board.occupancy,
        board.black_occupancy,
        board.white_occupancy,
        board.pawn_bitboard,
        board.knight_bitboard,
        board.bishop_bitboard,
        board.rook_bitboard,
        board.queen_bitboard,
        !board.black_turn,
    );
    let moves = generate_moves(&black_king_analysis, &white_king_analysis, board);

    assert_eq!(moves.0.len(), 4);
}

#[test]
pub fn move_generation_capture_the_threat_with_bishop_or_move_the_king() {
    let board = BoardRep::from_fen(
        "r3kb1r/p1Npqp2/1b2pnp1/n2PN3/1p2P3/5Q1p/PPPBBPPP/R3K2R b KQkq - 0 2".into(),
    );
    let white_king_analysis = analyze_king_position(
        board.white_king_position,
        false,
        board.occupancy,
        board.white_occupancy,
        board.black_occupancy,
        board.pawn_bitboard,
        board.knight_bitboard,
        board.bishop_bitboard,
        board.rook_bitboard,
        board.queen_bitboard,
        board.black_turn,
    );

    let black_king_analysis = analyze_king_position(
        board.black_king_position,
        true,
        board.occupancy,
        board.black_occupancy,
        board.white_occupancy,
        board.pawn_bitboard,
        board.knight_bitboard,
        board.bishop_bitboard,
        board.rook_bitboard,
        board.queen_bitboard,
        !board.black_turn,
    );
    let moves = generate_moves(&black_king_analysis, &white_king_analysis, board);

    assert_eq!(moves.0.len(), 2);
}

#[test]
pub fn move_generation_capture_the_threat_with_rook_to_avoid_smother() {
    let board = BoardRep::from_fen(
        "3nkb1r/p1Npqp2/4pnp1/1b1PN3/1p2P3/5Q1p/PPrBBPPP/R3K2R b KQk - 0 2".into(),
    );
    let white_king_analysis = analyze_king_position(
        board.white_king_position,
        false,
        board.occupancy,
        board.white_occupancy,
        board.black_occupancy,
        board.pawn_bitboard,
        board.knight_bitboard,
        board.bishop_bitboard,
        board.rook_bitboard,
        board.queen_bitboard,
        board.black_turn,
    );

    let black_king_analysis = analyze_king_position(
        board.black_king_position,
        true,
        board.occupancy,
        board.black_occupancy,
        board.white_occupancy,
        board.pawn_bitboard,
        board.knight_bitboard,
        board.bishop_bitboard,
        board.rook_bitboard,
        board.queen_bitboard,
        !board.black_turn,
    );
    let moves = generate_moves(&black_king_analysis, &white_king_analysis, board);

    assert_eq!(moves.0.len(), 1);
}

#[test]
pub fn move_generation_block_threat_with_bishop() {
    let board = BoardRep::from_fen(
        "r3kb2/pp3ppp/2n2n1r/1Bpp4/4b3/2N1PP2/PPPP2PP/R1B1q1KR w q - 0 11".into(),
    );
    let white_king_analysis = analyze_king_position(
        board.white_king_position,
        false,
        board.occupancy,
        board.white_occupancy,
        board.black_occupancy,
        board.pawn_bitboard,
        board.knight_bitboard,
        board.bishop_bitboard,
        board.rook_bitboard,
        board.queen_bitboard,
        board.black_turn,
    );

    let black_king_analysis = analyze_king_position(
        board.black_king_position,
        true,
        board.occupancy,
        board.black_occupancy,
        board.white_occupancy,
        board.pawn_bitboard,
        board.knight_bitboard,
        board.bishop_bitboard,
        board.rook_bitboard,
        board.queen_bitboard,
        !board.black_turn,
    );
    let moves = generate_moves(&white_king_analysis, &black_king_analysis, board);

    assert_eq!(moves.0.len(), 1);
}

#[test]
pub fn move_generation_capture_the_threat_with_pawn_to_avoid_smother() {
    let board = BoardRep::from_fen(
        "3nkb1r/p1pbnp2/3Np1p1/q3N3/1p2P3/2q2Q1p/PPPBBPPP/R3K2R b KQk - 0 2".into(),
    );
    let white_king_analysis = analyze_king_position(
        board.white_king_position,
        false,
        board.occupancy,
        board.white_occupancy,
        board.black_occupancy,
        board.pawn_bitboard,
        board.knight_bitboard,
        board.bishop_bitboard,
        board.rook_bitboard,
        board.queen_bitboard,
        board.black_turn,
    );

    let black_king_analysis = analyze_king_position(
        board.black_king_position,
        true,
        board.occupancy,
        board.black_occupancy,
        board.white_occupancy,
        board.pawn_bitboard,
        board.knight_bitboard,
        board.bishop_bitboard,
        board.rook_bitboard,
        board.queen_bitboard,
        !board.black_turn,
    );
    let moves = generate_moves(&black_king_analysis, &white_king_analysis, board);

    assert_eq!(moves.0.len(), 1);
}

#[test]
pub fn move_generation_block_with_pawn_or_move_king() {
    let board = BoardRep::from_fen(
        "r3kb2/pp3ppp/2n2n1r/1Bpp4/3qb3/2N2P2/PPPPP1PP/R1B3K1 w q - 0 11".into(),
    );
    let white_king_analysis = analyze_king_position(
        board.white_king_position,
        false,
        board.occupancy,
        board.white_occupancy,
        board.black_occupancy,
        board.pawn_bitboard,
        board.knight_bitboard,
        board.bishop_bitboard,
        board.rook_bitboard,
        board.queen_bitboard,
        board.black_turn,
    );

    let black_king_analysis = analyze_king_position(
        board.black_king_position,
        true,
        board.occupancy,
        board.black_occupancy,
        board.white_occupancy,
        board.pawn_bitboard,
        board.knight_bitboard,
        board.bishop_bitboard,
        board.rook_bitboard,
        board.queen_bitboard,
        !board.black_turn,
    );
    let moves = generate_moves(&white_king_analysis, &black_king_analysis, board);
    assert_eq!(moves.0.len(), 3);
}

#[test]
pub fn move_generation_block_or_capture_with_bishop() {
    let board = BoardRep::from_fen(
        "r3k2R/p1ppqpb1/bn2pn2/3PN1p1/1p2P3/2N5/PPPBBPPP/R3K3 b Qq - 0 2".into(),
    );
    let white_king_analysis = board.get_white_king_analysis();

    let black_king_analysis = board.get_black_king_analysis();

    let moves = generate_moves(&black_king_analysis, &white_king_analysis, board);

    assert_eq!(moves.0.len(), 4);
}

#[test]
pub fn pawn_move_gen_threatened_block_with_double_pawn_push() {
    let board = BoardRep::from_fen(
        "r2q1rk1/pP1p2pp/Q4n2/bbp1p3/Np6/1B3NBn/pPPP1PPP/R3K2R b KQ - 0 1".into(),
    );

    let moves = generate_index_moves(
        board,
        index_from_coords("d7"),
        &board.get_black_king_analysis(),
    );
    println!("{:?}", moves);
    assert_eq!(moves.len(), 1);
}

#[test]
pub fn pawn_move_gen_threatened_take_ep() {
    let board = BoardRep::from_fen("8/8/8/1Ppp3r/1KR2p1k/8/4P1P1/8 w - c6 0 3".into());

    let moves = generate_index_moves(
        board,
        index_from_coords("b5"),
        &board.get_white_king_analysis(),
    );
    println!("{:?}", moves);
    assert_eq!(moves.len(), 1);
}

#[test]
pub fn pawn_move_gen_threatened_take_threat() {
    let board = BoardRep::from_fen("8/2p5/3p4/KPR3kr/5p2/8/4P1P1/8 b - - 3 2".into());

    let moves = generate_index_moves(
        board,
        index_from_coords("d6"),
        &board.get_black_king_analysis(),
    );
    println!("{:?}", moves);
    assert_eq!(moves.len(), 2);
}

#[test]
pub fn move_generation_scenario_pawn_wrap_around_king_threat() {
    let board = BoardRep::from_fen(
        "r4rk1/p1ppqpb1/bn2pnp1/P2PN3/1p2P3/2N2Q1p/1PPBBPPP/R3K2R b KQ - 0 2".into(),
    );

    let white_king_analysis = analyze_king_position(
        board.white_king_position,
        false,
        board.occupancy,
        board.white_occupancy,
        board.black_occupancy,
        board.pawn_bitboard,
        board.knight_bitboard,
        board.bishop_bitboard,
        board.rook_bitboard,
        board.queen_bitboard,
        board.black_turn,
    );

    let black_king_analysis = analyze_king_position(
        board.black_king_position,
        true,
        board.occupancy,
        board.black_occupancy,
        board.white_occupancy,
        board.pawn_bitboard,
        board.knight_bitboard,
        board.bishop_bitboard,
        board.rook_bitboard,
        board.queen_bitboard,
        !board.black_turn,
    );

    let moves = generate_moves(&black_king_analysis, &white_king_analysis, board);
    assert!(moves.0.contains(&Move::new(
        index_from_coords("g8"),
        index_from_coords("h7"),
        0b0,
        piece_type::PieceType::King,
        true,
        0
    )));
}

#[test]
pub fn generate_moves_queen_check_bishop_can_capture() {
    let board = BoardRep::from_fen(
        "rnbqkbnr/pp4pp/2p1Qp2/3pp3/2B1P2P/8/PPPP1PP1/RNB1K1NR b KQkq - 1 5".into(),
    );

    let moves = generate_moves_for_board(board);
    assert_eq!(moves.len(), 4);
    assert!(moves.contains(&Move::new(
        index_from_coords("c8"),
        index_from_coords("e6"),
        MF_CAPTURE,
        PieceType::Bishop,
        true,
        calculate_see(PieceType::Bishop, PieceType::Queen)
    )));
}

#[test]
pub fn get_pawn_threatboard_no_wrap_around() {
    let r = get_pawn_threatboard(index_from_coords("a5"), false);
    println!("{}", r.to_board_format());
    assert_eq!(r, 1 << index_from_coords("b6"));
}

#[test]
pub fn get_pawn_threatboard_white_rank_7() {
    let r = get_pawn_threatboard(index_from_coords("b7"), false);
    println!("{}", r.to_board_format());
    assert_eq!(r, 1 << index_from_coords("c8") | 1 << index_from_coords("a8"));
}

#[test]
pub fn generate_pawn_moves_when_pinned_bishop() {
    let board =
        BoardRep::from_fen("rnbqkbnr/pppppp2/8/1B4pp/4P3/2N5/PPPP1PPP/R1BQK1NR b KQkq -".into());

    let pawn_moves = generate_index_moves(
        board,
        index_from_coords("d7"),
        &board.get_black_king_analysis(),
    );
    assert_eq!(pawn_moves.len(), 0);
}

#[test]
pub fn generate_pawn_moves_when_pinned_bishop_that_can_be_captured() {
    let board = BoardRep::from_fen(
        "rnbqkbnr/pppppp2/2B5/6pp/4P3/2N5/PPPP1PPP/R1BQK1NR b KQkq - 0 1".into(),
    );

    let pawn_moves = generate_index_moves(
        board,
        index_from_coords("d7"),
        &board.get_black_king_analysis(),
    );
    assert_eq!(pawn_moves.len(), 1);
    assert_eq!(pawn_moves[0].to(), index_from_coords("c6"));
}

#[test]
pub fn generate_pawn_moves_when_pinned_on_e_file() {
    let board = BoardRep::from_fen(
        "rnbqkbnr/pppppp2/2B5/6pp/4P3/2N5/PPPP1PPP/R1BQK1NR b KQkq - 0 1".into(),
    );

    let pawn_moves = generate_index_moves(
        board,
        index_from_coords("e7"),
        &board.get_black_king_analysis(),
    );
    assert_eq!(pawn_moves.len(), 2);
}

#[test]
pub fn generate_pawn_moves_pinned_and_threatened_where_threat_captureable_no_legal_moves() {
    let board = BoardRep::from_fen(
        "rnbqkbnr/1ppp1p1p/p3Q3/1B2p1p1/4P3/8/PPPP1PPP/RNB1K1NR b KQkq - 0 1".into(),
    );

    let pawn_moves = generate_index_moves(
        board,
        index_from_coords("d7"),
        &board.get_black_king_analysis(),
    );
    assert_eq!(pawn_moves.len(), 0);
}

#[test]
pub fn generate_pawn_moves_pinned_and_right_ep_capture_available_disallow_the_capture() {
    let board = BoardRep::from_fen("8/2p5/3p4/KP5r/1R3pPk/8/4P3/8 b - g3".into());
    let pawn_moves = generate_index_moves(
        board,
        index_from_coords("f4"),
        &board.get_black_king_analysis(),
    );
    println!("{pawn_moves:?}");
    assert_eq!(pawn_moves.len(), 1);
}

#[test]
pub fn generate_pawn_moves_pinned_and_left_ep_capture_available_disallow_the_capture() {
    let board = BoardRep::from_fen("8/2p5/3p4/KP5r/1R2Pp1k/8/6P1/8 b - e3 0 1".into());
    let pawn_moves = generate_index_moves(
        board,
        index_from_coords("f4"),
        &board.get_black_king_analysis(),
    );
    println!("{pawn_moves:?}");
    assert_eq!(pawn_moves.len(), 1);
}

#[test]
pub fn generate_knight_moves_when_king_threatened_can_not_block_or_capture() {
    let board =
        BoardRep::from_fen("rnbq1bnr/pppppkpp/8/5p1Q/4P2P/8/PPPP1PP1/RNB1KBNR b KQ -".into());

    let knight_moves = generate_index_moves(
        board,
        index_from_coords("g8"),
        &board.get_black_king_analysis(),
    );
    assert_eq!(knight_moves.len(), 0);
}

#[test]
pub fn generate_knight_moves_when_king_threatened_can_block_or_capture() {
    let board =
        BoardRep::from_fen("rnbq1b1r/pppppkpp/8/5p1Q/4Pn1P/8/PPPP1PP1/RNB1KBNR b KQ - 0 1".into());

    let knight_moves = generate_index_moves(
        board,
        index_from_coords("f4"),
        &board.get_black_king_analysis(),
    );
    assert_eq!(knight_moves.len(), 2);
}

#[test]
pub fn generate_knight_moves_when_king_threatened_can_only_block() {
    let board =
        BoardRep::from_fen("rnbq1b1r/pppppkpp/8/5pn1/4P2P/1Q6/PPPP1PP1/RNB1KBNR b KQ - 0 1".into());

    let knight_moves = generate_index_moves(
        board,
        index_from_coords("g5"),
        &board.get_black_king_analysis(),
    );
    assert_eq!(knight_moves.len(), 1);
}

#[test]
pub fn generate_knight_moves_when_knight_pinned_no_moves() {
    let board = BoardRep::from_fen(
        "rnbq1b1r/pppppkpp/8/3n1p2/4P2P/1Q6/PPPP1PP1/RNB1KBNR b KQ - 0 1".into(),
    );

    let moves = generate_index_moves(
        board,
        index_from_coords("d5"),
        &board.get_black_king_analysis(),
    );
    println!("{moves:?}");
    assert_eq!(moves.len(), 0);
}

#[test]
pub fn generate_bishop_moves_when_bishop_pinned_diagonally_should_include_capture_and_full_ray() {
    let board =
        BoardRep::from_fen("rnbq3r/pppppkpp/4b3/5p2/4P2P/1Q6/PPPP1PP1/RNB1KBNR b KQ - 0 1".into());

    let moves = generate_index_moves(
        board,
        index_from_coords("e6"),
        &board.get_black_king_analysis(),
    );
    println!("{moves:?}");
    assert_eq!(moves.len(), 3);
}

#[test]
pub fn generate_bishop_moves_when_bishop_pinned_diagonally_should_include_capture_and_full_ray_including_retreat(
) {
    let board =
        BoardRep::from_fen("rnbq2kr/ppppp1pp/8/3b1p2/4P2P/1Q6/PPPP1PP1/RNB1KBNR b KQ - 0 1".into());

    let moves = generate_index_moves(
        board,
        index_from_coords("d5"),
        &board.get_black_king_analysis(),
    );
    println!("{moves:?}");
    assert_eq!(moves.len(), 4);
}

#[test]
pub fn generate_bishop_moves_when_bishop_pinned_orthogonally_should_return_0() {
    let board =
        BoardRep::from_fen("rnbq3r/pppppkpp/8/5b2/1Q2P2P/5R2/PPPP1PP1/RNB1KBN1 b Q - 0 1".into());

    let moves = generate_index_moves(
        board,
        index_from_coords("f5"),
        &board.get_black_king_analysis(),
    );
    println!("{moves:?}");
    assert_eq!(moves.len(), 0);
}

#[test]
pub fn generate_rook_moves_when_rook_pinned_orthogonally_should_include_capture_and_full_ray_including_retreat(
) {
    let board =
        BoardRep::from_fen("rnbqk3/pppp2pp/3b3p/4rp2/7P/4Q3/PPPP1PP1/RNB1KBNR b KQ - 0 1".into());

    let moves = generate_index_moves(
        board,
        index_from_coords("e5"),
        &board.get_black_king_analysis(),
    );
    println!("{moves:?}");
    assert_eq!(moves.len(), 4);
}

#[test]
pub fn generate_rook_moves_when_rook_pinned_diagonally_should_return_none() {
    let board =
        BoardRep::from_fen("rnbqk3/pppp1rpp/3b3p/5p1Q/7P/8/PPPP1PP1/RNB1KBNR b KQ - 0 1".into());

    let moves = generate_index_moves(
        board,
        index_from_coords("f7"),
        &board.get_black_king_analysis(),
    );
    println!("{moves:?}");
    assert_eq!(moves.len(), 0);
}

#[test]
pub fn generate_queen_moves_when_pinned_orthogonally() {
    let board =
        BoardRep::from_fen("rnbqk3/ppppr1pp/3b3p/5p2/7P/4Q3/PPPP1PP1/RNB1KBNR w KQ - 0 1".into());

    let moves = generate_index_moves(
        board,
        index_from_coords("e3"),
        &&board.get_white_king_analysis(),
    );
    println!("{moves:?}");
    assert_eq!(moves.len(), 5);
}

#[test]
pub fn generate_queen_moves_when_pinned_and_king_threatened_by_capturable_piece() {
    let board = BoardRep::from_fen(
        "rnb1kbnr/1ppq1p1p/p3Q3/1B2p1p1/4P3/8/PPPP1PPP/RNB1K1NR b KQkq - 0 1".into(),
    );

    let moves = generate_index_moves(
        board,
        index_from_coords("d2"),
        &board.get_black_king_analysis(),
    );
    println!("{moves:?}");
    assert_eq!(moves.len(), 0);
}

#[test]
pub fn generate_queen_moves_when_pinned_orthogonally_and_forked_with_knight() {
    let board =
        BoardRep::from_fen("rnbqk3/ppppr1pp/3b3p/5p2/7P/4Q3/PPPP1Pn1/RNB1KBNR w KQ - 0 1".into());

    let moves = generate_index_moves(
        board,
        index_from_coords("e3"),
        &&board.get_white_king_analysis(),
    );
    println!("{moves:?}");
    assert_eq!(moves.len(), 0);
}

#[test]
pub fn generate_queen_moves_when_pinned_diagonally() {
    let board =
        BoardRep::from_fen("rnbqk3/ppppr1pp/3b3p/5p2/5Q1P/8/PPPP1P1K/RNB2BNR w - - 0 1".into());

    let moves = generate_index_moves(
        board,
        index_from_coords("f4"),
        &&board.get_white_king_analysis(),
    );
    println!("{moves:?}");
    assert_eq!(moves.len(), 3);
}

#[test]
pub fn generate_moves_king_in_check_no_blockers_only_retreat_to_f3() {
    let board =
        BoardRep::from_fen("1nb1kbnr/pp1rpppp/8/2p5/4PP2/8/PPPqK1PP/R4BNR w k - 0 1".into());

    let moves = generate_moves_for_board(board);
    assert_eq!(moves.len(), 1);
    assert_eq!(moves[0].to(), index_from_coords("f3"));
}
