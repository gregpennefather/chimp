use crate::{
    board::position::{MoveSegmentArray, Position},
    r#move::{
        move_data::MoveData,
        move_segment::{MoveSegment, MoveSegmentType},
        Move,
    },
    search::position_table::MoveTableLookup,
    shared::{
        board_utils::{get_coords_from_index, index_from_coords},
        constants::MF_EP_CAPTURE,
        piece_type::PieceType,
    },
    POSITION_TRANSPOSITION_TABLE,
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
        let (new_zorb, move_segments) = self.position.zorb_key_after_move(m);

        let lookup_result = lookup(new_zorb);
        let new_position = match lookup_result {
            // Some(new_position) => {
            //     let calc_position =  self.position.apply_segments(move_segments);
            //     assert_eq!(new_position, calc_position, "{self:?}:{m:?}");
            //     new_position
            // }
            Some(new_position) => new_position,
            None => self.position.apply_segments(move_segments),
        };

        let mut half_moves = self.half_moves;
        let mut full_moves = self.full_moves;

        if m.is_capture() || m.piece_type() == PieceType::Pawn {
            half_moves = 0;
        } else {
            half_moves += 1;
        }

        if self.position.black_turn {
            full_moves += 1;
        }

        Self {
            position: new_position,
            half_moves,
            full_moves,
        }
    }

    pub fn to_fen(&self) -> String {
        let mut result = self.position.to_fen();

        result = format!("{result} {}", self.half_moves);
        result = format!("{result} {}", self.full_moves);

        result
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
        f.debug_tuple("GameState")
            .field(&self.to_fen())
            .field(&self.position.zorb_key)
            .finish()
    }
}

fn lookup(zorb_key: u64) -> Option<Position> {
    // return None;
    POSITION_TRANSPOSITION_TABLE
        .try_read()
        .unwrap()
        .get(&zorb_key)
        .copied()
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
    pub fn bishop_to_c4() {
        let starting_state =
            GameState::new("rnbqkb1r/pppppppp/5n2/8/4P3/8/PPPP1PPP/RNBQKBNR w KQkq - 1 2".into());

        let next_state = starting_state.make(Move::new(
            index_from_coords("f1"),
            index_from_coords("c4"),
            0b0,
            PieceType::Bishop,
            false,
        ));

        let expected_game =
            GameState::new("rnbqkb1r/pppppppp/5n2/8/2B1P3/8/PPPP1PPP/RNBQK1NR b KQkq - 2 2".into());
        assert_eq!(next_state, expected_game);
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

        let expected_game_state_1 =
            GameState::new("rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1".into());
        assert_eq!(game_state, expected_game_state_1);

        game_state = game_state.make(Move::new(
            index_from_coords("g8"),
            index_from_coords("f6"),
            0b0,
            PieceType::Knight,
            true,
        ));

        let expected_game_state_1_2 = expected_game_state_1.make(Move::new(
            index_from_coords("g8"),
            index_from_coords("f6"),
            0b0,
            PieceType::Knight,
            true,
        ));

        let expected_game_state_2 =
            GameState::new("rnbqkb1r/pppppppp/5n2/8/4P3/8/PPPP1PPP/RNBQKBNR w KQkq - 1 2".into());
        assert_eq!(game_state, expected_game_state_2);
        assert_eq!(expected_game_state_1_2, expected_game_state_2);

        game_state = game_state.make(Move::new(
            index_from_coords("f1"),
            index_from_coords("c4"),
            0b0,
            PieceType::Bishop,
            false,
        ));

        let expected_game_state_3 =
            GameState::new("rnbqkb1r/pppppppp/5n2/8/2B1P3/8/PPPP1PPP/RNBQK1NR b KQkq - 2 2".into());
        assert_eq!(game_state, expected_game_state_3);
    }

    #[test]
    pub fn make_multiple_moves() {
        let game_state =
            GameState::new("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1".into());
        let output_state_1 = game_state.make(Move::new(
            index_from_coords("a2"),
            index_from_coords("a4"),
            MF_DOUBLE_PAWN_PUSH,
            PieceType::Pawn,
            false,
        ));
        let expected_game_state_1 =
            GameState::new("rnbqkbnr/pppppppp/8/8/P7/8/1PPPPPPP/RNBQKBNR b KQkq a3 0 1".into());
        assert_eq!(output_state_1, expected_game_state_1);

        let output_state_2 = output_state_1.make(Move::new(
            index_from_coords("b7"),
            index_from_coords("b5"),
            MF_DOUBLE_PAWN_PUSH,
            PieceType::Pawn,
            true,
        ));
        let expected_state_2 =
            GameState::new("rnbqkbnr/p1pppppp/8/1p6/P7/8/1PPPPPPP/RNBQKBNR w KQkq b6 0 2".into());

        assert_eq!(output_state_2, expected_state_2);
    }

    #[test]
    pub fn black_moving_king_to_clear_flags() {
        let state = GameState::new("r2k3r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R b Q - 3 2".into());

        let state_after_move = state.make(Move::new(index_from_coords("d8"), index_from_coords("e8"), 0b0, PieceType::King, true));

        let expected_state = GameState::new("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w Q - 4 3".into());

        assert_eq!(state_after_move, expected_state);
    }
}
