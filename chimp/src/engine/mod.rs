use crate::board::r#move::MoveFunctions;
use crate::board::state::BoardState;
use crate::engine::search::search;
use log::info;
use std::str::SplitAsciiWhitespace;
mod evaluate;
mod search;

pub struct ChimpEngine {
    board_state: BoardState,
    move_history: Vec<String>,
}

impl ChimpEngine {
    pub fn new() -> Self {
        Self {
            board_state: BoardState::default(),
            move_history: Vec::new(),
        }
    }

    pub fn position(&mut self, mut split_string: SplitAsciiWhitespace<'_>) {
        info!(target:"app:chimp", "Start position command");
        let first_word = split_string.next();
        match first_word {
            Some(word) => {
                if !word.eq_ignore_ascii_case("startpos") {
                    panic!("unexpected word");
                }
            }
            None => panic!("unexpected lack of word"),
        }

        let second_word = split_string.next();
        match second_word {
            Some(word) => {
                if !word.eq_ignore_ascii_case("moves") {
                    panic!("unexpected word");
                }
            }
            None => {
                info!(target:"app:chimp", "Loading initial boardstate");
                self.load_initial_board_state();
                return;
            }
        }

        info!(target:"app:chimp", "Current moves: {:?}", self.move_history);
        let mut move_index = 0;
        while let Some(word) = split_string.next() {
            if move_index >= self.move_history.len() {
                self.add_move(word);
            } else if !self.move_history[move_index].eq(word) {
                self.add_move(word);
            }
            move_index += 1;
        }

        info!(target:"app:chimp", "position command resulted in fen: {}", self.board_state.to_fen());
    }

    pub fn go(&mut self, mut split_string: SplitAsciiWhitespace<'_>) -> String {
        let best_move = search(self.board_state);
        info!(target:"app:chimp", "bestmove {}", best_move.0.uci());
        best_move.0.uci()
    }

    pub fn go_uci_and_san(&mut self) -> (String, String) {
        let best_move = search(self.board_state);
        info!(target:"app:chimp", "bestmove {}", best_move.0.uci());
        (
            best_move.0.uci(),
            best_move.0.san(self.board_state, best_move.1)
        )
    }

    fn load_initial_board_state(&mut self) {
        self.board_state = BoardState::default();
        self.move_history = Vec::new();
    }

    fn add_move(&mut self, uci_move: &str) {
        info!(target:"app:chimp", "add_word {uci_move}");
        let m = self.board_state.move_from_string(uci_move);
        self.board_state = self.board_state.apply_move(m);
        self.move_history.push(uci_move.to_string());
    }
}
