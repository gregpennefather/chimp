use std::{collections::HashMap, sync::RwLock};

use crate::{
    board::position::Position, r#move::move_data::MoveData
};

pub mod board;
pub mod engine;
pub mod match_state;
pub mod r#move;
pub mod search;
pub mod shared;

#[macro_use]
extern crate lazy_static;

lazy_static! {
    static ref MOVE_DATA: MoveData = MoveData::new();
    static ref POSITION_TRANSPOSITION_TABLE: RwLock<HashMap<u64, Position>> =
        RwLock::new(HashMap::with_capacity(4000000));
}
