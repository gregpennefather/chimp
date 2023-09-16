use std::{fs::read_to_string, time::{SystemTime, Instant, Duration}};

use ch_imp::engine::ChimpEngine;
use log::{info, LevelFilter};
use log4rs::{
    append::{console::ConsoleAppender, file::FileAppender},
    config::{Appender, Config, Root},
    encode::pattern::PatternEncoder,
};
use test_suite::{TestInstance, TestSuite, ResultScore};

mod test_suite;

const STRATEGIC_TEST_SUITE_FILES: [(&str, &str); 15] = [
    ("Re-Capturing", "STS6.epd"),
    ("Activity of the King", "STS11.epd"),
    ("Center Control", "STS12.epd"),
    ("Pawn Play in the Center", "STS13.epd"),
    ("Open Files and Diagonals", "STS2.epd"),
    ("Knight Outposts", "STS3.epd"),
    ("Undermining", "STS1.epd"),
    ("Avoid Pointless Exchange", "STS15.epd"),
    ("Square Vacancy", "STS4.epd"),
    ("Bishop vs Knight", "STS5.epd"),
    ("Offer of Simplification", "STS7.epd"),
    ("Advancement of f/g/h pawns", "STS8.epd"),
    ("Advancement of a/b/c Pawns", "STS9.epd"),
    ("Simplification", "STS10.epd"),
    ("Queens and Rooks to the 7th Rank", "STS14.epd"),
];

const RECORD: bool = true;
const TEST_TIME_SECONDS: i32 = 5;

fn main() {
    let stdout = ConsoleAppender::builder()
        .encoder(Box::new(PatternEncoder::new("{l} {m}{n}")))
        .build();

    let config = if RECORD {
        let chimp_logs: FileAppender = FileAppender::builder()
            .encoder(Box::new(PatternEncoder::new("{l} {m}{n}")))
            .build(format!(
                "reports/sts_report_{}s_{:?}.log",
                TEST_TIME_SECONDS,
                SystemTime::now()
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .unwrap()
                    .as_secs()
            ))
            .unwrap();
        Config::builder()
            .appender(Appender::builder().build("chimp", Box::new(chimp_logs)))
            .appender(Appender::builder().build("stdout", Box::new(stdout)))
            .build(
                Root::builder()
                    .appender("stdout")
                    .appender("chimp")
                    .build(LevelFilter::Info),
            )
            .unwrap()
    } else {
        Config::builder()
            .appender(Appender::builder().build("stdout", Box::new(stdout)))
            .build(Root::builder().appender("stdout").build(LevelFilter::Info))
            .unwrap()
    };
    let _handle = log4rs::init_config(config).unwrap();

    for test_suite_file_info in STRATEGIC_TEST_SUITE_FILES {
        let test_suite = read_file(test_suite_file_info.0, test_suite_file_info.1);
        run_suite(test_suite);
    }
}

fn read_file(theme: &str, file_name: &str) -> TestSuite {
    let mut tests = vec![];
    let mut max_score = 0;
    for line in read_to_string(format!("./files/{}", file_name))
        .unwrap()
        .lines()
    {
        let test = TestInstance::new(line);
        max_score += test.result_scores[0].score;
        tests.push(test);
    }
    return TestSuite {
        name: theme.to_string(),
        tests,
        max_score,
    };
}

fn run_suite(test_suite: TestSuite) -> usize {
    let mut score = 0;
    info!("===== {} =====", test_suite.name);
    let timems = TEST_TIME_SECONDS * 1000;
    for test in test_suite.tests {
        let mut engine = ChimpEngine::from_position(test.fen);
        let timeout = Instant::now()
            .checked_add(Duration::from_millis(timems as u64))
            .unwrap();
        let result = engine.iterative_deepening(&|| Instant::now() > timeout, vec![]);
        let san = engine.current_game_state.to_san(result[0]);
        let score_change = handle_result(&test.result_scores, &san);
        info!("'{}': {score_change}/10 M:{} BM:{} ({:?})", test.name, san, test.result_scores[0].m, result[0]);
        score += score_change;
    }
    info!("===== {}/{} =====", score, test_suite.max_score);
    score
}

fn handle_result(result_scores: &Vec<ResultScore>, bestmove: &String) -> usize {
    let mut score = 0;
    for result_score in result_scores {
        if bestmove.eq(&result_score.m) {
            score = result_score.score;
        }
    }
    score
}
