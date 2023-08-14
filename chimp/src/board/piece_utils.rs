use crate::shared::*;

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

pub fn is_white(piece: u8) -> bool {
    piece & BLACK_MASK == 0
}

pub fn get_piece_code(pieces: &u128, piece_index: u8) -> u8 {
    let piece: u8 = (pieces >> (piece_index * 4) & 0b1111).try_into().unwrap();
    return piece;
}
