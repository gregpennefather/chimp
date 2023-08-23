use log::info;

use crate::{
    board::{
        move_generation::is_legal_castling, position_node::PositionNode, r#move::Move,
        state::BoardStateFlagsTrait,
    },
    util::t_table::MoveTableLookup,
};

pub fn generate_nodes(
    node: &PositionNode,
    lookup_table: &mut impl MoveTableLookup,
) -> Vec<(Move, PositionNode)> {
    let black_turn = node.position.flags.is_black_turn();
    let psudolegal_moves = node.position.generate_psudolegals();
    generate_legal_moves(node, psudolegal_moves, lookup_table, black_turn)
}

pub fn generate_legal_moves(
    node: &PositionNode,
    psudolegal_moves: Vec<Move>,
    lookup_table: &mut impl MoveTableLookup,
    black_turn: bool,
) -> Vec<(Move, PositionNode)> {
    let mut result = Vec::new();
    for m in psudolegal_moves {
        if m.is_castling()
            && !is_legal_castling(m, black_turn, &node.metrics)
        {
            continue;
        }

        match apply_or_get_move(node, m, lookup_table) {
            Ok(node) => result.push((m, node)),
            _ => {}
        }
    }
    result
}

pub fn apply_or_get_move(
    node: &PositionNode,
    psudolegal_move: Move,
    lookup_table: &mut impl MoveTableLookup,
) -> Result<PositionNode, &'static str> {
    let black_turn = node.position.flags.is_black_turn();
    let new_position = lookup_table.lookup(psudolegal_move, node.position, node.position_zorb);

    if (!black_turn && !new_position.metrics.white_in_check)
        || (black_turn && !new_position.metrics.black_in_check)
    {
        Ok(new_position)
    } else {
        Err("Invalid position")
    }
}
