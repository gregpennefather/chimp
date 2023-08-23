use std::collections::{HashMap, LinkedList};

use log::info;

use crate::{
    board::{position_node::PositionNode, r#move::Move, state::BoardStateFlagsTrait},
    util::{t_table::PositionTranspositionTable, zorb_hash::ZorbSet},
};

use super::move_table::generate_nodes;

pub fn search(
    node: PositionNode,
    lookup_table: &mut PositionTranspositionTable,
) -> Result<(Move, PositionNode), bool> {
    let r = ab_search(
        &node,
        !node.position.flags.is_black_turn(),
        4,
        f32::MIN,
        f32::MAX,
        lookup_table,
    );

    if r.unwrap().0 == Move::default() {
        return Err(!node.position.flags.is_black_turn());
    }
    r
}

fn ab_search(
    origin_node: &PositionNode,
    maximize: bool,
    depth: u8,
    mut alpha: f32, // maximize
    mut beta: f32,
    lookup_table: &mut PositionTranspositionTable,
) -> Result<(Move, PositionNode), bool> {
    if depth == 0 {
        return Ok((Move::default(), *origin_node));
    }

    let moves_and_nodes = generate_nodes(origin_node, lookup_table);

    let mut chosen_move_index = 0;
    let mut chosen_move_eval = if maximize { f32::MIN } else { f32::MAX };

    if moves_and_nodes.len() == 0 {
        return Ok((Move::default(), *origin_node));
    }

    for i in 0..moves_and_nodes.len() {
        let (next_move, next_node) = match ab_search(
            &moves_and_nodes[i].1,
            !maximize,
            depth - 1,
            alpha,
            beta,
            lookup_table,
        ) {
            Ok(r) => r,
            Err(e) => return Err(e),
        };

        if maximize {
            if next_node.evaluation > chosen_move_eval {
                chosen_move_index = i;
                chosen_move_eval = next_node.evaluation;
            }
            alpha = f32::max(alpha, chosen_move_eval);
            if beta <= alpha {
                break;
            }
        } else {
            if next_node.evaluation < chosen_move_eval {
                chosen_move_index = i;
                chosen_move_eval = next_node.evaluation;
            }
            beta = f32::min(beta, chosen_move_eval);
            if beta <= alpha {
                break;
            }
        }
    }

    Ok(moves_and_nodes[chosen_move_index])
}

// #[cfg(test)]
// mod test {
//     use crate::{board::{r#move::Move, r#move::CAPTURE, state::BoardState}, engine::search::ab_search};

//     use super::search;

//     #[test]
//     fn scenario_1() {
//         let bs = BoardState::from_fen(&"7k/8/8/p7/P7/1P2R3/8/7K w - - 0 1".into());
//         let metrics = bs.generate_metrics();
//         let (m, e, o_m, line) = ab_search(&bs, &metrics, true, 3);
//         let e = Move::new(19u8, 35u8, 0b0);
//         assert_eq!(m, e, "{} != {}. Line: {}", m.uci(), e.uci(), line)
//     }

//     #[test]
//     fn scenario_2() {
//         let bs = BoardState::from_fen(&"k7/8/8/8/8/8/p7/K7 w - - 0 1".into());
//         let metrics = bs.generate_metrics();
//         let (m, e, o_m, line) = search_rec(&bs, &metrics, true, 1);
//         let e = Move::new(7u8, 15u8, CAPTURE);
//         assert_eq!(m, e, "{} != {}. Line: {}", m.uci(), e.uci(), line)
//     }
// }
