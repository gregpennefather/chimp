use chimp::board::{piece::get_piece_char, state::*, apply_move::standard_notation_to_move};
use colored::Colorize;

fn main() {
    misc_tests();
    from_fen_test_cases();
    apply_move_test_cases();
}

fn misc_tests() {
    let test_count = 3;
    let mut success_count = 0;

    // to_fen
    // initial_board
    let initial_input_fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1".into();
    let initial_output_fen = BoardState::from_fen(&initial_input_fen).to_fen();
    if !initial_input_fen.eq(&initial_output_fen) {
        print_test_result("to_fen initial board".to_string(), "FENs do not match".into(), false);
        println!("'{}' vs '{}'", initial_output_fen.red(), initial_input_fen.yellow())
    } else {
        success_count += 1;
    }
    // Dualing kings
    let dualing_kings_input_fen = "7k/7p/8/8/8/8/P7/K7 b - - 0 1".into();
    let dualing_kings_output_fen = BoardState::from_fen(&dualing_kings_input_fen).to_fen();
    if !dualing_kings_input_fen.eq(&dualing_kings_output_fen) {
        print_test_result("to_fen dualing kings".to_string(), "FENs do not match".into(), false);
        println!("'{}' vs '{}'", dualing_kings_output_fen.red(), dualing_kings_input_fen.yellow())
    } else {
        success_count += 1;
    }
    // Non-mirrored board
    let non_mirrored_fen = "8/8/8/8/8/8/PPPPPPPP/kNBQKBNR b K - 0 1".into();
    let non_mirrored_fen_output = BoardState::from_fen(&non_mirrored_fen).to_fen();
    if !non_mirrored_fen.eq(&non_mirrored_fen_output) {
        print_test_result("to_fen Non-mirrored board".to_string(), "FENs do not match".into(), false);
        println!("'{}' vs '{}'", non_mirrored_fen_output.red(), non_mirrored_fen.yellow())
    } else {
        success_count += 1;
    }


    print_test_group_result("misc_tests".into(), success_count, test_count);
}

fn from_fen_test_cases() {
    let test_count = 9;
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
        0b00011111,
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
        0b00000000,
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
        0b10011110,
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

fn test_fen_flags(fen: &String, desc: String, expected_flags: u8) -> bool {
    let board_state = BoardState::from_fen(fen);
    let r = board_state.flags == expected_flags;
    if !r {
        print_test_result(desc, "FLags do not match expected".into(), false);
        let p_r = format!("{:b}", board_state.flags);
        let p_e = format!("{:b}", expected_flags);
        println!("{} vs {}", p_r.red(), p_e.yellow());
    }
    r
}

fn apply_move_test_cases() {
    let test_count = 9;
    let mut success_count = 0;

    if test_move(
        "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1".into(),
        "e2e3".into(),
        "rnbqkbnr/pppppppp/8/8/8/4P3/PPPP1PPP/RNBQKBNR b KQkq - 0 1".into()
    ) {
        success_count += 1;
    }

    if test_move(
        "8/8/8/8/4KP2/8/r3k3/8 w - - 0 1".into(),
        "f4f5".into(),
        "8/8/8/5P2/4K3/8/r3k3/8 b - - 0 1".into()
    ) {
        success_count += 1;
    }

    if test_move(
        "8/8/8/8/4KP2/8/r3k3/8 b - - 0 1".into(),
        "a2a6".into(),
        "8/8/r7/8/4KP2/8/4k3/8 w - - 1 2".into()
    ) {
        success_count += 1;
    }

    if test_move(
        "8/8/8/5k2/5p2/1P6/1K6/8 w - - 0 1".into(),
        "b3b4".into(),
        "8/8/8/5k2/1P3p2/8/1K6/8 b - - 0 1".into()
    ) {
        success_count += 1;
    }

    if test_move(
        "8/8/4k3/4p3/2KP4/8/8/8 b - - 0 1".into(),
        "e5xd4".into(),
        "8/8/4k3/8/2Kp4/8/8/8 w - - 0 2".into()
    ) {
        success_count += 1;
    }

    if test_move(
        "8/8/4k3/4p3/2KP4/8/8/8 b - - 5 1".into(),
        "e5xd4".into(),
        "8/8/4k3/8/2Kp4/8/8/8 w - - 0 2".into()
    ) {
        success_count += 1;
    }

    if test_move(
        "rnbqkb1r/ppp1pp1p/5np1/3p4/2PP4/2N5/PP2PPPP/R1BQKBNR w KQkq - 0 5".into(),
        "c4xd5".into(),
        "rnbqkb1r/ppp1pp1p/5np1/3P4/3P4/2N5/PP2PPPP/R1BQKBNR b KQkq - 0 5".into()
    ) {
        success_count += 1;
    }

    if test_move(
        "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1".into(),
        "e7e5".into(),
        "rnbqkbnr/pppp1ppp/8/4p3/4P3/8/PPPP1PPP/RNBQKBNR w KQkq e6 0 2".into()
    ) {
        success_count += 1;
    }

    if test_move(
        "rnbqkb1r/ppp1pp1p/3p1np1/8/3PPP2/2N5/PPP3PP/R1BQKBNR b KQkq - 0 1".into(),
        "f6g4".into(),
        "rnbqkb1r/ppp1pp1p/3p2p1/8/3PPPn1/2N5/PPP3PP/R1BQKBNR w KQkq - 1 2".into()
    ) {
        success_count += 1;
    }

    print_test_group_result("apply_move_test_cases".into(), success_count, test_count);
}

fn test_move(init_fen: String, m: String, exp_fen: String) -> bool {
    let board = BoardState::from_fen(&init_fen);
    let move_code = standard_notation_to_move(&m);
    let after_move = board.apply_move(move_code);
    let after_move_fen = after_move.to_fen();
    let r= exp_fen.eq(&after_move_fen);
    if !r {
        print_test_result("Test Move".into(), "Move output does not match expected".into(), false);
        println!("{} vs {}", after_move_fen.red(), exp_fen.yellow());
        println!("{}", board_to_string(after_move.bitboard, after_move.pieces).red())
    }
    r
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

fn board_to_string(bitboard: u64, pieces: u128) -> String {
    let mut r: String = "".to_string();

    let mut index = 63;
    let mut piece_index = (bitboard.count_ones() - 1).try_into().unwrap();
    while index >= 0 {
        let occ = (bitboard >> index) & 1 == 1;
        if occ {
            r += &get_board_square_char(pieces, piece_index).to_string();
            piece_index -= 1;
        } else {
            r += &'0'.to_string();
        }
        index -= 1;
        if (index + 1) % 8 == 0 {
            r += "\n".into();
        }
    }

    r
}

fn get_board_square_char(pieces: u128, index: i32) -> char {
    let piece: u8 = (pieces >> (index * 4) & 0b1111).try_into().unwrap();
    return get_piece_char(piece);
}

fn bitboard_to_string(bitboard: u64) -> String {
    let mut r: String = "".to_string();

    r += format!("{:#010b}\n", get_bitboard_file(bitboard, 7)).as_str();
    r += format!("{:#010b}\n", get_bitboard_file(bitboard, 6)).as_str();
    r += format!("{:#010b}\n", get_bitboard_file(bitboard, 5)).as_str();
    r += format!("{:#010b}\n", get_bitboard_file(bitboard, 4)).as_str();
    r += format!("{:#010b}\n", get_bitboard_file(bitboard, 3)).as_str();
    r += format!("{:#010b}\n", get_bitboard_file(bitboard, 2)).as_str();
    r += format!("{:#010b}\n", get_bitboard_file(bitboard, 1)).as_str();
    r += format!("{:#010b}\n", get_bitboard_file(bitboard, 0)).as_str();

    r
}

fn get_bitboard_file(bitboard: u64, file: u8) -> u8 {
    let r: u8 = (bitboard >> (file * 8) & 255) as u8;
    r
}
