use std::collections::HashMap;

use super::zorb_set::ZorbSet;
use crate::{
    board::position::Position,
    match_state::game_state::GameState,
    r#move::{move_segment::MoveSegment, Move},
};

pub struct PositionTranspositionTable(HashMap<u64, Position>, ZorbSet);

pub trait MoveTableLookup {
    fn lookup(&mut self, m: Move, game_state: GameState, position_zorb: u64) -> Position;
}

impl PositionTranspositionTable {
    pub fn new(zorb_set: ZorbSet) -> Self {
        Self(HashMap::with_capacity(4000000), zorb_set)
    }
}

impl MoveTableLookup for PositionTranspositionTable {
    fn lookup(&mut self, m: Move, game_state: GameState, position_zorb: u64) -> Position {
        let mut key: u64 = position_zorb;
        let move_segments = game_state.generate_move_segments(&m);
        let mut iter = move_segments.iter();
        while let Some(&segment) = iter.next() {
            key = self.1.shift(key, segment)
        }
        key = self.1.colour_shift(key);

        *self
            .0
            .entry(key)
            .or_insert_with(|| apply_move_segments(game_state.position, move_segments))
    }
}

fn apply_move_segments(position: Position, segments: [MoveSegment; 5]) -> Position {
    position.apply_segments(segments)
}
