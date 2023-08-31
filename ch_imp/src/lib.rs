use std::{
    collections::HashMap,
    sync::{Mutex, RwLock},
};

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

#[macro_use]
extern crate lazy_static;

lazy_static! {
    static ref MOVE_DATA: MoveData = MoveData::new();
    static ref POSITION_TRANSPOSITION_TABLE: RwLock<HashMap<u64, Position>> =
        RwLock::new(HashMap::with_capacity(1000000));
    static ref PL_MOVES_TRANSPOSITION_TABLE: RwLock<HashMap<u64, Vec<Move>>> =
        RwLock::new(HashMap::with_capacity(1000000));
    static ref L_MOVES_TRANSPOSITION_TABLE: RwLock<HashMap<u64, Vec<Move>>> =
        RwLock::new(HashMap::with_capacity(1000000));
    static ref HASH_HITS: Mutex<usize> = Mutex::new(0);
    static ref HASH_MISSES: Mutex<usize> = Mutex::new(0);
}
