use crate::r#move::Move;

const KILLER_STORE_MAX_PLY: usize = 12;

#[derive(Copy, Clone, Default, Debug)]
pub struct PlyKillers {
    index: u8,
    moves: [Option<Move>; 3],
}

impl PlyKillers {
    fn set(&mut self, m: Move) {
        if self.moves[(self.index as usize + 1)%3] == Some(m) || self.moves[(self.index as usize + 2)%3] == Some(m) {
            return{}
        }
        self.moves[self.index as usize] = Some(m);
        self.index = (self.index + 1) % 3;
    }

    pub fn get(self, pos: usize) -> Option<Move> {
        let i = (self.index as usize + 5 + pos) % 3; // +5 is equivalent to -1
        self.moves[i]
    }
}

#[derive(Copy, Clone, Default)]
pub struct KillerStore {
    plys: [PlyKillers; KILLER_STORE_MAX_PLY],
}

impl KillerStore {
    pub fn set(&mut self, ply: usize, m: Move) {
        if ply < KILLER_STORE_MAX_PLY {
            self.plys[ply].set(m);
        }
    }

    pub fn get_ply(self, ply: usize) -> PlyKillers {
        if ply >= KILLER_STORE_MAX_PLY {
            PlyKillers::default()
        } else {
            self.plys[ply]
        }
    }
}
