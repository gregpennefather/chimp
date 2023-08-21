use crate::board::piece_utils::*;
use crate::shared::*;

use super::{bitboard::{Bitboard, BitboardExtensions}, piece_list::PieceList};

pub type BoardStateFlags = u8;

pub trait BoardStateFlagsTrait {
    fn is_black_turn(&self) -> bool;
    fn set_black_turn(&mut self, is_black_turn: bool);
    fn alternate_turn(&mut self);
}

impl BoardStateFlagsTrait for BoardStateFlags {
    fn is_black_turn(&self) -> bool {
        self & 0b1 > 0
    }

    fn set_black_turn(&mut self, is_black_turn: bool) {
        if is_black_turn {
            *self |= 0b1;
        } else {
            *self &= 0b0;
        }
    }

    fn alternate_turn(&mut self) {
        *self ^= 0b1
    }
}

#[derive(Clone, Copy)]
pub struct BoardState {
    pub bitboard: Bitboard,
    pub white_bitboard: Bitboard,
    pub black_bitboard: Bitboard,
    pub pieces: PieceList,
    pub flags: BoardStateFlags,
    pub ep_file: u8,
    pub half_moves: u8,
    pub full_moves: u32,
    pub piece_count: u8,
    pub white_king_index: u8,
    pub black_king_index: u8,
}

// Concepts:
// Bitboard: a u64 containing a 1 for every piece and a 0 for every empty square
// ordering is lsb=bottom right of board and msb = top right of board
// thus H1->A1;H2->A2;...;H8->H1

impl BoardState {
    pub fn from_fen(fen: &String) -> BoardState {
        let mut bitboard = Bitboard::default();
        let mut white_bitboard = Bitboard::default();
        let mut black_bitboard = Bitboard::default();
        let mut pieces = PieceList::default();
        let mut flags: BoardStateFlags = BoardStateFlags::default();
        let mut ep_file: u8 = u8::default();
        let mut white_king_index = 0;
        let mut black_king_index = 0;

        let mut i: usize = 0;
        let mut rank: i8 = 7;
        let mut file: i8 = 7;
        // Pieces
        while i < fen.len() {
            let char: char = fen.chars().nth(i).unwrap();
            i += 1;

            if char.is_ascii_digit() {
                let digit = (char as i16 - 0x30) as i8;
                file -= digit;
                continue;
            }

            if char == '/' {
                file = 7;
                rank -= 1;
                continue;
            }

            if char == ' ' {
                break;
            }

            if file < 0 {
                panic!("file shouldn't be < 0 at {}", &fen[..i]);
            }

            let position_index = ((rank * 8) + file) as u8; // I dont like this - we should generate the indexes the same everywhere

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
                'K' => {
                    white_king_index = position_index;
                    KING_INDEX
                }
                'k' => {
                    black_king_index = position_index;
                    KING_INDEX | BLACK_MASK
                }
                _ => 0,
            };

            bitboard = bitboard.set(position_index);
            if piece & BLACK_MASK > 0 {
                black_bitboard = black_bitboard.set(position_index);
            } else {
                white_bitboard = white_bitboard.set(position_index);
            }
            file = file - 1;

            let piece_u128: u128 = piece as u128;
            pieces = pieces.push(piece);
        }

        // Turn
        if fen.chars().nth(i).unwrap() == 'b' {
            flags.set_black_turn(true);
        } else {
            flags.set_black_turn(false);
        };

        i += 2;

        // Castling
        let mut can_castle = 0;
        while let c = fen.chars().nth(i).unwrap() {
            i += 1;
            match c {
                'K' => can_castle += 1,
                'Q' => can_castle += 2,
                'k' => can_castle += 4,
                'q' => can_castle += 8,
                ' ' => {
                    i -= 1;
                    break;
                }
                _ => break,
            }
        }
        flags += can_castle << 1;
        i += 1;

        // En Passant
        let ep_char = fen.chars().nth(i).unwrap().to_ascii_lowercase();
        if ep_char != '-' {
            ep_file = fileS.find(ep_char).unwrap() as u8;
            i += 1;
        } else {
            ep_file = u8::MAX
        }
        i += 2;

        // Half moves
        let remaining_fen = &fen[i..];
        let next_space = remaining_fen.find(' ').unwrap();
        let half_moves_str = &remaining_fen[0..next_space];
        let half_moves = half_moves_str.parse::<u8>().unwrap();

        // Full moves
        let full_remaining_fen = &remaining_fen[next_space + 1..];
        let next_space = match full_remaining_fen.find(' ') {
            Some(pos) => pos,
            _ => full_remaining_fen.len(),
        };
        let full_moves_str = &full_remaining_fen[0..next_space];
        let full_moves = full_moves_str.parse::<u32>().unwrap();
        let piece_count = bitboard.count_occupied();

        if piece_count > 32 {
            panic!("Fen code '{fen}' leading to >32 pieces");
        }

        Self {
            bitboard,
            white_bitboard,
            black_bitboard,
            pieces,
            flags,
            ep_file,
            half_moves,
            full_moves,
            piece_count,
            white_king_index,
            black_king_index,
        }
    }

    pub fn to_fen(&self) -> String {
        let mut rank_index = 7;
        let mut fen: String = String::default();
        let mut piece_index: i8 = (self.bitboard.count_occupied() - 1).try_into().unwrap();

        // Pieces
        loop {
            let rank: u8 = self.bitboard.get_rank(rank_index);
            let pieces_in_row: i8 = rank.count_ones().try_into().unwrap();
            fen += &rank_to_fen_string(rank, &self.pieces, piece_index);
            piece_index -= pieces_in_row;
            if rank_index > 0 {
                rank_index -= 1;
                fen += &"/".to_string();
            } else {
                break;
            }
            // if rank_index < 8 {
            //     fen += &"/".to_string();
            // }
        }

        // Flags
        // Move
        let black_turn: bool = self.flags.is_black_turn();
        fen += if black_turn { " b" } else { " w" };

        // Castling
        fen += " ";
        if (self.flags & 0b10) > 0 {
            fen += "K";
        }
        if (self.flags & 0b100) > 0 {
            fen += "Q";
        }
        if (self.flags & 0b1000) > 0 {
            fen += "k";
        }
        if (self.flags & 0b10000) > 0 {
            fen += "q";
        }

        if (self.flags & 0b11110) == 0 {
            fen += "-";
        }

        // En Passant
        fen += " ";
        if self.flags & 0b100000 > 0 {
            let file_char = fileS.chars().nth(self.ep_file as usize).unwrap();
            fen += &file_char.to_string();
            // if it's whites move next then this move was by black
            fen += if black_turn { "3".into() } else { "6".into() };
        } else {
            fen += "-";
        }

        // Half-moves
        fen += &format!(" {}", self.half_moves);
        // Full-moves
        fen += &format!(" {}", self.full_moves);

        fen
    }
}

impl Default for BoardState {
    fn default() -> Self {
        BoardState::from_fen(&"rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1".into())
    }
}

fn rank_to_fen_string(rank: u8, pieces: &PieceList, piece_index: i8) -> String {
    let mut i: i8 = 7;
    let mut pi = piece_index;
    let mut empty_count = 0;
    let mut r = String::default();
    while i >= 0 && pi < 32 {
        let occ = ((rank >> i) & 1) > 0;
        if occ {
            if empty_count > 0 {
                r += &empty_count.to_string();
                empty_count = 0;
            }
            let piece = pieces.get(pi as u8);
            r += &piece.to_string();
            pi -= 1;
        } else {
            empty_count += 1;
        }
        i -= 1;
    }
    if empty_count > 0 {
        r += &empty_count.to_string();
    }
    r
}
