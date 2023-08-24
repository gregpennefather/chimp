use std::fmt::Display;

use crate::shared::*;

pub enum PieceType {
    Pawn = 1,
    Knight = 2,
    Bishop = 3,
    Rook = 4,
    Queen = 5,
    King = 6,
}

#[derive(Default, PartialEq, Debug, Clone, Copy)]
pub struct Piece(pub u8);

impl Piece {
    pub fn new(code: u8) -> Self {
        Piece(code)
    }

    pub fn new_coloured(code: u8, is_black: bool) -> Self {
        Piece(code + if is_black { BLACK_MASK } else { 0 })
    }

    pub fn is(&self, t: PieceType) -> bool {
        match t {
            PieceType::Pawn => self.0 == PAWN_INDEX || self.0 == BLACK_PAWN,
            PieceType::Knight => self.0 == KNIGHT_INDEX || self.0 == BLACK_KNIGHT,
            PieceType::Bishop => self.0 == BISHOP_INDEX || self.0 == BLACK_BISHOP,
            PieceType::Rook => self.0 == ROOK_INDEX || self.0 == BLACK_ROOK,
            PieceType::Queen => self.0 == QUEEN_INDEX || self.0 == BLACK_QUEEN,
            PieceType::King => self.0 == KING_INDEX || self.0 == BLACK_KING,
        }
    }

    pub fn without_colour(&self) -> PieceType {
        match self.0 {
            PAWN_INDEX | BLACK_PAWN => PieceType::Pawn,
            KNIGHT_INDEX | BLACK_KNIGHT => PieceType::Knight,
            BISHOP_INDEX | BLACK_BISHOP => PieceType::Bishop,
            ROOK_INDEX | BLACK_ROOK => PieceType::Rook,
            QUEEN_INDEX | BLACK_QUEEN => PieceType::Queen,
            KING_INDEX | BLACK_KING => PieceType::King,
            _ => panic!("Invalid Piece {:b}", self.0),
        }
    }

    pub fn is_white(&self) -> bool {
        !self.is_black()
    }

    pub fn is_black(&self) -> bool {
        self.0 & BLACK_MASK > 1
    }

    // team_flag 1 = black; 0 = white
    pub fn matches_team(&self, team_flag: u8) -> bool {
        self.0 >> 3 == team_flag
    }

    fn get_char(&self) -> char {
        match self.0 {
            PAWN_INDEX => 'P',
            BLACK_PAWN => 'p',
            BISHOP_INDEX => 'B',
            BLACK_BISHOP => 'b',
            KNIGHT_INDEX => 'N',
            BLACK_KNIGHT => 'n',
            ROOK_INDEX => 'R',
            BLACK_ROOK => 'r',
            QUEEN_INDEX => 'Q',
            BLACK_QUEEN => 'q',
            KING_INDEX => 'K',
            BLACK_KING => 'k',
            _ => 'X',
        }
    }
}

impl Display for Piece {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.get_char())
    }
}
