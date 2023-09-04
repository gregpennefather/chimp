use std::{collections::HashMap};


use crate::{board::position::Position};

pub struct PositionTranspositionTable(pub HashMap<u64, Position>);

pub trait MoveTableLookup {
    fn set (&mut self, position: Position);
    fn find(&self, key: u64) -> Option<&Position>;
}

impl PositionTranspositionTable {
    pub fn new() -> Self {
        Self(HashMap::with_capacity(4000000))
    }
}

impl MoveTableLookup for PositionTranspositionTable {
    fn set (&mut self, position: Position) {
        let set_result = self.0.insert(position.board.zorb_key, position);
        match set_result {
            Some(old_result) => println!("Replacing old result {:?} => {:?}", old_result, position),
            None => {},
        }
    }
    fn find(&self, key: u64) -> Option<&Position> {
        let r = self.0.get(&key);
        match r {
            None=> println!("{key} not in hash"),
            Some(pos) => println!("Found {pos:?}")
        }
        self.0.get(&key)
    }
}
