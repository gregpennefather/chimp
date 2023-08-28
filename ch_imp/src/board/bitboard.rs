use std::{
    fmt::Display,
    ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign},
};

#[derive(Default, Debug, Clone, Copy)]
pub struct Bitboard(pub u64);

impl Bitboard {
    pub fn new(val: u64) -> Bitboard {
        Self(val)
    }

    pub fn occupied(&self, index: u8) -> bool {
        if index > 63 {
            println!("{}", index);
        }
        self.0 >> index & 0b1 > 0
    }

    pub fn flip(&self, index: u8) -> Bitboard {
        Bitboard(self.0 ^ (1 << index))
    }

    pub fn set(&self, index: u8) -> Bitboard {
        Bitboard(self.0 | (1 << index))
    }

    pub fn count_occupied(&self) -> u8 {
        self.0.count_ones() as u8
    }

    fn get_rank(&self, rank: u8) -> u8 {
        (self.0 >> (rank * 8) & 255) as u8
    }
}

impl BitOr for Bitboard {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl BitOrAssign for Bitboard {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0
    }
}

impl BitAnd for Bitboard {
    type Output = Self;
    fn bitand(self, rhs: Self) -> Self::Output {
        Self(self.0 & rhs.0)
    }
}

impl BitAndAssign for Bitboard {
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 &= rhs.0
    }
}

impl PartialEq for Bitboard {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}
impl Display for Bitboard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut r: String = "".to_string();

        r += format!("{:#010b}\n", self.get_rank(7)).as_str();
        r += format!("{:#010b}\n", self.get_rank(6)).as_str();
        r += format!("{:#010b}\n", self.get_rank(5)).as_str();
        r += format!("{:#010b}\n", self.get_rank(4)).as_str();
        r += format!("{:#010b}\n", self.get_rank(3)).as_str();
        r += format!("{:#010b}\n", self.get_rank(2)).as_str();
        r += format!("{:#010b}\n", self.get_rank(1)).as_str();
        r += format!("{:#010b}\n", self.get_rank(0)).as_str();

        write!(f, "{r}")
    }
}

impl Into<u64> for Bitboard {
    fn into(self) -> u64 {
        self.0
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    pub fn occupied_h2_is_occupied() {
        let bitboard = Bitboard(0b1111111110u64);
        assert!(bitboard.occupied(8));
    }

    #[test]
    pub fn set_h1() {
        let bitboard = Bitboard::default();
        let expected = Bitboard::new(0b1);
        assert_eq!(bitboard.set(0), expected);
    }
    #[test]
    pub fn set_g1() {
        let bitboard = Bitboard::default();
        let expected = Bitboard::new(u64::pow(2, 1));
        assert_eq!(bitboard.set(1), expected);
    }

    #[test]
    pub fn set_e6() {
        let bitboard = Bitboard::default();
        let expected = Bitboard::new(u64::pow(2, 41));
        assert_eq!(bitboard.set(41), expected);
    }
}
