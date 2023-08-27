use crate::{
    r#move::{move_segment::{MoveSegment, MoveSegmentType}, Move},
    shared::{
        board_utils::get_index_from_file_and_rank,
        piece_type::{get_piece_char, PieceType}, constants::MF_EP_CAPTURE,
    },
};

use super::bitboard::Bitboard;

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Position {
    pub occupancy: Bitboard,
    pub white_bitboard: Bitboard,
    pub black_bitboard: Bitboard,
    pub pawn_bitboard: Bitboard,
    pub knight_bitboard: Bitboard,
    pub bishop_bitboard: Bitboard,
    pub rook_bitboard: Bitboard,
    pub queen_bitboard: Bitboard,
    pub king_bitboard: Bitboard,
    pub white_in_check: bool,
    pub black_in_check: bool,
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
            occupancy: bitboard,
            white_bitboard,
            black_bitboard,
            pawn_bitboard,
            knight_bitboard,
            bishop_bitboard,
            rook_bitboard,
            queen_bitboard,
            king_bitboard,
            white_in_check,
            black_in_check,
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

    pub fn generate_move_segments(&self, m: &Move, black_turn: bool) -> [MoveSegment; 5] {
        let mut segments = [MoveSegment::default(); 5];

        let from_index = m.from();
        let to_index = m.to();
        let mut segment_index = 0;

        if m.is_castling() {
            let (rook_from_index, rook_to_index) = if m.is_king_castling() {
                (from_index - 3, to_index + 1)
            } else {
                (from_index + 4, to_index - 1)
            };

            segments[0] = MoveSegment::new(
                MoveSegmentType::Pickup,
                from_index,
                PieceType::King,
                black_turn,
            ); // remove king
            segments[1] = MoveSegment::new(
                MoveSegmentType::Pickup,
                rook_from_index,
                PieceType::Rook,
                black_turn,
            ); // remove rook
            segments[2] = MoveSegment::new(
                MoveSegmentType::Place,
                to_index,
                PieceType::King,
                black_turn,
            ); // place king
            segments[3] = MoveSegment::new(
                MoveSegmentType::Place,
                rook_to_index,
                PieceType::Rook,
                black_turn,
            ); // place rook
            segments[4] = MoveSegment::new(
                MoveSegmentType::ClearCastling,
                from_index,
                PieceType::King,
                black_turn,
            ); // place rook
            segment_index = 5;
        } else if m.is_promotion() {
            segments[segment_index] = MoveSegment::new(
                MoveSegmentType::Pickup,
                from_index,
                PieceType::Pawn,
                black_turn,
            ); // pickup Pawn
            segment_index += 1;

            if m.is_capture() {
                let captured_piece_type = self.get_piece_type_at_index(to_index);

                segments[segment_index] = MoveSegment::new(
                    MoveSegmentType::Pickup,
                    to_index,
                    captured_piece_type,
                    !black_turn,
                ); // pickup captured piece
                segment_index += 1;

                // TODO: Rook captures clear castling rights
            }

            let promotion_piece_type = match m.flags() {
                8 | 12 => PieceType::Knight,
                9 | 13 => PieceType::Bishop,
                10 | 14 => PieceType::Rook,
                11 | 15 => PieceType::Queen,
                _ => panic!("Unknown promotion: {:?}", m.flags()),
            };

            segments[segment_index] = MoveSegment::new(
                MoveSegmentType::Place,
                to_index,
                promotion_piece_type,
                black_turn,
            ); // place new piece
            segment_index += 1;
        } else {
            segments[segment_index] = MoveSegment::new(
                MoveSegmentType::Pickup,
                from_index,
                m.piece_type(),
                black_turn,
            ); // pickup piece
            segment_index += 1;

            if m.is_capture() {
                let captured_piece_type = if m.flags() == MF_EP_CAPTURE {
                    PieceType::Pawn
                } else {
                    self.get_piece_type_at_index(to_index)
                };
                segments[segment_index] = MoveSegment::new(
                    MoveSegmentType::Pickup,
                    to_index,
                    captured_piece_type,
                    !black_turn,
                ); // pickup captured piece
                segment_index += 1;

                if captured_piece_type == PieceType::Rook {
                    segments[segment_index] = MoveSegment::new(
                        MoveSegmentType::ClearCastling,
                        to_index,
                        PieceType::Rook,
                        !black_turn,
                    ); // place new piece
                    segment_index += 1;
                }
            }

            segments[segment_index] = MoveSegment::new(
                MoveSegmentType::Place,
                to_index,
                m.piece_type(),
                black_turn,
            ); // place new piece
            segment_index += 1;

            if m.is_double_pawn_push() {
                segments[segment_index] = MoveSegment::new(
                    MoveSegmentType::DoublePawnPush,
                    (from_index + to_index) / 2,
                    m.piece_type(),
                    black_turn,
                ); // place new piece
                segment_index += 1;
            }

            // Rook or King move clear castling for that piece
            if m.piece_type() == PieceType::Rook || m.piece_type() == PieceType::King {
                segments[segment_index] = MoveSegment::new(
                    MoveSegmentType::ClearCastling,
                    from_index,
                    m.piece_type(),
                    black_turn,
                ); // place new piece
                segment_index += 1;
            }
        }

        segments
    }


    pub(crate) fn apply_segments(&self, segments: [MoveSegment; 5]) -> Position {
        let mut bitboard = self.occupancy;
        let mut white_bitboard = self.white_bitboard;
        let mut black_bitboard = self.black_bitboard;
        let mut pawn_bitboard = self.pawn_bitboard;
        let mut knight_bitboard = self.knight_bitboard;
        let mut bishop_bitboard = self.bishop_bitboard;
        let mut rook_bitboard = self.rook_bitboard;
        let mut queen_bitboard = self.queen_bitboard;
        let mut king_bitboard = self.king_bitboard;

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
            occupancy: bitboard,
            white_bitboard,
            black_bitboard,
            pawn_bitboard,
            knight_bitboard,
            bishop_bitboard,
            rook_bitboard,
            queen_bitboard,
            king_bitboard,
            white_in_check,
            black_in_check
        }
    }
}

impl Default for Position {
    fn default() -> Self {
        Self {
            occupancy: Bitboard::new(18446462598732906495),
            white_bitboard: Bitboard::new(65535),
            black_bitboard: Bitboard::new(18446462598732840960),
            pawn_bitboard: Bitboard::new(71776119061282560),
            knight_bitboard: Bitboard::new(4755801206503243842),
            bishop_bitboard: Bitboard::new(2594073385365405732),
            rook_bitboard: Bitboard::new(9295429630892703873),
            queen_bitboard: Bitboard::new(1152921504606846992),
            king_bitboard: Bitboard::new(576460752303423496),
            white_in_check: false,
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
    use crate::shared::constants::{MF_DOUBLE_PAWN_PUSH, MF_KING_CASTLING, MF_CAPTURE};

    use super::*;

    #[test]
    pub fn new_start_pos() {
        let result = Position::new("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR".into());
        assert_eq!(result.occupancy, Bitboard::new(18446462598732906495));
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

    #[test]
    pub fn generate_move_segments_start_pos_e4() {
        let position = Position::default();
        let m = Move::new(11, 27, MF_DOUBLE_PAWN_PUSH, PieceType::Pawn, false);

        let segments = position.generate_move_segments(&m, false);

        assert_eq!(
            segments[0],
            MoveSegment::new(MoveSegmentType::Pickup, 11, PieceType::Pawn, false)
        );

        assert_eq!(
            segments[1],
            MoveSegment::new(MoveSegmentType::Place, 27, PieceType::Pawn, false)
        );

        assert_eq!(
            segments[2],
            MoveSegment::new(MoveSegmentType::DoublePawnPush, 19, PieceType::Pawn, false)
        );
        assert_eq!(segments[3], MoveSegment::default());
        assert_eq!(segments[4], MoveSegment::default());
    }

    #[test]
    pub fn generate_move_segments_black_castling_kingside() {
        let position = Position::new(
            "rnbqk2r/pppp1ppp/5n2/2b1p3/2B1P3/2NP4/PPP2PPP/R1BQK1NR".to_string(),
        );
        let m = Move::new(59, 57, MF_KING_CASTLING, PieceType::King, true);

        let segments = position.generate_move_segments(&m, true);

        assert_eq!(
            segments[0],
            MoveSegment::new(MoveSegmentType::Pickup, 59, PieceType::King, true)
        );

        assert_eq!(
            segments[1],
            MoveSegment::new(MoveSegmentType::Pickup, 56, PieceType::Rook, true)
        );

        assert_eq!(
            segments[2],
            MoveSegment::new(MoveSegmentType::Place, 57, PieceType::King, true)
        );
        assert_eq!(
            segments[3],
            MoveSegment::new(MoveSegmentType::Place, 58, PieceType::Rook, true)
        );
        assert_eq!(
            segments[4],
            MoveSegment::new(MoveSegmentType::ClearCastling, 59, PieceType::King, true)
        );
    }

    #[test]
    pub fn generate_move_segments_white_moves_queenside_rook_clear_castling() {
        let position = Position::new(
            "rnbqk2r/pppp1ppp/5n2/2b1p3/2B1P3/2NP4/PPP2PPP/R1BQK1NR".to_string(),
        );
        let m = Move::new(7, 6, 0b0, PieceType::Rook, true);

        let segments = position.generate_move_segments(&m, false);

        assert_eq!(
            segments[0],
            MoveSegment::new(MoveSegmentType::Pickup, 7, PieceType::Rook, false)
        );

        assert_eq!(
            segments[1],
            MoveSegment::new(MoveSegmentType::Place, 6, PieceType::Rook, false)
        );

        assert_eq!(
            segments[2],
            MoveSegment::new(MoveSegmentType::ClearCastling, 7, PieceType::Rook, false)
        );
        assert_eq!(segments[3], MoveSegment::default());
        assert_eq!(segments[4], MoveSegment::default());
    }

    #[test]
    pub fn generate_move_segments_white_captures_black_rook_clearing_kingside_castling() {
        let position = Position::new("r3k2r/8/8/8/8/8/8/R3K2R".to_string());
        let m = Move::new(63, 7, MF_CAPTURE, PieceType::Rook, true);

        let segments = position.generate_move_segments(&m, true);

        assert_eq!(
            segments[0],
            MoveSegment::new(MoveSegmentType::Pickup, 63, PieceType::Rook, true)
        );

        assert_eq!(
            segments[1],
            MoveSegment::new(MoveSegmentType::Pickup, 7, PieceType::Rook, false)
        );

        assert_eq!(
            segments[2],
            MoveSegment::new(MoveSegmentType::ClearCastling, 7, PieceType::Rook, false)
        );

        assert_eq!(
            segments[3],
            MoveSegment::new(MoveSegmentType::Place, 7, PieceType::Rook, true)
        );
        assert_eq!(
            segments[4],
            MoveSegment::new(MoveSegmentType::ClearCastling, 63, PieceType::Rook, true)
        );
    }

    #[test]
    pub fn generate_move_segments_black_moves_king_clearing_castling() {
        let position = Position::new(
            "rnbqkbnr/pppp1ppp/4p3/8/2B5/4P3/PPPP1PPP/RNBQK1NR".to_string(),
        );
        let m = Move::new(56, 48, 0b0, PieceType::King, true);

        let segments = position.generate_move_segments(&m, true);

        assert_eq!(
            segments[0],
            MoveSegment::new(MoveSegmentType::Pickup, 56, PieceType::King, true)
        );

        assert_eq!(
            segments[1],
            MoveSegment::new(MoveSegmentType::Place, 48, PieceType::King, true)
        );

        assert_eq!(
            segments[2],
            MoveSegment::new(MoveSegmentType::ClearCastling, 56, PieceType::King, true)
        );

        assert_eq!(segments[3], MoveSegment::default());
        assert_eq!(segments[4], MoveSegment::default());
    }

}
