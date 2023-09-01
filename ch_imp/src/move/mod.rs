use crate::shared::{
    board_utils::get_coords_from_index,
    constants::{
        MF_BISHOP_CAPTURE_PROMOTION, MF_BISHOP_PROMOTION, MF_CAPTURE, MF_DOUBLE_PAWN_PUSH,
        MF_EP_CAPTURE, MF_KING_CASTLING, MF_KNIGHT_CAPTURE_PROMOTION, MF_KNIGHT_PROMOTION,
        MF_PROMOTION, MF_QUEEN_CAPTURE_PROMOTION, MF_QUEEN_CASTLING, MF_QUEEN_PROMOTION,
        MF_ROOK_CAPTURE_PROMOTION, MF_ROOK_PROMOTION,
    },
    piece_type::PieceType,
};
use core::fmt::Debug;
use std::cmp::Ordering;

pub mod move_data;
pub mod move_generation;
pub mod move_magic_bitboards;
pub mod move_segment;

#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub struct Move(u16, PieceType, bool);

impl Move {
    pub fn new(
        from_index: u8,
        to_index: u8,
        flags: u16,
        piece_type: PieceType,
        is_black: bool,
    ) -> Move {
        let f: u16 = from_index.into();
        let t: u16 = to_index.into();
        let m: u16 = f << 10 | t << 4 | flags;
        Move(m, piece_type, is_black)
    }

    pub fn from(&self) -> u8 {
        (self.0 >> 10).try_into().unwrap()
    }

    pub fn to(&self) -> u8 {
        (self.0 >> 4 & 0b111111).try_into().unwrap()
    }

    pub fn flags(&self) -> u16 {
        self.0 & 0b1111
    }

    pub fn piece_type(&self) -> PieceType {
        self.1
    }

    pub fn is_black(&self) -> bool {
        self.2
    }

    pub fn is_castling(&self) -> bool {
        self.flags() == MF_KING_CASTLING || self.flags() == MF_QUEEN_CASTLING
    }

    pub fn is_king_castling(&self) -> bool {
        self.flags() == MF_KING_CASTLING
    }

    pub fn is_promotion(&self) -> bool {
        self.flags() & MF_PROMOTION == MF_PROMOTION
    }

    // Will return true if CAPTURE, EP_CAPTURE, or any CAPTURE_PROMOTION
    pub fn is_capture(&self) -> bool {
        self.flags() & MF_CAPTURE == MF_CAPTURE
    }

    pub fn is_ep_capture(&self) -> bool {
        self.flags() == MF_EP_CAPTURE
    }

    pub fn is_double_pawn_push(&self) -> bool {
        self.flags() == MF_DOUBLE_PAWN_PUSH
    }

    pub fn uci(&self) -> String {
        let promotion = match self.flags() {
            MF_KNIGHT_PROMOTION => "n",
            MF_KNIGHT_CAPTURE_PROMOTION => "n",
            MF_BISHOP_PROMOTION => "b",
            MF_BISHOP_CAPTURE_PROMOTION => "b",
            MF_ROOK_PROMOTION => "r",
            MF_ROOK_CAPTURE_PROMOTION => "r",
            MF_QUEEN_PROMOTION => "q",
            MF_QUEEN_CAPTURE_PROMOTION => "q",
            _ => "",
        };
        format!(
            "{}{}{}",
            get_coords_from_index(self.from()),
            get_coords_from_index(self.to()),
            promotion
        )
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.0 == 0
    }
}

impl Debug for Move {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Move")
            .field(&self.uci())
            .field(&self.piece_type())
            .field(&self.flags())
            .field(&self.is_black())
            .finish()
    }
}

impl PartialOrd for Move {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        let flags_result = other.flags().cmp(&self.flags());
        if flags_result != Ordering::Equal {
            return Some(flags_result);
        }

        Some(other.1.cmp(&self.1))
    }
}

impl Ord for Move {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match self.partial_cmp(other) {
            Some(r) => r,
            None => Ordering::Equal
        }
    }
}
#[cfg(test)]
mod test {
    use crate::{shared::{piece_type::PieceType, constants::MF_CAPTURE}, r#move::Move, engine::move_orderer::order};

    #[test]
    pub fn order_will_prioritize_greater_valued_flags() {
        let m1 = Move::new(2,4,1, PieceType::Queen, false);
        let m2 = Move::new(2,4,15, PieceType::Queen, false);

        let mut vec = vec![m1,m2];
        vec.sort();
        assert_eq!(vec[0],m2);
        assert_eq!(vec[1],m1);

    }

    #[test]
    pub fn order_will_prioritize_greater_valued_pieces() {
        let m1 = Move::new(2,4,1, PieceType::Pawn, false);
        let m2 = Move::new(2,4,1, PieceType::Queen, false);

        let mut vec = vec![m1,m2];
        vec.sort();
        assert_eq!(vec[0],m2);
        assert_eq!(vec[1],m1);

    }

    #[test]
    pub fn order_moves_case_capture_over_quiet() {
        let capture = Move::new(0, 1, MF_CAPTURE, PieceType::Pawn, true);
        let quiet = Move::new(0, 1, 0b0, PieceType::Pawn, true);
        let mut moves = vec![quiet, capture];

        moves.sort();
        assert_eq!(moves[0], capture);
        assert_eq!(moves[1], quiet);
    }

}