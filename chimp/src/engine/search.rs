use crate::board::state::BoardState;

pub fn search(board_state: BoardState) -> (u16, Vec<u16>) {
    let metrics = board_state.generate_metrics();
    let pl_moves = board_state.generate_psudolegals();
    let move_options = board_state.generate_legal_moves(&pl_moves, &metrics);
    let mut best_move_index = 0;
    let mut best_move_eval = f32::MIN;
    for i in 0..move_options.len() {
        let new_board_state = board_state.apply_move(move_options[i].0);
        let new_metrics = board_state.generate_metrics();
        let eval = new_board_state.evaluate(&new_metrics);
        if eval > best_move_eval {
            best_move_index = i;
            best_move_eval = eval;
        }
    }
    (move_options[best_move_index].0, pl_moves)
}
