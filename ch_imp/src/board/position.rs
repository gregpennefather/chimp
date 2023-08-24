use crate::{
    r#move::move_segment::{MoveSegment, MoveSegmentType},
    shared::{
        board_utils::get_position_from_coords,
        piece_type::{get_piece_char, PieceType},
    },
};

use super::bitboard::Bitboard;

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Position {
    pub bitboard: Bitboard,
    pub white_bitboard: Bitboard,
    pub black_bitboard: Bitboard,
    pub pawn_bitboard: Bitboard,
    pub knight_bitboard: Bitboard,
    pub bishop_bitboard: Bitboard,
    pub rook_bitboard: Bitboard,
    pub queen_bitboard: Bitboard,
    pub king_bitboard: Bitboard,
    pub check: bool,
}

impl Position {
    pub fn new(fen: String) -> Self {
        let mut bitboard = Bitboard::default();
        let mut white_bitboard = Bitboard::default();
        let mut black_bitboard = Bitboard::default();
        let mut pawn_bitboard = Bitboard::default();
        let mut knight_bitboard = Bitboard::default();
        let mut bishop_bitboard = Bitboard::default();
        let mut rook_bitboard = Bitboard::default();
        let mut queen_bitboard = Bitboard::default();
        let mut king_bitboard = Bitboard::default();
        let mut check = false;

        let mut position_index = 63;
        let mut rank_lenght = 0; // The number of spaces we've worked through in the current rank
        for i in 0..fen.len() {
            // Get char
            let char = fen.chars().nth(i).unwrap();

            // Check if is a shift digit
            if char.is_ascii_digit() {
                let digit = (char as i16 - 0x30) as i8;
                position_index -= digit;
                rank_lenght += digit;
                continue;
            }

            if char == '/' {
                position_index -= 8 - rank_lenght;
                rank_lenght = 0;
                continue;
            }

            let piece_type = match char {
                'P' | 'p' => PieceType::Pawn,
                'B' | 'b' => PieceType::Bishop,
                'N' | 'n' => PieceType::Knight,
                'R' | 'r' => PieceType::Rook,
                'Q' | 'q' => PieceType::Queen,
                'K' | 'k' => PieceType::King,
                _ => panic!("Unknown piece type {}", char),
            };

            let piece_is_black = char.is_ascii_lowercase();
            let mut colour_board_to_change = if piece_is_black {
                black_bitboard
            } else {
                white_bitboard
            };

            match piece_type {
                PieceType::Pawn => {
                    (pawn_bitboard, colour_board_to_change, bitboard) = flip_piece_i8(
                        position_index,
                        pawn_bitboard,
                        colour_board_to_change,
                        bitboard,
                    )
                }
                PieceType::Knight => {
                    (knight_bitboard, colour_board_to_change, bitboard) = flip_piece_i8(
                        position_index,
                        knight_bitboard,
                        colour_board_to_change,
                        bitboard,
                    )
                }
                PieceType::Bishop => {
                    (bishop_bitboard, colour_board_to_change, bitboard) = flip_piece_i8(
                        position_index,
                        bishop_bitboard,
                        colour_board_to_change,
                        bitboard,
                    )
                }
                PieceType::Rook => {
                    (rook_bitboard, colour_board_to_change, bitboard) = flip_piece_i8(
                        position_index,
                        rook_bitboard,
                        colour_board_to_change,
                        bitboard,
                    )
                }
                PieceType::Queen => {
                    (queen_bitboard, colour_board_to_change, bitboard) = flip_piece_i8(
                        position_index,
                        queen_bitboard,
                        colour_board_to_change,
                        bitboard,
                    )
                }
                PieceType::King => {
                    (king_bitboard, colour_board_to_change, bitboard) = flip_piece_i8(
                        position_index,
                        king_bitboard,
                        colour_board_to_change,
                        bitboard,
                    )
                }
                _ => panic!("Unknown piece type {:?}", piece_type),
            }

            if piece_is_black {
                black_bitboard = colour_board_to_change;
            } else {
                white_bitboard = colour_board_to_change;
            }

            position_index -= 1;
            rank_lenght += 1;
        }

        Self {
            bitboard,
            white_bitboard,
            black_bitboard,
            pawn_bitboard,
            knight_bitboard,
            bishop_bitboard,
            rook_bitboard,
            queen_bitboard,
            king_bitboard,
            check,
        }
    }

    pub fn to_fen(&self) -> String {
        let mut result = "".into();
        let mut i = 64;

        let mut gap = 0;
        while i > 0 {
            if i % 8 == 0 && i != 64 {
                if gap > 0 {
                    result = format!("{result}{gap}");
                    gap = 0;
                }
                result = format!("{result}/");
            }

            let piece_type = self.get_piece_type_at_index(i - 1);
            let black_piece = self.black_bitboard.occupied(i - 1);
            match piece_type {
                PieceType::None => {
                    gap += 1;
                }
                _ => {
                    if gap > 0 {
                        result = format!("{result}{gap}");
                        gap = 0;
                    }
                    result = format!("{result}{}", get_piece_char(piece_type, black_piece));
                }
            }
            i -= 1;
        }

        if gap > 0 {
            result = format!("{result}{gap}");
            gap = 0;
        }

        result
    }

    pub fn get_piece_type_at_index(&self, index: u8) -> PieceType {
        if self.pawn_bitboard.occupied(index) {
            return PieceType::Pawn;
        }
        if self.bishop_bitboard.occupied(index) {
            return PieceType::Bishop;
        }
        if self.knight_bitboard.occupied(index) {
            return PieceType::Knight;
        }
        if self.rook_bitboard.occupied(index) {
            return PieceType::Rook;
        }
        if self.queen_bitboard.occupied(index) {
            return PieceType::Queen;
        }
        if self.king_bitboard.occupied(index) {
            return PieceType::King;
        }
        return PieceType::None;
    }

    pub(crate) fn apply_segments(&self, segments: [MoveSegment; 5]) -> Position {
        let mut bitboard = self.bitboard;
        let mut white_bitboard = self.white_bitboard;
        let mut black_bitboard = self.black_bitboard;
        let mut pawn_bitboard = self.pawn_bitboard;
        let mut knight_bitboard = self.knight_bitboard;
        let mut bishop_bitboard = self.bishop_bitboard;
        let mut rook_bitboard = self.rook_bitboard;
        let mut queen_bitboard = self.queen_bitboard;
        let mut king_bitboard = self.king_bitboard;
        let mut check = false;

        for segment in segments {
            if segment.segment_type == MoveSegmentType::Pickup
                || segment.segment_type == MoveSegmentType::Place
            {
                if segment.black_piece {
                    match segment.piece_type {
                        PieceType::None => panic!("Unexpected lack of piece type {:?}", segment),
                        PieceType::Pawn => {
                            (pawn_bitboard, black_bitboard, bitboard) =
                                flip_piece(segment.index, pawn_bitboard, black_bitboard, bitboard)
                        }
                        PieceType::Knight => {
                            (knight_bitboard, black_bitboard, bitboard) =
                                flip_piece(segment.index, knight_bitboard, black_bitboard, bitboard)
                        }
                        PieceType::Bishop => {
                            (bishop_bitboard, black_bitboard, bitboard) =
                                flip_piece(segment.index, bishop_bitboard, black_bitboard, bitboard)
                        }
                        PieceType::Rook => {
                            (rook_bitboard, black_bitboard, bitboard) =
                                flip_piece(segment.index, rook_bitboard, black_bitboard, bitboard)
                        }
                        PieceType::Queen => {
                            (queen_bitboard, black_bitboard, bitboard) =
                                flip_piece(segment.index, queen_bitboard, black_bitboard, bitboard)
                        }
                        PieceType::King => {
                            (king_bitboard, black_bitboard, bitboard) =
                                flip_piece(segment.index, king_bitboard, black_bitboard, bitboard)
                        }
                    }
                } else {
                    match segment.piece_type {
                        PieceType::None => panic!("Unexpected lack of piece type {:?}", segment),
                        PieceType::Pawn => {
                            (pawn_bitboard, white_bitboard, bitboard) =
                                flip_piece(segment.index, pawn_bitboard, white_bitboard, bitboard)
                        }
                        PieceType::Knight => {
                            (knight_bitboard, white_bitboard, bitboard) =
                                flip_piece(segment.index, knight_bitboard, white_bitboard, bitboard)
                        }
                        PieceType::Bishop => {
                            (bishop_bitboard, white_bitboard, bitboard) =
                                flip_piece(segment.index, bishop_bitboard, white_bitboard, bitboard)
                        }
                        PieceType::Rook => {
                            (rook_bitboard, white_bitboard, bitboard) =
                                flip_piece(segment.index, rook_bitboard, white_bitboard, bitboard)
                        }
                        PieceType::Queen => {
                            (queen_bitboard, white_bitboard, bitboard) =
                                flip_piece(segment.index, queen_bitboard, white_bitboard, bitboard)
                        }
                        PieceType::King => {
                            (king_bitboard, white_bitboard, bitboard) =
                                flip_piece(segment.index, king_bitboard, white_bitboard, bitboard)
                        }
                    }
                }
            }
        }

        Self {
            bitboard,
            white_bitboard,
            black_bitboard,
            pawn_bitboard,
            knight_bitboard,
            bishop_bitboard,
            rook_bitboard,
            queen_bitboard,
            king_bitboard,
            check,
        }
    }
}

impl Default for Position {
    fn default() -> Self {
        Self {
            bitboard: Bitboard::new(18446462598732906495),
            white_bitboard: Bitboard::new(65535),
            black_bitboard: Bitboard::new(18446462598732840960),
            pawn_bitboard: Bitboard::new(71776119061282560),
            knight_bitboard: Bitboard::new(4755801206503243842),
            bishop_bitboard: Bitboard::new(2594073385365405732),
            rook_bitboard: Bitboard::new(9295429630892703873),
            queen_bitboard: Bitboard::new(1152921504606846992),
            king_bitboard: Bitboard::new(576460752303423496),
            check: false,
        }
    }
}

fn flip_piece_i8(
    index: i8,
    piece_bitboard: Bitboard,
    colour_bitboard: Bitboard,
    all_bitboard: Bitboard,
) -> (Bitboard, Bitboard, Bitboard) {
    flip_piece(index as u8, piece_bitboard, colour_bitboard, all_bitboard)
}

fn flip_piece(
    index: u8,
    piece_bitboard: Bitboard,
    colour_bitboard: Bitboard,
    all_bitboard: Bitboard,
) -> (Bitboard, Bitboard, Bitboard) {
    (
        piece_bitboard.flip(index),
        colour_bitboard.flip(index),
        all_bitboard.flip(index),
    )
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    pub fn new_start_pos() {
        let result = Position::new("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR".into());
        assert_eq!(result.bitboard, Bitboard::new(18446462598732906495));
        assert_eq!(result.black_bitboard, Bitboard::new(18446462598732840960));
        assert_eq!(result.white_bitboard, Bitboard::new(65535));
        assert_eq!(result.pawn_bitboard, Bitboard::new(71776119061282560));
        assert_eq!(result.bishop_bitboard, Bitboard::new(2594073385365405732));
        assert_eq!(result.knight_bitboard, Bitboard::new(4755801206503243842));
        assert_eq!(result.rook_bitboard, Bitboard::new(9295429630892703873));
        assert_eq!(result.queen_bitboard, Bitboard::new(1152921504606846992));
        assert_eq!(result.king_bitboard, Bitboard::new(576460752303423496));
    }

    #[test]
    pub fn to_fen_startpos() {
        let result = Position::new("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR".into());
        assert_eq!(
            result.to_fen(),
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR"
        );
    }

    #[test]
    pub fn to_fen_ending_in_empty_squares() {
        let result = Position::new("rnbq1rk1/ppp2pbp/3p1np1/4p3/2PPP3/2N2N2/PP2BPPP/R1BQ1RK1".into());
        assert_eq!(
            result.to_fen(),
            "rnbq1rk1/ppp2pbp/3p1np1/4p3/2PPP3/2N2N2/PP2BPPP/R1BQ1RK1"
        );
    }
}
