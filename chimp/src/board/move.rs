use std::{
    cmp::Ordering,
    fmt::{Debug, Display},
};

use crate::board::board_utils::{char_from_file, get_file, get_friendly_name_for_index};

use super::state::BoardState;

#[derive(PartialEq, Clone, Copy, Eq, Hash)]
pub struct Move(u16);

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

impl Move {
    pub fn new(from_index: u8, to_index: u8, flags: u16) -> Move {
        let f: u16 = from_index.into();
        let t: u16 = to_index.into();
        let m: u16 = f << 10 | t << 4 | flags;
        Move(m)
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

    pub fn is_castling(&self) -> bool {
        self.flags() == KING_CASTLING || self.flags() == QUEEN_CASTLING
    }

    pub fn is_king_castling(&self) -> bool {
        self.flags() == KING_CASTLING
    }

    pub fn is_promotion(&self) -> bool {
        self.flags() & PROMOTION == PROMOTION
    }

    // Will return true if CAPTURE, EP_CAPTURE, or any CAPTURE_PROMOTION
    pub fn is_capture(&self) -> bool {
        self.flags() & CAPTURE == CAPTURE
    }

    pub fn is_ep_capture(&self) -> bool {
        self.flags() == EP_CAPTURE
    }

    pub fn is_double_pawn_push(&self) -> bool {
        self.flags() == DOUBLE_PAWN_PUSH
    }

    pub fn uci(&self) -> String {
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

    pub fn san(&self, board_state: BoardState, other_moves: Vec<Move>) -> String {
        let piece = board_state
            .pieces
            .get_by_position_index(board_state.bitboard, self.from());
        let piece_letter = piece.to_string().to_ascii_uppercase();

        let mut r = if !piece_letter.eq("P") {
            format!("{}", piece_letter)
        } else {
            "".into()
        };

        if self.is_castling() {
            if self.is_king_castling() {
                return "O-O".into();
            } else {
                return "O-O-O".into();
            }
        }

        let mut moves_targeting_square = Vec::new();
        for c_m in other_moves {
            let cm_to = c_m.to();
            let cm_from = c_m.from();
            let cm_piece = board_state
                .pieces
                .get_by_position_index(board_state.bitboard, cm_from);
            if cm_to == self.to()
                && (cm_piece == piece || piece.is(crate::board::piece::PieceType::Pawn))
            {
                moves_targeting_square.push(c_m);
            }
        }

        if moves_targeting_square.len() >= 0 {
            let from_file = char_from_file(get_file(self.from()));
            r = format!("{r}{from_file}");
        }

        if self.is_capture() {
            r = format!("{r}x");
        }

        format!("{r}{}", get_friendly_name_for_index(self.to()))
    }
}

impl Ord for Move {
    fn cmp(&self, other: &Self) -> Ordering {
        if self.flags() > other.flags() {
            std::cmp::Ordering::Less
        } else {
            std::cmp::Ordering::Greater
        }
    }
}

impl PartialOrd for Move {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(if self.flags() > other.flags() {
            std::cmp::Ordering::Less
        } else {
            std::cmp::Ordering::Greater
        })
    }

    fn lt(&self, other: &Self) -> bool {
        matches!(self.partial_cmp(other), Some(std::cmp::Ordering::Less))
    }

    fn le(&self, other: &Self) -> bool {
        matches!(
            self.partial_cmp(other),
            Some(std::cmp::Ordering::Less | std::cmp::Ordering::Equal)
        )
    }

    fn gt(&self, other: &Self) -> bool {
        matches!(self.partial_cmp(other), Some(std::cmp::Ordering::Greater))
    }

    fn ge(&self, other: &Self) -> bool {
        matches!(
            self.partial_cmp(other),
            Some(std::cmp::Ordering::Greater | std::cmp::Ordering::Equal)
        )
    }
}

impl Display for Move {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!("{}", self.uci()))
    }
}

impl Debug for Move {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Move").field(&self.to_string()).finish()
    }
}

impl Default for Move {
    fn default() -> Self {
        Self(Default::default())
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
        assert_eq!(r.0, 0b0010110100110000);
    }

    #[test]
    pub fn build_move_a7_a8_pawn_push() {
        let from_index = 63; // 111111
        let to_index = 55; // 110111
        let r = Move::new(from_index, to_index, 0b0u16);
        assert_eq!(r.0, 0b1111111101110000);
    }

    #[test]
    pub fn move_sort() {
        let quiet = Move::new(3, 4, 0b0);
        let capture = Move::new(6, 2, CAPTURE);
        let promote_queen_capture = Move::new(1, 2, QUEEN_CAPTURE_PROMOTION);
        let mut moves = [
            quiet,
            capture,
            promote_queen_capture,
        ];
        moves.sort();
        assert_eq!(moves[0], promote_queen_capture);
        assert_eq!(moves[1], capture);
        assert_eq!(moves[2], quiet);
    }
}
