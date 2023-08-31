use std::{
    mem,
    time::{Instant, SystemTime},
};

use ch_imp::{
    board::{bitboard::Bitboard, position::Position},
    engine::{perft::perft, san::build_san, ChimpEngine},
    evaluation::{base_eval::base_eval, early_eval},
    match_state::game_state::{GameState, MatchResultState, self},
    r#move::Move,
};
use log::{info, LevelFilter};
use log4rs::{
    append::{console::ConsoleAppender, file::FileAppender},
    config::{Appender, Config, Logger, Root},
    encode::pattern::PatternEncoder,
};
fn main() {
    //perfts();
    // let magic_table = MagicTable::new();
    // //println!("{}", Bitboard::new(magic_table.get_bishop_attacks(4, 18446462598732906495)));
    // //generate_blocker_patterns(rook_mask_generation(0));

    //let move_data = MoveData::new();
    //let game_state = GameState::new("rnbqkbnr/pppppppp/8/8/P7/8/1PPPPPPP/RNBQKBNR b KQkq - 0 1".into());
    //let moves = move_data.generate_position_moves(game_state.position, 54, true, 23, true, true, true, true);
    //println!("{moves:?}");
    //for m in &moves {
    //    println!("{}: 1",m.uci())
    //}
    // println!("{}", moves.len());
    // println!("{}", Bitboard::new(magic_table.get_bishop_attacks(index_from_coords("f4") as usize, game_state.position.occupancy.into())));
    // println!("{}",index_from_coords("f4"));

    //println!("{}", mem::size_of::<Position>());
    //println!("{}", mem::size_of::<Move>());
    // println!("{}", mem::size_of_val::<Position>(&Position::from_fen(
    //     "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1".into(),
    // )));

    //  let mut position =
    //      Position::from_fen("r3k2r/p1ppqpb1/b3pnp1/3PN3/1pn1P3/2N2Q1p/PPPB1PPP/R3KB1R w KQkq - 2 2".into());
    // // position = position.make(Move::new(9, 1, MF_QUEEN_PROMOTION, PieceType::Pawn, true));
    // // println!("{}", position.to_fen());
    // let mut c = 0;
    // for mi in 0..64 {
    //     let m = if position.black_turn { position.black_moves[mi]} else {position.white_moves[mi]};
    //     if m.from() == m.to() && m.from() == 0 {
    //         break;
    //     }
    //     let applied = position.make(m);
    //     if  applied.legal() {
    //         c+= 1;
    //         println!("{}: 1", m.uci());
    //     }
    // }
    // println!("Count: {}", c);

    // perft(
    //     "rnbqkbnr/ppp1pppp/3p4/8/4P3/8/PPPP1PPP/RNBQKBNR w KQkq - 0 2".into(),
    //     vec![30, 781, 24086],
    //   )

    // let zorb_set = ZorbSet::new();
    // println!("{zorb_set:?}");

    // let mut magics = [0; 64];
    // for i in 0..64usize {
    //     magics[i] = find_rook_magics(i as i64, ROOK_LEFT_SHIFT_BITS[i]);
    // }
    // println!("{magics:?}");

    // let mut magics = [0; 64];
    // for i in 0..64usize {
    //     magics[i] = find_bishop_magics(i as i64, BISHOP_LEFT_SHIFT_BITS[i]);
    // }
    // println!("{magics:?}");

    // let gs = GameState::default();
    // println!("{:?}", gs.move_from_uci("a2a4"));

    // let gs = GameState::new("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1".into());
    // println!("{:?}", gs.move_from_uci("e1c1"));
    // println!("{:?}", gs.move_from_uci("e1g1"));

    // let gs = GameState::new("R1brk3/3p4/4p3/6B1/2B1P1p1/6N1/1PPK1pP1/8 b - - 0 30".into());
    // let legal_moves = gs.get_legal_moves();
    // for m in legal_moves {
    //     println!("{:?}:{:?}", gs.to_san(m), m);
    // }

    // let bitboard = 0.set(27).set(28).set(35).set(36);
    // println!("{}", bitboard.to_board_format());
    // println!("{}", bitboard);

    // let game_state = GameState::new("rnbqkbnr/ppppp1p1/5pQ1/7p/8/4P3/PPPP1PPP/RNB1KBNR b KQkq - 1 3".into());
    // println!("{:?}", game_state.result_state());

    compare_evals("r6r/1pk1p3/1nb2p2/p5pp/2pPP3/2P2B1P/PP1B1KP1/R6R b - - 1 28".into(), "r6r/1pk1p3/1nb2p2/p5pp/2pPP3/2P2B1P/PP1B1KP1/4RR2 b - - 1 28".into());

    // park_table();
}

fn compare_evals(fen_1: String, fen_2: String) {
    let p1 = Position::from_fen(fen_1);
    let p2 = Position::from_fen(fen_2);

    println!("{} vs {}", p1.eval, p2.eval)
}

fn park_table() {
    let stdout = ConsoleAppender::builder().build();
    let chimp_logs = FileAppender::builder()
        .encoder(Box::new(PatternEncoder::new("{m}{n}")))
        .build(format!(
            "log/chimp_v0.0.0.5_{:?}.log",
            SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_secs()
        ))
        .unwrap();

    let config = Config::builder()
        .appender(Appender::builder().build("chimp", Box::new(chimp_logs)))
        .appender(Appender::builder().build("stdout", Box::new(stdout)))
        .build(Root::builder().appender("stdout").appender("chimp").build(LevelFilter::Info))
        .unwrap();
    let _handle = log4rs::init_config(config).unwrap();

    let start = Instant::now();
    let mut engine: ChimpEngine = ChimpEngine::new();
    let mut moves = Vec::new();
    let mut move_ucis = Vec::new();
    info!("Park Table:");
    for _i in 0..200 {
        let m = engine.go();
        move_ucis.push(m.uci());
        moves.push(m);
        engine.position(get_moves_string(&move_ucis).split_ascii_whitespace());
        if engine.current_game_state.result_state() != MatchResultState::Active {
            break;
        }
    }
    let duration = start.elapsed();
    info!("Result: {:?}", engine.current_game_state.result_state());
    info!("Runtime: {:?}", duration);
    info!("SAN: {}", build_san(moves));
    info!("Final state: {:?}", engine.current_game_state);
}

fn get_moves_string(moves: &Vec<String>) -> String {
    let mut result = "startpos moves".into();
    for m in moves {
        result = format!("{} {}", result, m);
    }
    result
}

fn perfts() {
    perft(
        "Perft".into(),
        "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1".into(),
        vec![20, 400, 8902, 197281, 4865609],
    );

    perft(
        "Kiwipete Perft".into(),
        "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1".into(),
        vec![48, 2039, 97862, 4085603],
    );

    perft(
        "Perft Position 3".into(),
        "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1".into(),
        vec![14, 191, 2812, 43238, 674624],
    );

    perft(
        "Perft Position 4".into(),
        "r2q1rk1/pP1p2pp/Q4n2/bbp1p3/Np6/1B3NBn/pPPP1PPP/R3K2R b KQ - 0 1".into(),
        vec![6, 264, 9467, 422333],
    );

    perft(
        "Perft Position 5".into(),
        "rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8".into(),
        vec![44, 1486, 62379, 2103487],
    );

    perft(
        "Perft Position 6".into(),
        "r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10".into(),
        vec![46, 2079, 89890, 3894594],
    );
}
