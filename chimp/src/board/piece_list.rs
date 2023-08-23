use std::fmt::Display;

use crate::shared::binary_utils::BinaryUtils;

use super::{
    bitboard::{Bitboard, BitboardExtensions},
    piece::Piece,
};

#[derive(Default, PartialEq, Debug, Clone, Copy)]
pub struct PieceList(u128);

impl PieceList {
    pub fn new(pieces: u128) -> Self {
        Self(pieces)
    }

    pub fn get(&self, piece_index: u8) -> Piece {
        if piece_index > 31 {
            return Piece::default();
        }
        let piece: u8 = (self.0 >> (piece_index * 4) & 0b1111).try_into().unwrap();
        Piece::new(piece)
    }

    pub fn get_by_position_index(self, bitboard: Bitboard, position_index: u8) -> Piece {
        let piece_index: usize = bitboard.position_to_piece_index(position_index);
        let piece: u8 = (self.0 >> (piece_index * 4) & 0b1111).try_into().unwrap();
        Piece::new(piece)
    }

    pub fn pickup(&self, bitboard: Bitboard, position_index: u8) -> (Piece, PieceList) {
        let piece_index: usize = bitboard.position_to_piece_index(position_index);
        let piece = self.0.copy_b(piece_index * 4, 4) as u8;
        let board = self.0.remove_b(piece_index * 4, 4);
        (Piece(piece), PieceList(board))
    }

    pub fn remove(&self, bitboard: Bitboard, position_index: u8) -> PieceList {
        let piece_index = bitboard.position_to_piece_index(position_index);
        let list = self.0.remove_b(piece_index * 4, 4);
        PieceList(list)
    }

    pub fn place(&self, bitboard: Bitboard, position_index: u8, piece: Piece) -> PieceList {
        let piece_index = bitboard.position_to_piece_index(position_index);
        PieceList(self.0.insert_b(piece_index * 4, piece.0 as u128, 4))
    }

    pub fn push(&self, piece: u8) -> PieceList {
        PieceList((self.0 << 4) | piece as u128)
    }
}

impl Display for PieceList {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut r = "".to_string();
        for i in 0..32 {
            let piece = self.get(i);
            if piece == Piece::default() {
                break;
            }
            r += &piece.to_string();
        }

        write!(f, "{}", r)
    }
}
