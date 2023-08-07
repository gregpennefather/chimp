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

pub fn is_white_piece(piece_code: u8) -> bool {
    return piece_code >> 3 == 0;
}

// pub fn move_bits_u128()

pub trait ShiftBits {
    fn shift_bits(&self, from: usize, to:  usize, len: usize) -> Self;
}

impl ShiftBits for u128 {
    fn shift_bits(&self, from: usize, to:  usize, len: usize) -> Self {
        let r = self.clone();

        // Example self: 10011, from: 2, to: 0, len 2
        // Expected result: 11100
        // 10011 >> 2 = 100
        let c = r >> from;

        // 100 & 11 = 00;
        let c1 = c & (1<<len);
        // 100 >> 2 = 1;
        let c2 = c >> len;
        let b = r & (1<<from);
        let a = b | (c1 << to) | (c2 << to+len);

        println!("c {c:b}, c1: {c1:b}, c2: {c2:b}, b:{b:b} = a:{a:b}");
        a
    }
}


mod tests {
    use super::*;

    #[test]
    fn shift_bits_swap_1_and_0() {
        let t:u128 = 0b10;
        let t_shifted = t.shift_bits(1,0,1);
        assert_eq!(t_shifted, 0b1);
    }
}