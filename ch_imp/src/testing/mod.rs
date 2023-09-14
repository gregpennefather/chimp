use std::fs::read_to_string;
use colored::Colorize;

use crate::{engine::ChimpEngine, match_state::game_state::GameState};

// const TEST_SUITES: [(&str, usize); 3] = [ ("AH_Endgames-250.epd", 5000),("bk_test.txt", 5000),("KaufmanTestSuite.txt", 5000)];
const TEST_SUITES: [(&str, usize); 1] = [("KaufmanTestSuite.txt", 20000)];

#[derive(Debug)]
enum CommandType {
    bm = 0,
    am = 1,
}

pub fn test_engine() {
    for test_suite in TEST_SUITES {
        println!("=== Start of suite: {} ===", test_suite.0);
        let mut suite_size = 0;
        let mut successes = 0;

        for line in read_to_string(format!("./test-suites/{}", test_suite.0))
            .unwrap()
            .lines()
        {
            let (command, fn_end) = get_command_pos(line);

            let id_pos = line.find("id").unwrap();

            let fen = line[0..fn_end].to_string();
            let command_str = trim_command(line[fn_end..id_pos].to_string());
            let line_len = line.len();
            let id = trim_id(line[id_pos..line_len].to_string());

            if perform_test(command, fen, command_str, id, test_suite.1) {
                successes += 1;
            }
            suite_size += 1;
        }
        let r = format!("{}/{}", successes, suite_size);
        println!("=== Suite result: {} ===", if successes == suite_size { r.green() } else { r.yellow() } );
    }
}

fn perform_test(
    command: CommandType,
    fen: String,
    command_str: String,
    id: String,
    timeout: usize,
) -> bool {
    let mut engine = ChimpEngine::from_position(fen);

    let (bestmove, ponder) = engine.go(0, 0, timeout as i32, timeout as i32);

    let san = engine.current_game_state.to_san(bestmove);

    let result = match command {
        CommandType::bm => san == command_str,
        CommandType::am => san != command_str,
    };

    println!(
        "{id} searchTime:{timeout} {command:?} result:{} | {san} vs {command_str}",
        if result {
            "SUCCESS".green()
        } else {
            "FAILED".red()
        }
    );
    result
}

fn get_command_pos(line: &str) -> (CommandType, usize) {
    match line.find("bm") {
        Some(pos) => (CommandType::bm, pos),
        None => match line.find("am") {
            Some(ampos) => (CommandType::am, ampos),
            None => panic!(),
        },
    }
}

fn trim_command(cmd: String) -> String {
    let space_pos = cmd.find(" ").unwrap() + 1;
    let len = cmd.len();
    cmd[space_pos..len - 2].into()
}

fn trim_id(id: String) -> String {
    let space_pos = id.find("\"").unwrap() + 1;
    let len = id.len();
    id[space_pos..len - 2].into()
}
