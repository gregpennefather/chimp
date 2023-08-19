use crate::{
    board::{
        board_utils::{
            char_from_rank, get_friendly_name_for_index, get_piece_from_position_index, get_rank,
        },
        piece_utils::get_piece_char,
    },
    shared::{BLACK_PAWN, PAWN_INDEX},
};

use super::state::BoardState;

pub type Move = u16;

// https://www.chessprogramming.org/Encoding_Moves
pub const DOUBLE_PAWN_PUSH: u16 = 0b0001;
pub const KING_CASTLING: u16 = 0b0010;
pub const QUEEN_CASTLING: u16 = 0b0011;
pub const CAPTURE: u16 = 0b0100;
pub const EP_CAPTURE: u16 = 0b0101;
pub const PROMOTION: u16 = 0b1000;
pub const KNIGHT_PROMOTION: u16 = 0b1000;
pub const KNIGHT_CAPTURE_PROMOTION: u16 = 0b1100;
pub const BISHOP_PROMOTION: u16 = 0b1001;
pub const BISHOP_CAPTURE_PROMOTION: u16 = 0b1101;
pub const ROOK_PROMOTION: u16 = 0b1010;
pub const ROOK_CAPTURE_PROMOTION: u16 = 0b1110;
pub const QUEEN_PROMOTION: u16 = 0b1011;
pub const QUEEN_CAPTURE_PROMOTION: u16 = 0b1111;

pub trait MoveFunctions {
    fn new(from_index: u8, to_index: u8, flags: u16) -> Move;
    fn from(&self) -> u8;
    fn to(&self) -> u8;
    fn flags(&self) -> u16;
    fn is_castling(&self) -> bool;
    fn is_king_castling(&self) -> bool;
    fn is_promotion(&self) -> bool;
    fn is_capture(&self) -> bool;
    fn is_ep_capture(&self) -> bool;
    fn is_double_pawn_push(&self) -> bool;
    fn uci(&self) -> String;
    fn san(&self, board_state: BoardState, other_moves: Vec<Move>) -> String;
}

impl MoveFunctions for Move {
    fn new(from_index: u8, to_index: u8, flags: u16) -> Move {
        let f: u16 = from_index.into();
        let t: u16 = to_index.into();
        let m: u16 = f << 10 | t << 4 | flags;
        m
    }

    fn from(&self) -> u8 {
        (*self >> 10).try_into().unwrap()
    }

    fn to(&self) -> u8 {
        (*self >> 4 & 0b111111).try_into().unwrap()
    }

    fn flags(&self) -> u16 {
        *self & 0b1111
    }

    fn is_castling(&self) -> bool {
        self.flags() == KING_CASTLING || self.flags() == QUEEN_CASTLING
    }

    fn is_king_castling(&self) -> bool {
        self.flags() == KING_CASTLING
    }

    fn is_promotion(&self) -> bool {
        self.flags() & PROMOTION == PROMOTION
    }

    // Will return true if CAPTURE, EP_CAPTURE, or any CAPTURE_PROMOTION
    fn is_capture(&self) -> bool {
        self.flags() & CAPTURE == CAPTURE
    }

    fn is_ep_capture(&self) -> bool {
        self.flags() == EP_CAPTURE
    }

    fn is_double_pawn_push(&self) -> bool {
        self.flags() == DOUBLE_PAWN_PUSH
    }

    fn uci(&self) -> String {
        let promotion = match self.flags() {
            KNIGHT_PROMOTION => "n",
            KNIGHT_CAPTURE_PROMOTION => "n",
            BISHOP_PROMOTION => "b",
            BISHOP_CAPTURE_PROMOTION => "b",
            ROOK_PROMOTION => "r",
            ROOK_CAPTURE_PROMOTION => "r",
            QUEEN_PROMOTION => "q",
            QUEEN_CAPTURE_PROMOTION => "q",
            _ => "",
        };
        format!(
            "{}{}{}",
            get_friendly_name_for_index(self.from()),
            get_friendly_name_for_index(self.to()),
            promotion
        )
    }

    fn san(&self, board_state: BoardState, other_moves: Vec<Move>) -> String {
        let piece =
            get_piece_from_position_index(board_state.bitboard, board_state.pieces, self.from());
        let piece_letter = get_piece_char(piece).to_ascii_uppercase();

        let mut r = if piece_letter != 'P' {
            format!("{}", piece_letter)
        } else {
            "".into()
        };

        if self.is_capture() {
            if self.is_king_castling() {
                return "O-O".into();
            } else {
                return "O-O-O".into();
            }
        }

        let mut moves_targeting_square = Vec::new();
        for c_m in other_moves {
            let cm_to = (c_m >> 4 & 0b111111) as u8;
            let cm_from = (c_m >> 10) as u8;
            let cm_piece =
                get_piece_from_position_index(board_state.bitboard, board_state.pieces, cm_from);
            if cm_to == self.to()
                && (cm_piece == piece || piece == PAWN_INDEX || piece == BLACK_PAWN)
            {
                moves_targeting_square.push(c_m);
            }
        }

        if moves_targeting_square.len() >= 1 {
            let from_rank = char_from_rank(get_rank(self.from()));
            r = format!("{r}{from_rank}");
        }

        if self.is_capture() {
            r = format!("{r}x");
        }

        format!("{r}{}", get_friendly_name_for_index(self.to()))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    pub fn build_move_e1_e2_pawn_push() {
        let from_index = 11; // 001011
        let to_index = 19; // 010011
        let r = Move::new(from_index, to_index, 0b0u16);
        println!("{r:#018b}");
        assert_eq!(r, 0b0010110100110000);
    }

    #[test]
    pub fn build_move_a7_a8_pawn_push() {
        let from_index = 63; // 111111
        let to_index = 55; // 110111
        let r = Move::new(from_index, to_index, 0b0u16);
        println!("{r:#018b}");
        assert_eq!(r, 0b1111111101110000);
    }
}
