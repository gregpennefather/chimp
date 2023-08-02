// MSB: 1 = white 0 = black
// 0: None
// 1: Pawn
// 10: Knight
// 11: Bishop
// 100: Rook
// 101: Queen
// 110: King

// Example: Black King = {0}{110}
// White Pawn = {1}{001}

// Bit bits per square
// 8x8 squares => 8x8x4 = 256;

use self::{position::*, r#move::Move};
use super::constants::*;
use crate::chess::board::piece::*;

mod piece;
mod position;
mod r#move;
mod utils;


pub struct BoardState(pub u64, pub u128);

impl BoardState {
    pub fn check_position(&self, file: u8, rank: u8) -> bool {
        let index = (file * 8) + rank;
        let check_result = self.0 & (1 << index);
        check_result > 0
    }

    pub fn get_piece(&self, piece_index: usize) -> u8 {
        let pieces = self.1;
        let sub = pieces >> (4 * piece_index) & (COLOURED_PIECE_MASK as u128);

        sub as u8
    }

    pub fn from_fen(fen: String) -> BoardState {
        let mut occ: u64 = 0;
        let mut file: i64 = 7;
        let mut rank: u64 = 0;
        let mut piece_index: u16 = 0;
        let mut p: u128 = 0;
        let mut clause = 0;

        for i in 0..fen.len() {
            let char: char = fen.chars().nth(i).unwrap();

            if clause == 0 {
                if char.is_ascii_digit() {
                    let digit = char as i32 - 0x30;
                    rank += digit as u64;
                    continue;
                }

                if char == '/' {
                    rank = 0;
                    file -= 1;
                    continue;
                }

                if char == ' ' {
                    clause += 1;
                    continue;
                }

                let piece_position: u64 = 1 << ((file * 8) as u64 + rank);

                occ = occ + piece_position;
                rank = rank + 1;

                let piece: u8 = match char {
                    'P' => PAWN_INDEX,
                    'p' => PAWN_INDEX | BLACK_MASK,
                    'B' => BISHOP_INDEX,
                    'b' => BISHOP_INDEX | BLACK_MASK,
                    'N' => KNIGHT_INDEX,
                    'n' => KNIGHT_INDEX | BLACK_MASK,
                    'R' => ROOK_INDEX,
                    'r' => ROOK_INDEX | BLACK_MASK,
                    'Q' => QUEEN_INDEX,
                    'q' => QUEEN_INDEX | BLACK_MASK,
                    'K' => KING_INDEX,
                    'k' => KING_INDEX | BLACK_MASK,
                    _ => 0,
                };

                let piece_u128: u128 = (piece as u128) << (4 * piece_index);
                p = p | piece_u128;
                piece_index += 1;
            }
        }

        BoardState(occ, p)
    }
}

pub struct Board {
    state: BoardState,
    pub pieces: [Piece; 32],
}

impl Board {
    pub fn new(state: BoardState) -> Board {
        let mut pieces: [Piece; 32] = [Piece::default(); 32];

        let mut piece_index: usize = 0;
        for y in 0..8 {
            for rank in 0..8 {
                let file = 7 - y;
                if state.check_position(file, rank) {
                    let code = state.get_piece(piece_index);
                    let piece = Piece { pos: Position {file, rank}, code };
                    pieces[piece_index] = piece;
                    piece_index += 1;
                }
            }
        }

        Self {
            state: state,
            pieces: pieces,
        }
    }

    pub fn get_moves(&self) -> Vec<Move> {
        todo!();
    }
}
