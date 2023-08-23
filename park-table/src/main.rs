use std::time::Instant;

use chimp::engine::ChimpEngine;
use log::{info, LevelFilter};
use log4rs::{
    append::{console::ConsoleAppender, file::FileAppender},
    config::{Appender, Root},
    encode::pattern::PatternEncoder,
    Config,
};
use uuid::Uuid;

const MOVE_2_PLY_COUNT: usize = 30;

fn main() {
    let stdout = ConsoleAppender::builder().build();

    let id = Uuid::new_v4();
    let chimp_logs = FileAppender::builder()
        .encoder(Box::new(PatternEncoder::new("{d} - {m}{n}")))
        .build(format!("log/chimp-{}.log", id.to_string()))
        .unwrap();

    let config = Config::builder()
        .appender(Appender::builder().build("chimp", Box::new(chimp_logs)))
        .appender(Appender::builder().build("stdout", Box::new(stdout)))
        .build(Root::builder().appender("stdout").build(LevelFilter::Info))
        .unwrap();

    let _handle = log4rs::init_config(config).unwrap();

    let mut engine = ChimpEngine::new();
    let mut moves = Vec::new();

    let start_pos: String = "startpos".into();

    let start = Instant::now();
    engine.position(start_pos.split_ascii_whitespace());
    let mut move_list_length = MOVE_2_PLY_COUNT;
    for _i in 0..(MOVE_2_PLY_COUNT * 2) {
        let (best_uci, best_san) = engine.go_uci_and_san();

        if best_san.len() == 0 {
            println!("{best_uci}");
            move_list_length = (_i / 2) + 1;
            break;
        }

        moves.push((best_uci.clone(), best_san.clone()));
        engine.position(get_moves_string(&moves).split_ascii_whitespace());
    }

    let mut output: String = "".into();
    for full_move_index in 0..move_list_length {
        let white_pgn = &moves[full_move_index * 2].1;
        let black_pgn = moves.get(full_move_index * 2 + 1);
        let clause = match black_pgn {
            Some(b_png) => format!("{}. {white_pgn} {}", full_move_index + 1, b_png.1),
            None => format!("{}. {white_pgn}", full_move_index + 1),
        };

        output = format!("{} {}", output, clause);
    }
    info!("{}", output);
    let duration = start.elapsed();
    info!("Runtime: {:?}", duration);
}

fn get_moves_string(moves: &Vec<(String, String)>) -> String {
    let mut result = "startpos moves".into();
    for m in moves {
        result = format!("{} {}", result, m.0);
    }
    result
}
