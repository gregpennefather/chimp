use std::collections::HashMap;

use super::zorb_set::ZorbSet;
use crate::{board::position::Position, r#move::Move};

pub struct PositionTranspositionTable(HashMap<u64, Position>, ZorbSet);

pub trait MoveTableLookup {
    fn lookup(&mut self, m: Move, position: Position) -> Position;
}

impl PositionTranspositionTable {
    pub fn new(zorb_set: ZorbSet) -> Self {
        Self(HashMap::with_capacity(4000000), zorb_set)
    }
}

impl MoveTableLookup for PositionTranspositionTable {
    fn lookup(&mut self, m: Move, position: Position) -> Position {
        let key = position.zorb_key_after_move(m);
        // let mut iter = move_segments.iter();
        // while let Some(&segment) = iter.next() {
        //     key = self.1.shift(key, segment)
        // }
        // key = self.1.colour_shift(key);

        *self.0.entry(key).or_insert_with(|| position.make(m))
    }
}
