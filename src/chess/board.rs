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

use std::fmt;

pub const PAWN_INDEX: u8 = 1;
pub const KNIGHT_INDEX: u8 = 2;
pub const BISHOP_INDEX: u8 = 3;
pub const ROOK_INDEX: u8 = 4;
pub const QUEEN_INDEX: u8 = 5;
pub const KING_INDEX: u8 = 6;
pub const BLACK_MASK: u8 = 8;
pub const COLOURED_PIECE_MASK: u8 = 15;
pub const PIECE_MASK: u8 = 7;

static RANKS: &str = "ABCDEFGH";

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

#[derive(Default, Copy, Clone)]
pub struct Piece {
    pub file: u8,
    pub rank: u8,
    pub code: u8,
}

impl fmt::Debug for Piece {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let colour = if (&self.code >> 3) > 0 {
            "Black"
        } else {
            "White"
        };
        let piece_code = &self.code & PIECE_MASK;
        let piece_type = match piece_code {
            PAWN_INDEX => "Pawn",
            KNIGHT_INDEX => "Knight",
            BISHOP_INDEX => "Bishop",
            ROOK_INDEX => "Rook",
            QUEEN_INDEX => "Queen",
            KING_INDEX => "King",
            _ => "Unknown",
        };
        let rank_c = RANKS.chars().nth(self.rank.into()).unwrap();
        let pos = format!("{rank_c}{}", self.file + 1);

        f.debug_struct("Piece")
            .field("pos", &pos)
            .field("colour", &colour)
            .field("type", &piece_type)
            .finish()
    }
}

impl Piece {
    pub fn empty(&self) -> bool {
        return &self.code <= &0;
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
                    let piece = Piece { file, rank, code };
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
}
