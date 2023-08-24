use super::constants::file_names;

pub fn get_position_from_coords(file: u8, rank: u8) -> u8 {
    ((rank) * 8) + (7 - file)
}

pub fn position_from_coords(coords: &str) -> u8 {
    let file_letter = coords.chars().nth(0).unwrap();
    let file = file_names.find(file_letter).unwrap();
    let rank = coords.chars().nth(1).unwrap().to_digit(8).unwrap() - 1;
    get_position_from_coords(file as u8, rank as u8)
}