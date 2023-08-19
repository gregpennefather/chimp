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

// Todo - maybe refactor this out
pub fn is_piece_white(piece: u8) -> bool {
    !is_piece_black(piece)
}

pub fn is_piece_black(piece: u8) -> bool {
    piece & BLACK_MASK > 1
}

// team_flag 1 = black; 0 = white
pub fn matches_team(piece:u8, team_flag: u8) -> bool {
    piece >> 3 == team_flag
}

pub fn get_piece_code(pieces: &u128, piece_index: u8) -> u8 {
    let piece: u8 = (pieces >> (piece_index * 4) & 0b1111).try_into().unwrap();
    return piece;
}
