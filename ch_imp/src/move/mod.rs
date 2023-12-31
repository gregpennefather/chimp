use crate::{
    board::{
        attack_and_defend_lookups::AttackedBy,
        see::{see, see_from_capture},
    },
    shared::{
        board_utils::get_coords_from_index,
        constants::{
            MF_BISHOP_CAPTURE_PROMOTION, MF_BISHOP_PROMOTION, MF_CAPTURE, MF_DOUBLE_PAWN_PUSH,
            MF_EP_CAPTURE, MF_KING_CASTLING, MF_KNIGHT_CAPTURE_PROMOTION, MF_KNIGHT_PROMOTION,
            MF_PROMOTION, MF_QUEEN_CAPTURE_PROMOTION, MF_QUEEN_CASTLING, MF_QUEEN_PROMOTION,
            MF_ROOK_CAPTURE_PROMOTION, MF_ROOK_PROMOTION,
        },
        piece_type::{get_piece_char, PieceType, PIECE_TYPE_EXCHANGE_VALUE},
    },
};
use core::fmt::Debug;
use std::{cmp::Ordering, fmt::Display};

pub mod move_data;
pub mod move_magic_bitboards;
pub mod move_segment;

#[derive(Default, Clone, Copy, Eq)]
pub struct Move(u16, PieceType, bool, i8,i16);

impl Move {
    pub fn new(
        from_index: u8,
        to_index: u8,
        flags: u16,
        piece_type: PieceType,
        is_black: bool,
        see_value: i8,
        square_delta: i16
    ) -> Move {
        let f: u16 = from_index.into();
        let t: u16 = to_index.into();
        let m: u16 = f << 10 | t << 4 | flags;
        Move(m, piece_type, is_black, see_value, square_delta)
    }

    pub fn capture_move(
        from_index: u8,
        to_index: u8,
        attacker_piece_type: PieceType,
        attacked_piece_type: PieceType,
        is_black: bool,
        friendly_attacked_by: AttackedBy,
        opponent_attacked_by: AttackedBy,
        square_delta: i16
    ) -> Self {
        let f: u16 = from_index.into();
        let t: u16 = to_index.into();
        let m: u16 = f << 10 | t << 4 | MF_CAPTURE;
        let see_value = see_from_capture(
            attacker_piece_type,
            friendly_attacked_by,
            attacked_piece_type,
            opponent_attacked_by,
        );
        Move(m, attacker_piece_type, is_black, see_value, square_delta)
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

    pub fn is_quiet(&self) -> bool {
        self.3 == 0
    }

    pub fn see(&self) -> i8 {
        self.3
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

    pub fn is_empty(&self) -> bool {
        self.0 == 0
    }
}

impl Debug for Move {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut m_info = String::default();
        m_info += &format!("{}", get_piece_char(self.piece_type(), self.is_black()));
        m_info += match self.flags() {
            MF_EP_CAPTURE | MF_CAPTURE => "+",
            MF_KING_CASTLING => "o-o",
            MF_QUEEN_CASTLING => "o-o-o",
            MF_KNIGHT_PROMOTION | MF_KNIGHT_CAPTURE_PROMOTION => "n",
            MF_BISHOP_PROMOTION | MF_BISHOP_CAPTURE_PROMOTION => "b",
            MF_ROOK_PROMOTION | MF_ROOK_CAPTURE_PROMOTION => "r",
            MF_QUEEN_PROMOTION | MF_QUEEN_CAPTURE_PROMOTION => "q",
            MF_DOUBLE_PAWN_PUSH => "dpp",
            _ => "",
        };

        if self.see() != 0 {
            m_info += &format!("({})", self.see())
        }

        f.debug_tuple("Move")
            .field(&format!("{}-{}", self.uci(), m_info))
            .finish()
    }
}

impl Display for Move {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut m_info = String::default();
        m_info += &format!("{}", get_piece_char(self.piece_type(), self.is_black()));
        m_info += match self.flags() {
            MF_EP_CAPTURE | MF_CAPTURE => "+",
            MF_KING_CASTLING => "o-o",
            MF_QUEEN_CASTLING => "o-o-o",
            MF_KNIGHT_PROMOTION | MF_KNIGHT_CAPTURE_PROMOTION => "n",
            MF_BISHOP_PROMOTION | MF_BISHOP_CAPTURE_PROMOTION => "b",
            MF_ROOK_PROMOTION | MF_ROOK_CAPTURE_PROMOTION => "r",
            MF_QUEEN_PROMOTION | MF_QUEEN_CAPTURE_PROMOTION => "q",
            MF_DOUBLE_PAWN_PUSH => "dpp",
            _ => "",
        };

        f.debug_tuple("Move")
            .field(&format!("{}-{}", self.uci(), m_info))
            .finish()
    }
}

impl PartialOrd for Move {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        // Better SEE
        let see_result = other.see().cmp(&self.3);
        if see_result != Ordering::Equal {
            return Some(see_result);
        }

        // Captures should be LV first
        if self.is_capture() {
            if !other.is_capture() {
                return Some(Ordering::Less);
            }
            return Some(self.1.cmp(&other.1));
        }

        if other.is_capture() {
            if !self.is_capture() {
                return Some(Ordering::Greater);
            }
            return Some(self.1.cmp(&other.1));
        }

        // Flag priority (Promotion Captures -> Promotions -> EP Capture -> Captures -> Castling -> DPP -> Quiet)
        let flags_result = other.flags().cmp(&self.flags());
        if flags_result != Ordering::Equal {
            return Some(flags_result);
        }

        // Check Pawn captures first
        if self.piece_type() == PieceType::Pawn
            && (self.flags() == MF_CAPTURE || self.flags() == MF_EP_CAPTURE)
        {
            return Some(Ordering::Less);
        }
        if other.piece_type() == PieceType::Pawn
            && (other.flags() == MF_CAPTURE || other.flags() == MF_EP_CAPTURE)
        {
            return Some(Ordering::Less);
        }

        Some(other.4.cmp(&self.4))
    }
}

impl Ord for Move {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match self.partial_cmp(other) {
            Some(r) => r,
            None => Ordering::Equal,
        }
    }
}

impl PartialEq for Move {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0 && self.1 == other.1 && self.2 == other.2
    }
}

#[cfg(test)]
mod test {
    use crate::{
        r#move::Move,
        shared::{
            constants::{MF_CAPTURE, MF_DOUBLE_PAWN_PUSH, MF_QUEEN_PROMOTION},
            piece_type::PieceType,
        },
    };

    #[test]
    pub fn order_will_prioritize_greater_valued_flags() {
        let m1 = Move::new(2, 4, MF_DOUBLE_PAWN_PUSH, PieceType::Queen, false, 0, 0);
        let m2 = Move::new(2, 4, MF_QUEEN_PROMOTION, PieceType::Queen, false, 0, 0);

        let mut vec = vec![m1, m2];
        vec.sort();
        assert_eq!(vec[0], m2);
        assert_eq!(vec[1], m1);
    }

    #[test]
    pub fn order_moves_case_capture_over_quiet() {
        let capture = Move::new(0, 1, MF_CAPTURE, PieceType::Pawn, true, 2, 0);
        let quiet = Move::new(0, 1, 0b0, PieceType::Pawn, true, 0, 0);
        let mut moves = vec![quiet, capture];

        moves.sort();
        assert_eq!(moves[0], capture);
        assert_eq!(moves[1], quiet);
    }

    #[test]
    pub fn order_moves_case_capture_with_higher_see_over_lower() {
        let major_capture = Move::new(0, 1, MF_CAPTURE, PieceType::Pawn, true, 4, 0);
        let minor_capture = Move::new(0, 1, MF_CAPTURE, PieceType::Queen, true, 1, 0);
        let mut moves = vec![minor_capture, major_capture];

        moves.sort();
        assert_eq!(moves[0], major_capture);
        assert_eq!(moves[1], minor_capture);
    }

    #[test]
    pub fn order_moves_case_equal_see_captures_check_least_valuable_piece_first() {
        let rook_capture = Move::new(0, 1, MF_CAPTURE, PieceType::Rook, true, 4, 0);
        let queen_capture = Move::new(0, 1, MF_CAPTURE, PieceType::Queen, true, 4, 0);
        let mut moves = vec![queen_capture, rook_capture];

        moves.sort();
        assert_eq!(moves[0], rook_capture);
        assert_eq!(moves[1], queen_capture);
    }

    #[test]
    pub fn order_equal_see_capture_over_dpp() {
        let pawn_capture = Move::new(0, 1, MF_CAPTURE, PieceType::Pawn, true, 0, 0);
        let dpp = Move::new(0, 1, MF_DOUBLE_PAWN_PUSH, PieceType::Pawn, true, 0, 0);
        let mut moves = vec![dpp, pawn_capture];

        moves.sort();
        assert_eq!(moves[0], pawn_capture);
        assert_eq!(moves[1], dpp);
    }

    #[test]
    pub fn order_better_square_delta_when_moves_quiet() {
        let better_move = Move::new(0, 1, 0b0, PieceType::Pawn, true, 0, 23);
        let worse_move = Move::new(0, 1, 0b0, PieceType::Queen, true, 0, 11);
        let mut moves = vec![worse_move, better_move];

        moves.sort();
        assert_eq!(moves[0], better_move);
        assert_eq!(moves[1], worse_move);
    }


    // #[test]
    // pub fn order_moves_case_equal_see_king_last() {
    //     let king_move = Move::new(0, 1, 0b0, PieceType::King, true, 0);
    //     let pawn_move = Move::new(0, 1, 0b0, PieceType::Pawn, true, 0);
    //     let mut moves = vec![king_move, pawn_move];

    //     moves.sort();
    //     assert_eq!(moves[0], pawn_move);
    //     assert_eq!(moves[1], king_move);
    // }
}
