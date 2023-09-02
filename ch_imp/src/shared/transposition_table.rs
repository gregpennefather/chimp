use crate::{board::position::Position, r#move::Move, POSITION_TRANSPOSITION_TABLE};

pub fn lookup_position_table(zorb_key: u64) -> Option<(Position, Vec<Move>)> {
    // return None;
    POSITION_TRANSPOSITION_TABLE
        .try_read()
        .unwrap()
        .get(&zorb_key)
        .cloned()
}

pub fn insert_into_position_table(position: Position, pl_moves: Vec<Move>) {
    POSITION_TRANSPOSITION_TABLE
        .write()
        .unwrap()
        .insert(position.zorb_key, (position, pl_moves));
}

pub fn clear_tables() {
    POSITION_TRANSPOSITION_TABLE.write().unwrap().clear();
}
