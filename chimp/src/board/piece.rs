use crate::shared::*;

pub fn rank_from_char(char: char) -> u8 {
    match char {
        'a' => 0,
        'b' => 1,
        'c' => 2,
        'd' => 3,
        'e' => 4,
        'f' => 5,
        'g' => 6,
        'h' => 7,
        _ => u8::MAX
    }
}

pub fn char_from_rank(rank: u8) -> char {
    match rank {
        0 => 'a',
        1 => 'b',
        2 => 'c',
        3 => 'd',
        4 => 'e',
        5 => 'f',
        6 => 'g',
        7 => 'h',
        _ => '_'
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

pub fn get_piece_code(pieces: &u128, piece_index: u8) -> u8 {
    let piece: u8 = (pieces >> (piece_index * 4) & 0b1111).try_into().unwrap();
    return piece;
}

pub fn get_file(index: u8) -> u8 {
    index / 0b1000u8
}

pub fn get_rank(index: u8) -> u8 {
    7 - (index % 8)
}
pub fn get_rank_i8(index: i8) -> u8 {
    7 - (index % 8) as u8
}

pub fn get_friendly_name_for_index(index: u8) -> String {
    let file = get_file(index)+1;
    let rank = get_rank(index);
    format!("{}{file}", char_from_rank(rank))
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    pub fn get_friendly_name_for_index_0_h1() {
        let friendly_name = get_friendly_name_for_index(0);
        assert!(friendly_name.eq("h1".into()), "not h1 but {friendly_name}");
    }
}