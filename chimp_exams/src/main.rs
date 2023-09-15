use std::{fs::read_to_string, time::SystemTime};

use log::{info, LevelFilter};
use log4rs::{
    append::{console::ConsoleAppender, file::FileAppender},
    config::{Appender, Config, Root},
    encode::pattern::PatternEncoder,
};
use test_suite::{TestInstance, TestSuite};

mod test_suite;

const STRATEGIC_TEST_SUITE_FILES: [(&str, &str); 15] = [
    ("Undermining", "STS1.epd"),
    ("Open Files and Diagonals", "STS2.epd"),
    ("Knight Outposts", "STS3.epd"),
    ("Square Vacancy", "STS4.epd"),
    ("Bishop vs Knight", "STS5.epd"),
    ("Re-Capturing", "STS6.epd"),
    ("Offer of Simplification", "STS7.epd"),
    ("Advancement of f/g/h pawns", "STS8.epd"),
    ("Advancement of a/b/c Pawns", "STS9.epd"),
    ("Simplification", "STS10.epd"),
    ("Activity of the King", "STS11.epd"),
    ("Center Control", "STS12.epd"),
    ("Pawn Play in the Center", "STS13.epd"),
    ("Queens and Rooks to the 7th Rank", "STS14.epd"),
    ("Avoid Pointless Exchange", "STS15.epd"),
];

const RECORD: bool = false;

fn main() {
    let stdout = ConsoleAppender::builder()
        .encoder(Box::new(PatternEncoder::new("{d(%H%M%S)} {l} {m}{n}")))
        .build();

    let config = if RECORD {
        let chimp_logs: FileAppender = FileAppender::builder()
            .encoder(Box::new(PatternEncoder::new("{d(%H%M%S)} {l} {m}{n}")))
            .build(format!(
                "reports/sts_report_{:?}.log",
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
                    .build(LevelFilter::Debug),
            )
            .unwrap()
    } else {
        Config::builder()
            .appender(Appender::builder().build("stdout", Box::new(stdout)))
            .build(Root::builder().appender("stdout").build(LevelFilter::Debug))
            .unwrap()
    };
    let _handle = log4rs::init_config(config).unwrap();

    for test_suite_file_info in STRATEGIC_TEST_SUITE_FILES {
        let test_suite = read_file(test_suite_file_info.0, test_suite_file_info.1);
        println!("{test_suite}");
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
