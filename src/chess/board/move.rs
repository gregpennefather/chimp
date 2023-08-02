use super::position::Position;

pub struct Move {
    pub from: Position,
    pub to: Position,
    pub promote: u8,
    pub castling: bool,
    pub capture: bool,
}
impl Move {
    pub(crate) fn new(pos: Position, new_pos: Position, capture: bool) -> Move {
        Self {
            from: pos,
            to: new_pos,
            promote: 0,
            castling: false,
            capture: capture,
        }
    }
}
