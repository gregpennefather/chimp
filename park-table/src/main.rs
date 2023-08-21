use chimp::engine::ChimpEngine;
use log::LevelFilter;
use log4rs::{append::console::ConsoleAppender, Config, config::{Appender, Root}};

const MOVE_2_PLY_COUNT: usize = 30;

fn main() {
    let stdout = ConsoleAppender::builder().build();

    let config = Config::builder()
        .appender(Appender::builder().build("stdout", Box::new(stdout)))
        .build(Root::builder().appender("stdout").build(LevelFilter::Info))
        .unwrap();

    let handle = log4rs::init_config(config).unwrap();

    let mut engine = ChimpEngine::new();
    let mut moves = Vec::new();

    let go_dummy: String = "".into();
    let start_pos: String = "startpos".into();

    engine.position(start_pos.split_ascii_whitespace());
    for i in 0..(MOVE_2_PLY_COUNT * 2) {
        let (best_uci, best_san) = engine.go_uci_and_san();
        moves.push((best_uci.clone(), best_san.clone()));
        engine.position(get_moves_string(&moves).split_ascii_whitespace());
    }

    let mut output: String = "".into();
    for full_move_index in 0..(MOVE_2_PLY_COUNT) {
        let white_pgn = &moves[full_move_index * 2].1;
        let black_pgn = &moves[full_move_index * 2 + 1].1;
        let clause = format!("{}. {white_pgn} {black_pgn}", full_move_index + 1);
        output = format!("{} {}", output, clause);
    }
    println!("{}", output)
}

fn get_moves_string(moves: &Vec<(String, String)>) -> String {
    let mut result = "startpos moves".into();
    for m in moves {
        result = format!("{} {}", result, m.0);
    }
    result
}
