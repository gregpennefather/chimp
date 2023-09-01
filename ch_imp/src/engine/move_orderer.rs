use log::trace;

use crate::r#move::Move;

pub fn order(moves: Vec<Move>, priority_moves: Vec<Move>) -> Vec<Move> {
    trace!("moves:{moves:?}");
    trace!("priority_moves:{priority_moves:?}");

    let mut working = moves.clone();
    let mut output = Vec::new();
    for m in moves {
        let search_result = priority_moves.binary_search(&m);
        match search_result {
            Ok(index) => {
                output.push(m);
                working.remove(index);
            },
            _ => {}
        }
    }

    output.extend(working);

    output
}

#[cfg(test)]
mod test {
    use crate::{shared::piece_type::PieceType, r#move::Move, engine::move_orderer::order};

    #[test]
    pub fn order_priority_move_to_top() {
        let priority_move = Move::new(2,4,1, PieceType::Queen, false);
        let moves = vec![Move::new(0,1,0,PieceType::Pawn, false), priority_move];
        let r = order(moves, vec![priority_move]);
        assert_eq!(r[0], priority_move);
    }
}
