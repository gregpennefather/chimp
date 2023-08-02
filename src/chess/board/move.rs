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
}
