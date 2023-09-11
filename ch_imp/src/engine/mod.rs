use log::{debug, error, info, trace};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::{str::SplitAsciiWhitespace, time::Duration, time::Instant};

use crate::shared::board_utils::get_rank;
use crate::shared::piece_type::PieceType;
use crate::{
    match_state::game_state::{GameState, MatchResultState},
    r#move::Move,
    shared::transposition_table::clear_tables,
    POSITION_TRANSPOSITION_TABLE,
};
use crate::{PONDERING, PONDERING_RESULT};

pub mod move_orderer;
pub mod perft;
pub mod san;

const MAX_EXTENSIONS: i8 = 8;
const WHITE_WIN_THRESHOLD: i32 = i32::MAX - 5;
const BLACK_WIN_THRESHOLD: i32 = i32::MIN + 5;

pub struct ChimpEngine {
    pub current_game_state: GameState,
    moves: Vec<Move>,
    previous_best_line: Vec<Move>
}

impl ChimpEngine {
    pub fn new() -> Self {
        clear_tables();
        let current_game_state = GameState::default();
        let moves = Vec::new();
        Self {
            current_game_state,
            moves,
            previous_best_line: Vec::new()
        }
    }

    pub fn from_position(fen: String) -> Self {
        let current_game_state = GameState::new(fen);
        let moves = Vec::new();
        Self {
            current_game_state,
            moves,
            previous_best_line: Vec::new()
        }
    }

    pub fn black_turn(&self) -> bool {
        self.current_game_state.position.board.black_turn
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

    pub fn go(&mut self, wtime: i32, btime: i32, winc: i32, binc: i32) -> (Move, Option<Move>) {
        let ms = if winc == -1 || binc == -1 {
            info!("go movetime 10000");
            wtime
        } else if self.current_game_state.position.board.black_turn {
            if btime < binc {
                binc / 3 * 2
            } else {
                i32::max(binc - 50, i32::min(btime / 10, binc + (btime / 12)))
            }
        } else {
            if wtime < winc {
                winc / 3 * 2
            } else {
                i32::max(winc - 50, i32::min(wtime / 10, winc + (wtime / 12)))
            }
        };
        info!(
            "{}: go {} {wtime} {btime} {winc} {binc} => {ms:?}",
            self.moves.len(),
            if self.current_game_state.position.board.black_turn {
                "black"
            } else {
                "white"
            }
        );
        let timeout = Instant::now()
            .checked_add(Duration::from_millis(ms as u64))
            .unwrap();

        let previous_line = if self.previous_best_line.len() > 0 && self.moves.iter().last() == self.previous_best_line.iter().nth(0) {
            info!("line hit! : {:?}", self.previous_best_line);
            let num_priority_moves = self.previous_best_line.len();
            self.previous_best_line[1..num_priority_moves].to_vec()
        } else {
            info!("line miss! {:?} vs  {:?}", self.moves.iter().last(), self.previous_best_line);
            Vec::new()
        };

        let eval_result = iterative_deepening(self.current_game_state.clone(), timeout, previous_line);
        self.previous_best_line = eval_result.2;
        (eval_result.0, eval_result.1)
    }

    pub fn go_post_ponder(
        &self,
        wtime: i32,
        btime: i32,
        winc: i32,
        binc: i32,
        ponder_moves: Vec<Move>,
    ) -> (Move, Option<Move>) {
        let ms = if ponder_moves.len() >= 5 {
            if self.black_turn() {
                binc / 2
            } else {
                winc / 2
            }
        } else if self.current_game_state.position.board.black_turn {
            if btime < binc {
                binc / 3 * 2
            } else {
                i32::max(binc - 50, i32::min(btime / 10, binc + (btime / 12)))
            }
        } else {
            if wtime < winc {
                winc / 3 * 2
            } else {
                i32::max(winc - 50, i32::min(wtime / 10, winc + (wtime / 12)))
            }
        };

        info!(
            "{}: go postponder {} {wtime} {btime} {winc} {binc} => {ms:?}",
            self.moves.len(),
            if self.current_game_state.position.board.black_turn {
                "black"
            } else {
                "white"
            }
        );
        let timeout = Instant::now()
            .checked_add(Duration::from_millis(ms as u64))
            .unwrap();
        let (m,ponder,_c) = iterative_deepening(self.current_game_state.clone(), timeout, ponder_moves);
        (m,ponder)
    }

    fn reset_state(&mut self) {
        self.current_game_state = GameState::default();
        POSITION_TRANSPOSITION_TABLE.write().unwrap().clear();
    }

    fn add_move(&mut self, move_uci: &str) {
        let m = self.current_game_state.move_from_uci(move_uci);
        self.current_game_state = self.current_game_state.make(m).unwrap();
        self.moves.push(m);
    }

    pub fn ponder_miss(&mut self) {
        let binding = Arc::clone(&PONDERING);
        let mut mut_pondering = binding.lock().unwrap();
        *mut_pondering = false;
        let len = self.moves.len();
        let pre_ponder_moves = self.moves[0..len - 1].to_vec();
        self.current_game_state = GameState::default();
        self.moves = vec![];
        for m in pre_ponder_moves {
            self.add_move(&m.uci());
        }
    }

    pub fn ponder_hit(&self) {
        let binding = Arc::clone(&PONDERING);
        let mut mut_pondering = binding.lock().unwrap();
        *mut_pondering = false;
    }

    pub fn ponder(&mut self) -> JoinHandle<Vec<Move>> {
        let ponder_state = self.current_game_state.clone();
        let pondering = Arc::clone(&PONDERING);
        let mut thread_pondering = pondering.lock().unwrap();
        *thread_pondering = true;

        thread::spawn(move || {
            let ponder_outcome = ponder_deepening(ponder_state);
            info!("info ponder_outcome: {ponder_outcome:?}");
            ponder_outcome
        })
    }
}

pub fn iterative_deepening(
    game_state: GameState,
    timeout: Instant,
    pondered_moves: Vec<Move>,
) -> (Move, Option<Move>, Vec<Move>) {
    let mut depth = pondered_moves.len() as i8;

    let mut output_r = (
        Vec::<(Move, i32)>::new(),
        0,
        if game_state.position.board.black_turn {
            i32::MAX
        } else {
            i32::MIN
        },
    );
    let mut priority_moves = pondered_moves.clone();

    let t_time = Instant::now();

    let mut cur_time = Instant::now();
    while cur_time < timeout && depth < 10 {
        let alpha = i32::MIN;
        let beta = i32::MAX;

        depth += 1;

        let r = ab_search(
            &game_state,
            &priority_moves,
            depth,
            0,
            timeout,
            0,
            alpha,
            beta,
        )
        .unwrap();

        output_r = r;
        info!(
            "depth: {depth} complete {:?} => val : {:?}",
            t_time.elapsed(),
            output_r
        );

        if (!game_state.position.board.black_turn && output_r.2 > WHITE_WIN_THRESHOLD)
            || (game_state.position.board.black_turn && output_r.2 < BLACK_WIN_THRESHOLD)
        {
            break;
        }

        cur_time = Instant::now();

        priority_moves = output_r.0.iter().map(|&f| f.0).collect();
    }

    let m_history = output_r.0;
    // If the chosen_move_eval is equal to a max it means this branch will end in a mate
    if output_r.2 == i32::MAX || output_r.2 == i32::MIN {
        info!("Mate in {}: {:?}", m_history.len(), m_history)
    }

    if m_history.len() == 0 {
        return (Move::default(), None, Vec::new());
    }

    info!(
        "go {:?} (depth: {}) path:{:?}\n",
        m_history[0],
        depth - 1,
        m_history
    );
    if m_history.len() > 1 {
        let num_priority_moves = priority_moves.len();
        let slice = priority_moves[1..num_priority_moves].to_vec();
        (m_history[0].0, Some(m_history[1].0), slice)
    } else {
        (m_history[0].0, None, Vec::new())
    }
}

pub fn ponder_deepening(game_state: GameState) -> Vec<Move> {
    let mut depth = 0;

    let mut still_pondering = true;

    let mut output_r = (
        Vec::<(Move, i32)>::new(),
        0,
        if game_state.position.board.black_turn {
            i32::MAX
        } else {
            i32::MIN
        },
    );

    while still_pondering {
        let alpha = i32::MIN;
        let beta = i32::MAX;

        depth += 1;

        let priority_moves = output_r.0.iter().map(|&f| f.0).collect();

        output_r = ponder_search(&game_state, &priority_moves, depth, 0, 0, alpha, beta).unwrap();

        let pondering_arc = Arc::clone(&PONDERING);
        let pondering_lock = pondering_arc.lock().unwrap();
        still_pondering = *pondering_lock;
        drop(pondering_lock)
    }

    output_r.0.into_iter().map(|e| e.0).collect::<Vec<Move>>()
}

pub fn ab_search(
    game_state: &GameState,
    priority_moves: &Vec<Move>,
    depth: i8,
    ply: u8,
    timeout: Instant,
    total_extensions: i8,
    mut alpha: i32, // maximize
    mut beta: i32,
) -> Result<(Vec<(Move, i32)>, i32, i32), String> {
    if game_state.result_state != MatchResultState::Active {
        return match game_state.result_state {
            MatchResultState::WhiteVictory => Ok((vec![], i32::MAX - 1, i32::MAX - 1)),
            MatchResultState::BlackVictory => Ok((vec![], i32::MIN + 1, i32::MIN + 1)),
            _ => Ok((vec![], game_state.position.eval, game_state.position.eval)),
        };
    }

    if depth <= 0 {
        return quiescence_search(game_state, timeout, 0, alpha, beta);
    }

    let now: Instant = Instant::now();
    if now > timeout {
        return Ok(if game_state.position.board.black_turn {
            (vec![], i32::MIN + 1, i32::MIN + 1)
        } else {
            (vec![], i32::MAX - 1, i32::MAX - 1)
        });
    }

    let mut chosen_move = Move::default();
    let mut chosen_move_eval = if !game_state.position.board.black_turn {
        i32::MIN
    } else {
        i32::MAX
    };
    let mut next_node_eval = 0;
    let mut move_history = Vec::new();

    let mut ordered_moves = game_state.position.moves.clone();
    match priority_moves.iter().nth(ply as usize) {
        Some(r) => {
            ordered_moves.sort_by(|a: &Move, b| move_orderer::top_priority(a, b, &r));
        }
        None => {}
    }
    for move_index in 0..ordered_moves.len() {
        let test_move = ordered_moves[move_index];
        let new_state = match game_state.make(test_move) {
            Some(new_state) => new_state,
            None => continue,
        };
        let extensions: i8 = get_extensions(&new_state, test_move, total_extensions);

        // if move_index >= 5 {
        //     extensions -= 1; // Lower priority moves get a less deep search
        // }

        let mut full_search = true;
        let mut shallow_eval = (Vec::new(), 0, 0);

        // Attempt to reduce the search depth
        if extensions == 0 && depth >= 2 && move_index >= 4 && test_move.is_quiet() {
            full_search = false;
            shallow_eval = match ab_search(
                &new_state,
                &priority_moves,
                depth - 1 - 1,
                ply + 1,
                timeout,
                total_extensions + 0,
                alpha,
                beta,
            ) {
                Ok(r) => r,
                Err(e) => {
                    error!("{e}");
                    panic!("{e}")
                }
            };

            // If the shallow eval is better than our current move redo the search with full depth
            if !game_state.position.board.black_turn && shallow_eval.2 > chosen_move_eval {
                full_search = true;
            } else if shallow_eval.2 < chosen_move_eval {
                full_search = true;
            }
        }

        // if we dont need to redo a full search return the partial_search
        let (path, node_eval, result_eval) = if full_search {
            match ab_search(
                &new_state,
                &priority_moves,
                depth - 1 + extensions,
                ply + 1,
                timeout,
                total_extensions + extensions,
                alpha,
                beta,
            ) {
                Ok(r) => r,
                Err(e) => {
                    error!("{e}");
                    panic!("{e}")
                }
            }
        } else {
            shallow_eval
        };

        let now = Instant::now();
        if now > timeout {
            if !chosen_move.is_empty() {
                break;
            }
        }

        if !game_state.position.board.black_turn {
            if result_eval > chosen_move_eval {
                trace!("{depth}:chosen move change: {test_move:?}{result_eval:?} > {chosen_move:?}:{chosen_move_eval:?}");
                chosen_move = test_move;
                chosen_move_eval = result_eval;
                next_node_eval = node_eval;
                move_history = path;
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
                next_node_eval = node_eval;
                move_history = path;
            }
            beta = i32::min(beta, chosen_move_eval);
            if beta <= alpha {
                break;
            }
        }
    }

    // If we couldn't choose a move it means that none of the PL moves are actually legal so abandon this branch
    if chosen_move.is_empty() {
        trace!("No legal moves found at {}", game_state.to_fen());
        return Ok(if game_state.position.board.black_turn {
            (vec![], i32::MAX - 1, i32::MAX - 1)
        } else {
            (vec![], i32::MIN + 1, i32::MIN + 1)
        });
    }

    // If the chosen_move_eval is equal to a max it means this branch will end in a mate
    if chosen_move_eval == i32::MAX || chosen_move_eval == i32::MIN {
        debug!(
            "chosen_move_eval {chosen_move_eval} at {depth} for black:{} => {chosen_move:?}",
            game_state.position.board.black_turn
        );
    }

    let mut final_move_history = vec![(chosen_move, next_node_eval)];
    final_move_history.extend(move_history);

    Ok((
        final_move_history,
        game_state.position.eval,
        chosen_move_eval,
    ))
}

pub fn ponder_search(
    game_state: &GameState,
    priority_moves: &Vec<Move>,
    depth: i8,
    ply: u8,
    total_extensions: i8,
    mut alpha: i32, // maximize
    mut beta: i32,
) -> Result<(Vec<(Move, i32)>, i32, i32), String> {
    if depth <= 0 || game_state.result_state != MatchResultState::Active {
        return match game_state.result_state {
            MatchResultState::Draw => Ok((vec![], 0, 0)),
            MatchResultState::WhiteVictory => Ok((vec![], i32::MAX - 1, i32::MAX - 1)),
            MatchResultState::BlackVictory => Ok((vec![], i32::MIN + 1, i32::MIN + 1)),
            _ => Ok((vec![], game_state.position.eval, game_state.position.eval)),
        };
    }

    let pondering_arc = Arc::clone(&PONDERING);
    let pondering_lock = pondering_arc.lock().unwrap();
    let still_pondering = *pondering_lock;
    drop(pondering_lock);

    if !still_pondering {
        return Ok(if game_state.position.board.black_turn {
            (vec![], i32::MIN + 1, i32::MIN + 1)
        } else {
            (vec![], i32::MAX - 1, i32::MAX - 1)
        });
    }

    let mut chosen_move = Move::default();
    let mut chosen_move_eval = if !game_state.position.board.black_turn {
        i32::MIN
    } else {
        i32::MAX
    };
    let mut next_node_eval = 0;
    let mut move_history = Vec::new();

    let mut ordered_moves = game_state.position.moves.clone();
    match priority_moves.iter().nth(ply as usize) {
        Some(r) => {
            ordered_moves.sort_by(|a: &Move, b| move_orderer::top_priority(a, b, &r));
        }
        None => {}
    }
    for move_index in 0..ordered_moves.len() {
        let test_move = ordered_moves[move_index];
        let new_state = match game_state.make(test_move) {
            Some(new_state) => new_state,
            None => continue,
        };
        let extensions: i8 = get_extensions(&new_state, test_move, total_extensions);

        // if move_index >= 5 {
        //     extensions -= 1; // Lower priority moves get a less deep search
        // }

        let (path, node_eval, result_eval) = match ponder_search(
            &new_state,
            &priority_moves,
            depth - 1 + extensions,
            ply + 1,
            total_extensions + extensions,
            alpha,
            beta,
        ) {
            Ok(r) => r,
            Err(e) => {
                error!("{e}");
                panic!("{e}")
            }
        };

        if !still_pondering {
            if !chosen_move.is_empty() {
                break;
            }
        }

        if !game_state.position.board.black_turn {
            if result_eval > chosen_move_eval {
                trace!("{depth}:chosen move change: {test_move:?}{result_eval:?} > {chosen_move:?}:{chosen_move_eval:?}");
                chosen_move = test_move;
                chosen_move_eval = result_eval;
                next_node_eval = node_eval;
                move_history = path;
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
                next_node_eval = node_eval;
                move_history = path;
            }
            beta = i32::min(beta, chosen_move_eval);
            if beta <= alpha {
                break;
            }
        }
    }

    // If we couldn't choose a move it means that none of the PL moves are actually legal so abandon this branch
    if chosen_move.is_empty() {
        trace!("No legal moves found at {}", game_state.to_fen());
        return Ok(if game_state.position.board.black_turn {
            (vec![], i32::MAX - 1, i32::MAX - 1)
        } else {
            (vec![], i32::MIN + 1, i32::MIN + 1)
        });
    }

    // If the chosen_move_eval is equal to a max it means this branch will end in a mate
    if chosen_move_eval == i32::MAX || chosen_move_eval == i32::MIN {
        debug!(
            "chosen_move_eval {chosen_move_eval} at {depth} for black:{} => {chosen_move:?}",
            game_state.position.board.black_turn
        );
    }

    let mut final_move_history = vec![(chosen_move, next_node_eval)];
    final_move_history.extend(move_history);

    Ok((
        final_move_history,
        game_state.position.eval,
        chosen_move_eval,
    ))
}

fn quiescence_search(
    game_state: &GameState,
    timeout: Instant,
    ply: u8,
    mut alpha: i32, // maximize
    mut beta: i32,
) -> Result<(Vec<(Move, i32)>, i32, i32), String> {

    // Dont force a bad capture
    if !game_state.position.board.black_turn {
        if game_state.position.eval < alpha {
            return Ok((vec![], alpha, alpha));
        }
    } else {
        if game_state.position.eval > beta {
            return Ok((vec![], beta, beta));
        }
    }

    let capture_moves: Vec<Move> = game_state
        .position
        .moves
        .clone()
        .into_iter()
        .filter(|m| !m.is_quiet())
        .collect();

    if capture_moves.len() == 0 {
        return Ok((vec![], game_state.position.eval, game_state.position.eval));
    }

    let now: Instant = Instant::now();
    if now > timeout {
        return Ok(if game_state.position.board.black_turn {
            (vec![], i32::MIN + 1, i32::MIN + 1)
        } else {
            (vec![], i32::MAX - 1, i32::MAX - 1)
        });
    }

    let mut chosen_move = Move::default();
    let mut chosen_move_eval = if !game_state.position.board.black_turn {
        i32::MIN
    } else {
        i32::MAX
    };
    let mut next_node_eval = 0;
    let mut move_history = Vec::new();

    for m in capture_moves {
        let new_state = match game_state.make(m) {
            Some(new_state) => new_state,
            None => continue,
        };

        let (path, node_eval, result_eval) =
            match quiescence_search(&new_state, timeout, ply + 1, alpha, beta) {
                Ok(r) => r,
                Err(e) => {
                    error!("{e}");
                    panic!("{e}")
                }
            };

        let now = Instant::now();
        if now > timeout {
            if !chosen_move.is_empty() {
                break;
            }
        }

        if !game_state.position.board.black_turn {
            if result_eval > chosen_move_eval {
                chosen_move = m;
                chosen_move_eval = result_eval;
                next_node_eval = node_eval;
                move_history = path;
            }
            alpha = i32::max(alpha, chosen_move_eval);
            if beta <= alpha {
                break;
            }
        } else {
            if result_eval < chosen_move_eval {
                chosen_move = m;
                chosen_move_eval = result_eval;
                next_node_eval = node_eval;
                move_history = path;
            }
            beta = i32::min(beta, chosen_move_eval);
            if beta <= alpha {
                break;
            }
        }
    }

    if chosen_move.is_empty() {
        trace!(
            "{ply}: no legal captures {} {}",
            game_state.to_fen(),
            game_state.position.eval
        );
        return Ok((vec![], game_state.position.eval, game_state.position.eval));
    }

    let mut final_move_history = vec![(chosen_move, next_node_eval)];
    final_move_history.extend(move_history);

    Ok((
        final_move_history,
        game_state.position.eval,
        chosen_move_eval,
    ))
}

fn get_extensions(new_state: &GameState, test_move: Move, total_extensions: i8) -> i8 {
    if total_extensions >= MAX_EXTENSIONS {
        return 0;
    }
    if new_state.position.black_in_check || new_state.position.white_in_check {
        return 1;
    }

    let m_rank = get_rank(test_move.to());
    if test_move.piece_type() == PieceType::Pawn && (m_rank == 1 || m_rank == 6) {
        return 1;
    }
    return 0;
}