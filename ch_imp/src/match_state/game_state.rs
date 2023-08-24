use crate::{
    board::position::Position,
    r#move::{
        move_segment::{MoveSegment, MoveSegmentType},
        Move,
    },
    shared::{board_utils::position_from_coords, piece_type::PieceType},
};

#[derive(Clone, Copy)]
pub struct GameState {
    pub position: Position,
    pub black_turn: bool,
    pub white_queen_side_castling: bool,
    pub white_king_side_castling: bool,
    pub black_queen_side_castling: bool,
    pub black_king_side_castling: bool,
    pub half_moves: u8,
    pub full_moves: u32,
    pub ep_position: u8,
}

impl GameState {
    pub fn new(fen: String) -> Self {
        let mut fen_segments = fen.split_whitespace();

        let position = Position::new(fen_segments.nth(0).unwrap().to_string());

        let mut white_queen_side_castling = false;
        let mut white_king_side_castling = false;
        let mut black_queen_side_castling = false;
        let mut black_king_side_castling = false;

        let black_turn = fen_segments.nth(0).unwrap().eq_ignore_ascii_case("b");
        let castling = fen_segments.nth(0).unwrap();

        if !castling.eq_ignore_ascii_case("-") {
            if castling.contains("Q") {
                white_queen_side_castling = true;
            }
            if castling.contains("K") {
                white_king_side_castling = true;
            }
            if castling.contains("q") {
                black_queen_side_castling = true;
            }
            if castling.contains("k") {
                black_king_side_castling = true;
            }
        }

        let ep_string = fen_segments.nth(0).unwrap();
        let ep_position = if ep_string.eq("-") {
            u8::MAX
        } else {
            position_from_coords(ep_string)
        };

        let half_moves = fen_segments.nth(0).unwrap().parse::<u8>().unwrap();
        let full_moves = fen_segments.nth(0).unwrap().parse::<u32>().unwrap();

        Self {
            position,
            black_turn,
            white_queen_side_castling,
            white_king_side_castling,
            black_queen_side_castling,
            black_king_side_castling,
            half_moves,
            full_moves,
            ep_position,
        }
    }

    pub fn make(&self, m: Move) -> Self {
        let move_segments = self.generate_move_segments(m);

        *self
    }

    fn generate_move_segments(&self, m: Move) -> [MoveSegment; 5] {
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
                let captured_piece_type = self.position.get_piece_type_at_index(to_index);

                segments[segment_index] = MoveSegment::new(
                    MoveSegmentType::Pickup,
                    to_index,
                    captured_piece_type,
                    !self.black_turn,
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
                let captured_piece_type = self.position.get_piece_type_at_index(to_index);

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
}

impl Default for GameState {
    fn default() -> Self {
        Self::new("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1".into())
    }
}
#[cfg(test)]
mod test {
    use crate::shared::constants::{MF_CAPTURE, MF_DOUBLE_PAWN_PUSH, MF_KING_CASTLING};

    use super::*;

    #[test]
    pub fn new_start_pos() {
        let result =
            GameState::new("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1".into());
        assert_eq!(result.position, Position::default());
        assert_eq!(result.black_turn, false);
        assert_eq!(result.white_queen_side_castling, true);
        assert_eq!(result.white_king_side_castling, true);
        assert_eq!(result.black_queen_side_castling, true);
        assert_eq!(result.black_king_side_castling, true);
        assert_eq!(result.half_moves, 0);
        assert_eq!(result.full_moves, 1);
        assert_eq!(result.ep_position, u8::MAX);
    }

    #[test]
    pub fn new_case_king_only_end_game() {
        let result = GameState::new("k7/8/8/8/8/8/8/7K b - - 5 25".into());

        assert_eq!(result.black_turn, true);
        assert_eq!(result.white_queen_side_castling, false);
        assert_eq!(result.white_king_side_castling, false);
        assert_eq!(result.black_queen_side_castling, false);
        assert_eq!(result.black_king_side_castling, false);
        assert_eq!(result.half_moves, 5);
        assert_eq!(result.full_moves, 25);
        assert_eq!(result.ep_position, u8::MAX);
    }

    #[test]
    pub fn new_case_white_can_ep_capture() {
        let result = GameState::new(
            "rnbqkbnr/pppp3p/6p1/4ppP1/4P3/8/PPPP1P1P/RNBQKBNR w KQkq f6 0 4".into(),
        );
        assert_eq!(result.black_turn, false);
        assert_eq!(result.white_queen_side_castling, true);
        assert_eq!(result.white_king_side_castling, true);
        assert_eq!(result.black_queen_side_castling, true);
        assert_eq!(result.black_king_side_castling, true);
        assert_eq!(result.half_moves, 0);
        assert_eq!(result.full_moves, 4);
        assert_eq!(result.ep_position, 42);
    }

    #[test]
    pub fn generate_move_segments_start_pos_e4() {
        let game_state = GameState::default();
        let m = Move::new(11, 27, MF_DOUBLE_PAWN_PUSH, PieceType::Pawn);

        let segments = game_state.generate_move_segments(m);
        println!("{:?}", segments);

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
        let game_state = GameState::new(
            "rnbqk2r/pppp1ppp/5n2/2b1p3/2B1P3/2NP4/PPP2PPP/R1BQK1NR b KQkq - 0 1".to_string(),
        );
        let m = Move::new(59, 57, MF_KING_CASTLING, PieceType::King);

        let segments = game_state.generate_move_segments(m);
        println!("{:?}", segments);

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
        let game_state = GameState::new(
            "rnbqk2r/pppp1ppp/5n2/2b1p3/2B1P3/2NP4/PPP2PPP/R1BQK1NR w KQkq - 0 1".to_string(),
        );
        let m = Move::new(7, 6, 0b0, PieceType::Rook);

        let segments = game_state.generate_move_segments(m);
        println!("{:?}", segments);

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
        let game_state = GameState::new("r3k2r/8/8/8/8/8/8/R3K2R b KQkq - 0 1".to_string());
        let m = Move::new(63, 7, MF_CAPTURE, PieceType::Rook);

        let segments = game_state.generate_move_segments(m);
        println!("{:?}", segments);

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
        let game_state = GameState::new("rnbqkbnr/pppp1ppp/4p3/8/2B5/4P3/PPPP1PPP/RNBQK1NR b KQkq - 0 1".to_string());
        let m = Move::new(56, 48, 0b0, PieceType::King);

        let segments = game_state.generate_move_segments(m);
        println!("{:?}", segments);

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

        assert_eq!(
            segments[3],
            MoveSegment::default()
        );
        assert_eq!(
            segments[4],
            MoveSegment::default()
        );
    }
}
