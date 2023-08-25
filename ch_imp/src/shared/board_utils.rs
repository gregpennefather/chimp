pub fn get_index_from_file_and_rank(file: u8, rank: u8) -> u8 {
    ((rank) * 8) + (7 - file)
}

pub fn index_from_coords(coords: &str) -> u8 {
    let file_char = coords.chars().nth(0).unwrap();
    let file = file_from_char(file_char);
    let rank = coords.chars().nth(1).unwrap().to_digit(16).unwrap() - 1;
    get_index_from_file_and_rank(file as u8, rank as u8)
}

pub fn get_rank(index: u8) -> u8 {
    index / 0b1000u8
}

pub fn get_file(index: u8) -> u8 {
    7 - (index % 8)
}
pub fn get_file_i8(index: i8) -> u8 {
    7 - (index % 8) as u8
}

pub fn file_from_char(char: char) -> u8 {
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

pub fn char_from_file(file: u8) -> char {
    match file {
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

pub fn get_coords_from_index(index: u8) -> String {
    let rank = get_rank(index) + 1;
    let file = get_file(index);
    format!("{}{rank}", char_from_file(file))
}