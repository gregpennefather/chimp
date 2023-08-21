use crate::shared::*;

use super::piece::Piece;

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
