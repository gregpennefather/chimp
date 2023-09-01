use crate::{
    board::position::Position, r#move::Move,
    POSITION_TRANSPOSITION_TABLE,
};

pub fn lookup_position_table(zorb_key: u64) -> Option<(Position, Option<Vec<Move>>)> {
    // return None;
    POSITION_TRANSPOSITION_TABLE
        .try_read()
        .unwrap()
        .get(&zorb_key)
        .cloned()
}

pub fn insert_into_position_table(position: Position, legal_moves: Option<Vec<Move>>) {
    POSITION_TRANSPOSITION_TABLE
        .write()
        .unwrap()
        .insert(position.zorb_key, (position, legal_moves));
}