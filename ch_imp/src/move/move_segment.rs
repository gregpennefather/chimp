use crate::shared::piece_type::PieceType;

use core::fmt::Debug;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum MoveSegmentType {
    None,
    Pickup,
    Place,
    ClearCastling, // When set the index will represent from of the piece thats move or removal cleared the castling
    DoublePawnPush,
    ClearEP
}

#[derive(Clone, Copy, PartialEq)]
pub struct MoveSegment {
    pub segment_type: MoveSegmentType,
    pub index: u8,
    pub piece_type: PieceType,
    pub black_piece: bool,
}

impl Debug for MoveSegment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Type").field(&self.segment_type).field(&self.index).field(&self.piece_type).field(&self.black_piece).finish()
    }
}

impl MoveSegment {
    pub fn new(segment_type: MoveSegmentType, index: u8, piece_type: PieceType, black_piece: bool) -> MoveSegment {
        Self {segment_type, index, piece_type, black_piece}
    }

    pub fn is_empty(&self) -> bool {
        self.segment_type == MoveSegmentType::None
    }
}

impl Default for MoveSegment {
    fn default() -> Self {
        Self { segment_type: MoveSegmentType::None, index: Default::default(), piece_type: PieceType::None, black_piece: Default::default() }
    }
}