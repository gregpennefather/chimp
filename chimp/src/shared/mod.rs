pub mod binary_utils;

pub const PAWN_INDEX: u8 = 0b0001;
pub const KNIGHT_INDEX: u8 = 0b0010;
pub const BISHOP_INDEX: u8 = 0b0011;
pub const ROOK_INDEX: u8 = 0b0100;
pub const QUEEN_INDEX: u8 = 0b0101;
pub const KING_INDEX: u8 = 0b0110;
pub const BLACK_MASK: u8 = 0b1000;
pub const BLACK_PAWN: u8 = 0b1001;
pub const BLACK_KNIGHT: u8 = 0b1010;
pub const BLACK_BISHOP: u8 = 0b1011;
pub const BLACK_ROOK: u8 = 0b1100;
pub const BLACK_QUEEN: u8 = 0b1101;
pub const BLACK_KING: u8 = 0b1110;
pub const COLOURED_PIECE_MASK: u8 = 15;
pub const PIECE_MASK: u8 = 7;

pub static fileS: &str = "abcdefgh";
