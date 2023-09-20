use std::{sync::{Arc, Mutex}};

use crate::{
    r#move::{move_data::MoveData, Move}, evaluation::pawn_structure::PawnZorb,
};

pub mod board;
pub mod engine;
pub mod evaluation;
pub mod match_state;
pub mod r#move;
pub mod search;
pub mod shared;
pub mod testing;
pub mod move_generation;
pub mod move_ordering;

#[macro_use]
extern crate lazy_static;

lazy_static! {
    static ref PAWN_ZORB: PawnZorb = PawnZorb::new();
    pub static ref MOVE_DATA: MoveData = MoveData::new();
    static ref PONDERING: Arc<Mutex<bool>> = Arc::new(Mutex::new(false));

    static ref PONDERING_RESULT: Arc<Mutex<Option<Vec<Move>>>> = Arc::new(Mutex::new(None));
}
