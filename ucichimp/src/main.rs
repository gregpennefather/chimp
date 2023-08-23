use std::panic;
use std::time::Instant;

use chimp::engine::ChimpEngine;
use log::{debug, LevelFilter, info};
use log4rs::append::file::FileAppender;
use log4rs::config::{Appender, Config, Logger, Root};
use log4rs::encode::pattern::PatternEncoder;

fn main() {
    let chimp_logs = FileAppender::builder()
        .encoder(Box::new(PatternEncoder::new("{d} - {m}{n}")))
        .build("log/chimp.log")
        .unwrap();

    let config = Config::builder()
        .appender(Appender::builder().build("chimp", Box::new(chimp_logs)))
        .build(Root::builder().appender("chimp").build(LevelFilter::Debug))
        .unwrap();

    let handle = log4rs::init_config(config).unwrap();
    let start = Instant::now();

    let result = panic::catch_unwind(|| run());
    let duration = start.elapsed();
    info!("Runtime: {:?}", duration);
    if result.is_err() {
        let err = result.unwrap_err();
        let err_text = format!("{:#?}", err.downcast_ref::<panic::PanicInfo>());
        log::error!(target:"app:chimp", "chimp encountered an error {}", err_text);
        log::error!("{:#?}", err.downcast_ref::<&str>());
    }
}

fn run() -> bool {
    let mut input = String::new();
    let mut engine = ChimpEngine::new();
    debug!(target:"app:chimp", "\n==================================== Chimp Started ===============================\n");
    loop {
        std::io::stdin().read_line(&mut input).unwrap();
        debug!(target:"app:chimp", ">> {}", input);
        let trimmed = input.trim_end();
        let mut split_string = trimmed.split_ascii_whitespace();
        let first_word = split_string.next();
        match first_word {
            Some(word) => match word {
                "uci" => println!("uciok"),
                "isready" => println!("readyok"),
                "ucinewgame" => {
                    engine = ChimpEngine::new();
                }
                "position" => {
                    engine.position(split_string);
                }
                "go" => {
                    println!("bestmove {}", engine.go(split_string));
                }
                "quit" => break,
                _ => {
                    println!("Unknown command {}", input);
                    break;
                }
            },
            None => {}
        }
        input = String::new();
    }
    info!("wrangler quit");
    true
}
