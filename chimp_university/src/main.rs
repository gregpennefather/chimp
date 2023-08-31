use std::{
    mem,
    time::{Instant, SystemTime},
};

use ch_imp::{
    board::{bitboard::Bitboard, position::Position},
    engine::{perft::perft, san::build_san, ChimpEngine},
    evaluation::{base_eval::base_eval, early_eval},
    r#move::Move,
};
use log::{info, LevelFilter};
use log4rs::{
    append::{console::ConsoleAppender, file::FileAppender},
    config::{Appender, Root},
    encode::pattern::PatternEncoder,
    Config,
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

    // let gs = GameState::new("r3k2r/p1ppqpb1/bn1Npnp1/3PN3/1p2P3/5Q2/PPPBBPpP/R3K2R b KQkq - 1 2".into());
    // println!("{:?}", gs.move_from_uci("g2h1q"));
    // println!("{:?}", gs.move_from_uci("g2g1n"));

    // let bitboard = 0.set(27).set(28).set(35).set(36);
    // println!("{}", bitboard.to_board_format());
    // println!("{}", bitboard);

    //let position = Position::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1".into());
    // println!("{}", position.base_eval);

    // compare_evals("r1bqkb1r/1pp2pp1/5p1p/3p4/pn1P4/2N2N2/PPPKPPPP/1R1Q1B1R b kq - 1 9".into(), "r1bqkb1r/1pp2pp1/5p1p/3p4/pn1P4/P1N2N2/1PP1PPPP/1R1QKB1R b Kkq - 0 9".into());

    park_table(50);
}

fn compare_evals(fen_1: String, fen_2: String) {
    let p1 = Position::from_fen(fen_1);
    let p2 = Position::from_fen(fen_2);

    println!(
        "{} vs {}",
        p1.eval,
        p2.eval
    )
}

fn park_table(ply_count: usize) {
    let stdout = ConsoleAppender::builder().build();
    let chimp_logs = FileAppender::builder()
        .encoder(Box::new(PatternEncoder::new("{d} - {m}{n}")))
        .build(format!(
            "log/chimp_v0.0.0.4_{:?}.log",
            SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_secs()
        ))
        .unwrap();

    let config = Config::builder()
        .appender(Appender::builder().build("chimp", Box::new(chimp_logs)))
        .appender(Appender::builder().build("stdout", Box::new(stdout)))
        .build(Root::builder().appender("chimp").build(LevelFilter::Info))
        .unwrap();
    let _handle = log4rs::init_config(config).unwrap();

    let start = Instant::now();
    let mut engine: ChimpEngine = ChimpEngine::new();
    let mut moves = Vec::new();
    let mut move_ucis = Vec::new();
    info!("Park Table (ply_count:{ply_count}):");
    for _i in 0..(ply_count) {
        let m = engine.go();
        move_ucis.push(m.uci());
        moves.push(m);
        engine.position(get_moves_string(&move_ucis).split_ascii_whitespace())
    }
    let duration = start.elapsed();
    info!("Runtime: {:?}", duration);
    info!("Moves: {move_ucis:?}");
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
