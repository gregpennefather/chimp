use std::{cell::OnceCell, time::Instant};

use log::{debug, info, trace};

use crate::{
    match_state::game_state::{self, GameState},
    move_generation::generate_moves_for_board,
    move_ordering::move_orderer::{MoveOrderer, MOVE_CACHE},
    r#move::Move,
    shared::{
        board_utils::{get_rank, index_from_coords},
        piece_type::PieceType,
        transposition_table::NodeType,
    }, board::board_rep::BoardRep,
};

const MAX_EXTENSIONS: u8 = 12;
pub const AB_MIN: i16 = -32767;
pub const AB_MAX: i16 = 32767;

use super::{move_orderer, ChimpEngine};

impl ChimpEngine {
    fn make(&mut self, game_state: GameState, m: Move) -> Option<GameState> {
        assert!(!m.is_empty());
        let (new_zorb, move_segments) = game_state.position.board.zorb_key_after_move(m);

        let lookup_result = self.position_cache.lookup(new_zorb);

        let position = match lookup_result {
            Some(r) => r,
            None => {
                let new_pos = game_state.position.apply_segments(move_segments, new_zorb);
                self.position_cache.record(new_zorb, new_pos);
                new_pos
            }
        };

        game_state.after_position(position, m)
    }

    pub fn iterative_deepening<CutoffFunc>(
        &mut self,
        cutoff: &CutoffFunc,
        priority_line: Vec<Move>,
    ) -> Vec<Move>
    where
        CutoffFunc: Fn() -> bool,
    {
        let mut depth = priority_line.len() as u8;

        let mut output = (0, priority_line);

        let timer = Instant::now();

        while !cutoff() && depth < 12 {
            depth += 1;

            let result = self.alpha_beta_search(
                self.current_game_state,
                cutoff,
                depth,
                0,
                AB_MIN,
                AB_MAX,
                &output.1,
                0,
            );

            if result.1.len() == 0 {
                break;
            }

            output = result;

            let dur = timer.elapsed();
            debug!("{depth}: {} \t{:?} \t {:?}", output.0, dur, output.1);
        }
        output.1
    }

    pub fn alpha_beta_search<CutoffFunc>(
        &mut self,
        game_state: GameState,
        cutoff: &CutoffFunc,
        depth: u8,
        ply: u8,
        mut alpha: i16,
        beta: i16,
        priority_line: &Vec<Move>,
        total_extensions: u8,
    ) -> (i16, Vec<Move>)
    where
        CutoffFunc: Fn() -> bool,
    {
        // If we have an entry in the TT table shortcut the search using its value
        let tt_entry =
            self.transposition_table
                .lookup(game_state.position.board.zorb_key, depth, alpha, beta);
        if tt_entry != None {
            let (eval, m) = tt_entry.unwrap();
            return (eval, vec![m]);
        }

        // If we're at depth 0 we're on a leaf node so store its value in the TT table and return
        if depth == 0 {
            let q_result = self.quiescence_search(game_state, cutoff, alpha, beta);
            self.transposition_table.record(
                game_state.position.board.zorb_key,
                depth,
                q_result.0,
                crate::shared::transposition_table::NodeType::PVNode,
                Some(game_state.entry_move),
            );

            return q_result;
        }

        if cutoff() {
            return (AB_MIN, vec![]);
        }

        // We need to evaluate this node
        let mut node_type = NodeType::AllNode;
        let mut line = vec![];
        let mut has_legal_move = false;

        let pv = priority_line.iter().nth(ply as usize);
        let hm = self
            .transposition_table
            .get_move(game_state.position.board.zorb_key);
        let board = game_state.position.board;
        let move_orderer = MoveOrderer::new(pv, hm, game_state.position, self.killer_store.get_ply(ply as usize));

        let mut move_index = -1;
        let legal_moves = get_moves(board);
        for m in move_orderer {
            move_index += 1;
            if !legal_moves.contains(&m) {
                println!("position is {}", game_state.to_fen());
                println!("pv at ply {ply} is {pv:?}");
                println!("hm at ply {ply} is {hm:?}");
                panic!("move {m:?} not in legal moves list {legal_moves:?}");
            }
            let new_game_state = match self.make(game_state, m) {
                Some(s) => s,
                None => {
                    debug!(
                        "Illegal move {m} from {}",
                        game_state.position.board.to_fen()
                    );
                    continue;
                }
            };
            has_legal_move = true;

            let extension = get_extensions(new_game_state, m, total_extensions);

            // Reduce late moves if possible
            let shallow_eval = if extension == 0 && depth > 2 && move_index > 3 && m.is_quiet() {
                let (opponent_val, moves) = self.alpha_beta_search(
                    new_game_state,
                    cutoff,
                    depth - 2,
                    ply + 1,
                    -beta,
                    -alpha,
                    priority_line,
                    total_extensions + extension,
                );
                let val = opponent_val * -1;
                // if val > alpha then we need a full search, else use this shallow eval
                if val > alpha {
                    None
                } else {
                    Some((val, moves))
                }
            } else {
                None
            };

            let (val, moves) = match shallow_eval {
                Some(eval_pair) => eval_pair,
                None => {
                    let (opponent_val, moves) = self.alpha_beta_search(
                        new_game_state,
                        cutoff,
                        depth - 1 + extension,
                        ply + 1,
                        -beta,
                        -alpha,
                        priority_line,
                        total_extensions + extension,
                    );
                    (opponent_val * -1, moves)
                }
            };

            if line.len() != 0 && cutoff() {
                break;
            }

            // Fail high, this move is too good and must be cut
            if val >= beta {
                self.transposition_table.record(
                    new_game_state.position.board.zorb_key,
                    depth,
                    beta,
                    NodeType::CutNode,
                    Some(m),
                );
                if !m.is_capture() {
                    self.killer_store.set(ply as usize, m);
                }
                return (beta, vec![]);
            }

            // This move is inside the alpha-beta window and is thus considered a PV node
            if val > alpha {
                node_type = NodeType::PVNode;
                alpha = val;
                line = vec![m];
                line.extend(moves);
            }
        }

        if !has_legal_move {
            alpha = if game_state.position.black_in_check | game_state.position.white_in_check {
                AB_MIN
            } else {
                0
            };
        } else {
            self.transposition_table.record(
                game_state.position.board.zorb_key,
                depth,
                alpha,
                node_type,
                if node_type == NodeType::AllNode {
                    None
                } else {
                    Some(line[0])
                },
            );
        }
        return (alpha, line);
    }

    pub fn quiescence_search<CutoffFunc>(
        &mut self,
        game_state: GameState,
        cutoff: &CutoffFunc,
        mut alpha: i16,
        beta: i16,
    ) -> (i16, Vec<Move>)
    where
        CutoffFunc: Fn() -> bool,
    {
        if cutoff() {
            return (0, vec![]); // TODO: Confirm this
        }

        if game_state.subjective_eval >= beta {
            return (beta, vec![]);
        }

        if game_state.subjective_eval > alpha {
            alpha = game_state.subjective_eval;
        }

        let mut line = vec![];
        let moves = get_moves(game_state.position.board);
        for m in moves {
            if m.is_quiet() {
                continue;
            }

            let new_game_state = match self.make(game_state, m) {
                Some(gs) => gs,
                None => continue,
            };

            let (opponent_val, search_line) =
                self.quiescence_search(new_game_state, cutoff, -beta, -alpha);
            let val = opponent_val * -1;

            if val >= beta {
                return (beta, vec![]);
            }

            if val > alpha {
                alpha = val;
                line = vec![m];
                line.extend(search_line)
            }
        }

        (alpha, line)
    }
}

fn get_moves(board: BoardRep) -> Vec<Move> {
    MOVE_CACHE.lock().unwrap().get_moves(board)
}

fn get_extensions(new_state: GameState, test_move: Move, total_extensions: u8) -> u8 {
    if total_extensions >= MAX_EXTENSIONS {
        return 0;
    }
    if new_state.position.black_in_check || new_state.position.white_in_check {
        return 1;
    }

    if test_move.is_promotion() {
        return 1;
    }

    let m_rank = get_rank(test_move.to());
    if test_move.piece_type() == PieceType::Pawn && (m_rank == 1 || m_rank == 6) {
        return 1;
    }
    return 0;
}
