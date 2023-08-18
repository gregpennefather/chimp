use crate::board::{state::BoardState, board_metrics::BoardMetrics};

pub fn search(board_state: BoardState) -> (u16, BoardMetrics) {
    let pl_metrics = board_state.generate_psudolegals();
    let move_options = board_state.generate_legal_moves(pl_metrics);
    let mut best_move_index = 0;
    let mut best_move_eval = i32::MIN;
    for i in 0..move_options.len() {
        let new_board_state = board_state.apply_move(move_options[i].0);
        let eval = new_board_state.evaluate(&move_options[i].2);
        if eval > best_move_eval {
            best_move_index = i;
            best_move_eval = eval;
        }
    }
    (move_options[best_move_index].0, move_options[best_move_index].2.clone())
}
