pub trait BitboardExtensions {
    fn occupied(&self, index: u8) -> bool;
    fn occupied_i8(&self, index: i8) -> bool;
    fn flip(&self, index: u8) -> u64;
}

impl BitboardExtensions for u64 {
    fn occupied(&self, index: u8) -> bool {
        self >> index & 0b1 > 0
    }

    fn occupied_i8(&self, index: i8) -> bool {
        self >> index & 0b1 > 0
    }

    fn flip(&self, index: u8) -> u64 {
        self ^ u64::pow(2, index.into())
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
}
