use std::time::{Duration, Instant, SystemTime};

use ch_imp::{
    board::{position::Position, bitboard::Bitboard},
    engine::{ab_search, iterative_deepening, perft::perft, san::build_san, ChimpEngine},
    match_state::game_state::{GameState, MatchResultState, self}, shared::board_utils::get_index_from_file_and_rank,
};
use log::{info, LevelFilter};
use log4rs::{
    append::{console::ConsoleAppender, file::FileAppender},
    config::{Appender, Config, Root},
    encode::pattern::PatternEncoder,
};
fn main() {

    // let mut bb = 0;
    // for i in 0..8 {
    //     bb = bb.flip(get_index_from_file_and_rank(7-i,i));
    // }
    // println!("{}", bb.to_board_format());
    // println!("{}", bb);

    //perft("t".into(), "rnb1qk1r/pp1Pbppp/2pQ4/8/2B5/8/PPP1NnPP/RNB1K2R w KQ - 3 9".into(),vec![66]);

    //println!("{}",Position::from_fen("rnbq1bnr/ppppkppp/8/3Np3/8/8/PPPPPPPP/R1BQKBNR w KQ - 0 1".into()).board.zorb_key);

    // let game_state = GameState::new("8/2p5/3p4/KP1rR3/5p1k/8/4P1P1/8 b - - 3 2".into());
    // for m in game_state.position.moves {
    //     if m.is_empty() {
    //         break;
    //     }
    //     println!("{:?}:{:?}",m,game_state.make(m));
    // }
    // println!("{}", game_state.position.legal);
    // let new_state = game_state.make(game_state.move_from_uci("f4f3"));

    // assert_eq!(new_state, None);


    perfts();
    //let magic_table = MagicTable::new();
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

    //debug_evals("rnbqkb1r/ppp2pp1/3p1n2/4pQ1p/8/4P3/PPPPNPPP/RNB1KBR1 b Qkq - 1 5".into(), "r1b2k1r/ppp2pp1/2nq1n2/1Qbpp2p/8/2N1PN2/PPPPBPPP/R1B2K1R b - - 1 13".into());

    //debug_deepening("r1b1kb1r/ppp1pppp/2nq1n2/3p2N1/8/2N1P3/PPPP1PPP/R1BQKB1R w KQkq - 2 5".into(), 2000);

    //  debug_search(
    //      "r1bq1k1r/pppp1ppp/2n2n2/4p1N1/8/2P1PQ2/P1PP1PPP/1RB1KB1R b K - 2 7".into(),
    //      2,
    //  );

    //println!("{r:?}");

    //timed_depth_test();
    //target_depth_test();

    // park_table();
}

fn debug_evals(fen_1: String, fen_2: String) {
    let stdout = ConsoleAppender::builder().build();
    let config = Config::builder()
        .appender(Appender::builder().build("stdout", Box::new(stdout)))
        .build(Root::builder().appender("stdout").build(LevelFilter::Trace))
        .unwrap();
    let _handle = log4rs::init_config(config).unwrap();

    let p1 = Position::from_fen(fen_1);
    let p2 = Position::from_fen(fen_2);

    println!("{} vs {}", p1.eval, p2.eval)
}

fn debug_deepening(fen_1: String, ms: u64) {
    let stdout = ConsoleAppender::builder().build();
    let config = Config::builder()
        .appender(Appender::builder().build("stdout", Box::new(stdout)))
        .build(Root::builder().appender("stdout").build(LevelFilter::Info))
        .unwrap();
    let _handle = log4rs::init_config(config).unwrap();

    let timer = Instant::now();
    let game_state = GameState::new(fen_1);
    let timeout = Instant::now()
        .checked_add(Duration::from_millis(ms))
        .unwrap();
    iterative_deepening(game_state, timeout);
    let dur = timer.elapsed();
    println!("dur: {dur:?}");
}

fn debug_search(fen_1: String, depth: u8) {
    let stdout = ConsoleAppender::builder().build();
    let config = Config::builder()
        .appender(Appender::builder().build("stdout", Box::new(stdout)))
        .build(Root::builder().appender("stdout").build(LevelFilter::Debug))
        .unwrap();
    let _handle = log4rs::init_config(config).unwrap();

    let timer = Instant::now();

    let game_state = GameState::new(fen_1);
    let mut i = 1;
    while i <= depth {
        let timeout = Instant::now().checked_add(Duration::from_secs(30)).unwrap();

        let r = ab_search(
            &game_state,
            vec![],
            i,
            timeout,
            0,
            i32::MIN,
            i32::MAX,
        )
        .unwrap();
        let dur = timer.elapsed();
        println!("{i}:{:?} {:?}", r, dur);
        i += 1;
    }
}

fn timed_depth_test() {
    let stdout = ConsoleAppender::builder().build();
    let chimp_logs = FileAppender::builder()
        .encoder(Box::new(PatternEncoder::new("{m}{n}")))
        .build(format!(
            "log/timed_depth/{:?}.log",
            SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_secs()
        ))
        .unwrap();

    let config = Config::builder()
        .appender(Appender::builder().build("chimp", Box::new(chimp_logs)))
        .appender(Appender::builder().build("stdout", Box::new(stdout)))
        .build(
            Root::builder()
                .appender("stdout")
                .appender("chimp")
                .build(LevelFilter::Info),
        )
        .unwrap();
    let _handle = log4rs::init_config(config).unwrap();

    let game_state = GameState::default();
    let timeout = Instant::now()
        .checked_add(Duration::from_secs(120))
        .unwrap();
    info!("Timed depth test: 120s");
    iterative_deepening(game_state, timeout);
}

fn target_depth_test() {
    let stdout = ConsoleAppender::builder().build();
    let chimp_logs = FileAppender::builder()
        .encoder(Box::new(PatternEncoder::new("{m}{n}")))
        .build(format!(
            "log/target_depth/{:?}.log",
            SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_secs()
        ))
        .unwrap();

    let config = Config::builder()
        .appender(Appender::builder().build("chimp", Box::new(chimp_logs)))
        .appender(Appender::builder().build("stdout", Box::new(stdout)))
        .build(
            Root::builder()
                .appender("stdout")
                .appender("chimp")
                .build(LevelFilter::Info),
        )
        .unwrap();
    let _handle = log4rs::init_config(config).unwrap();

    let game_state = GameState::default();
    let depth = 8;
    info!("Target depth test: 8");
    let mut i = 0;
    let timer = Instant::now();
    while i <= depth {
        let timeout = Instant::now().checked_add(Duration::from_secs(300)).unwrap();

        let r = ab_search(
            &game_state,
            vec![],
            i,
            timeout,
            0,
            i32::MIN,
            i32::MAX,
        )
        .unwrap();
        let dur = timer.elapsed();
        info!("{i}:{:?} {:?}", r, dur);
        i += 1;
    }
}

fn park_table() {
    let stdout = ConsoleAppender::builder().build();
    let chimp_logs = FileAppender::builder()
        .encoder(Box::new(PatternEncoder::new("{m}{n}")))
        .build(format!(
            "log/chimp_v0.0.0.6_{:?}.log",
            SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_secs()
        ))
        .unwrap();

    let config = Config::builder()
        .appender(Appender::builder().build("chimp", Box::new(chimp_logs)))
        .appender(Appender::builder().build("stdout", Box::new(stdout)))
        .build(
            Root::builder()
                .appender("stdout")
                .appender("chimp")
                .build(LevelFilter::Info),
        )
        .unwrap();
    let _handle = log4rs::init_config(config).unwrap();

    let start = Instant::now();
    let mut engine: ChimpEngine = ChimpEngine::new();
    let mut moves = Vec::new();
    let mut move_ucis = Vec::new();
    let mut white_ms = 60000;
    let mut black_ms = 60000;
    let inc_ms = 1000;
    info!("Park Table:");
    for _i in 0..200 {
        let timer = Instant::now();
        let (m, ponder) = if _i == 0 || _i == 1 {
            engine.go(10000, 10000, -1, -1)
        } else {
            engine.go(white_ms, black_ms, inc_ms, inc_ms)
        };
        if _i > 1 {
            if engine.black_turn() {
                black_ms += inc_ms;
            } else {
                white_ms += inc_ms;
            }
        }
        if m.is_empty() {
            info!("No legal moves found! FF");
            break;
        }
        move_ucis.push(m.uci());
        let delay = timer.elapsed().as_millis() as i32;
        if engine.black_turn() {
            black_ms -= delay;
            if black_ms < 0 {
                info!("Black out of time!");
                break;
            }
        } else {
            white_ms -= delay;
            if white_ms < 0 {
                info!("White out of time!");
                break;
            }
        }
        moves.push(m);
        engine.position(get_moves_string(&move_ucis).split_ascii_whitespace());
        if engine.current_game_state.result_state != MatchResultState::Active {
            break;
        }
    }
    let duration = start.elapsed();
    info!("Result: {:?}", engine.current_game_state.result_state);
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
    let chimp_logs = FileAppender::builder()
        .encoder(Box::new(PatternEncoder::new("{m}{n}")))
        .build(format!(
            "log/perfts_{:?}.log",
            SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_secs()
        ))
        .unwrap();

    let config = Config::builder()
        .appender(Appender::builder().build("chimp", Box::new(chimp_logs)))
        .build(
            Root::builder()
                .appender("chimp")
                .build(LevelFilter::Info),
        )
        .unwrap();
    let _handle = log4rs::init_config(config).unwrap();
    perft(
        "Perft".into(),
        "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1".into(),
        vec![20, 400, 8902, 197281],
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
