use std::{cell::OnceCell, sync::Mutex};

use crate::{board::position::Position, r#move::Move, shared::cache::MovesCache};

use super::killer_store::PlyKillers;

lazy_static! {
    pub static ref MOVE_CACHE: Mutex<MovesCache> = Mutex::<MovesCache>::new(MovesCache::new());
}

pub struct MoveOrderer {
    index: usize,
    principal_variation: Option<Move>,
    hash_move: Option<Move>,
    ply_killers: PlyKillers,
    position: Position,
    moves: OnceCell<Vec<Move>>,
}

impl MoveOrderer {
    pub fn new(pv: Option<&Move>, hm: Option<Move>, position: Position, ply_killers: PlyKillers) -> Self {
        Self {
            index: 0,
            principal_variation: match pv {
                Some(&m) => Some(m),
                None => None,
            },
            hash_move: hm,
            ply_killers,
            position: position,
            moves: OnceCell::new(),
        }
    }

    fn get_moves(&mut self) -> &Vec<Move> {
        let moves = self
            .moves
            .get_or_init(|| MOVE_CACHE.lock().unwrap().get_moves(self.position.board));
        moves
    }
}

impl Iterator for MoveOrderer {
    type Item = Move;

    fn next(&mut self) -> Option<Self::Item> {
        let mut result = None;
        while result == None {
            // If this is the first move, check if the principal_variation move at this depth is legal
            if self.index == 0 {
                result = match self.principal_variation {
                    Some(pv) => {
                        if self.position.is_legal_move(pv) {
                            Some(pv)
                        } else {
                            None
                        }
                    }
                    None => None,
                };
            }
            // If this is the second move, check if the hash_move at this depth is legal
            else if self.index == 1 {
                result = match self.hash_move {
                    Some(hm) => {
                        if self.position.is_legal_move(hm) {
                            Some(hm)
                        } else {
                            None
                        }
                    }
                    None => None,
                };
            }
            // If this is the 3rd-5th move try our killer moves
            else if self.index < 5 {
                let killer_move = self.ply_killers.get(self.index-2);
                result = match killer_move {
                    Some(km) => {
                        if self.position.is_legal_move(km) {
                            Some(km)
                        } else {
                            None
                        }
                    },
                    None => None
                }
            }
            // Not the PV or HM so lets generate the moves (if we haven't already) and find the next move that isn't the PV or HM
            else {
                let arr_pos = self.index - 5;
                let moves = self.get_moves();

                if arr_pos >= moves.len() {
                    break;
                }

                result = Some(moves[arr_pos]);

                // Don't repeatedly check the PV or HM
                if result == self.principal_variation || result == self.hash_move {
                    result = None
                }
            }
            self.index += 1
        }
        result
    }
}
