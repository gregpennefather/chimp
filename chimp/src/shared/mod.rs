pub mod binary_utils;

pub const PAWN_INDEX: u8 = 0b0001;
pub const KNIGHT_INDEX: u8 = 0b0010;
pub const BISHOP_INDEX: u8 = 0b0011;
pub const ROOK_INDEX: u8 = 0b0100;
pub const QUEEN_INDEX: u8 = 0b0101;
pub const KING_INDEX: u8 = 0b0110;
pub const BLACK_MASK: u8 = 8;
pub const BLACK_PAWN: u8 = 0b1001;
pub const BLACK_KNIGHT: u8 = 0b1010;
pub const BLACK_BISHOP: u8 = 0b1011;
pub const BLACK_ROOK: u8 = 0b1100;
pub const BLACK_QUEEN: u8 = 0b1101;
pub const BLACK_KING: u8 = 0b1110;
pub const COLOURED_PIECE_MASK: u8 = 15;
pub const PIECE_MASK: u8 = 7;

pub const PROMOTION_FLAG: u16 = 0b1000;
pub const CAPTURE_FLAG: u16 = 0b0100;
pub const EP_CAPTURE_FLAG: u16 = 0b0101;

pub static RANKS: &str = "abcdefgh";



pub fn bitboard_to_string(bitboard: u64) -> String {
    let mut r: String = "".to_string();

    r += format!("{:#010b}\n", get_bitboard_file(bitboard, 7)).as_str();
    r += format!("{:#010b}\n", get_bitboard_file(bitboard, 6)).as_str();
    r += format!("{:#010b}\n", get_bitboard_file(bitboard, 5)).as_str();
    r += format!("{:#010b}\n", get_bitboard_file(bitboard, 4)).as_str();
    r += format!("{:#010b}\n", get_bitboard_file(bitboard, 3)).as_str();
    r += format!("{:#010b}\n", get_bitboard_file(bitboard, 2)).as_str();
    r += format!("{:#010b}\n", get_bitboard_file(bitboard, 1)).as_str();
    r += format!("{:#010b}\n", get_bitboard_file(bitboard, 0)).as_str();

    r
}



fn get_bitboard_file(bitboard: u64, file: u8) -> u8 {
    let r: u8 = (bitboard >> (file * 8) & 255) as u8;
    r
}
