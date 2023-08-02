use crate::chess::constants::PIECE_MASK;

use super::position::Position;

pub fn get_position_index(rank: i8, file: i8 ) -> i8 {
    (file * 8) + rank
}

pub fn check_board_position(bitboard: u64, rank: i8, file: i8) -> bool {
    let index = get_position_index(rank, file);
    let check_result = bitboard & (1 << index);
    check_result > 0
}

pub fn valid_position(rank: i8, file: i8) -> bool {
    return rank >= 0 && rank <= 7 && file >= 0 && file <= 7;
}

pub fn build_bitboard(positions: &[Position]) -> u64 {
    let mut bitboard: u64 = 0;
    for (index,item) in positions.iter().enumerate() {
        let pos_index = get_position_index(item.rank, item.file) as u64;
        bitboard += 1 << pos_index;
    }
    bitboard
}

pub fn is_piece_type(piece_code: u8, piece_type: u8) -> bool {
    let colourless_piece_code = piece_code & PIECE_MASK;
    colourless_piece_code == piece_type
}