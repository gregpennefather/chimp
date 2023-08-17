use std::str::SplitAsciiWhitespace;
use rand::Rng;
use log::info;

use crate::board::{state::BoardState, move_utils::get_move_uci};

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
            },
            None => panic!("unexpected lack of word")
        }

        let second_word = split_string.next();
        match second_word {
            Some(word) => {
                if !word.eq_ignore_ascii_case("moves") {
                    panic!("unexpected word");
                }
            },
            None => {
                info!(target:"app:chimp", "Loading initial boardstate");
                self.load_initial_board_state();
                return
            }
        }

        info!(target:"app:chimp", "Current moves: {:?}", self.move_history);
        let mut moveIndex = 0;
        while let Some(word) = split_string.next() {
            if moveIndex >= self.move_history.len() {
                self.add_move(word);
            } else if !self.move_history[moveIndex].eq(word) {
                self.add_move(word);
            }
            moveIndex += 1;
        }

        info!(target:"app:chimp", "position command resulted in fen: {}", self.board_state.to_fen());
    }

    pub fn best_move(&mut self, mut split_string: SplitAsciiWhitespace<'_>) -> String {
        let moves = self.get_ordered_moves();
        let mut rng = rand::thread_rng();
        let r = rng.gen_range(0..moves.len());
        let selected_move = moves[r];
        info!(target:"app:chimp", "bestmove {} of {}", get_move_uci(selected_move), moves.len());
        get_move_uci(selected_move)
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

    fn get_ordered_moves(&self) -> Vec<u16> {
        let metrics = self.board_state.generate_psudolegals();
        let legal_moves = self.board_state.generate_legal_moves(metrics);
        let mut result = Vec::new();
        for m in legal_moves {
            result.push(m.0);
        }
        result
    }


}
