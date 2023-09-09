const H1: u64 = 0x5555555555555555u64;
const H2: u64 = 0x3333333333333333u64;
const H4: u64 = 0x0F0F0F0F0F0F0F0Fu64;
const V1: u64 = 0x00FF00FF00FF00FFu64;
const V2: u64 = 0x0000FFFF0000FFFFu64;

const K1 : u64 = 0x5555555555555555u64;
const K2 : u64 = 0x3333333333333333u64;
const K4 : u64 = 0x0f0f0f0f0f0f0f0fu64;

pub trait Bitboard {
    fn occupied(&self, index: u8) -> bool;
    fn flip(&self, index: u8) -> u64;
    fn set(&self, index: u8) -> u64;
    fn count_occupied(&self) -> u8;
    fn get_rank(&self, rank: u8) -> u8;
    fn set_file(&self, file: u8) -> u64;
    fn set_rank(&self, rank: u8) -> u64;
    fn to_board_format(&self) -> String;
    fn rotate_180(&self) -> u64;
    fn mirror_horizontally(&self) -> Self;
}

impl Bitboard for u64 {
    fn occupied(&self, index: u8) -> bool {
        assert!(index <= 63);
        self >> index & 0b1 > 0
    }

    fn flip(&self, index: u8) -> u64 {
        self ^ (1 << index)
    }

    fn set(&self, index: u8) -> u64 {
        self | (1 << index)
    }

    fn count_occupied(&self) -> u8 {
        self.count_ones() as u8
    }

    fn get_rank(&self, rank: u8) -> u8 {
        (self >> (rank * 8) & 255) as u8
    }

    fn set_file(&self, file: u8) -> Self {
        self | (0b100000001000000010000000100000001000000010000000100000001u64 << file as u64)
    }

    fn set_rank(&self, rank: u8) -> Self {
        self | (0xff << (8 * rank))
    }

    fn to_board_format(&self) -> String {
        let mut r: String = "".to_string();

        r += format!("{:#010b}\n", self.get_rank(7)).as_str();
        r += format!("{:#010b}\n", self.get_rank(6)).as_str();
        r += format!("{:#010b}\n", self.get_rank(5)).as_str();
        r += format!("{:#010b}\n", self.get_rank(4)).as_str();
        r += format!("{:#010b}\n", self.get_rank(3)).as_str();
        r += format!("{:#010b}\n", self.get_rank(2)).as_str();
        r += format!("{:#010b}\n", self.get_rank(1)).as_str();
        r += format!("{:#010b}\n", self.get_rank(0)).as_str();

        r
    }

    /**
     * Rotate a bitboard by 180 degrees.
     * Square a1 is mapped to h8, and a8 is mapped to h1.
     * @param x any bitboard
     * @return bitboard x rotated 180 degrees
     */
    fn rotate_180(&self) -> Self {
        let mut r = *self;
        r = ((r >> 1) & H1) | ((r & H1) << 1);
        r = ((r >> 2) & H2) | ((r & H2) << 2);
        r = ((r >> 4) & H4) | ((r & H4) << 4);
        r = ((r >> 8) & V1) | ((r & V1) << 8);
        r = ((r >> 16) & V2) | ((r & V2) << 16);
        r = (r >> 32) | (r << 32);
        r
    }


/**
 * Mirror a bitboard horizontally about the center files.
 * File a is mapped to file h and vice versa.
 * @param x any bitboard
 * @return bitboard x mirrored horizontally
 */
    fn mirror_horizontally(&self) -> Self {
        let mut r = *self;
        r = ((r >> 1) & K1) | ((r & K1) << 1);
        r = ((r >> 2) & K2) | ((r & K2) << 2);
        r = ((r >> 4) & K4) | ((r & K4) << 4);
        r
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    pub fn occupied_h2_is_occupied() {
        let bitboard = 0b1111111110u64;
        assert!(bitboard.occupied(8));
    }

    #[test]
    pub fn set_h1() {
        let bitboard = 0;
        let expected = 0b1;
        assert_eq!(bitboard.set(0), expected);
    }
    #[test]
    pub fn set_g1() {
        let bitboard = 0;
        let expected = u64::pow(2, 1);
        assert_eq!(bitboard.set(1), expected);
    }

    #[test]
    pub fn set_e6() {
        let bitboard = 0;
        let expected = u64::pow(2, 41);
        assert_eq!(bitboard.set(41), expected);
    }

    #[test]
    pub fn mirror_horizontally_tests() {
        let bb = 0.set_rank(6).flip(6*8);
        println!("{}", bb.to_board_format());
        println!("{}", bb.reverse_bits().mirror_horizontally().to_board_format());
        let e = 0.set_rank(1).flip(8);
        println!("{}", e.to_board_format());
        assert_eq!(bb.reverse_bits().mirror_horizontally(),e);
    }
}
