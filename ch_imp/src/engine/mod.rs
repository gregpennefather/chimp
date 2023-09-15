use log::{debug, error, info, trace};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::{str::SplitAsciiWhitespace, time::Duration, time::Instant};

use crate::shared::cache::{MovesCache, PositionCache};

use crate::shared::transposition_table::TranspositionTable;
use crate::{
    match_state::game_state::{GameState, MatchResultState},
    r#move::Move,
};

pub mod move_orderer;
pub mod perft;
pub mod san;
pub mod search;

const MAX_EXTENSIONS: i8 = 8;
const WHITE_WIN_THRESHOLD: i32 = i32::MAX - 5;
const BLACK_WIN_THRESHOLD: i32 = i32::MIN + 5;

pub struct ChimpEngine {
    pub current_game_state: GameState,
    moves: Vec<Move>,
    previous_best_line: Vec<Move>,
    pub(super) transposition_table: TranspositionTable,
    pub position_cache: PositionCache,
    pub moves_cache: MovesCache,
}

impl ChimpEngine {
    pub fn new() -> Self {
        let current_game_state = GameState::default();
        let moves = Vec::new();
        Self {
            current_game_state,
            moves,
            previous_best_line: Vec::new(),
            transposition_table: TranspositionTable::new(),
            position_cache: PositionCache::new(),
            moves_cache: MovesCache::new(),
        }
    }

    pub fn from_position(fen: String) -> Self {
        let current_game_state = GameState::new(fen);
        let moves = Vec::new();
        Self {
            current_game_state,
            moves,
            previous_best_line: Vec::new(),
            transposition_table: TranspositionTable::new(),
            position_cache: PositionCache::new(),
            moves_cache: MovesCache::new(),
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
    }

    pub fn go(&mut self, wtime: i32, btime: i32, winc: i32, binc: i32) -> (Move, Option<Move>) {
        let ms = if winc == -1 || binc == -1 {
            info!("go movetime {wtime}");
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

        let previous_line = if self.previous_best_line.len() > 0
            && self.moves.iter().last() == self.previous_best_line.iter().nth(0)
        {
            let num_priority_moves = self.previous_best_line.len();
            self.previous_best_line[1..num_priority_moves].to_vec()
        } else {
            Vec::new()
        };

        let cutoff = || Instant::now() > timeout;

        let eval_result = self.iterative_deepening(&cutoff, previous_line);

        if (eval_result.len()) == 0 {
            return (Move::default(), None);
        }

        let num_priority_moves = eval_result.len();
        self.previous_best_line = eval_result[1..num_priority_moves].to_vec();

        info!("go {:?} path:{:?}\n", eval_result[0], eval_result);

        let ponder = if eval_result.len() > 1 {
            Some(eval_result[1])
        } else {
            None
        };

        (eval_result[0], ponder)
    }

    // pub fn go_post_ponder(
    //     &self,
    //     wtime: i32,
    //     btime: i32,
    //     winc: i32,
    //     binc: i32,
    //     ponder_moves: Vec<Move>,
    // ) -> (Move, Option<Move>) {
    //     let ms = if ponder_moves.len() >= 5 {
    //         if self.black_turn() {
    //             binc / 2
    //         } else {
    //             winc / 2
    //         }
    //     } else if self.current_game_state.position.board.black_turn {
    //         if btime < binc {
    //             binc / 3 * 2
    //         } else {
    //             i32::max(binc - 50, i32::min(btime / 10, binc + (btime / 12)))
    //         }
    //     } else {
    //         if wtime < winc {
    //             winc / 3 * 2
    //         } else {
    //             i32::max(winc - 50, i32::min(wtime / 10, winc + (wtime / 12)))
    //         }
    //     };

    //     info!(
    //         "{}: go postponder {} {wtime} {btime} {winc} {binc} => {ms:?}",
    //         self.moves.len(),
    //         if self.current_game_state.position.board.black_turn {
    //             "black"
    //         } else {
    //             "white"
    //         }
    //     );
    //     let timeout = Instant::now()
    //         .checked_add(Duration::from_millis(ms as u64))
    //         .unwrap();
    //     let (m, ponder, _c) =
    //         iterative_deepening(self.current_game_state.clone(), timeout, ponder_moves);
    //     (m, ponder)
    // }

    fn reset_state(&mut self) {
        self.current_game_state = GameState::default();
    }

    fn add_move(&mut self, move_uci: &str) {
        let m = self.current_game_state.move_from_uci(move_uci);
        self.current_game_state = self.current_game_state.make(m);
        self.moves.push(m);
    }

    // pub fn ponder_miss(&mut self) {
    //     let binding = Arc::clone(&PONDERING);
    //     let mut mut_pondering = binding.lock().unwrap();
    //     *mut_pondering = false;
    //     let len = self.moves.len();
    //     let pre_ponder_moves = self.moves[0..len - 1].to_vec();
    //     self.current_game_state = GameState::default();
    //     self.moves = vec![];
    //     for m in pre_ponder_moves {
    //         self.add_move(&m.uci());
    //     }
    // }

    // pub fn ponder_hit(&self) {
    //     let binding = Arc::clone(&PONDERING);
    //     let mut mut_pondering = binding.lock().unwrap();
    //     *mut_pondering = false;
    // }

    // pub fn ponder(&mut self) -> JoinHandle<Vec<Move>> {
    //     let ponder_state = self.current_game_state.clone();
    //     let pondering = Arc::clone(&PONDERING);
    //     let mut thread_pondering = pondering.lock().unwrap();
    //     *thread_pondering = true;

    //     thread::spawn(move || {
    //         let ponder_outcome = ponder_deepening(ponder_state);
    //         info!("info ponder_outcome: {ponder_outcome:?}");
    //         ponder_outcome
    //     })
    // }
}
