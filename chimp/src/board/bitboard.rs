pub trait BitboardExtensions {
    fn occupied(&self, index: u8) -> bool;
    fn occupied_i8(&self, index: i8) -> bool;
}

impl BitboardExtensions for u64 {
    fn occupied(&self, index: u8) -> bool {
        self >> index & 0b1 > 0
    }

    fn occupied_i8(&self, index: i8) -> bool {
        self >> index & 0b1 > 0
    }
}