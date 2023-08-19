pub type Bitboard = u64;

pub trait BitboardExtensions {
    fn occupied(&self, index: u8) -> bool;
    fn occupied_i8(&self, index: i8) -> bool;
    fn flip(&self, index: u8) -> Bitboard;
    fn set(&self, index: u8) -> Bitboard;
    fn position_to_piece_index(&self, position_index: u8) -> usize;
}

impl BitboardExtensions for Bitboard {
    fn occupied(&self, index: u8) -> bool {
        self >> index & 0b1 > 0
    }

    fn occupied_i8(&self, index: i8) -> bool {
        self >> index & 0b1 > 0
    }

    fn flip(&self, index: u8) -> Bitboard {
        self ^ (1 << index)
    }

    fn set(&self, index: u8) -> Bitboard {
        self | (1 << index)
    }

    fn position_to_piece_index(&self, position_index: u8) -> usize {
        let bitboard_relevant = self & (u64::pow(2, position_index.into()) - 1);
        bitboard_relevant.count_ones() as usize // TODO: replace with u8 or something
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
