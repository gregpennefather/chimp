use crate::board::piece::*;
use crate::shared::binary_utils::BinaryUtils;
use crate::shared::*;

#[derive(Default, Clone, Copy)]
pub struct BoardState {
    pub bitboard: u64,
    pub pieces: u128,
    pub flags: u8,
    pub half_moves: u8,
    pub full_moves: u32,
    pub piece_count: u8
}

// Concepts:
// Bitboard: a u64 containing a 1 for every piece and a 0 for every empty square
// ordering is lsb=bottom right of board and msb = top right of board
// thus H1->A1;H2->A2;...;H8->H1

impl BoardState {
    pub fn from_fen(fen: &String) -> BoardState {
        let mut bitboard: u64 = 0;
        let mut pieces: u128 = 0;
        let mut flags = 0;

        let mut i: usize = 0;
        let mut file: i16 = 7;
        let mut rank: i16 = 7;
        // Pieces
        while i < fen.len() {
            let char: char = fen.chars().nth(i).unwrap();
            i += 1;

            if char.is_ascii_digit() {
                let digit = char as i16 - 0x30;
                rank -= digit;
                continue;
            }

            if char == '/' {
                rank = 7;
                file -= 1;
                continue;
            }

            if char == ' ' {
                break;
            }

            if rank < 0 {
                panic!("Rank shouldn't be < 0 at {}", &fen[..i]);
            }

            let piece_position: u64 = 1 << ((file * 8) + rank);

            bitboard = bitboard + piece_position;
            rank = rank - 1;

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

            let piece_u128: u128 = piece as u128;
            pieces = (pieces << 4) | piece_u128;
        }

        // Turn
        let white_turn = if fen.chars().nth(i).unwrap() == 'w' {
            1
        } else {
            0
        };
        flags += white_turn;
        i += 2;

        // Castling
        let mut can_castle: u8 = 0;
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
            let rank = RANKS.find(ep_char).unwrap() as u8;
            flags += rank << 5;
            i += 1;
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
        let piece_count = bitboard.count_ones() as u8;

        Self {
            bitboard,
            pieces,
            flags,
            half_moves,
            full_moves,
            piece_count
        }
    }

    pub fn to_fen(&self) -> String {
        let mut file_index = 7;
        let mut fen: String = String::default();
        let mut piece_index: i8 = (self.bitboard.count_ones() - 1).try_into().unwrap();

        // Pieces
        loop {
            let file: u8 = self.bitboard.copy_b(file_index * 8, 8).try_into().unwrap();
            let pieces_in_row: i8 = file.count_ones().try_into().unwrap();
            fen += &file_to_fen_string(file, &self.pieces, piece_index);
            piece_index -= pieces_in_row;
            if (file_index > 0) {
                file_index -= 1;
                fen += &"/".to_string();
            } else {
                break;
            }
            // if file_index < 8 {
            //     fen += &"/".to_string();
            // }
        }

        // Flags
        // Move
        let white_move = (self.flags & 1) > 0;
        fen += if white_move { " w" } else { " b" };

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
        let rank = self.flags >> 5;
        fen += " ";
        if rank > 0 {
            let rank_char = RANKS.chars().nth(rank as usize).unwrap();
            fen += &rank_char.to_string();
            // if it's whites move next then this move was by black
            fen += if white_move { "6".into() } else { "3".into() };
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

fn file_to_fen_string(file: u8, pieces: &u128, piece_index: i8) -> String {
    let mut i: i8 = 7;
    let mut pi = piece_index;
    let mut empty_count = 0;
    let mut r = String::default();
    while i >= 0 && pi < 32 {
        let occ = ((file >> i) & 1) > 0;
        if occ {
            if empty_count > 0 {
                r += &empty_count.to_string();
                empty_count = 0;
            }
            let piece = get_piece_code(pieces, pi as u8);
            r += &get_piece_char(piece).to_string();
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
