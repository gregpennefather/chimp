use std::{str::SplitAsciiWhitespace, time::Duration, time::Instant};

use log::{debug, error, info, trace};

use crate::{
    match_state::game_state::{self, GameState, MatchResultState},
    r#move::Move,
    shared::piece_type::PieceType,
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

    pub fn from_position(fen: String) -> Self {
        let current_game_state = GameState::new(fen);
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

    pub fn go(&self, wtime: i32, btime: i32, winc: i32, binc: i32) -> Move {
        let ms = if (self.current_game_state.position.black_turn) {
            binc + i32::min(15000, btime / 10)
        } else {
            winc + i32::min(15000, wtime / 10)
        };
        info!(
            "{}: go {} {ms:?}",
            self.moves.len(),
            if self.current_game_state.position.black_turn {
                "black"
            } else {
                "white"
            }
        );
        let timeout = Instant::now()
            .checked_add(Duration::from_millis(ms as u64))
            .unwrap();
        iterative_deepening(self.current_game_state.clone(), timeout)
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

pub fn iterative_deepening(game_state: GameState, timeout: Instant) -> Move {
    let mut depth = 0;

    let mut output_r = (
        Move::default(),
        if game_state.position.black_turn {
            i32::MAX
        } else {
            i32::MIN
        },
    );

    let t_time = Instant::now();

    let mut cur_time = Instant::now();
    while cur_time < timeout && depth < 20 {
        let mut alpha = i32::MIN;
        let mut beta = i32::MAX;
        // This should maybe be re-created through more intelligent move ordering
        // if game_state.position.black_turn {
        //     beta = output_r.1;
        // } else {
        //     alpha = output_r.1;
        // }
        depth += 1;
        info!(
            "depth: {depth} -> val : {:?} alpha: {alpha}, beta: {beta}",
            output_r
        );
        let r = ab_search(game_state.clone(), depth, timeout, alpha, beta).unwrap(); // Possible optimization with alpha + beta

        info!("depth: {depth} complete {:?}", t_time.elapsed());
        if Instant::now() > timeout {
            if (!game_state.position.black_turn && r.1 > output_r.1)
                || (game_state.position.black_turn && r.1 < output_r.1)
            {
                info!("Premature exit - {:?} vs {r:?}", output_r);
                output_r = r;
            }
        } else {
            output_r = r;
        }
        cur_time = Instant::now();
    }
    let m = output_r.0;
    info!("go {m:?} (depth: {depth})\n");
    m
}

pub fn ab_search(
    game_state: GameState,
    depth: u8,
    timeout: Instant,
    mut alpha: i32, // maximize
    mut beta: i32,
) -> Result<(Move, i32), String> {
    if depth == 0 || game_state.result_state() != MatchResultState::Active {
        return match game_state.result_state() {
            MatchResultState::Draw => Ok((Move::default(), 0)),
            MatchResultState::WhiteVictory => Ok((Move::default(), i32::MAX - 1)),
            MatchResultState::BlackVictory => Ok((Move::default(), i32::MIN + 1)),
            _ => Ok((Move::default(), game_state.position.eval)),
        };
    }

    let now = Instant::now();
    if now > timeout {
        debug!(
            "game_state: {} timeout at depth {depth}",
            game_state.to_fen()
        );
        return Ok((
            Move::default(),
            if game_state.position.black_turn {
                i32::MIN + 1
            } else {
                i32::MAX - 1
            },
        ));
    }

    let legal_moves = game_state.get_legal_moves();

    let mut chosen_move = Move::default();
    let mut chosen_move_eval = if !game_state.position.black_turn {
        i32::MIN
    } else {
        i32::MAX
    };

    for &test_move in &legal_moves {
        let new_state = game_state.make(test_move);
        let (m, result_eval) = match ab_search(new_state, depth - 1, timeout, alpha, beta) {
            Ok(r) => r,
            Err(e) => {
                error!("{e}");
                panic!("{e}")
            }
        };
        if !game_state.position.black_turn {
            if (result_eval > 1000) {
                debug!("{depth}:killer white move: {test_move:?}:{result_eval:?}");
            }

            if result_eval > chosen_move_eval {
                trace!("{depth}:chosen move change: {test_move:?}{result_eval:?} > {chosen_move:?}:{chosen_move_eval:?}");
                chosen_move = test_move;
                chosen_move_eval = result_eval;
            }
            alpha = i32::max(alpha, chosen_move_eval);
            if beta <= alpha {
                break;
            }
        } else {
            if result_eval < chosen_move_eval {
                trace!("{depth}:chosen move change: {test_move:?}{result_eval:?} < {chosen_move:?}:{chosen_move_eval:?}");
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
        println!("legal_moves: {legal_moves:?}");
        panic!(
            "empty chosen move! depth:{depth}:value{} => {}",
            chosen_move_eval,
            game_state.to_fen()
        );
    }

    if (chosen_move_eval == i32::MAX || chosen_move_eval == i32::MIN) {
        debug!(
            "chosen_move_eval {chosen_move_eval} at {depth} for black:{} => {chosen_move:?}",
            game_state.position.black_turn
        );
    }

    Ok((chosen_move, chosen_move_eval))
}
