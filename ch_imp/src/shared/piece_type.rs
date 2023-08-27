
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PieceType {
    None = 0,
    Pawn = 1,
    Knight = 2,
    Bishop = 3,
    Rook = 4,
    Queen = 5,
    King = 6,
}

impl Default for PieceType {
    fn default() -> Self {
        PieceType::None
    }
}

pub fn get_piece_char(piece_type: PieceType, black_turn: bool) -> char {
    match (piece_type, black_turn) {
        (PieceType::Pawn, false) => 'P',
        (PieceType::Pawn, true) => 'p',
        (PieceType::Bishop, false) => 'B',
        (PieceType::Bishop, true) => 'b',
        (PieceType::Knight, false) => 'N',
        (PieceType::Knight, true) => 'n',
        (PieceType::Rook, false) => 'R',
        (PieceType::Rook, true) => 'r',
        (PieceType::Queen, false) => 'Q',
        (PieceType::Queen, true) => 'q',
        (PieceType::King, false) => 'K',
        (PieceType::King, true) => 'k',
        _ => 'X',
    }
}
