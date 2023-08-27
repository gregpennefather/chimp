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
    pub black_turn: bool,
    pub white_queen_side_castling: bool,
    pub white_king_side_castling: bool,
    pub black_queen_side_castling: bool,
    pub black_king_side_castling: bool,
    pub half_moves: u8,
    pub full_moves: u32,
    pub ep_index: u8,
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
            if castling.contains("K") {
                white_king_side_castling = true;
            }
            if castling.contains("Q") {
                white_queen_side_castling = true;
            }
            if castling.contains("k") {
                black_king_side_castling = true;
            }
            if castling.contains("q") {
                black_queen_side_castling = true;
            }
        }

        let ep_string = fen_segments.nth(0).unwrap();
        let ep_position = if ep_string.eq("-") {
            u8::MAX
        } else {
            index_from_coords(ep_string)
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
            ep_index: ep_position,
        }
    }

    pub fn generate_moves(&self, move_data: &MoveData) -> Vec<Move> {
        let mut moves = Vec::new();
        for generated_move in move_data.generate_moves(
            self.position,
            self.black_turn,
            self.ep_index,
            self.white_king_side_castling,
            self.white_queen_side_castling,
            self.black_king_side_castling,
            self.black_queen_side_castling,
        ) {
            if generated_move.is_black() == self.black_turn {
                moves.push(generated_move);
            }
        }

        moves
    }

    pub fn make(&self, m: Move) -> Self {
        let move_segments = self.position.generate_move_segments(&m, self.black_turn);
        self.apply_move_segments(m, move_segments)
    }

    pub fn to_fen(&self) -> String {
        let mut result = self.position.to_fen();

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

        result = format!("{result} {}", self.half_moves);
        result = format!("{result} {}", self.full_moves);

        result
    }


    fn apply_move_segments(&self, m: Move, move_segments: [MoveSegment; 5]) -> Self {
        let black_turn = self.black_turn;
        let mut wqc = self.white_queen_side_castling;
        let mut wkc = self.white_king_side_castling;
        let mut bqc = self.black_queen_side_castling;
        let mut bkc = self.black_king_side_castling;
        let mut half_moves = self.half_moves;
        let mut full_moves = self.full_moves;
        let mut ep_position = u8::MAX;

        let position = self.position.apply_segments(move_segments);

        for i in 0..5 {
            let segment = move_segments[i];

            match segment.segment_type {
                MoveSegmentType::None => break,
                MoveSegmentType::Pickup => continue,
                MoveSegmentType::Place => continue,
                MoveSegmentType::ClearCastling => {
                    (wqc, wkc, bqc, bkc) = modify_castling(segment.index, wqc, wkc, bqc, bkc)
                }
                MoveSegmentType::DoublePawnPush => ep_position = segment.index,
            }
        }

        if m.is_capture() || m.piece_type() == PieceType::Pawn {
            half_moves = 0;
        } else {
            half_moves += 1;
        }

        if black_turn {
            full_moves += 1;
        }

        Self {
            position,
            black_turn: !black_turn,
            white_queen_side_castling: wqc,
            white_king_side_castling: wkc,
            black_queen_side_castling: bqc,
            black_king_side_castling: bkc,
            half_moves,
            full_moves,
            ep_index: ep_position,
        }
    }
}

impl Debug for GameState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("GameState")
            .field(&self.to_fen())
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
        assert_eq!(result.ep_index, u8::MAX);
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
        assert_eq!(result.ep_index, u8::MAX);
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
        assert_eq!(result.ep_index, 42);
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
    pub fn make_multiple_moves() {
        let game_state = GameState::default();
        let s1 = game_state.make(Move::new(
            index_from_coords("a2"),
            index_from_coords("a4"),
            MF_DOUBLE_PAWN_PUSH,
            PieceType::Pawn,
            false,
        ));
        assert_eq!(s1, GameState::new("rnbqkbnr/pppppppp/8/8/P7/8/1PPPPPPP/RNBQKBNR b KQkq a3 0 1".into()));
        let s2 = s1.make(Move::new(
            index_from_coords("b7"),
            index_from_coords("b5"),
            MF_DOUBLE_PAWN_PUSH,
            PieceType::Pawn,
            true,
        ));
        assert_eq!(s2, GameState::new("rnbqkbnr/p1pppppp/8/1p6/P7/8/1PPPPPPP/RNBQKBNR w KQkq b6 0 2".into()));
    }
}
