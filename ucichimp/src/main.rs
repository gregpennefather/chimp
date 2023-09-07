use std::thread::JoinHandle;
use std::time::{Instant, SystemTime};
use std::{default, panic};

use ch_imp::engine::*;
use ch_imp::r#move::Move;
use log::{debug, info, LevelFilter};
use log4rs::append::file::FileAppender;
use log4rs::config::{Appender, Config, Logger, Root};
use log4rs::encode::pattern::PatternEncoder;

fn main() {
    let chimp_logs = FileAppender::builder()
        .encoder(Box::new(PatternEncoder::new("{m}{n}")))
        .build(format!(
            "log/chimp_v0.0.0.11_{:?}.log",
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

#[derive(Default)]
struct TimeInfo {
    pub wtime: i32,
    pub btime: i32,
    pub winc: i32,
    pub binc: i32,
}

fn run() -> bool {
    let mut input = String::new();
    let mut engine: ChimpEngine = ChimpEngine::new();
    debug!(target:"app:chimp", "\n==================================== Chimp Started ===============================\n");
    let mut last_time_info = TimeInfo::default();
    let mut ponder_handler: Option<JoinHandle<Vec<Move>>> = None;
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
                    let (should_ponder, time_info) = get_go_params(split_string);
                    last_time_info = time_info;
                    if should_ponder {
                        ponder_handler = Some(engine.ponder());
                    } else {
                        let (bestmove, ponder) = engine.go(
                            last_time_info.wtime,
                            last_time_info.btime,
                            last_time_info.winc,
                            last_time_info.binc,
                        );
                        handle_go_result(bestmove, ponder);
                    }
                }
                "ponderhit" => {
                    engine.ponder_hit();
                    let ponder_result = await_ponder_handler_result(ponder_handler);
                    let (bestmove, ponder) = engine.go_post_ponder(
                        last_time_info.wtime,
                        last_time_info.btime,
                        last_time_info.winc,
                        last_time_info.binc,
                        ponder_result,
                    );
                    handle_go_result(bestmove, ponder);
                    ponder_handler = None;
                }
                "pondermiss" | "stop" => {
                    engine.ponder_miss();
                    let _ponder_result = await_ponder_handler_result(ponder_handler);
                    let (bestmove, ponder) = engine.go(
                        last_time_info.wtime,
                        last_time_info.btime,
                        last_time_info.winc,
                        last_time_info.binc,
                    );
                    handle_go_result(bestmove, ponder);
                    ponder_handler = None;
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

fn handle_go_result(bestmove: Move, ponder: Option<Move>) {
    if bestmove.is_empty() {
        println!("ff")
    } else {
        let message = format!(
            "bestmove {}{}",
            bestmove.uci(),
            match ponder {
                Some(r) => format!(" ponder {}", r.uci()),
                None => "".into(),
            }
        );
        info!("{}", message);
        println!("{}", message);
    }
}

fn await_ponder_handler_result(ponder_handler: Option<JoinHandle<Vec<Move>>> ) -> Vec<Move> {
    match ponder_handler {
        Some(handler) => {
            let handler_result = handler.join();
            match handler_result {
                Ok(r) => {
                    return r
                }
                Err(e) => panic!("{e:?}"),
            };
        }
        None => panic!("handler none?!"),
    }
    vec![]
}

fn get_go_params(mut split_string: std::str::SplitAsciiWhitespace<'_>) -> (bool, TimeInfo) {
    let first_word = split_string.next().unwrap();
    let mut ponder = false;
    if first_word.eq("movetime") {
        let r = split_string.next().unwrap();
        let v = r.parse::<i32>().unwrap();
        return (
            false,
            TimeInfo {
                wtime: v,
                btime: v,
                winc: -1,
                binc: -1,
            },
        );
    }

    if first_word.eq("ponder") {
        ponder = true;
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
    (
        ponder,
        TimeInfo {
            wtime,
            btime,
            winc,
            binc,
        },
    )
}
