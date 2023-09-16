use std::fmt::Display;

#[derive(Clone, Copy)]
pub enum TestType {
    bm = 0,
    am = 1,
}

#[derive(Clone, Debug)]
pub struct ResultScore {
    pub m: String,
    pub score: usize
}


impl std::fmt::Display for TestType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::bm => write!(f, "bm"),
            Self::am => write!(f, "am"),
        }
    }
}

#[derive(Clone)]
pub struct TestSuite {
    pub name: String,
    pub max_score: usize,
    pub tests: Vec<TestInstance>,
}

impl Display for TestSuite {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} - {} points available across {} tests",
            self.name, self.max_score, self.tests.len()
        )
    }
}

#[derive(Clone)]
pub struct TestInstance {
    pub name: String,
    pub fen: String,
    pub test_type: TestType,
    pub expected_result: String,
    pub comment: String,
    pub result_scores: Vec<ResultScore>
}
impl TestInstance {
    pub fn new(s: &str) -> Self {
        let (test_type, fn_end) = get_test_type(s);
        let fen = s[0..fn_end].to_string();

        let expected_move = get_command(s, fn_end);

        let id = get_id(s);
        let comment = get_comment(s);

        let result_scores = get_result_scores(&comment, &expected_move);

        Self {
            name: id,
            fen,
            test_type,
            expected_result: expected_move,
            comment: comment,
            result_scores
        }
    }
}

fn get_comment(s: &str) -> String {
    let comment_pos = match s.find("c0") {
        None => 0,
        Some(cp) => cp,
    };
    if comment_pos == 0 {
        return "".into();
    }

    let eos = s.len();
    let ss = s[comment_pos..eos].to_string();
    let c1 = ss.find("\"").unwrap();
    let c2 = ss[c1 + 1..].find("\"").unwrap();
    let start = comment_pos + c1 + 1;
    let end = start + c2;
    s[start..end].to_string()
}

fn get_command(s: &str, fn_end: usize) -> String {
    let ss = s[fn_end..].to_string();
    let command_end_pos = fn_end
        + match ss.find(";") {
            Some(pos) => pos,
            None => 5,
        };
    trim_command(s[fn_end..command_end_pos].to_string())
}

fn trim_command(cmd: String) -> String {
    let space_pos = cmd.find(" ").unwrap() + 1;
    let len = cmd.len();
    cmd[space_pos..len].into()
}

impl Display for TestInstance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} - {} '{}' - {}",
            self.name, self.test_type, self.expected_result, self.comment
        )
    }
}

fn get_test_type(line: &str) -> (TestType, usize) {
    match line.find("bm") {
        Some(pos) => (TestType::bm, pos),
        None => match line.find("am") {
            Some(ampos) => (TestType::am, ampos),
            None => panic!(),
        },
    }
}

fn get_id(s: &str) -> String {
    let id_pos = s.find("id").unwrap();
    let comment_pos = match s.find("c0") {
        None => s.len(),
        Some(cp) => {
            if cp < id_pos {
                s.len()
            } else {
                cp
            }
        }
    };
    trim_id(s[id_pos..comment_pos].to_string())
}

fn trim_id(id: String) -> String {
    let start = id.find("\"").unwrap() + 1;
    let ss = id[start..].to_string();
    let end = start + match ss.find("\"") {
        None => ss.len(),
        Some(pos) => pos
    };
    id[start..end].into()
}

fn get_result_scores(comment: &String, expected_move: &String) -> Vec<ResultScore> {
    let mut s = comment.clone();
    let mut r = Vec::new();

    loop {
        let i = match s.find(",") {
            None => break,
            Some(pos) => pos
        };

        r.push(get_result_score(&s[..i]));
        s = s[i+1..].to_string();
    }

    if r.len() == 0 {
        if s.len() == 0 {
            r.push(ResultScore { m: expected_move.to_string(), score: 10 })
        } else {
            r.push(get_result_score(&s));
        }
    }

    r
}

fn get_result_score(s: &str) -> ResultScore {
    let e = s.find("=").unwrap();
    let m = s[..e].trim().to_string();
    let score = str::parse::<usize>(&s[e+1..]).unwrap();
    ResultScore { m, score }
}
