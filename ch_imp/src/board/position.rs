use crate::{
    r#move::{
        move_data::MoveData,
        move_segment::{MoveSegment, MoveSegmentType},
        Move,
    },
    search::zorb_set_precomputed::ZORB_SET,
    shared::{
        board_utils::{get_coords_from_index, get_index_from_file_and_rank, index_from_coords},
        constants::MF_EP_CAPTURE,
        piece_type::{get_piece_char, PieceType},
    },
    MOVE_DATA,
};
use std::fmt::Debug;

use super::bitboard::Bitboard;

#[derive(Copy, Clone, PartialEq)]
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
    pub white_queen_side_castling: bool,
    pub white_king_side_castling: bool,
    pub black_queen_side_castling: bool,
    pub black_king_side_castling: bool,
    pub ep_index: u8,
    pub zorb_key: u64,
    pub black_turn: bool,
    pub white_moves: [Move; 128],
    pub black_moves: [Move; 128],
    pub white_threatboard: u64,
    pub black_threatboard: u64,
    pub white_mobility_board: u64,
    pub black_mobility_board: u64,
}

impl Position {
    pub fn new(
        position_segment: String,
        turn_segment: String,
        castling_segment: String,
        ep_segment: String,
    ) -> Self {
        let mut occupancy = Bitboard::default();
        let mut white_bitboard = Bitboard::default();
        let mut black_bitboard = Bitboard::default();
        let mut pawn_bitboard = Bitboard::default();
        let mut knight_bitboard = Bitboard::default();
        let mut bishop_bitboard = Bitboard::default();
        let mut rook_bitboard = Bitboard::default();
        let mut queen_bitboard = Bitboard::default();
        let mut king_bitboard = Bitboard::default();
        let mut white_queen_side_castling = false;
        let mut white_king_side_castling = false;
        let mut black_queen_side_castling = false;
        let mut black_king_side_castling = false;

        let mut position_index = 63;
        let mut rank_length = 0; // The number of spaces we've worked through in the current rank
        for i in 0..position_segment.len() {
            // Get char
            let char = position_segment.chars().nth(i).unwrap();

            // Check if is a shift digit
            if char.is_ascii_digit() {
                let digit = (char as i16 - 0x30) as i8;
                position_index -= digit;
                rank_length += digit;
                continue;
            }

            if char == '/' {
                position_index -= 8 - rank_length;
                rank_length = 0;
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
                    (pawn_bitboard, colour_board_to_change, occupancy) = flip_piece_i8(
                        position_index,
                        pawn_bitboard,
                        colour_board_to_change,
                        occupancy,
                    )
                }
                PieceType::Knight => {
                    (knight_bitboard, colour_board_to_change, occupancy) = flip_piece_i8(
                        position_index,
                        knight_bitboard,
                        colour_board_to_change,
                        occupancy,
                    )
                }
                PieceType::Bishop => {
                    (bishop_bitboard, colour_board_to_change, occupancy) = flip_piece_i8(
                        position_index,
                        bishop_bitboard,
                        colour_board_to_change,
                        occupancy,
                    )
                }
                PieceType::Rook => {
                    (rook_bitboard, colour_board_to_change, occupancy) = flip_piece_i8(
                        position_index,
                        rook_bitboard,
                        colour_board_to_change,
                        occupancy,
                    )
                }
                PieceType::Queen => {
                    (queen_bitboard, colour_board_to_change, occupancy) = flip_piece_i8(
                        position_index,
                        queen_bitboard,
                        colour_board_to_change,
                        occupancy,
                    )
                }
                PieceType::King => {
                    (king_bitboard, colour_board_to_change, occupancy) = flip_piece_i8(
                        position_index,
                        king_bitboard,
                        colour_board_to_change,
                        occupancy,
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
            rank_length += 1;
        }

        let black_turn = turn_segment.eq_ignore_ascii_case("b");

        if !castling_segment.eq_ignore_ascii_case("-") {
            if castling_segment.contains("K") {
                white_king_side_castling = true;
            }
            if castling_segment.contains("Q") {
                white_queen_side_castling = true;
            }
            if castling_segment.contains("k") {
                black_king_side_castling = true;
            }
            if castling_segment.contains("q") {
                black_queen_side_castling = true;
            }
        }

        let ep_index = if ep_segment.eq("-") {
            u8::MAX
        } else {
            index_from_coords(&ep_segment)
        };

        let mut output = Self {
            occupancy,
            white_bitboard,
            black_bitboard,
            pawn_bitboard,
            knight_bitboard,
            bishop_bitboard,
            rook_bitboard,
            queen_bitboard,
            king_bitboard,
            white_in_check: false, // TODO
            black_in_check: false, // TODO
            ep_index,
            white_queen_side_castling,
            white_king_side_castling,
            black_king_side_castling,
            black_queen_side_castling,
            black_turn,
            zorb_key: 0,
            white_moves: [Move::default(); 128],
            black_moves: [Move::default(); 128],
            white_threatboard: 0,
            black_threatboard: 0,
            white_mobility_board: 0,
            black_mobility_board: 0,
        };

        output.zorb_key = ZORB_SET.hash(output);

        let (
            white_moves,
            black_moves,
            white_threatboard,
            black_threatboard,
            white_mobility_board,
            black_mobility_board,
        ) = MOVE_DATA.generate_moves(output);

        output.white_moves = to_move_array(white_moves);
        output.black_moves = to_move_array(black_moves);
        output.white_threatboard = white_threatboard;
        output.black_threatboard = black_threatboard;
        output.white_mobility_board = white_mobility_board;
        output.black_mobility_board = black_mobility_board;

        output.white_in_check = (Bitboard::new(black_threatboard) & white_bitboard & king_bitboard)
            .count_occupied()
            != 0;
        output.black_in_check = (Bitboard::new(white_threatboard) & black_bitboard & king_bitboard)
            .count_occupied()
            != 0;

        output
    }

    pub fn from_fen(fen: String) -> Self {
        let mut fen_segments = fen.split_whitespace();

        let position_segment = fen_segments.nth(0).unwrap().to_string();
        let turn_segment = fen_segments.nth(0).unwrap().to_string();
        let castling_segment = fen_segments.nth(0).unwrap().to_string();
        let ep_segment = fen_segments.nth(0).unwrap().to_string();
        Position::new(position_segment, turn_segment, castling_segment, ep_segment)
    }

    pub fn legal(&self) -> bool {
        return !((self.black_turn && self.white_in_check) || (!self.black_turn && self.black_in_check))
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

        result += if self.black_turn { " b" } else { " w" };
        result += if self.white_king_side_castling {
            " K"
        } else {
            " "
        };
        result += if self.white_queen_side_castling {
            "Q"
        } else {
            ""
        };
        result += if self.black_king_side_castling {
            "k"
        } else {
            ""
        };
        result += if self.black_queen_side_castling {
            "q"
        } else {
            ""
        };
        result += if !self.white_queen_side_castling
            && !self.white_king_side_castling
            && !self.black_queen_side_castling
            && !self.black_king_side_castling
        {
            "-"
        } else {
            ""
        };

        result += " ";
        if self.ep_index != u8::MAX {
            result = format!("{result}{}", get_coords_from_index(self.ep_index));
        } else {
            result += "-";
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

    pub fn make(&self, m: Move) -> Self {
        let move_segments = self.generate_move_segments(&m);
        self.apply_segments(move_segments)
    }

    pub fn zorb_key_after_move(&self, m: Move) -> u64 {
        let segments = self.generate_move_segments(&m);
        ZORB_SET.apply_segments(self.zorb_key, self.ep_index, segments)
    }

    pub fn generate_move_segments(&self, m: &Move) -> [MoveSegment; 5] {
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
                self.black_turn,
            ); // remove king
            segments[1] = MoveSegment::new(
                MoveSegmentType::Pickup,
                rook_from_index,
                PieceType::Rook,
                self.black_turn,
            ); // remove rook
            segments[2] = MoveSegment::new(
                MoveSegmentType::Place,
                to_index,
                PieceType::King,
                self.black_turn,
            ); // place king
            segments[3] = MoveSegment::new(
                MoveSegmentType::Place,
                rook_to_index,
                PieceType::Rook,
                self.black_turn,
            ); // place rook
            segments[4] = MoveSegment::new(
                MoveSegmentType::ClearCastling,
                from_index,
                PieceType::King,
                self.black_turn,
            ); // place rook
            segment_index = 5;
        } else if m.is_promotion() {
            segments[segment_index] = MoveSegment::new(
                MoveSegmentType::Pickup,
                from_index,
                PieceType::Pawn,
                self.black_turn,
            ); // pickup Pawn
            segment_index += 1;

            if m.is_capture() {
                let captured_piece_type = self.get_piece_type_at_index(to_index);

                segments[segment_index] = MoveSegment::new(
                    MoveSegmentType::Pickup,
                    to_index,
                    captured_piece_type,
                    !self.black_turn,
                ); // pickup captured piece
                segment_index += 1;

                if captured_piece_type == PieceType::Rook {
                    segments[segment_index] = MoveSegment::new(
                        MoveSegmentType::ClearCastling,
                        to_index,
                        PieceType::Rook,
                        !self.black_turn,
                    ); // place new piece
                    segment_index += 1;
                }
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
                self.black_turn,
            ); // place new piece
            segment_index += 1;
        } else {
            segments[segment_index] = MoveSegment::new(
                MoveSegmentType::Pickup,
                from_index,
                m.piece_type(),
                self.black_turn,
            ); // pickup piece
            segment_index += 1;

            if m.is_capture() {
                let (captured_piece_type, captured_index) = if m.flags() == MF_EP_CAPTURE {
                    (PieceType::Pawn, if m.is_black() { self.ep_index + 8 } else { self.ep_index - 8})
                } else {
                    (self.get_piece_type_at_index(to_index), to_index)
                };
                segments[segment_index] = MoveSegment::new(
                    MoveSegmentType::Pickup,
                    captured_index,
                    captured_piece_type,
                    !self.black_turn,
                ); // pickup captured piece
                segment_index += 1;

                if captured_piece_type == PieceType::Rook {
                    segments[segment_index] = MoveSegment::new(
                        MoveSegmentType::ClearCastling,
                        captured_index,
                        PieceType::Rook,
                        !self.black_turn,
                    ); // place new piece
                    segment_index += 1;
                }
            }

            segments[segment_index] = MoveSegment::new(
                MoveSegmentType::Place,
                to_index,
                m.piece_type(),
                self.black_turn,
            ); // place new piece
            segment_index += 1;

            if m.is_double_pawn_push() {
                segments[segment_index] = MoveSegment::new(
                    MoveSegmentType::DoublePawnPush,
                    (from_index + to_index) / 2,
                    m.piece_type(),
                    self.black_turn,
                ); // place new piece
                segment_index += 1;
            }

            // Rook or King move clear castling for that piece
            if m.piece_type() == PieceType::Rook || m.piece_type() == PieceType::King {
                segments[segment_index] = MoveSegment::new(
                    MoveSegmentType::ClearCastling,
                    from_index,
                    m.piece_type(),
                    self.black_turn,
                ); // place new piece
                segment_index += 1;
            }
        }

        segments
    }

    pub(crate) fn apply_segments(&self, segments: [MoveSegment; 5]) -> Position {
        let mut occupancy = self.occupancy;
        let mut white_bitboard = self.white_bitboard;
        let mut black_bitboard = self.black_bitboard;
        let mut pawn_bitboard = self.pawn_bitboard;
        let mut knight_bitboard = self.knight_bitboard;
        let mut bishop_bitboard = self.bishop_bitboard;
        let mut rook_bitboard = self.rook_bitboard;
        let mut queen_bitboard = self.queen_bitboard;
        let mut king_bitboard = self.king_bitboard;
        let mut white_king_side_castling = self.white_king_side_castling;
        let mut white_queen_side_castling = self.white_queen_side_castling;
        let mut black_king_side_castling = self.black_king_side_castling;
        let mut black_queen_side_castling = self.black_queen_side_castling;
        let mut ep_index = u8::MAX;
        let mut zorb_key = self.zorb_key;

        for segment in segments {
            match segment.segment_type {
                MoveSegmentType::Pickup | MoveSegmentType::Place => {
                    if segment.black_piece {
                        match segment.piece_type {
                            PieceType::None => {
                                panic!("Unexpected lack of piece type {:?}", segment)
                            }
                            PieceType::Pawn => {
                                (pawn_bitboard, black_bitboard, occupancy) = flip_piece(
                                    segment.index,
                                    pawn_bitboard,
                                    black_bitboard,
                                    occupancy,
                                )
                            }
                            PieceType::Knight => {
                                (knight_bitboard, black_bitboard, occupancy) = flip_piece(
                                    segment.index,
                                    knight_bitboard,
                                    black_bitboard,
                                    occupancy,
                                )
                            }
                            PieceType::Bishop => {
                                (bishop_bitboard, black_bitboard, occupancy) = flip_piece(
                                    segment.index,
                                    bishop_bitboard,
                                    black_bitboard,
                                    occupancy,
                                )
                            }
                            PieceType::Rook => {
                                (rook_bitboard, black_bitboard, occupancy) = flip_piece(
                                    segment.index,
                                    rook_bitboard,
                                    black_bitboard,
                                    occupancy,
                                )
                            }
                            PieceType::Queen => {
                                (queen_bitboard, black_bitboard, occupancy) = flip_piece(
                                    segment.index,
                                    queen_bitboard,
                                    black_bitboard,
                                    occupancy,
                                )
                            }
                            PieceType::King => {
                                (king_bitboard, black_bitboard, occupancy) = flip_piece(
                                    segment.index,
                                    king_bitboard,
                                    black_bitboard,
                                    occupancy,
                                )
                            }
                        }
                    } else {
                        match segment.piece_type {
                            PieceType::None => {
                                panic!("Unexpected lack of piece type {:?}", segment)
                            }
                            PieceType::Pawn => {
                                (pawn_bitboard, white_bitboard, occupancy) = flip_piece(
                                    segment.index,
                                    pawn_bitboard,
                                    white_bitboard,
                                    occupancy,
                                )
                            }
                            PieceType::Knight => {
                                (knight_bitboard, white_bitboard, occupancy) = flip_piece(
                                    segment.index,
                                    knight_bitboard,
                                    white_bitboard,
                                    occupancy,
                                )
                            }
                            PieceType::Bishop => {
                                (bishop_bitboard, white_bitboard, occupancy) = flip_piece(
                                    segment.index,
                                    bishop_bitboard,
                                    white_bitboard,
                                    occupancy,
                                )
                            }
                            PieceType::Rook => {
                                (rook_bitboard, white_bitboard, occupancy) = flip_piece(
                                    segment.index,
                                    rook_bitboard,
                                    white_bitboard,
                                    occupancy,
                                )
                            }
                            PieceType::Queen => {
                                (queen_bitboard, white_bitboard, occupancy) = flip_piece(
                                    segment.index,
                                    queen_bitboard,
                                    white_bitboard,
                                    occupancy,
                                )
                            }
                            PieceType::King => {
                                (king_bitboard, white_bitboard, occupancy) = flip_piece(
                                    segment.index,
                                    king_bitboard,
                                    white_bitboard,
                                    occupancy,
                                )
                            }
                        }
                    }
                }
                MoveSegmentType::None => break,
                MoveSegmentType::ClearCastling => {
                    (
                        white_queen_side_castling,
                        white_king_side_castling,
                        black_queen_side_castling,
                        black_king_side_castling,
                    ) = modify_castling(
                        segment.index,
                        white_queen_side_castling,
                        white_king_side_castling,
                        black_queen_side_castling,
                        black_king_side_castling,
                    )
                }
                MoveSegmentType::DoublePawnPush => ep_index = segment.index,
                _ => {}
            }
        }
        zorb_key = ZORB_SET.apply_segments(zorb_key, self.ep_index, segments);

        let mut output = Self {
            occupancy,
            white_bitboard,
            black_bitboard,
            pawn_bitboard,
            knight_bitboard,
            bishop_bitboard,
            rook_bitboard,
            queen_bitboard,
            king_bitboard,
            white_in_check: false, // TODO
            black_in_check: false, // TODO
            white_king_side_castling,
            white_queen_side_castling,
            black_king_side_castling,
            black_queen_side_castling,
            ep_index,
            black_turn: !self.black_turn,
            zorb_key,
            white_moves: [Move::default(); 128],
            black_moves: [Move::default(); 128],
            white_threatboard: 0,
            black_threatboard: 0,
            white_mobility_board: 0,
            black_mobility_board: 0,
        };

        let (
            white_moves,
            black_moves,
            white_threatboard,
            black_threatboard,
            white_mobility_board,
            black_mobility_board,
        ) = MOVE_DATA.generate_moves(output);

        output.white_moves = to_move_array(white_moves);
        output.black_moves = to_move_array(black_moves);
        output.white_threatboard = white_threatboard;
        output.black_threatboard = black_threatboard;
        output.white_mobility_board = white_mobility_board;
        output.black_mobility_board = black_mobility_board;

        output.white_in_check = (Bitboard::new(black_threatboard) & white_bitboard & king_bitboard)
            .count_occupied()
            != 0;
        output.black_in_check = (Bitboard::new(white_threatboard) & black_bitboard & king_bitboard)
            .count_occupied()
            != 0;

        output
    }
}

impl Default for Position {
    fn default() -> Self {
        let mut output = Self {
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
            black_in_check: false,
            white_queen_side_castling: true,
            white_king_side_castling: true,
            black_queen_side_castling: true,
            black_king_side_castling: true,
            ep_index: u8::MAX,
            black_turn: false,
            zorb_key: 0, // TODO
            white_moves: [Move::default(); 128],
            black_moves: [Move::default(); 128],
            white_threatboard: 0,
            black_threatboard: 0,
            white_mobility_board: 0,
            black_mobility_board: 0,
        };
        output.zorb_key = ZORB_SET.hash(output);

        let (
            white_moves,
            black_moves,
            white_threatboard,
            black_threatboard,
            white_mobility_board,
            black_mobility_board,
        ) = MOVE_DATA.generate_moves(output);

        output.white_moves = to_move_array(white_moves);
        output.black_moves = to_move_array(black_moves);
        output.white_threatboard = white_threatboard;
        output.black_threatboard = black_threatboard;
        output.white_mobility_board = white_mobility_board;
        output.black_mobility_board = black_mobility_board;

        output
    }
}

impl Debug for Position {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Position")
            .field(&self.to_fen())
            .field(&self.zorb_key)
            .finish()
    }
}

fn modify_castling(
    index: u8,
    wqc: bool,
    wkc: bool,
    bqc: bool,
    bkc: bool,
) -> (bool, bool, bool, bool) {
    match index {
        0 => (wqc, false, bqc, bkc),
        3 => (false, false, bqc, bkc),
        7 => (false, wkc, bqc, bkc),
        56 => (wqc, wkc, bqc, false),
        59 => (wqc, wkc, false, false),
        63 => (wqc, wkc, false, bkc),
        _ => (wqc, wkc, bqc, bkc),
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

fn to_move_array(vec: Vec<Move>) -> [Move; 128] {
    let mut move_array = [Move::default(); 128];
    if (vec.len() >= 128) {
        println!("Too many moves! {}", vec.len());
    }
    for i in 0..vec.len() {
        move_array[i] = vec[i];
    }
    move_array
}

#[cfg(test)]
mod test {
    use crate::shared::constants::{MF_CAPTURE, MF_DOUBLE_PAWN_PUSH, MF_KING_CASTLING};

    use super::*;

    #[test]
    pub fn new_start_pos() {
        let result = Position::new(
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR".into(),
            "w".into(),
            "KQkq".into(),
            "-".into(),
        );
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
        let result = Position::new(
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR".into(),
            "w".into(),
            "KQkq".into(),
            "-".into(),
        );
        assert_eq!(
            result.to_fen(),
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq -"
        );
    }

    #[test]
    pub fn to_fen_ending_in_empty_squares() {
        let result = Position::new(
            "rnbq1rk1/ppp2pbp/3p1np1/4p3/2PPP3/2N2N2/PP2BPPP/R1BQ1RK1".into(),
            "b".into(),
            "Kq".into(),
            "-".into(),
        );
        assert_eq!(
            result.to_fen(),
            "rnbq1rk1/ppp2pbp/3p1np1/4p3/2PPP3/2N2N2/PP2BPPP/R1BQ1RK1 b Kq -"
        );
    }

    #[test]
    pub fn generate_move_segments_start_pos_e4() {
        let position = Position::default();
        let m = Move::new(11, 27, MF_DOUBLE_PAWN_PUSH, PieceType::Pawn, false);

        let segments = position.generate_move_segments(&m);

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
        let position = Position::from_fen(
            "rnbqk2r/pppp1ppp/5n2/2b1p3/2B1P3/2NP4/PPP2PPP/R1BQK1NR b KQkq -".to_string(),
        );
        let m = Move::new(59, 57, MF_KING_CASTLING, PieceType::King, true);

        let segments = position.generate_move_segments(&m);

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
        let position = Position::from_fen(
            "rnbqk2r/pppp1ppp/5n2/2b1p3/2B1P3/2NP4/PPP2PPP/R1BQK1NR w KQkq -".to_string(),
        );
        let m = Move::new(7, 6, 0b0, PieceType::Rook, true);

        let segments = position.generate_move_segments(&m);

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
        let position = Position::from_fen("r3k2r/8/8/8/8/8/8/R3K2R w KQkq -".to_string());
        let m = Move::new(63, 7, MF_CAPTURE, PieceType::Rook, false);

        let segments = position.generate_move_segments(&m);

        assert_eq!(
            segments[0],
            MoveSegment::new(MoveSegmentType::Pickup, 63, PieceType::Rook, false)
        );

        assert_eq!(
            segments[1],
            MoveSegment::new(MoveSegmentType::Pickup, 7, PieceType::Rook, true)
        );

        assert_eq!(
            segments[2],
            MoveSegment::new(MoveSegmentType::ClearCastling, 7, PieceType::Rook, true)
        );

        assert_eq!(
            segments[3],
            MoveSegment::new(MoveSegmentType::Place, 7, PieceType::Rook, false)
        );
        assert_eq!(
            segments[4],
            MoveSegment::new(MoveSegmentType::ClearCastling, 63, PieceType::Rook, false)
        );
    }

    #[test]
    pub fn generate_move_segments_black_moves_king_clearing_castling() {
        let position = Position::from_fen(
            "rnbqkbnr/pppp1ppp/4p3/8/2B5/4P3/PPPP1PPP/RNBQK1NR b KQkq -".to_string(),
        );
        let m = Move::new(56, 48, 0b0, PieceType::King, true);

        let segments = position.generate_move_segments(&m);

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
