use std::fmt;

use crate::chess::constants::RANKS;

#[derive(Default, Copy, Clone)]
pub struct Position {
    pub rank: i8,
    pub file: i8
}

impl fmt::Debug for Position {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let rank_c = RANKS.chars().nth(self.rank.try_into().unwrap()).unwrap();
        let pos = format!("{rank_c}{}", self.file + 1);

        f.debug_struct("Position")
            .field("name", &pos)
            .finish()
    }
}