use crate::{board::position::Position, POSITION_TRANSPOSITION_TABLE};

pub fn lookup_position_table(zorb_key: u64) -> Option<Position> {
    // return None;
    POSITION_TRANSPOSITION_TABLE
        .try_read()
        .unwrap()
        .get(&zorb_key)
        .cloned()
}

pub fn insert_into_position_table(position: Position) {
    POSITION_TRANSPOSITION_TABLE
        .write()
        .unwrap()
        .insert(position.board.zorb_key, position);
}

pub fn clear_tables() {
    POSITION_TRANSPOSITION_TABLE.write().unwrap().clear();
}
