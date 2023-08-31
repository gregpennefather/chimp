use crate::{
    board::position::Position, r#move::Move, L_MOVES_TRANSPOSITION_TABLE,
    PL_MOVES_TRANSPOSITION_TABLE, POSITION_TRANSPOSITION_TABLE,
};

pub fn lookup_position_table(zorb_key: u64) -> Option<Position> {
    // return None;
    POSITION_TRANSPOSITION_TABLE
        .try_read()
        .unwrap()
        .get(&zorb_key)
        .cloned()
}

pub fn lookup_pl_moves_table(zorb_key: u64) -> Option<Vec<Move>> {
    // return None;
    PL_MOVES_TRANSPOSITION_TABLE
        .try_read()
        .unwrap()
        .get(&zorb_key)
        .cloned()
}
pub fn lookup_l_moves_table(zorb_key: u64) -> Option<Vec<Move>> {
    // return None;
    L_MOVES_TRANSPOSITION_TABLE
        .try_read()
        .unwrap()
        .get(&zorb_key)
        .cloned()
}

pub fn insert_into_position_table(position: Position, moves: Vec<Move>) {
    POSITION_TRANSPOSITION_TABLE
        .write()
        .unwrap()
        .insert(position.zorb_key, position);

    PL_MOVES_TRANSPOSITION_TABLE
        .write()
        .unwrap()
        .insert(position.zorb_key, moves);
}

pub fn insert_into_l_moves_table(key: u64, legal_moves: Vec<Move>) {
    L_MOVES_TRANSPOSITION_TABLE
        .write()
        .unwrap()
        .insert(key, legal_moves);
}
