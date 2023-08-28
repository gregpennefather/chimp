use std::str::SplitAsciiWhitespace;

use log::info;

use crate::{match_state::game_state::GameState, r#move::Move, POSITION_TRANSPOSITION_TABLE};

pub mod perft;

pub struct ChimpEngine {
    pub current_game_state: GameState,
    moves: Vec<Move>
}

impl ChimpEngine {
    pub fn new() -> Self {
        let current_game_state = GameState::default();
        let moves = Vec::new();
        Self { current_game_state, moves }
    }

    pub fn position(&mut self, mut split_string: SplitAsciiWhitespace<'_>) {
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
                self.reset_state();
                return;
            }
        }

        let mut move_index = 0;
        while let Some(move_uci) = split_string.next() {
            if move_index >= self.moves.len() {
                self.add_move(move_uci);
            } else if !self.moves[move_index].uci().eq(move_uci) {
                self.add_move(move_uci);
            }
            move_index += 1;
        }
        // if (self.current_node.metrics.black_in_check || self.current_node.metrics.white_in_check) {
        //     let psudolegal_moves = self.current_node.position.generate_psudolegals();
        //     let legal_moves = generate_legal_moves(
        //         &self.current_node,
        //         psudolegal_moves,
        //         &mut self.lookup_table,
        //         self.current_node.position.flags.is_black_turn(),
        //     );
        //     if (legal_moves.len() == 0) {
        //         println!("Checkmate!");
        //     }
        // }
    }

    pub fn go(&self) -> Move {
        self.current_game_state.get_moves()[0]
    }

    fn reset_state(&mut self) {
        self.current_game_state = GameState::default();
        POSITION_TRANSPOSITION_TABLE.write().unwrap().clear();
    }

    fn add_move(&mut self, move_uci: &str) {
        let m = self.current_game_state.move_from_uci(move_uci);
        self.current_game_state = self.current_game_state.make(m);
        self.moves.push(m);
    }
}