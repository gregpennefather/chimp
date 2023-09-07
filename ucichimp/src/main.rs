use std::panic;
use std::time::{Instant, SystemTime};

use ch_imp::engine::*;
use log::{debug, info, LevelFilter};
use log4rs::append::file::FileAppender;
use log4rs::config::{Appender, Config, Logger, Root};
use log4rs::encode::pattern::PatternEncoder;

fn main() {
    let chimp_logs = FileAppender::builder()
        .encoder(Box::new(PatternEncoder::new("{m}{n}")))
        .build(format!(
            "log/chimp_v0.0.0.7_{:?}.log",
            SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_secs()
        ))
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
    let mut engine: ChimpEngine = ChimpEngine::new();
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
                    let (wtime, btime, winc, binc) = get_go_params(split_string);
                    let (bestmove, ponder) = engine.go(wtime, btime, winc, binc);
                    if bestmove.is_empty() {
                        println!("ff")
                    } else {
                        let message = format!(
                            "bestmove {}{}",
                            bestmove.uci(),
                            ""
                            // match ponder {
                            //     Some(r) => "".into(), // Disable ponder for now: format!(" ponder {}", r.uci()),
                            //     None => "".into(),
                            // }
                        );
                        info!("{}", message);
                        println!("{}", message);
                    }
                }
                "quit" => break,
                _ => {
                    info!("Unknown command {}", input);
                    println!("Unknown command {}", input);
                    break;
                }
            },
            None => {}
        }
        input = String::new();
    }
    info!("ucichimp quit");
    true
}

fn get_go_params(mut split_string: std::str::SplitAsciiWhitespace<'_>) -> (i32, i32, i32, i32) {
    let first_word = split_string.next().unwrap();

    if first_word.eq("movetime") {
        let r = split_string.next().unwrap();
        let v = r.parse::<i32>().unwrap();
        return (v, v, -1, -1);
    }

    if first_word.eq("ponder") {
        split_string.next().unwrap();
    }

    let r = split_string.next().unwrap();
    let wtime = r.parse::<i32>().unwrap();

    split_string.next();
    let r = split_string.next().unwrap();
    let btime = r.parse::<i32>().unwrap();

    split_string.next();
    let r = split_string.next().unwrap();
    let winc = r.parse::<i32>().unwrap();

    split_string.next();
    let r = split_string.next().unwrap();
    let binc = r.parse::<i32>().unwrap();
    (wtime, btime, winc, binc)
}
