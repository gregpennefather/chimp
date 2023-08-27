use crate::r#move::move_data::MoveData;

pub mod board;
pub mod r#move;
pub mod match_state;
pub mod shared;
pub mod search;
pub mod engine;

#[macro_use]
extern crate lazy_static;

lazy_static! {
    static ref MOVE_DATA: MoveData = MoveData::new();
}