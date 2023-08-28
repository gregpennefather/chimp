use crate::search::{zorb_set::ZorbSet};

use super::game_state::GameState;

pub struct Match {
    pub current_game_state: GameState
}

impl Match {}

impl Default for Match {
    fn default() -> Self {
        Self {
            current_game_state: Default::default()
        }
    }
}
