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

pub mod move_data;
pub mod move_generation;
pub mod move_magic_bitboards;
pub mod move_segment;

#[derive(Default, Clone, Copy, PartialEq)]
pub struct Move(u16, PieceType, bool);

impl Move {
    pub fn new(from_index: u8, to_index: u8, flags: u16, piece_type: PieceType, is_black: bool) -> Move {
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
}

impl Debug for Move {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Move")
            .field(&self.uci())
            .field(&self.piece_type())
            .field(&self.flags())
            .finish()
    }
}