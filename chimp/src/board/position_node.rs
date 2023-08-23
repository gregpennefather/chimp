use crate::util::zorb_hash::ZorbSet;
use std::fmt::Debug;

use super::{
    bitboard::BitboardExtensions, board_metrics::BoardMetrics, r#move::Move, state::BoardState,
};

#[derive(Clone, Copy)]
pub struct PositionNode {
    pub position_zorb: u64,
    pub position: BoardState,
    pub metrics: BoardMetrics,
    pub evaluation: f32
}

impl Debug for PositionNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("PositionNode").field(&self.position.to_fen()).finish()
    }
}

impl PositionNode {
    pub fn new(zorb_set: ZorbSet) -> Self {
        let position = BoardState::default();
        let position_zorb = zorb_set.hash(position);
        let current_metrics = position.generate_metrics();
        Self {
            position_zorb,
            position,
            metrics: current_metrics,
            evaluation: 0.0
        }
    }
}

impl Default for PositionNode {
    fn default() -> Self {
        Self {
            position_zorb: Default::default(),
            position: Default::default(),
            metrics: Default::default(),
            evaluation: 0.0
        }
    }
}