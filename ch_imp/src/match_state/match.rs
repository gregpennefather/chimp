use crate::search::{position_table::PositionTranspositionTable, zorb_set::ZorbSet};

use super::game_state::GameState;

pub struct Match {
    zorb_set: ZorbSet,
    pub current_game_state: GameState,
    transposition_table: PositionTranspositionTable,
}

impl Match {}

impl Default for Match {
    fn default() -> Self {
        let zorb_set = ZorbSet::new();
        Self {
            zorb_set: zorb_set,
            current_game_state: Default::default(),
            transposition_table: PositionTranspositionTable::new(zorb_set),
        }
    }
}
