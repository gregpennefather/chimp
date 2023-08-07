use crate::chess::board::{piece::Piece, utils::is_white_piece};

pub trait BinaryModification {
    fn copy_b(&self, start: usize, len: usize) -> Self;
    fn remove_b(&self, start: usize, len: usize) -> Self;
    fn insert_b(&self, start: usize, val: Self, len: usize) -> Self;
    fn move_b(&self, start: usize, end: usize, len: usize) -> Self;
}

impl BinaryModification for u64 {
    fn copy_b(&self, start: usize, len: usize) -> u64 {
        (self >> start) & (u64::pow(2, len as u32) - 1)
    }

    fn remove_b(&self, start: usize, len: usize) -> u64 {
        let a = if start > 0 { self.copy_b(0, start) } else { 0 };
        let b = self.copy_b(start + len, 64 - start - len);
        a | (b << start)
    }

    fn insert_b(&self, start: usize, val: u64, len: usize) -> u64 {
        let a = if start > 0 { self.copy_b(0, start) } else { 0 };
        let b = self.copy_b(start, 64-start-len);
        a | (val << start) | (b << (start + len))
    }

    fn move_b(&self, start: usize, end: usize, len: usize) -> u64 {
        let word = self.copy_b(start, len);
        let without = self.remove_b(start, len);
        without.insert_b(end, word, len)
    }
}

impl BinaryModification for u128 {
    fn copy_b(&self, start: usize, len: usize) -> u128 {
        (self >> start) & (u128::pow(2, len as u32) - 1)
    }

    fn remove_b(&self, start: usize, len: usize) -> u128 {
        let a = if start > 0 { self.copy_b(0, start) } else { 0 };
        let b = self.copy_b(start + len, 128 - start - len);
        a | (b << start)
    }

    fn insert_b(&self, start: usize, val: u128, len: usize) -> u128 {
        let a = if start > 0 { self.copy_b(0, start) } else { 0 };
        println!("start {start}, len: {len}");
        let b = self.copy_b(start, 128-start-len);
        a | (val << start) | (b << (start + len))
    }

    fn move_b(&self, start: usize, end: usize, len: usize) -> u128 {
        let word = self.copy_b(start, len);
        let without = self.remove_b(start, len);
        println!("Move from start {start} to end {end} with len {len} and word {word:b} which is white:{}", is_white_piece(word.try_into().unwrap()));
        without.insert_b(end, word, len)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn copy_b_copy_three_digits() {
        let val: u64 = 0b1010110;
        let result = val.copy_b(2, 3);
        assert_eq!(result, 0b101);
    }

    #[test]
    pub fn remove_b_first_digit() {
        let val: u64 = 0b01;
        let result = val.remove_b(0, 1);
        assert_eq!(result, 0b0);
    }

    #[test]
    pub fn remove_b_remove_two_digits() {
        let val: u64 = 0b11010100;
        let result = val.remove_b(4, 2);
        println!("{result:b}");
        assert_eq!(result, 0b110100);
    }

    #[test]
    pub fn insert_b_at_start() {
        let val: u64 = 0b10;
        let result = val.insert_b(0, 0b01, 2);
        println!("{result:b}");
        assert_eq!(result, 0b1001);
    }

    #[test]
    pub fn insert_b_three_digits() {
        let val: u64 = 0b10;
        let result = val.insert_b(1, 0b101, 3);
        println!("{result:b}");
        assert_eq!(result, 0b11010);
    }


    #[test]
    pub fn move_b_two_digits_to_start() {
        let val: u64 = 0b10110;
        let result = val.move_b(3, 2, 2);
        println!("{result:b}");
        assert_eq!(result, 0b11010);
    }


    #[test]
    pub fn move_b_word_2_to_start() {
        // 0b1001-1011-0011
        // word 0b1011
        // without 0b1001-0011
        let val: u64 = 0b100110110011;
        let result = val.move_b(4, 0, 4);
        println!("{result:b}");
        assert_eq!(result, 0b100100111011);
    }

    #[test]
    pub fn move_b_word_1_to_middle() {
        // 0b1001-1011-0011
        // word 0b0011
        // without 0b1001-1011
        let val: u64 = 0b100110110011;
        let result = val.move_b(0, 4, 4);
        println!("{result:b}");
        assert_eq!(result, 0b100100111011);
    }
}
