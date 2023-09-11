use crate::{
    match_state::game_state::{self, GameState},
    r#move::{move_generation::generate_moves_for_board, Move},
    shared::transposition_table::NodeType,
};

pub const AB_MIN: i16 = -32767;
pub const AB_MAX: i16 = 32767;

use super::ChimpEngine;

impl ChimpEngine {
    pub fn iterative_deepening<CutoffFunc>(
        &mut self,
        cutoff: &CutoffFunc,
        mut priority_line: Vec<Move>,
    ) -> (i16, Vec<Move>)
    where
        CutoffFunc: Fn() -> bool,
    {
        let mut depth = priority_line.len() as u8;

        let mut output = (0, vec![]);

        while !cutoff() && depth < 12 {
            depth += 1;

            // TODO: Add priority line
            output =
                self.alpha_beta_search(self.current_game_state, cutoff, depth, AB_MIN, AB_MAX);

            // TODO: Add mate detection
        }
        output
    }

    pub fn alpha_beta_search<CutoffFunc>(
        &mut self,
        game_state: GameState,
        cutoff: &CutoffFunc,
        depth: u8,
        mut alpha: i16,
        beta: i16,
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
                game_state.entry_move,
            );

            return q_result;
        }

        if cutoff() {
            panic!("cutoff!")
        }

        // We need to evaluate this node
        let mut node_type = NodeType::AllNode;
        let mut line = vec![];

        let legal_moves = generate_moves_for_board(game_state.position.board);
        for m in legal_moves {
            let new_game_state = match game_state.make(m) {
                Some(s) => s,
                None => continue,
            };

            let (opponent_val, moves) =
                self.alpha_beta_search(new_game_state, cutoff, depth - 1, -beta, -alpha);
            let val = opponent_val * -1;

            // Fail high, this move is too good and must be cut
            if val >= beta {
                self.transposition_table.record(
                    new_game_state.position.board.zorb_key,
                    depth,
                    beta,
                    NodeType::CutNode,
                    m,
                );
                return (beta, vec![game_state.entry_move]);
            }

            // This move is inside the alpha-beta window and is thus considered a PV node
            if val > alpha {
                node_type = NodeType::PVNode;
                alpha = val;
                line = vec![m];
                line.extend(moves)
            }
        }
        self.transposition_table.record(
            game_state.position.board.zorb_key,
            depth,
            alpha,
            node_type,
            game_state.entry_move,
        );

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
        let moves = generate_moves_for_board(game_state.position.board); // Add a quiet flag
        for m in moves {
            if m.is_quiet() {
                continue;
            }

            let new_game_state = match game_state.make(m) {
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
