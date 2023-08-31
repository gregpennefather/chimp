use std::str::SplitAsciiWhitespace;

use log::{error, info};

use crate::{
    match_state::game_state::{self, GameState},
    r#move::Move,
    HASH_HITS, HASH_MISSES, POSITION_TRANSPOSITION_TABLE,
};

pub mod perft;
pub mod san;

pub struct ChimpEngine {
    pub current_game_state: GameState,
    moves: Vec<Move>,
}

impl ChimpEngine {
    pub fn new() -> Self {
        let current_game_state = GameState::default();
        let moves = Vec::new();
        Self {
            current_game_state,
            moves,
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
        let (m, state) = ab_search(
            self.current_game_state.clone(),
            4,
            i32::MIN,
            i32::MAX,
        )
        .unwrap();
        println!("\ngo {m:?}");
        println!(
            "Table size: {}",
            POSITION_TRANSPOSITION_TABLE.read().unwrap().len()
        );
        let mut hits = HASH_HITS.lock().unwrap();
        let mut misses = HASH_MISSES.lock().unwrap();
        println!("Hits: {hits}");
        println!("Misses: {misses}");
        *hits = 0;
        *misses = 0;
        m
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

fn ab_search(
    game_state: GameState,
    depth: u8,
    mut alpha: i32, // maximize
    mut beta: i32,
) -> Result<(Move, i32), String> {
    if depth == 0 {
        return Ok((Move::default(), game_state.position.eval));
    }

    if !game_state.legal() {
        // Todo: The whole stalemate, checkmate, etc detection needs to be improved
        return Ok((Move::default(), if !game_state.position.black_turn { i32::MAX } else { i32::MIN }));
    }

    let moves = game_state.get_moves();

    let mut chosen_move = Move::default();
    let mut chosen_move_eval = if !game_state.position.black_turn { i32::MIN } else { i32::MAX };

    let mut next_position_is_illegal = 0;

    for test_move in moves
        .into_iter()
        .filter(|m| m.is_black() == game_state.position.black_turn)
    {
        let new_state = game_state.make(test_move);
        let (m, result_eval) = match ab_search(new_state, depth - 1, alpha, beta) {
            Ok(r) => r,
            Err(e) => {
                error!("{e}");
                panic!("{e}")
            }
        };

        if !game_state.position.black_turn {
            if result_eval > chosen_move_eval {
                chosen_move = test_move;
                chosen_move_eval = result_eval;
            }
            alpha = i32::max(alpha, chosen_move_eval);
            if beta <= alpha {
                break;
            }
        } else {
            if result_eval < chosen_move_eval {
                chosen_move = test_move;
                chosen_move_eval = result_eval;
            }
            beta = i32::min(beta, chosen_move_eval);
            if beta <= alpha {
                break;
            }
        }
    }

    if (chosen_move.is_empty()) {
        panic!(
            "empty chosen move! depth:{depth},invalids:{next_position_is_illegal} => {}",
            game_state.to_fen()
        );
    }

    Ok((chosen_move, chosen_move_eval))
}
