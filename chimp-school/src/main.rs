use std::{cmp::Ordering, collections::HashMap, time::Instant};

use chimp::{
    board::{
        board_metrics::BoardMetrics,
        board_utils::{board_to_string, rank_and_file_to_index},
        piece_utils::get_piece_code,
        state::BoardState, r#move::MoveFunctions,
    },
    shared::bitboard_to_string,
};
use colored::Colorize;

fn main() {
    let quiet = true;

    misc_tests();
    from_fen_test_cases();
    apply_move_test_cases();
    apply_move_deep_test_cases(quiet);
    metric_generation_check_test_cases(quiet);
    move_generation_test_cases();
    move_chain_test_cases(quiet);
    perft(quiet);
    kiwipete_perft(quiet);
    perft_position_3(quiet);
    perft_position_4(quiet);
    perft_position_5(quiet);
    perft_position_6(quiet);

    // Clearly we have a apply_move issue that we need to start testing for
    //test_move_generation_count("r3k2N/p1ppq3/bn2pnpb/3P4/1p2P3/2N2Q1p/PPPBBPPP/R3K2R b KQq - 0 2".into(), 44);
    /*node_debug_test(
        "r3k2r/p1ppqpb1/bn2pnp1/3PN3/Pp2P3/2N2Q1p/1PPBBPPP/R3K2R b KQkq a3 0 1".into(),
        vec![44],
        false,
    )*/
}

fn misc_tests() {
    let test_count = 3;
    let mut success_count = 0;

    // to_fen
    // initial_board
    let initial_input_fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1".into();
    let initial_output_fen = BoardState::from_fen(&initial_input_fen).to_fen();
    if !initial_input_fen.eq(&initial_output_fen) {
        print_test_result(
            "to_fen initial board".to_string(),
            "FENs do not match".into(),
            false,
        );
        println!(
            "'{}' vs '{}'",
            initial_output_fen.red(),
            initial_input_fen.yellow()
        )
    } else {
        success_count += 1;
    }
    // Dualing kings
    let dualing_kings_input_fen = "7k/7p/8/8/8/8/P7/K7 b - - 0 1".into();
    let dualing_kings_output_fen = BoardState::from_fen(&dualing_kings_input_fen).to_fen();
    if !dualing_kings_input_fen.eq(&dualing_kings_output_fen) {
        print_test_result(
            "to_fen dualing kings".to_string(),
            "FENs do not match".into(),
            false,
        );
        println!(
            "'{}' vs '{}'",
            dualing_kings_output_fen.red(),
            dualing_kings_input_fen.yellow()
        )
    } else {
        success_count += 1;
    }
    // Non-mirrored board
    let non_mirrored_fen = "8/8/8/8/8/8/PPPPPPPP/kNBQKBNR b K - 0 1".into();
    let non_mirrored_fen_output = BoardState::from_fen(&non_mirrored_fen).to_fen();
    if !non_mirrored_fen.eq(&non_mirrored_fen_output) {
        print_test_result(
            "to_fen Non-mirrored board".to_string(),
            "FENs do not match".into(),
            false,
        );
        println!(
            "'{}' vs '{}'",
            non_mirrored_fen_output.red(),
            non_mirrored_fen.yellow()
        )
    } else {
        success_count += 1;
    }

    print_test_group_result("misc_tests".into(), success_count, test_count);
}

fn from_fen_test_cases() {
    let test_count = 10;
    let mut success_count = 0;
    // Initial Board State
    let initial_board_fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1".into();
    if test_fen_bitboard(
        &initial_board_fen,
        "Initial Board State bitboard".into(),
        18446462598732906495,
    ) {
        success_count += 1;
    }
    if test_fen_pieces(
        &initial_board_fen,
        "Initial Board State pieces".into(),
        269490179295853796843097322727436280612,
    ) {
        success_count += 1;
    }
    if test_fen_flags(
        &initial_board_fen,
        "Initial Board State flags".into(),
        0b11110,
        u8::MAX,
    ) {
        success_count += 1;
    }
    if test_fen_king_positions(
        &initial_board_fen,
        "Initial Board State flags".into(),
        rank_and_file_to_index(4, 0),
        rank_and_file_to_index(4, 7),
    ) {
        success_count += 1;
    }
    // Dualing Kings
    let dualing_kings_fen = "7k/7p/8/8/8/8/P7/K b - - 0 1".into();
    if test_fen_bitboard(
        &dualing_kings_fen,
        "Dualing Kings opposite corners bitboard".into(),
        72339069014671488,
    ) {
        success_count += 1;
    }
    if test_fen_pieces(
        &dualing_kings_fen,
        "Dualing Kings opposite corners pieces".into(),
        0b1110100100010110,
    ) {
        success_count += 1;
    }
    if test_fen_flags(
        &dualing_kings_fen,
        "Dualing Kings opposite corners flags".into(),
        0b1,
        u8::MAX,
    ) {
        success_count += 1;
    }
    // White E pawn opening
    let white_e_pawn_opening_fen =
        "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e4 0 2".into();
    if test_fen_bitboard(
        &white_e_pawn_opening_fen,
        "White E pawn opening bitboard".into(),
        18446462598867122175,
    ) {
        success_count += 1;
    }
    if test_fen_pieces(
        &white_e_pawn_opening_fen,
        "White E pawn opening pieces".into(),
        269490179295853796843097322727436280612,
    ) {
        success_count += 1;
    }
    if test_fen_flags(
        &white_e_pawn_opening_fen,
        "White E pawn opening flags".into(),
        0b11111,
        4,
    ) {
        success_count += 1;
    }

    print_test_group_result("from_fen_test_cases".into(), success_count, test_count);
}

fn test_fen_bitboard(fen: &String, desc: String, expected_bitboard: u64) -> bool {
    let board_state = BoardState::from_fen(fen);
    let r = board_state.bitboard == expected_bitboard;
    if !r {
        print_test_result(desc, "Bitboard does not match expected".into(), false);
        let bb_r = format!("{:b}", board_state.bitboard);
        let bb_e = format!("{:b}", expected_bitboard);
        println!("{} vs {}", bb_r.red(), bb_e.yellow());
        println!("{}", bitboard_to_string(board_state.bitboard).red());
        println!("{}", bitboard_to_string(expected_bitboard).yellow());
    }
    r
}

fn test_fen_pieces(fen: &String, desc: String, expected_pieces: u128) -> bool {
    let board_state = BoardState::from_fen(fen);
    let r = board_state.pieces == expected_pieces;
    if !r {
        print_test_result(desc, "Pieceboard does not match expected".into(), false);
        let p_r = format!("{:b}", board_state.pieces);
        let p_e = format!("{:b}", expected_pieces);
        println!("{} vs {}", p_r.red(), p_e.yellow());
        println!(
            "{}",
            board_to_string(board_state.bitboard, board_state.pieces).red()
        );
    }
    r
}

fn test_fen_flags(fen: &String, desc: String, expected_flags: u8, expected_ep: u8) -> bool {
    let board_state = BoardState::from_fen(fen);
    let r = if board_state.flags != expected_flags {
        print_test_result(desc, "Flags do not match expected".into(), false);
        let p_r = format!("{:b}", board_state.flags);
        let p_e = format!("{:b}", expected_flags);
        println!("{} vs {}", p_r.red(), p_e.yellow());
        return false;
    };
    if board_state.ep_rank != expected_ep {
        print_test_result(desc, "EP does not match expected".into(), false);
        let p_r = format!("{:b}", board_state.ep_rank);
        let p_e = format!("{:b}", expected_ep);
        println!("{} vs {}", p_r.red(), p_e.yellow());
        return false;
    }
    true
}

fn test_fen_king_positions(
    fen: &String,
    desc: String,
    expected_white_king_position_index: u8,
    expected_black_king_position_index: u8,
) -> bool {
    let board_state = BoardState::from_fen(fen);
    let mut flag = true;
    if board_state.white_king_index != expected_white_king_position_index {
        print_test_result(
            desc.clone(),
            "White King positions do not match expected".into(),
            false,
        );
        println!(
            "white: {} vs {}",
            board_state.white_king_index.to_string().red(),
            expected_white_king_position_index.to_string().yellow()
        );
        flag = false;
    };
    if board_state.black_king_index != expected_black_king_position_index {
        print_test_result(
            desc,
            "Black King positions do not match expected".into(),
            false,
        );
        println!(
            "black: {} vs {}",
            board_state.black_king_index.to_string().red(),
            expected_black_king_position_index.to_string().yellow()
        );
        flag = false;
    }
    flag
}

fn apply_move_test_cases() {
    let test_count = 10;
    let mut success_count = 0;

    if test_move(
        "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1".into(),
        "e2e3".into(),
        "rnbqkbnr/pppppppp/8/8/8/4P3/PPPP1PPP/RNBQKBNR b KQkq - 0 1".into(),
    ) {
        success_count += 1;
    }

    if test_move(
        "8/8/8/8/4KP2/8/r3k3/8 w - - 0 1".into(),
        "f4f5".into(),
        "8/8/8/5P2/4K3/8/r3k3/8 b - - 0 1".into(),
    ) {
        success_count += 1;
    }

    if test_move(
        "8/8/8/8/4KP2/8/r3k3/8 b - - 0 1".into(),
        "a2a6".into(),
        "8/8/r7/8/4KP2/8/4k3/8 w - - 1 2".into(),
    ) {
        success_count += 1;
    }

    if test_move(
        "8/8/8/5k2/5p2/1P6/1K6/8 w - - 0 1".into(),
        "b3b4".into(),
        "8/8/8/5k2/1P3p2/8/1K6/8 b - - 0 1".into(),
    ) {
        success_count += 1;
    }

    if test_move(
        "8/8/4k3/4p3/2KP4/8/8/8 b - - 0 1".into(),
        "e5d4".into(),
        "8/8/4k3/8/2Kp4/8/8/8 w - - 0 2".into(),
    ) {
        success_count += 1;
    }

    if test_move(
        "8/8/4k3/4p3/2KP4/8/8/8 b - - 5 1".into(),
        "e5d4".into(),
        "8/8/4k3/8/2Kp4/8/8/8 w - - 0 2".into(),
    ) {
        success_count += 1;
    }

    if test_move(
        "rnbqkb1r/ppp1pp1p/5np1/3p4/2PP4/2N5/PP2PPPP/R1BQKBNR w KQkq - 0 5".into(),
        "c4d5".into(),
        "rnbqkb1r/ppp1pp1p/5np1/3P4/3P4/2N5/PP2PPPP/R1BQKBNR b KQkq - 0 5".into(),
    ) {
        success_count += 1;
    }

    if test_move(
        "rnbqkb1r/ppp1pp1p/3p1np1/8/3PPP2/2N5/PPP3PP/R1BQKBNR b KQkq - 0 1".into(),
        "f6g4".into(),
        "rnbqkb1r/ppp1pp1p/3p2p1/8/3PPPn1/2N5/PPP3PP/R1BQKBNR w KQkq - 1 2".into(),
    ) {
        success_count += 1;
    }

    if test_move(
        "rnbqkbnr/pppppppp/8/8/8/P7/1PPPPPPP/RNBQKBNR b KQkq - 0 1".into(),
        "a7a6".into(),
        "rnbqkbnr/1ppppppp/p7/8/8/P7/1PPPPPPP/RNBQKBNR w KQkq - 0 2".into(),
    ) {
        success_count += 1;
    }

    if test_move(
        "r3k2r/p1ppqpb1/bn2pnp1/3PN3/Pp2P3/2N2Q1p/1PPBBPPP/R3K2R b KQkq a3 0 1".into(),
        "b4a3".into(),
        "r3k2r/p1ppqpb1/bn2pnp1/3PN3/4P3/p1N2Q1p/1PPBBPPP/R3K2R w KQkq - 0 2".into(),
    ) {
        success_count += 1;
    }

    print_test_group_result("apply_move_test_cases".into(), success_count, test_count);
}

fn apply_move_deep_test_cases(quiet: bool) {
    let mut tests: Vec<(String, String, String, String)> = Vec::new();
    let mut success_count = 0;

    tests.push((
        "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1".into(),
        "e5f7".into(),
        "r3k2r/p1ppqNb1/bn2pnp1/3P4/1p2P3/2N2Q1p/PPPBBPPP/R3K2R b KQkq - 0 1".into(),
        "Knight takes f7 pawn".into(),
    ));

    tests.push((
        "r3k2r/p1ppqpb1/bn2pnp1/3PN3/Pp2P3/2N2Q1p/1PPBBPPP/R3K2R b KQkq a3 0 1".into(),
        "b4a3".into(),
        "r3k2r/p1ppqpb1/bn2pnp1/3PN3/4P3/p1N2Q1p/1PPBBPPP/R3K2R w KQkq - 0 2".into(),
        "Black pawn EP capture into A rank".into(),
    ));

    tests.push((
        "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1".into(),
        "e1c1".into(),
        "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/2KR3R b kq - 1 1".into(),
        "White queenside castle".into(),
    ));

    tests.push((
        "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/2KR3R b kq - 1 1".into(),
        "c7c5".into(),
        "r3k2r/p2pqpb1/bn2pnp1/2pPN3/1p2P3/2N2Q1p/PPPBBPPP/2KR3R w kq c6 0 2".into(),
        "Black double pawn push after white queenside castle".into(),
    ));

    tests.push((
        "r3k2r/p2pqpb1/bn2pnp1/2pPN3/1p2P3/2N2Q1p/PPPBBPPP/2KR3R w kq c6 0 2".into(),
        "h1g1".into(),
        "r3k2r/p2pqpb1/bn2pnp1/2pPN3/1p2P3/2N2Q1p/PPPBBPPP/2KR2R1 b kq - 1 2".into(),
        "White castle move following black C double pawn push".into(),
    ));

    tests.push((
        "r3k2r/p2pqpb1/bn2pnp1/2pPN3/1p2P3/2N2Q1p/PPPBBPPP/R3K1R1 w Qkq c6 0 2".into(),
        "e1c1".into(),
        "r3k2r/p2pqpb1/bn2pnp1/2pPN3/1p2P3/2N2Q1p/PPPBBPPP/2KR2R1 b kq - 1 2".into(),
        "White castling queenside after black C double pawn push".into(),
    ));

    tests.push((
        "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1".into(),
        "h1g1".into(),
        "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K1R1 b Qkq - 1 1".into(),
        "Black rook move to clear king side castling".into(),
    ));

    tests.push((
        "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K1R1 b Qkq - 3 2".into(),
        "h8h7".into(),
        "r3k3/p1ppqpbr/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K1R1 w Qq - 4 3".into(),
        "Black rook move to clear king side castling".into(),
    ));

    tests.push((
        "r3k3/p1ppqpbr/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K1R1 w Qq - 4 3".into(),
        "g1h1".into(),
        "r3k3/p1ppqpbr/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R b Qq - 5 3".into(),
        "Black rook move to clear king side castling".into(),
    ));

    tests.push((
        "k7/8/8/8/8/K7/6p1/7R b - - 0 1".into(),
        "g2g1n".into(),
        "k7/8/8/8/8/K7/8/6nR w - - 0 2".into(),
        "Black Pawn Promotion".into(),
    ));

    tests.push((
        "r3k2r/p1ppqN2/bn2pnpb/3P4/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 1 2".into(),
        "f7h8".into(),
        "r3k2N/p1ppq3/bn2pnpb/3P4/1p2P3/2N2Q1p/PPPBBPPP/R3K2R b KQq - 0 2".into(),
        "White takes Knight Black could previously castle with".into(),
    ));

    tests.push((
        "r3k2r/p1p1qpb1/bn2pnp1/2NP4/1p2P3/2N2Q2/PPPBBPpP/R3K2R b KQkq - 1 2".into(),
        "g2h1q".into(),
        "r3k2r/p1p1qpb1/bn2pnp1/2NP4/1p2P3/2N2Q2/PPPBBP1P/R3K2q w Qkq - 0 3".into(),
        "Black takes whites kingside rook, promoting to queen and clearing black kingside castling"
            .into(),
    ));

    tests.push((
        "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1".into(),
        "e7e5".into(),
        "rnbqkbnr/pppp1ppp/8/4p3/4P3/8/PPPP1PPP/RNBQKBNR w KQkq e6 0 2".into(),
        "Black double pawn push in same file as white".into(),
    ));

    tests.push((
        "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1".into(),
        "a2a4".into(),
        "r3k2r/p1ppqpb1/bn2pnp1/3PN3/Pp2P3/2N2Q1p/1PPBBPPP/R3K2R b KQkq a3 0 1".into(),
        "White double pawn push opening self up to EP".into(),
    ));

    tests.push((
        "rnbqkbnr/p1p1pppp/3p4/1p6/4P3/2N2N2/PPPP1PPP/R1BQKB1R b KQkq - 1 3".into(),
        "f7f5".into(),
        "rnbqkbnr/p1p1p1pp/3p4/1p3p2/4P3/2N2N2/PPPP1PPP/R1BQKB1R w KQkq f6 0 4".into(),
        "Black pawn push from test game".into(),
    ));

    for test in &tests {
        let board = BoardState::from_fen(&test.0);
        let move_code = board.move_from_string(&test.1);
        let after_move_board_state = board.apply_move(move_code);
        let expected_board_state = BoardState::from_fen(&test.2);
        if !quiet {
            println!("Begin {}", test.3.yellow());
        }
        let r = board_deep_equal(after_move_board_state, expected_board_state);
        if !r || !quiet {
            print_test_result(
                format!("Apply Move {}", test.3),
                "After move board state does not match".into(),
                r,
            );
        }
        if r {
            success_count += 1;
        }
    }
    print_test_group_result(
        "apply_move_deep_test_cases".into(),
        success_count,
        tests.len().try_into().unwrap(),
    );
}

fn move_chain_test_cases(quiet:bool) {
    let mut tests: Vec<(String, Vec<String>, String)> = Vec::new();
    let mut success_count = 0;

    tests.push((
        "White pawn capture from test game".into(),
        vec!["e2e4".into(),"b7b5".into(),"g1f3".into(),"d7d6".into(),"b1c3".into(),"f7f5".into(),"e4f5".into()],
        "rnbqkbnr/p1p1p1pp/3p4/1p3P2/8/2N2N2/PPPP1PPP/R1BQKB1R b KQkq - 0 4".into(),
    ));


    for test in &tests {
        let mut board = BoardState::default();
        for m in test.1.iter() {
            let move_code = board.move_from_string(&m);
            board = board.apply_move(move_code);
        }
        let expected_board_state = BoardState::from_fen(&test.2);
        if !quiet {
            println!("Begin {}", test.0.yellow());
        }
        let r = board_deep_equal(board, expected_board_state);
        if !r || !quiet {
            print_test_result(
                format!("Move chain {}", test.0),
                "After move chain board state does not match".into(),
                r,
            );
        }
        if r {
            success_count += 1;
        }
    }
    print_test_group_result(
        "apply_move_deep_test_cases".into(),
        success_count,
        tests.len().try_into().unwrap(),
    );
}

fn board_deep_equal(a: BoardState, b: BoardState) -> bool {
    let mut flag = true;

    if a.bitboard != b.bitboard {
        println!("Bitboard do not match:");
        println!("{}", bitboard_to_string(a.bitboard).red());
        println!("{}", bitboard_to_string(b.bitboard).yellow());
        flag = false;
    }

    if a.pieces != b.pieces {
        println!("---Pieces do not match---");
        for i in 0..32 {
            let aw = get_piece_code(&a.pieces, i);
            let bw = get_piece_code(&b.pieces, i);
            if (aw != bw) {
                println!(
                    "{i}: {} vs {}",
                    aw.to_string().red(),
                    bw.to_string().yellow()
                );
            }
        }
        println!("--- End ---");
        flag = false;
    }

    if a.white_bitboard != b.white_bitboard {
        println!("white_bitboards do not match:");
        println!("{}", bitboard_to_string(a.white_bitboard).red());
        println!("{}", bitboard_to_string(b.white_bitboard).yellow());
        flag = false;
    }

    if a.black_bitboard != b.black_bitboard {
        println!("black_bitboards do not match:");
        println!("{}", bitboard_to_string(a.black_bitboard).red());
        println!("{}", bitboard_to_string(b.black_bitboard).yellow());
        flag = false;
    }

    if a.white_king_index != b.white_king_index {
        println!(
            "white_king_index do not match {} vs {}",
            a.white_king_index.to_string().red(),
            b.white_king_index.to_string().yellow()
        );
        flag = false;
    }

    if a.black_king_index != b.black_king_index {
        println!(
            "black_king_index do not match {} vs {}",
            a.black_king_index.to_string().red(),
            b.black_king_index.to_string().yellow()
        );
        flag = false;
    }

    if a.flags != b.flags {
        println!(
            "flags do not match {} vs {} aka {:b} vs {:b}",
            a.flags.to_string().red(),
            b.flags.to_string().yellow(),
            a.flags,
            b.flags
        );
        flag = false;
    }

    if a.ep_rank != b.ep_rank {
        println!(
            "ep_rank do not match {} vs {}",
            a.ep_rank.to_string().red(),
            b.ep_rank.to_string().yellow()
        );
        flag = false;
    }

    if a.half_moves != b.half_moves {
        println!(
            "half_moves do not match {} vs {}",
            a.half_moves.to_string().red(),
            b.half_moves.to_string().yellow()
        );
        flag = false;
    }

    if a.full_moves != b.full_moves {
        println!(
            "full_moves do not match {} vs {}",
            a.full_moves.to_string().red(),
            b.full_moves.to_string().yellow()
        );
        flag = false;
    }

    flag
}

fn move_generation_test_cases() {
    let test_count = 21;
    let mut success_count = 0;

    // 1
    if test_move_generation_count(
        "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1".into(),
        20,
    ) {
        success_count += 1;
    }

    // 2
    if test_move_generation_count("7k/7p/8/8/8/7N/PPPPPPPP/RNBQKB1R b KQ - 1 1".into(), 4) {
        success_count += 1;
    }

    // 3
    if test_move_generation_count(
        "rnbqkbnr/pppppppp/8/8/8/7N/PPPPPPPP/RNBQKB1R b KQkq - 1 1".into(),
        20,
    ) {
        success_count += 1;
    }

    // 4
    if test_move_generation_count(
        "rnbqkbnr/pppppppp/8/8/8/P7/1PPPPPPP/RNBQKBNR b KQkq - 0 1".into(),
        20,
    ) {
        success_count += 1;
    }

    // 5
    if test_move_generation_count(
        "rnbqkbnr/1ppppppp/p7/8/8/P7/1PPPPPPP/RNBQKBNR w KQkq - 0 1".into(),
        19,
    ) {
        success_count += 1;
    }

    // 6
    if test_move_generation_count(
        "rnbqkbnr/1ppppppp/p7/8/8/P7/1PPPPPPP/RNBQKBNR w KQkq - 0 1".into(),
        19,
    ) {
        success_count += 1;
    }

    // 7
    if test_move_generation_count(
        "rnbqkbnr/pppppppp/8/8/P7/8/1PPPPPPP/RNBQKBNR b KQkq a3 0 1".into(),
        20,
    ) {
        success_count += 1;
    }

    // 8
    if test_move_generation_count(
        "rnbqkbnr/1ppppppp/8/p7/P7/8/1PPPPPPP/RNBQKBNR w KQkq a6 0 1".into(),
        20,
    ) {
        success_count += 1;
    }

    // 9
    if test_move_generation_count(
        "rnbqkb1r/pppppppp/7n/8/P7/8/1PPPPPPP/RNBQKBNR w KQkq - 0 1".into(),
        21,
    ) {
        success_count += 1;
    }

    // 10
    if test_move_generation_count(
        "r1bqkbnr/pppppppp/n7/8/1P6/8/P1PPPPPP/RNBQKBNR w KQkq - 0 1".into(),
        21,
    ) {
        success_count += 1;
    }

    // 11
    if test_move_generation_count(
        "rnbqkb1r/pppppppp/5n2/8/8/3P4/PPP1PPPP/RNBQKBNR w KQkq - 0 1".into(),
        27,
    ) {
        success_count += 1;
    }

    // 12
    if test_move_generation_count(
        "rnbqkbnr/pppppp1p/8/6p1/8/3P4/PPP1PPPP/RNBQKBNR w KQkq g6 0 1".into(),
        26,
    ) {
        success_count += 1;
    }

    // 13
    if test_move_generation_count(
        "rnbqkbnr/ppp1pppp/8/1B1p4/4P3/8/PPPP1PPP/RNBQK1NR b KQkq - 0 1".into(),
        5,
    ) {
        success_count += 1;
    }

    // 14
    if test_move_generation_count("1k6/1P6/2K5/8/8/8/8/8 b - - 0 1".into(), 1) {
        success_count += 1;
    };

    // 15
    if test_move_generation_count(
        "rnbqkbnr/pppp1ppp/8/4p1B1/3P4/8/PPP1PPPP/RN1QKBNR b KQkq - 0 1".into(),
        28,
    ) {
        success_count += 1;
    };

    // 16
    if test_move_generation_count(
        "r3k2r/p1ppqpb1/bnN1pnp1/3P4/1p2P3/2N2Q1p/PPPBBPPP/R3K2R b KQkq - 1 1".into(),
        41,
    ) {
        success_count += 1;
    };

    // 17
    if test_move_generation_count(
        "r3k2r/p1pNqpb1/bn2pnp1/3P4/1p2P3/2N2Q1p/PPPBBPPP/R3K2R b KQkq - 0 1".into(),
        45,
    ) {
        success_count += 1;
    };

    // 18
    if test_move_generation_count(
        "r3k2r/p1ppqpb1/bn2pnp1/3PN3/4P3/p1N2Q1p/1PPBBPPP/R3K2R w KQkq - 0 2".into(),
        51,
    ) {
        success_count += 1;
    };

    // 19
    if test_move_generation_count(
        "r3k2r/p1ppqNb1/bn2pnp1/3P4/1p2P3/2N2Q2/PPPBBPpP/2KR3R b kq - 0 2".into(),
        53,
    ) {
        success_count += 1;
    };

    // 20
    if test_move_generation_count(
        "r3k2r/p1ppqpb1/bn2pnp1/3PN3/Pp2P3/2N2Q1p/1PPBBPPP/R3K2R b KQkq a3 0 1".into(),
        44,
    ) {
        success_count += 1;
    };

    // 21
    if test_move_generation_count(
        "r2q1rk1/pP1p2pp/Q4n2/bb2p3/Npp5/1B3NBn/pPPP1PPP/R3K2R w KQ - 0 2".into(),
        43,
    ) {
        success_count += 1;
    };

    print_test_group_result(
        "move_generation_test_cases".into(),
        success_count,
        test_count,
    );
}

fn test_move_generation_count(fen: String, expected_count: usize) -> bool {
    let board = BoardState::from_fen(&fen);
    let metrics = board.generate_metrics();
    let pl_moves = board.generate_psudolegals();
    let moves = board.generate_legal_moves(&pl_moves, &metrics);
    let r = moves.len() == expected_count;
    if !r {
        print_test_result(
            format!("Test Move Generation ({fen})"),
            format!(
                "Move count of {} does not match expected {}",
                (moves.len().to_string()).red(),
                (expected_count.to_string()).yellow()
            ),
            false,
        );
        let mut vec = Vec::new();
        for m in moves {
            vec.push(m.0.uci());
        }
        vec.sort();
        for m_s in vec {
            println!("{}", m_s.red());
        }
    }
    r
}

fn test_move(init_fen: String, m: String, exp_fen: String) -> bool {
    let board = BoardState::from_fen(&init_fen);
    let move_code = board.move_from_string(&m);
    let after_move = board.apply_move(move_code);
    let after_move_fen = after_move.to_fen();
    let r = exp_fen.eq(&after_move_fen);
    if !r {
        print_test_result(
            "Test Move".into(),
            "Move output does not match expected".into(),
            false,
        );
        println!("{} vs {}", after_move_fen.red(), exp_fen.yellow());
        println!(
            "{}",
            board_to_string(after_move.bitboard, after_move.pieces).red()
        )
    }
    r
}

fn metric_generation_check_test_cases(quiet: bool) {
    let mut tests: Vec<(String, String, bool, bool)> = Vec::new();
    let mut success_count = 0;

    tests.push((
        "Bishop threatens Black King".into(),
        "rnbqkbnr/ppp1pppp/8/1B1p4/4P3/8/PPPP1PPP/RNBQK1NR b KQkq - 0 1".into(),
        false,
        true
    ));

    for test in &tests {
        let board = BoardState::from_fen(&test.1);
        let metrics = board.generate_metrics();
        let r = metrics.white_in_check == test.2 && metrics.black_in_check == test.3;
        if !r || !quiet {
            print_test_result(
                format!("Metrics Check Tests {}", test.0),
                format!("Expected: {},{}. Result: {},{}", test.2.to_string().yellow(), test.3.to_string().yellow(), metrics.white_in_check.to_string().red(), metrics.black_in_check.to_string().red()),
                r,
            );
        }
        if r {
            success_count += 1;
        }
    }
    print_test_group_result(
        "apply_move_deep_test_cases".into(),
        success_count,
        tests.len().try_into().unwrap(),
    );
}

fn print_test_result(name: String, result: String, success: bool) {
    match success {
        true => println!("{name}: {} - {result}", "SUCCESS".green()),
        _ => println!("{name}: {} - {result}", "FAILURE".red()),
    }
}

fn print_test_group_result(name: String, success_count: i32, test_count: i32) {
    let result = format!("{}/{}", success_count, test_count);
    if success_count == test_count {
        println!("GROUP: {name} {}", result.green());
    } else {
        println!("GROUP: {name} {}", result.red());
    }
}

#[derive(Default, Clone)]
struct MovePath(u16, Vec<(BoardState, BoardMetrics)>);

fn node_debug_test(fen: String, counts: Vec<usize>, quiet: bool) {
    let mut depth = 0;
    let desired_depth = counts.len();
    let mut node_count = 0;
    let mut move_node_count: HashMap<u16, usize> = HashMap::new();


    let initial_board_state = BoardState::from_fen(&fen);
    let mut paths: Vec<MovePath> = Vec::new();
    // Todo: refactor to all happen in the loop
    let metrics = initial_board_state.generate_metrics();
    let pl_moves = initial_board_state.generate_psudolegals();
    let legal_moves = initial_board_state.generate_legal_moves(&pl_moves, &metrics);
    for m in legal_moves {
        node_count += 1;
        move_node_count.entry(m.0).or_insert(1);
        let new_board_states = vec![(m.1, m.2)];
        paths.push(MovePath(m.0, new_board_states))
    }

    print_test_result(
        format!("Perft {}", depth + 1),
        format!("Nodes {}/{}", node_count, counts[0]).into(),
        node_count == counts[0],
    );
    if node_count != counts[0] {
        let mut keys = Vec::new();
        for &key in move_node_count.keys() {
            keys.push(key)
        }
        keys.sort_by(|a, b| sort_uci(*a, *b));
        for key in keys {
            println!(
                "{}: {}",
                key.uci(),
                move_node_count.get(&key).unwrap()
            )
        }
        return;
    }
    depth += 1;

    while depth < desired_depth {
        let start = Instant::now();
        node_count = 0;
        move_node_count = HashMap::new();
        let mut new_path_entries = Vec::new();
        for path in paths {
            let mut new_board_states = Vec::new();
            for board_state_with_metrics in path.1.iter() {
                // if depth == 3 {
                //     println!("path: {}, board: {}, bitboard {}", get_move_uci(path.0), board_state.to_fen(), board_state.bitboard);
                // }
                let board_state = board_state_with_metrics.0;
                let metrics = board_state_with_metrics.1.clone();
                let pl_moves = board_state.generate_psudolegals();

                let legal_moves = board_state.generate_legal_moves(&pl_moves, &metrics);
                for m in legal_moves {
                    new_board_states.push((m.1, m.2));
                    move_node_count
                        .entry(path.0)
                        .and_modify(|v| *v += 1)
                        .or_insert(1);
                    node_count += 1;
                }
            }
            new_path_entries.push(MovePath(path.0, new_board_states));
        }
        paths = new_path_entries;
        let duration = start.elapsed();

        let success = node_count == counts[depth];

        print_test_result(
            format!("Perft {}", depth + 1),
            format!(
                "Nodes {}/{} - ({:?})",
                node_count,
                counts.get(depth).unwrap(),
                duration
            )
            .into(),
            success,
        );

        if !quiet && !success {
            let mut keys = Vec::new();
            for &key in move_node_count.keys() {
                keys.push(key)
            }
            keys.sort_by(|a, b| sort_uci(*a, *b));
            for key in keys {
                println!(
                    "{}: {}",
                    key.uci(),
                    move_node_count.get(&key).unwrap()
                )
            }
            return;
        }
        depth += 1;
    }
}

fn perft(quiet: bool) {
    println!("--- Perft ---");
    node_debug_test(
        "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1".into(),
        vec![20, 400, 8902, 197281],
        quiet,
    );
}

fn kiwipete_perft(quiet: bool) {
    println!("--- Kiwipete Perft ---");
    node_debug_test(
        "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1".into(),
        vec![48, 2039, 97862, 4085603],
        quiet,
    );
}

fn perft_position_3(quiet: bool) {
    println!("--- Perft Position 3 ---");
    node_debug_test(
        "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1".into(),
        vec![14, 191, 2812, 43238, 674624],
        quiet,
    );
}

fn perft_position_4(quiet: bool) {
    println!("--- Perft Position 4 ---");
    node_debug_test(
        "r2q1rk1/pP1p2pp/Q4n2/bbp1p3/Np6/1B3NBn/pPPP1PPP/R3K2R b KQ - 0 1".into(),
        vec![6, 264, 9467, 422333],
        quiet,
    );
}

fn perft_position_5(quiet: bool) {
    println!("--- Perft Position 5 ---");
    node_debug_test(
        "rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8".into(),
        vec![44, 1486, 62379, 2103487],
        quiet,
    );
}

fn perft_position_6(quiet: bool) {
    println!("--- Perft Position 6 ---");
    node_debug_test(
        "r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10".into(),
        vec![46, 2079, 89890, 3894594],
        quiet,
    );
}

fn sort_uci(a: u16, b: u16) -> Ordering {
    let a_from_index: u8 = (a >> 10).try_into().unwrap();
    let b_from_index: u8 = (b >> 10).try_into().unwrap();

    if a_from_index > b_from_index {
        return Ordering::Less;
    } else if a_from_index < b_from_index {
        return Ordering::Greater;
    }

    let a_to_index: u8 = (a >> 4 & 0b111111).try_into().unwrap();
    let b_to_index: u8 = (b >> 4 & 0b111111).try_into().unwrap();
    if a_to_index > b_to_index {
        return Ordering::Greater;
    } else if a_to_index < b_to_index {
        return Ordering::Less;
    }
    return Ordering::Equal;
}
