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

use self::{
    position::*,
    r#move::Move,
    utils::{check_board_position, is_white_piece},
};
use super::constants::*;
use crate::chess::board::piece::*;

mod r#move;
mod piece;
mod position;
mod utils;

pub struct BoardState(pub u64, pub u128);

impl BoardState {
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
    white_bitboard: u64,
    black_bitboard: u64,
    pub pieces: [Piece; 32],
}

impl Board {
    pub fn new(state: BoardState) -> Board {
        let mut pieces: [Piece; 32] = [Piece::default(); 32];
        let mut white_bitboard: u64 = 0;
        let mut black_bitboard: u64 = 0;
        let mut piece_index: usize = 0;
        for y in 0..8 {
            for rank in 0..8 {
                let file = 7 - y;
                if check_board_position(state.0, rank, file) {
                    let code = state.get_piece(piece_index);
                    let piece = Piece {
                        pos: Position { file, rank },
                        code,
                    };
                    if is_white_piece(code) {
                        white_bitboard += 1 << ((file * 8) as u64 + rank as u64);
                    } else {
                        black_bitboard += 1 << ((file * 8) as u64 + rank as u64);
                    }
                    pieces[piece_index] = piece;
                    piece_index += 1;
                }
            }
        }

        print!("num pieces {}, whitebb {}, blackbb {}", pieces.len(), white_bitboard, black_bitboard);

        Self {
            state,
            white_bitboard,
            black_bitboard,
            pieces,
        }
    }

    pub fn get_moves(&self, white_move: bool) -> Vec<Move> {
        let mut moves = Vec::new();
        let friendly_bitboard = if white_move {
            self.white_bitboard
        } else {
            self.black_bitboard
        };
        let opponent_bitboard = if white_move {
            self.black_bitboard
        } else {
            self.white_bitboard
        };
        for piece in self.pieces {
            if is_white_piece(piece.code) == white_move {
                let new_moves = match piece.code & PIECE_MASK {
                    PAWN_INDEX => get_pawn_moves(
                        piece.code,
                        piece.pos,
                        Move::default(),
                        true,
                        friendly_bitboard,
                        opponent_bitboard,
                    ),
                    KNIGHT_INDEX => get_knight_moves(
                        piece.code,
                        piece.pos,
                        friendly_bitboard,
                        opponent_bitboard,
                    ),
                    BISHOP_INDEX => get_bishop_moves(
                        piece.code,
                        piece.pos,
                        friendly_bitboard,
                        opponent_bitboard,
                    ),
                    ROOK_INDEX => get_rook_moves(
                        piece.code,
                        piece.pos,
                        friendly_bitboard,
                        opponent_bitboard,
                    ),
                    QUEEN_INDEX => {
                        get_queen_moves(piece.code, piece.pos, friendly_bitboard, opponent_bitboard)
                    }
                    KING_INDEX => {
                        get_king_moves(piece.code, piece.pos, friendly_bitboard, opponent_bitboard)
                    }
                    _ => panic!("Unknown {piece:?}!"),
                };
                println!("{} new moves for {piece:?}", new_moves.len());
                moves.extend(new_moves);
            }
        }
        moves
    }
}
