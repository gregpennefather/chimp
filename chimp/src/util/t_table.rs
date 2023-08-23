use std::collections::HashMap;

use log::{info, debug};

use crate::board::{
    board_metrics::BoardMetrics, position_node::PositionNode, r#move::Move, state::{BoardState, BoardStateFlagsTrait},
};

use super::zorb_hash::ZorbSet;

pub trait MoveTableLookup {
    fn lookup(&mut self, m: Move, position: BoardState, position_zorb: u64) -> PositionNode;
}

pub struct DummyLookupTable();

impl MoveTableLookup for DummyLookupTable {
    fn lookup(&mut self, m: Move, position: BoardState, position_zorb: u64) -> PositionNode {
        apply_move(0u64, position, m)
    }
}

pub struct PositionTranspositionTable(HashMap<u64, PositionNode>, ZorbSet);

impl PositionTranspositionTable {
    pub fn new(zorb_set: ZorbSet) -> Self {
        Self(HashMap::with_capacity(4000000), zorb_set)
    }
}

impl MoveTableLookup for PositionTranspositionTable {
    fn lookup(&mut self, m: Move, position: BoardState, position_zorb: u64) -> PositionNode {
        let mut key: u64 = position_zorb;
        let position_changes = position.get_position_changes(m);
        let mut iter = position_changes.iter();
        while let Some(&change) = iter.next() {
            key = self.1.shift(key, change)
        }
        key = self.1.colour_shift(key);

        let got = self.0.contains_key(&key);

        let r =*self
            .0
            .entry(key)
            .or_insert_with(|| apply_move(key, position, m));

        let test = position.apply_move(m);

        if test.bitboard != r.position.bitboard || test.pieces != r.position.pieces || test.flags.is_black_turn() != r.position.flags.is_black_turn() {
            println!("zorbshift: {:?}", position_changes);
            println!("pre-existing {:?}", got);
            println!("Err at {key} from='{}' for {} vs {} move={}", position.to_fen(), test.to_fen(), &r.position.to_fen(), m.uci());
        }

        r
    }
}

fn apply_move(key: u64, position: BoardState, m: Move) -> PositionNode {
    let new_position = position.apply_move(m);
    let metrics = new_position.generate_metrics();
    let evaluation = position.evaluate(&metrics);
    PositionNode {
        position_zorb: key,
        position: new_position,
        metrics,
        evaluation
    }
}
