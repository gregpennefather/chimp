pub trait Bitboard {
    fn occupied(&self, index: u8) -> bool;
    fn flip(&self, index: u8) -> u64;
    fn set(&self, index: u8) -> u64;
    fn count_occupied(&self) -> u8;
    fn get_rank(&self, rank: u8) -> u8;
    fn set_file(&self, file: u8) -> u64;
    fn set_rank(&self, rank: u8) -> u64;
    fn to_board_format(&self) -> String;
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
}
