use super::{
    bitboard::{Bitboard, BitboardExtensions},
    piece_list::PieceList,
};

pub fn rank_and_file_to_index(rank: u8, file: u8) -> u8 {
    ((file) * 8) + (7 - rank)
}

pub fn rank_and_file_to_index_i8(rank: i8, file: i8) -> u8 {
    (((file) * 8) + (7 - rank)) as u8
}

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
        _ => u8::MAX,
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
        _ => '_',
    }
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
    let file = get_file(index) + 1;
    let rank = get_rank(index);
    format!("{}{file}", char_from_rank(rank))
}

pub fn board_to_string(bitboard: Bitboard, pieces: PieceList) -> String {
    let mut r: String = "".to_string();

    let mut index = 0;
    let mut piece_index = (bitboard.count_occupied() - 1).try_into().unwrap();
    while index < 63 {
        let position_index = 63 - index;
        if bitboard.occupied(position_index) {
            let piece = pieces.get(piece_index);
            r += &piece.to_string();
            piece_index -= 1;
        } else {
            r += &'0'.to_string();
        }
        index += 1;
        if (position_index + 1) % 8 == 0 {
            r += "\n".into();
        }
    }

    r
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
