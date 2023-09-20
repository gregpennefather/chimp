use crate::{
    match_state::game_state::GameState, move_generation::generate_moves_for_board, r#move::Move, board::position::Position,
};

pub struct MoveOrderer {
    index: usize,
    principal_variation: Option<Move>,
    hash_move: Option<Move>,
    position: Position,
    moves: Option<Vec<Move>>,
}

impl MoveOrderer {
    pub fn new(pv: Option<&Move>, hm: Option<Move>, position: Position) -> Self {
        Self {
            index: 0,
            principal_variation: match pv { Some(&m) => Some(m), None => None},
            hash_move: hm,
            position: position,
            moves: None
        }
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
            // Not the PV or HM so lets generate the moves (if we haven't already) and find the next move that isn't the PV or HM
            else {
                if self.moves == None {
                    self.moves = Some(generate_moves_for_board(self.position.board))
                };

                let arr_pos = self.index - 2;
                let moves = self.moves.as_ref().unwrap();

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
