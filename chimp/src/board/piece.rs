use crate::shared::*;

pub fn file_from_char(char: char) -> u16 {
    match char {
        'a' => 0,
        'b' => 1,
        'c' => 2,
        'd' => 3,
        'e' => 4,
        'f' => 5,
        'g' => 6,
        'h' => 7,
        _ => u16::MAX
    }
}

pub fn get_piece_char(piece: u8) -> char {
    match piece {
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

pub fn get_piece_code(pieces: &u128, piece_index: i8) -> u8 {
    let piece: u8 = (pieces >> (piece_index * 4) & 0b1111).try_into().unwrap();
    return piece;
}