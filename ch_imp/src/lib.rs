use std::{collections::HashMap, sync::{RwLock, Arc, Mutex}};

use crate::{
    board::position::Position,
    r#move::{move_data::MoveData, Move},
};

pub mod board;
pub mod engine;
pub mod evaluation;
pub mod match_state;
pub mod r#move;
pub mod search;
pub mod shared;
pub mod testing;

#[macro_use]
extern crate lazy_static;

lazy_static! {
    static ref MOVE_DATA: MoveData = MoveData::new();
    static ref POSITION_TRANSPOSITION_TABLE: RwLock<HashMap<u64, Position>> =
        RwLock::new(HashMap::with_capacity(1000000));
    static ref PONDERING: Arc<Mutex<bool>> = Arc::new(Mutex::new(false));

    static ref PONDERING_RESULT: Arc<Mutex<Option<Vec<Move>>>> = Arc::new(Mutex::new(None));
}
