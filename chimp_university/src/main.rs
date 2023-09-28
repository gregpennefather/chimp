use std::{
    mem::size_of,
    time::{Duration, Instant, SystemTime},
    vec,
};

use ch_imp::{
    board::{bitboard::Bitboard, position::Position, board_rep::BoardRep},
    engine::{
        perft::perft,
        san::build_san,
        search::{AB_MAX, AB_MIN},
        ChimpEngine,
    },
    match_state::game_state::{self, GameState, MatchResultState},
    move_generation::generate_moves_for_board,
    r#move::move_data::MoveData,
    shared::board_utils::{get_index_from_file_and_rank, index_from_coords},
    testing::test_engine,
    MOVE_DATA,
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

    //perft("t".into(), "rn1qk2r/pp2bppp/2Q5/8/2B5/8/PPP1NnPP/RNBQK2R b KQ - 0 9".into(),vec![5]);

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

    //println!("{:?}", [0.set_file(7), 0.set_file(6), 0.set_file(5), 0.set_file(4), 0.set_file(3), 0.set_file(2), 0.set_file(1), 0.set_file(0)]);

    //build_pawn_frontspan_board();

    // let rank2 : [u64; 8] = [
    //     0.flip(15).flip(14).flip(13),
    //     0.flip(15).flip(14).flip(13),
    //     0.flip(15).flip(14).flip(13),
    //     0,
    //     0,
    //     0.flip(8).flip(9).flip(10),
    //     0.flip(8).flip(9).flip(10),
    //     0.flip(8).flip(9).flip(10),
    // ];
    // println!("{rank2:?}");

    // let rank3 : [u64; 8] = [
    //     0.flip(23).flip(22).flip(21),
    //     0.flip(23).flip(22).flip(21),
    //     0.flip(23).flip(22).flip(21),
    //     0,
    //     0,
    //     0.flip(16).flip(17).flip(18),
    //     0.flip(16).flip(17).flip(18),
    //     0.flip(16).flip(17).flip(18),
    // ];
    // println!("{rank3:?}");

    // let pos1 = Position::default();
    // let pos2 = Position::from_fen("rnbqkbnr/pppppppp/8/8/8/5N2/PPPPPPPP/RNBQKB1R b KQkq - 1 1".into());
    // assert_eq!(pos1.board.king_pawn_zorb, pos2.board.king_pawn_zorb);
    // let gs = GameState::new("rnbqkbnr/pppppppp/8/8/8/5N2/PPPPPPPP/RNBQKB1R b KQkq - 1 1".into());
    // let gs2 = gs.make(gs.move_from_uci("g8h6")).unwrap();
    // assert_eq!(pos1.board.king_pawn_zorb, gs2.position.board.king_pawn_zorb);

    // let game_state = GameState::new("8/5p2/8/p1p3P1/P1P5/7P/1P6/8 w - - 0 1".into());
    // println!("{:?}", game_state.result_state());

    //  debug_evals(
    //      "rnbqk1nr/pp4bp/2p2p2/3pp3/2B1P2P/8/PPPP1PP1/RNB1K1NR w KQkq - 0 6".into(),
    //      "rnbqkbnr/pp4Qp/2p2p2/4p3/2p1P2P/8/PPPP1PP1/RNB1K1NR w KQkq - 0 6".into(),
    //  );
    // debug_evals(
    //     "rnb1kbnr/pppp1ppp/8/4Q3/4Pq2/5N2/PPPP1PPP/RNB1KB1R b KQkq - 0 4".into(),
    //     "rnb1kbnr/pppp1ppp/8/1B2p2Q/4Pq2/5N2/PPPP1PPP/RNB1K2R b KQkq - 5 4".into(),
    // );

    // let gs = GameState::new("rn2kbnr/pp1ppppp/2p5/q5b1/4P3/8/PPPP1PPP/RNBQKBNR w KQkq - 0 1".into());

    // let mut engine = ChimpEngine::new();

    // println!("{:?}", engine.quiescence_search(gs, &|| false, AB_MIN, AB_MAX));

    //debug_deepening("rn1qkb1r/pbppnppp/1p6/1P6/P3p2P/5NP1/2PPPPB1/RNBQK2R b KQkq - 0 7".into(), 1000);

    //debug_deepening("rnb1kbr1/ppq1pppp/1npp4/8/2BPP2N/2N5/PPPB1PPP/R2Q1RK1 w q - 6 12".into(), 5000);

    // debug_search(
    //     "rn1qkb1r/pbppnppp/1p6/1P6/P3p2P/5NP1/2PPPPB1/RNBQK2R b KQkq - 0 7".into(),
    //     8,
    // );

    //println!("{r:?}");

    //timed_depth_test();
    // target_depth_test();

    // let pos = Position::from_fen("7k/3p4/4b3/4R3/8/2B5/8/1K6 w - - 0 1".into());
    // println!("{:? }", generate_moves_for_board(pos.board))

    // println!("{}", Position::from_fen("rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR w KQkq - 0 1".into()).eval);


    // println!("Q takes: {}", Position::from_fen("1q2rnk1/5rb1/bp1p1np1/pNpP2Bp/P1P1PQ1P/3B2P1/4NRR1/7K b - - 0 1".into()).eval);
    // println!("P takes: {}", Position::from_fen("1q2rnk1/5rb1/bp1p1np1/pNpP2Bp/P1P1PP1P/3B4/3QNRR1/7K b - - 0 1".into()).eval);

    // println!("r takes: {}", Position::from_fen("1q3nk1/5rb1/bp1p1np1/pNpP2Bp/P1P1rQ1P/3B2P1/4NRR1/7K w - - 0 2".into()).eval);

    // test_it_deep_search("rnb1kb1r/pp1p1ppp/2p2n2/q3p3/3PP3/2N5/PPPQ1PPP/R1B1KBNR w KQkq - 2 5".into(), 2000);

    //let board = Position::from_fen("r1bqkbnr/ppp2ppp/2np4/4N3/4P3/8/PPPP1PPP/RNBQKB1R w KQkq - 0 1".into());

    // println!("{:?}", generate_moves_for_board(board));

    // println!("p take: {}", Position::from_fen("rnb1kb1r/pp1p1ppp/2p2n2/4q3/4P3/2N5/PPPQ1PPP/R1B1KBNR w KQkq - 0 6".into()).eval);
    // println!("k move: {}", Position::from_fen("rnb1kb1r/pp3ppp/2pp1n2/q3p3/3PP3/2N5/PPPQKPPP/R1B2BNR w kq - 0 6".into()).eval);
    // println!("k move: {}", Position::from_fen("rnb1kb1r/pp1p1ppp/2p2n2/4q3/4P3/2N2N2/PPPQ1PPP/R1B1KB1R b KQkq - 1 6".into()).eval);

    //test_ab_search("7K/8/8/6rk/8/8/8/8 b - - 11 6".into(), 10);

    //test_ab_search("8/8/8/8/8/7R/4K1k1/8 w - - 22 12".into(), 10);

    // println!("{:?}", MOVE_DATA.is_slide_legal(0, 8));
    // println!("{:?}", MOVE_DATA.is_slide_legal(0, 9));
    // println!("{:?}", MOVE_DATA.is_slide_legal(1, 9));
    // println!("{}", MOVE_DATA.get_slide_inbetween(1, 17).to_board_format());
    // println!("{}", MOVE_DATA.get_slide_inbetween(1, 28).to_board_format());
    // println!("{}", MOVE_DATA.get_slide_inbetween(index_from_coords("e2"), index_from_coords("e8")).to_board_format());
    // println!("start: {}\n", Position::from_fen("8/8/8/6K1/8/8/8/kr6 b - - 1 1".into()).eval);
    // println!("w moves left: {}\n", Position::from_fen("8/8/8/5K2/8/8/8/kr6 b - - 1 1".into()).eval);
    // println!("w moves right: {}\n", Position::from_fen("8/8/8/7K/8/8/8/kr6 b - - 1 1".into()).eval);
    // println!("close black king: {}\n", Position::from_fen("8/8/8/5k1K/8/8/8/1r6 b - - 1 1".into()).eval);
    // println!("black king in center: {}\n", Position::from_fen("8/8/8/4k2K/8/8/8/1r6 b - - 1 1".into()).eval);


    // println!("1: {}", Position::from_fen("7K/8/8/6rk/8/8/8/8 b - - 11 6".into()).eval);
    // println!("2: {}", Position::from_fen("7K/8/6k1/6r1/8/8/8/8 w - - 12 7".into()).eval);
    // println!("3: {}", Position::from_fen("6K1/8/6k1/6r1/8/8/8/8 b - - 13 7".into()).eval);
    // println!("4: {}", Position::from_fen("6K1/8/6k1/5r2/8/8/8/8 w - - 14 8".into()).eval);
    // println!("5: {}", Position::from_fen("7K/8/6k1/5r2/8/8/8/8 b - - 15 8".into()).eval);

    //perfts();
    park_table();
    //test_engine();
}

fn test_ab_search(fen: String, depth: u8) {
    let stdout = ConsoleAppender::builder().build();
    let config = Config::builder()
        .appender(Appender::builder().build("stdout", Box::new(stdout)))
        .build(Root::builder().appender("stdout").build(LevelFilter::Debug))
        .unwrap();
    let _handle = log4rs::init_config(config).unwrap();

    let timer = Instant::now();
    let mut priority_line = vec![];
    let mut engine = ChimpEngine::from_position(fen);
    let mut i: u8 = 0;
    while i <= depth {
        let timeout = Instant::now()
            .checked_add(Duration::from_secs(3600))
            .unwrap();
        let cutoff = || Instant::now() > timeout;
        let (eval, moves) = engine.alpha_beta_search(
            engine.current_game_state,
            &cutoff,
            i,
            0,
            AB_MIN - 1,
            AB_MAX + 1,
            &priority_line,
            0,
        );
        let dur = timer.elapsed();
        println!("{i}: {eval} \t{:?} \t {moves:?}", dur);
        priority_line = moves.clone();
        i += 1;
        if eval == AB_MAX || eval == AB_MIN {
            break;
        }
    }
    println!(
        "Position:\n- hits: {}\n- misses: {}",
        engine.position_cache.hits, engine.position_cache.misses
    );
}

fn test_it_deep_search(fen: String, ms: u64) {
    let stdout = ConsoleAppender::builder().build();
    let config = Config::builder()
        .appender(Appender::builder().build("stdout", Box::new(stdout)))
        .build(Root::builder().appender("stdout").build(LevelFilter::Debug))
        .unwrap();
    let _handle = log4rs::init_config(config).unwrap();

    let mut engine = ChimpEngine::from_position(fen);
    println!("Eval before: {}", engine.current_game_state.position.eval);
    let timeout = Instant::now().checked_add(Duration::from_millis(ms)).unwrap();
    let cutoff = || Instant::now() > timeout;
    let o = engine.iterative_deepening(&cutoff, vec![]);
    println!("{o:?}");
    println!(
        "hits: {}\nmisses: {}",
        engine.position_cache.hits, engine.position_cache.misses
    );
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

// fn timed_depth_test() {
//     let stdout = ConsoleAppender::builder().build();
//     let chimp_logs = FileAppender::builder()
//         .encoder(Box::new(PatternEncoder::new("{m}{n}")))
//         .build(format!(
//             "log/timed_depth/{:?}.log",
//             SystemTime::now()
//                 .duration_since(SystemTime::UNIX_EPOCH)
//                 .unwrap()
//                 .as_secs()
//         ))
//         .unwrap();

//     let config = Config::builder()
//         .appender(Appender::builder().build("chimp", Box::new(chimp_logs)))
//         .appender(Appender::builder().build("stdout", Box::new(stdout)))
//         .build(
//             Root::builder()
//                 .appender("stdout")
//                 .appender("chimp")
//                 .build(LevelFilter::Info),
//         )
//         .unwrap();
//     let _handle = log4rs::init_config(config).unwrap();

//     let game_state = GameState::default();
//     let timeout = Instant::now()
//         .checked_add(Duration::from_secs(120))
//         .unwrap();
//     info!("Timed depth test: 120s");
//     iterative_deepening(game_state, timeout, vec![]);
// }

// fn target_depth_test() {
//     let stdout = ConsoleAppender::builder().build();
//     let chimp_logs = FileAppender::builder()
//         .encoder(Box::new(PatternEncoder::new("{m}{n}")))
//         .build(format!(
//             "log/target_depth/{:?}.log",
//             SystemTime::now()
//                 .duration_since(SystemTime::UNIX_EPOCH)
//                 .unwrap()
//                 .as_secs()
//         ))
//         .unwrap();

//     let config = Config::builder()
//         .appender(Appender::builder().build("chimp", Box::new(chimp_logs)))
//         .appender(Appender::builder().build("stdout", Box::new(stdout)))
//         .build(
//             Root::builder()
//                 .appender("stdout")
//                 .appender("chimp")
//                 .build(LevelFilter::Info),
//         )
//         .unwrap();
//     let _handle = log4rs::init_config(config).unwrap();

//     let game_state = GameState::default();
//     let depth = 8;
//     info!("Target depth test: 8");
//     let mut i = 0;
//     let timer = Instant::now();
//     while i <= depth {
//         let timeout = Instant::now()
//             .checked_add(Duration::from_secs(300))
//             .unwrap();

//         let r = ab_search(&game_state, &vec![], i, 0, timeout, 0, i32::MIN, i32::MAX).unwrap();
//         let dur = timer.elapsed();
//         info!("{i}:{:?} {:?}", r, dur);
//         i += 1;
//     }
// }

fn park_table() {
    let stdout = ConsoleAppender::builder()
        .encoder(Box::new(PatternEncoder::new("{d(%H%M%S)} {l} {m}{n}")))
        .build();
    let chimp_logs: FileAppender = FileAppender::builder()
        .encoder(Box::new(PatternEncoder::new("{d(%H%M%S)} {l} {m}{n}")))
        .build(format!(
            "log/chimp_{:?}.log",
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
                .build(LevelFilter::Debug),
        )
        .unwrap();
    let _handle = log4rs::init_config(config).unwrap();

    let start = Instant::now();
    let position: String = "1R6/R7/8/7K/8/5k2/8/8 b - - 1 1".into();
    let mut w_engine: ChimpEngine = ChimpEngine::from_position(position.clone());
    let mut b_engine: ChimpEngine = ChimpEngine::from_position(position.clone());
    let mut white_turn = !w_engine.black_turn();
    let mut moves = Vec::new();
    let mut move_ucis = Vec::new();
    let mut white_ms = 5000;
    let mut black_ms = 5000;
    let inc_ms = 1000;
    info!("Park Table:");
    for _i in 0..30 {
        let timer = Instant::now();
        if white_turn {
            w_engine.position(get_moves_string(&move_ucis).split_ascii_whitespace());
        } else {
            b_engine.position(get_moves_string(&move_ucis).split_ascii_whitespace());
        }
        let (m, ponder) = if _i == 0 || _i == 1 {
            if white_turn {
                w_engine.go(5000, 5000, -1, -1)
            } else {
                b_engine.go(5000, 5000, -1, -1)
            }
        } else {
            if white_turn {
                w_engine.go(white_ms, black_ms, inc_ms, inc_ms)
            } else {
                b_engine.go(white_ms, black_ms, inc_ms, inc_ms)
            }
        };

        if _i > 1 {
            if !white_turn {
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
        if _i > 1 {
            let delay = timer.elapsed().as_millis() as i32;
            if !white_turn {
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
        }
        white_turn = !white_turn;
        moves.push(m);
        if b_engine.current_game_state.result_state != MatchResultState::Active {
            break;
        }
    }
    let duration = start.elapsed();
    info!("Result: {:?}", b_engine.current_game_state.result_state);
    info!("Runtime: {:?}", duration);
    info!("SAN: {}", build_san(moves, position));
    info!("Final state: {:?}", b_engine.current_game_state);
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
        .build(Root::builder().appender("chimp").build(LevelFilter::Info))
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
