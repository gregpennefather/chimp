use crate::board::{
    board_metrics::BoardMetrics,
    r#move::{Move, MoveFunctions},
    state::BoardState,
};

pub fn search(board_state: BoardState) -> (Move, f32, Vec<Move>, String) {
    let metrics = board_state.generate_metrics();
    return search_rec(&board_state, &metrics, true, 0, 3);
}

fn search_rec(
    board_state: &BoardState,
    metrics: &BoardMetrics,
    friendly: bool,
    depth: u8,
    max_depth: u8,
) -> (Move, f32, Vec<Move>, String) {
    if depth == max_depth {
        let sign = if friendly { 1.0 } else { -1.0 };
        return (
            Move::default(),
            sign * board_state.evaluate(&metrics),
            Vec::new(),
            "".into(),
        );
    }
    let pl_moves = board_state.generate_psudolegals();
    let move_options = board_state.generate_legal_moves(&pl_moves, &metrics);

    let mut chosen_move_index = 0;
    let mut chosen_move_eval = if friendly { f32::MIN } else { f32::MAX };
    let mut chosen_line = "".into();

    if (move_options.len() == 0) {
        return (Move::default(), chosen_move_eval, Vec::new(), "".into());
    }

    for i in 0..move_options.len() {
        let (m, new_board_state, new_metrics) = &move_options[i];
        let (s_m, eval, o_m, line) = search_rec(
            new_board_state,
            &new_metrics,
            !friendly,
            depth + 1,
            max_depth,
        );

        if friendly {
            if eval > chosen_move_eval {
                chosen_move_index = i;
                chosen_move_eval = eval;
                chosen_line = line;
            }
        } else {
            if eval < chosen_move_eval {
                chosen_move_index = i;
                chosen_move_eval = eval;
                chosen_line = line;
            }
        }
    }
    let mut other_moves = Vec::new();
    for i in 0..move_options.len() {
        if i != chosen_move_index {
            other_moves.push(move_options[i].0)
        }
    }
    (
        move_options[chosen_move_index].0,
        chosen_move_eval,
        other_moves,
        format!(
            "-> {} {}",
            move_options[chosen_move_index].0.uci(),
            chosen_line
        ),
    )
}

#[cfg(test)]
mod test {
    use crate::{
        board::{
            r#move::Move,
            r#move::{MoveFunctions, CAPTURE},
            state::BoardState,
        },
        engine::search::search_rec,
    };

    use super::search;

    #[test]
    fn scenario_1() {
        let bs = BoardState::from_fen(&"7k/8/8/p7/P7/1P2R3/8/7K w - - 0 1".into());
        let metrics = bs.generate_metrics();
        let (m, e, o_m, line) = search_rec(&bs, &metrics, true, 0, 3);
        let e = Move::new(19u8, 35u8, 0b0);
        assert_eq!(m, e, "{} != {}. Line: {}", m.uci(), e.uci(), line)
    }

    #[test]
    fn scenario_2() {
        let bs = BoardState::from_fen(&"k7/8/8/8/8/8/p7/K7 w - - 0 1".into());
        let metrics = bs.generate_metrics();
        let (m, e, o_m, line) = search_rec(&bs, &metrics, true, 0, 1);
        let e = Move::new(7u8, 15u8, CAPTURE);
        assert_eq!(m, e, "{} != {}. Line: {}", m.uci(), e.uci(), line)
    }
}
