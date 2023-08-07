use std::fmt;
use super::position::Position;

#[derive(Default)]
pub struct Move {
    pub from: Position,
    pub to: Position,
    pub piece: u8,
    pub promote: u8,
    pub castling: bool,
    pub capture: bool,
}
impl Move {
    pub(crate) fn new(pos: Position, new_pos: Position, piece: u8, capture: bool, promote: u8) -> Move {
        Self {
            from: pos,
            to: new_pos,
            piece: piece,
            promote,
            castling: false,
            capture: capture,
        }
    }

    pub(crate) fn empty(&self) -> bool {
        return self.from.rank == 0 && self.from.file == 0 && self.to.rank == 0 && self.to.file == 0;
    }
}

impl fmt::Debug for Move {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {

        f.debug_struct("Move")
            .field("Piece", &self.piece)
            .field("From", &self.from)
            .field("To", &self.to)
            .finish()
    }
}