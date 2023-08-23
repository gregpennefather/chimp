use crate::board::position_node::PositionNode;
use crate::board::state::{BoardState, BoardStateFlagsTrait};
use crate::engine::move_table::*;
use crate::engine::search::search;
use crate::util::t_table::PositionTranspositionTable;
use crate::util::zorb_hash::ZorbSet;
use log::info;
use std::collections::HashMap;
use std::str::SplitAsciiWhitespace;
mod evaluate;
pub mod move_table;
mod search;

pub struct ChimpEngine {
    current_node: PositionNode,
    move_history: Vec<String>,
    lookup_table: PositionTranspositionTable,
    zorb_set: ZorbSet,
}

impl ChimpEngine {
    pub fn new() -> Self {
        let zorb_set = ZorbSet::new();
        Self {
            current_node: PositionNode::default(),
            move_history: Vec::new(),
            lookup_table: PositionTranspositionTable::new(zorb_set),
            zorb_set,
        }
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
                self.load_initial_board_state();
                return;
            }
        }

        let mut move_index = 0;
        while let Some(word) = split_string.next() {
            if move_index >= self.move_history.len() {
                self.add_move(word);
            } else if !self.move_history[move_index].eq(word) {
                self.add_move(word);
            }
            move_index += 1;
        }
        if (self.current_node.metrics.black_in_check || self.current_node.metrics.white_in_check) {
            let psudolegal_moves = self.current_node.position.generate_psudolegals();
            let legal_moves = generate_legal_moves(
                &self.current_node,
                psudolegal_moves,
                &mut self.lookup_table,
                self.current_node.position.flags.is_black_turn(),
            );
            if (legal_moves.len() == 0) {
                println!("Checkmate!");
            }
        }
    }

    pub fn go(&mut self, mut split_string: SplitAsciiWhitespace<'_>) -> String {
        match (search(self.current_node, &mut self.lookup_table)) {
            Ok(best_move) => {
                info!(target:"app:chimp", "bestmove {}", best_move.0.uci());
                best_move.0.uci()
            }
            Err(e) => {
                if e {
                    "Black Wins!".into()
                } else {
                    "White Wins! ".into()
                }
            }
        }
    }

    pub fn go_uci_and_san(&mut self) -> (String, String) {
        let result = search(self.current_node, &mut self.lookup_table);
        let legal_nodes = generate_nodes(&self.current_node, &mut self.lookup_table);
        let mut legal_moves = Vec::new();
        for node in legal_nodes {
            legal_moves.push(node.0);
        }
        match (result) {
            Ok(best_move) => {
                info!(
                    "{}. {} + {} -> {} : {}",
                    self.move_history.len() + 1,
                    self.current_node.position.to_fen(),
                    best_move.0.uci(),
                    best_move.1.position.to_fen(),
                    best_move.1.evaluation
                );
                (
                    best_move.0.uci(),
                    best_move.0.san(self.current_node.position, legal_moves),
                )
            }
            Err(e) => {
                if e {
                    ("Black Wins!".into(), "".into())
                } else {
                    ("White Wins!".into(), "".into())
                }
            }
        }
    }

    fn load_initial_board_state(&mut self) {
        let position = BoardState::from_fen(&"rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1".into());
        let position_zorb = self.zorb_set.hash(position);
        let metrics = position.generate_metrics();
        self.current_node = PositionNode {
            position,
            position_zorb,
            metrics,
            evaluation: 0.0,
        };
        self.move_history = Vec::new();
    }

    fn add_move(&mut self, uci_move: &str) {
        let m = self.current_node.position.move_from_string(uci_move);

        self.current_node = match apply_or_get_move(&self.current_node, m, &mut self.lookup_table) {
            Ok(n) => n,
            Err(e) => panic!("{}", e),
        };

        self.move_history.push(uci_move.to_string());
    }
}
