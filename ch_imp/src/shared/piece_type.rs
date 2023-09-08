pub(crate) const PIECE_TYPE_EXCHANGE_VALUE : [i8;7] = [0, 1, 3, 3, 4, 5, 25];

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum PieceType {
    None = 0,
    Pawn = 1,
    Knight = 2,
    Bishop = 3,
    Rook = 4,
    Queen = 5,
    King = 6,
}

impl Into::<usize> for PieceType {
    fn into(self) -> usize {
        self as usize
    }
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

pub fn get_piece_type_from_char(c: char) -> PieceType {
    match c {
        'P' | 'p' => PieceType::Pawn,
        'B' | 'b' => PieceType::Bishop,
        'N' | 'n' => PieceType::Knight,
        'R' | 'r' => PieceType::Rook,
        'Q' | 'q' => PieceType::Queen,
        'K' | 'k' => PieceType::King,
        _ => panic!("Unknown piece type {}", c),
    }
}