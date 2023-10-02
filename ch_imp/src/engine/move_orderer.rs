use std::cmp::Ordering;

use crate::r#move::Move;

pub fn priority_cmp(a: &Move, b: &Move, priority_moves: &Vec<Move>) -> Ordering {
    if priority_moves.contains(&a) {
        return Ordering::Less;
    }
    if priority_moves.contains(&b) {
        return Ordering::Greater;
    }
    return Ordering::Equal;
}

pub fn top_priority(a: &Move, b: &Move, m: &Move) -> Ordering {
    if m == a {
        return Ordering::Less;
    }
    if m == b {
        return Ordering::Greater;
    }
    return Ordering::Equal;
}

#[cfg(test)]
mod test {
    use crate::{engine::move_orderer::priority_cmp, r#move::Move, shared::piece_type::PieceType};

    #[test]
    pub fn order_priority_move_to_top() {
        let priority_move = Move::new(2, 4, 1, PieceType::Queen, false, 0,0);
        let mut moves = vec![Move::new(0, 1, 0, PieceType::Pawn, false, 0,0), priority_move];
        moves.sort_by(|a, b| priority_cmp(a, b, &vec![priority_move]));
        assert_eq!(moves[0], priority_move);
    }
}
