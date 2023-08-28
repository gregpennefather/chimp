use crate::{
    board::position::Position,
    r#move::{
        move_data::MoveData,
        move_segment::{MoveSegment, MoveSegmentType},
        Move,
    },
    shared::{
        board_utils::{get_coords_from_index, index_from_coords},
        constants::MF_EP_CAPTURE,
        piece_type::PieceType,
    },
};
use core::fmt::Debug;

#[derive(Clone, Copy, PartialEq)]
pub struct GameState {
    pub position: Position,
    pub half_moves: u8,
    pub full_moves: u32,
}

impl GameState {
    pub fn new(fen: String) -> Self {
        let mut fen_segments = fen.split_whitespace();

        let position_segment = fen_segments.nth(0).unwrap().to_string();
        let turn_segment = fen_segments.nth(0).unwrap().to_string();
        let castling_segment = fen_segments.nth(0).unwrap().to_string();
        let ep_segment = fen_segments.nth(0).unwrap().to_string();
        let position = Position::new(position_segment, turn_segment, castling_segment, ep_segment);

        let half_moves = fen_segments.nth(0).unwrap().parse::<u8>().unwrap();
        let full_moves = fen_segments.nth(0).unwrap().parse::<u32>().unwrap();

        Self {
            position,
            half_moves,
            full_moves,
        }
    }

    // pub fn generate_moves(&self, move_data: &MoveData) -> Vec<Move> {
    //     let mut moves = Vec::new();
    //     for generated_move in move_data.generate_moves(self.position) {
    //         if generated_move.is_black() == self.position.black_turn {
    //             moves.push(generated_move);
    //         }
    //     }

    //     moves
    // }

    pub fn legal(&self) -> bool {
        self.position.legal()
    }

    pub fn make(&self, m: Move) -> Self {
        let move_segments = self
            .position
            .generate_move_segments(&m);
        self.apply_move_segments(m, move_segments)
    }

    pub fn to_fen(&self) -> String {
        let mut result = self.position.to_fen();

        result = format!("{result} {}", self.half_moves);
        result = format!("{result} {}", self.full_moves);

        result
    }

    fn apply_move_segments(&self, m: Move, move_segments: [MoveSegment; 5]) -> Self {
        let mut half_moves = self.half_moves;
        let mut full_moves = self.full_moves;

        let position = self.position.apply_segments(move_segments);

        if m.is_capture() || m.piece_type() == PieceType::Pawn {
            half_moves = 0;
        } else {
            half_moves += 1;
        }

        if self.position.black_turn {
            full_moves += 1;
        }

        Self {
            position,
            half_moves,
            full_moves
        }
    }

    pub(crate) fn get_moves(&self) -> [Move; 128] {
        if self.position.black_turn {
            self.position.black_moves
        } else {
            self.position.white_moves
        }
    }
}

impl Debug for GameState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("GameState").field(&self.to_fen()).field(&self.position.zorb_key).finish()
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
        assert_eq!(result.half_moves, 0);
        assert_eq!(result.full_moves, 1);
    }

    #[test]
    pub fn new_case_king_only_end_game() {
        let result = GameState::new("k7/8/8/8/8/8/8/7K b - - 5 25".into());

        assert_eq!(result.half_moves, 5);
        assert_eq!(result.full_moves, 25);
    }

    #[test]
    pub fn new_case_white_can_ep_capture() {
        let result = GameState::new(
            "rnbqkbnr/pppp3p/6p1/4ppP1/4P3/8/PPPP1P1P/RNBQKBNR w KQkq f6 0 4".into(),
        );
        assert_eq!(result.half_moves, 0);
        assert_eq!(result.full_moves, 4);
    }

    #[test]
    pub fn to_fen_startpos() {
        let game_state = GameState::default();
        assert_eq!(
            game_state.to_fen(),
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"
        );
    }

    #[test]
    pub fn to_fen_startpos_kings_indian_defense() {
        let game_state = GameState::new(
            "rnbq1rk1/ppp1ppbp/3p1np1/8/2PPP3/2N2N2/PP2BPPP/R1BQK2R b KQ - 0 1".into(),
        );
        assert_eq!(
            game_state.to_fen(),
            "rnbq1rk1/ppp1ppbp/3p1np1/8/2PPP3/2N2N2/PP2BPPP/R1BQK2R b KQ - 0 1"
        );
    }

    #[test]
    pub fn make_pawn_e4_opening() {
        let mut game_state = GameState::default();
        game_state = game_state.make(Move::new(
            11,
            27,
            MF_DOUBLE_PAWN_PUSH,
            PieceType::Pawn,
            true,
        ));
        assert_eq!(
            game_state.to_fen(),
            "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1"
        );
    }

    #[test]
    pub fn make_white_king_side_castling_during_indian_defense() {
        let mut game_state = GameState::new(
            "rnbq1rk1/ppp2pbp/3p1np1/4p3/2PPP3/2N2N2/PP2BPPP/R1BQK2R w KQ - 0 2".into(),
        );
        game_state = game_state.make(Move::new(3, 1, MF_KING_CASTLING, PieceType::King, true));
        assert_eq!(
            game_state.to_fen(),
            "rnbq1rk1/ppp2pbp/3p1np1/4p3/2PPP3/2N2N2/PP2BPPP/R1BQ1RK1 b - - 1 2"
        );
    }


    #[test]
    pub fn make_multiple_moves_case_0() {
        let mut game_state: GameState = GameState::default();
        game_state = game_state.make(Move::new(
            index_from_coords("e2"),
            index_from_coords("e4"),
            MF_DOUBLE_PAWN_PUSH,
            PieceType::Pawn,
            false,
        ));

        let expected_game_state = GameState::new("rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1".into());
        assert_eq!(game_state, expected_game_state);

        game_state = game_state.make(Move::new(
            index_from_coords("g8"),
            index_from_coords("f6"),
            0b0,
            PieceType::Knight,
            true,
        ));

        let expected_game_state = GameState::new("rnbqkb1r/pppppppp/5n2/8/4P3/8/PPPP1PPP/RNBQKBNR w KQkq - 1 2".into());
        assert_eq!(game_state, expected_game_state);

        game_state = game_state.make(Move::new(
            index_from_coords("f1"),
            index_from_coords("c4"),
            0b0,
            PieceType::Bishop,
            false,
        ));

        let expected_game_state = GameState::new("rnbqkb1r/pppppppp/5n2/8/2B1P3/8/PPPP1PPP/RNBQK1NR b KQkq - 2 2".into());
        assert_eq!(game_state, expected_game_state);

    }

    #[test]
    pub fn make_multiple_moves() {
        let game_state = GameState::default();
        let s1 = game_state.make(Move::new(
            index_from_coords("a2"),
            index_from_coords("a4"),
            MF_DOUBLE_PAWN_PUSH,
            PieceType::Pawn,
            false,
        ));
        let expected_state_1 = GameState::new("rnbqkbnr/pppppppp/8/8/P7/8/1PPPPPPP/RNBQKBNR b KQkq a3 0 1".into());
        let expected_position_1 = Position::from_fen("rnbqkbnr/pppppppp/8/8/P7/8/1PPPPPPP/RNBQKBNR b KQkq a3".into());
        assert_eq!(
            s1.position,
            expected_position_1
        );
        assert_eq!(
            s1,
            expected_state_1,
            "\nleft: {}:{}\nright:{}:{}", s1.to_fen(), s1.position.zorb_key, expected_state_1.to_fen(), expected_state_1.position.zorb_key
        );
        let s2 = s1.make(Move::new(
            index_from_coords("b7"),
            index_from_coords("b5"),
            MF_DOUBLE_PAWN_PUSH,
            PieceType::Pawn,
            true,
        ));
        let s2_from_expected_state_1 = expected_state_1.make(Move::new(
            index_from_coords("b7"),
            index_from_coords("b5"),
            MF_DOUBLE_PAWN_PUSH,
            PieceType::Pawn,
            true,
        ));
        assert_eq!(
            s2,
            s2_from_expected_state_1,
            "\nleft: {}:{}\nright:{}:{}", s2.to_fen(), s2.position.zorb_key, s2_from_expected_state_1.to_fen(), s2_from_expected_state_1.position.zorb_key
        );
        let expected_state_2 = GameState::new("rnbqkbnr/p1pppppp/8/1p6/P7/8/1PPPPPPP/RNBQKBNR w KQkq b6 0 2".into());
        let expected_position_2 = Position::from_fen("rnbqkbnr/p1pppppp/8/1p6/P7/8/1PPPPPPP/RNBQKBNR w KQkq b6".into());
        assert_eq!(
            s2.position,
            expected_position_2
        );
        assert_eq!(
            s2,
            expected_state_2,
            "\nleft: {}:{}\nright:{}:{}", s2.to_fen(), s2.position.zorb_key, expected_state_2.to_fen(), expected_state_2.position.zorb_key
        );
    }
}
