use crate::{
    board::{bitboard::Bitboard, position::Position},
    r#move::Move,
    shared::{
        board_utils::{get_file, index_from_coords},
        constants::{
            MF_BISHOP_CAPTURE_PROMOTION, MF_BISHOP_PROMOTION, MF_CAPTURE, MF_DOUBLE_PAWN_PUSH,
            MF_EP_CAPTURE, MF_KING_CASTLING, MF_KNIGHT_CAPTURE_PROMOTION, MF_KNIGHT_PROMOTION,
            MF_QUEEN_CAPTURE_PROMOTION, MF_QUEEN_CASTLING, MF_QUEEN_PROMOTION,
            MF_ROOK_CAPTURE_PROMOTION, MF_ROOK_PROMOTION,
        },
        piece_type::{get_piece_type_from_char, PieceType},
    },
};
use core::fmt::Debug;

#[repr(u8)]
#[derive(PartialEq, Debug, Copy, Clone)]
pub enum MatchResultState {
    Active = 0u8,
    Draw = 1u8,
    WhiteVictory = 2u8,
    BlackVictory = 3,
}

#[derive(Copy, Clone, PartialEq)]
pub struct GameState {
    pub position: Position,
    pub half_moves: u8,
    pub full_moves: u32,
    pub result_state: MatchResultState,
    pub entry_move: Move,
    pub subjective_eval: i16,
    recent_moves: [Move; 6],
}

impl GameState {
    pub fn new(fen: String) -> Self {
        let mut fen_segments = fen.split_whitespace();

        let position_segment = fen_segments.nth(0).unwrap().to_string();
        let turn_segment = fen_segments.nth(0).unwrap().to_string();
        let castling_segment = fen_segments.nth(0).unwrap().to_string();
        let ep_segment = fen_segments.nth(0).unwrap().to_string();
        let position = Position::new(position_segment, turn_segment, castling_segment, ep_segment);

        let half_moves = match fen_segments.nth(0) {
            Some(hm) => hm.parse::<u8>().unwrap(),
            None => 0,
        };
        let full_moves = match fen_segments.nth(0) {
            Some(hm) => hm.parse::<u32>().unwrap(),
            None => 0,
        };
        let recent_moves = [Move::default(); 6];
        let result_state = result_state(half_moves, recent_moves, &position);
        let subjective_eval = get_subjective_eval(&position);
        Self {
            position,
            half_moves,
            full_moves,
            result_state,
            recent_moves,
            entry_move: Move::default(),
            subjective_eval,
        }
    }

    pub fn legal(&self) -> bool {
        self.position.legal
    }

    pub fn make(&self, m: Move) -> Self {
        let (new_zorb, move_segments) = self.position.board.zorb_key_after_move(m);

        let new_position = self.position.apply_segments(move_segments, new_zorb);

        if !new_position.legal {
            panic!(
                "unexpected illegal position after {m:?} at {}",
                self.position.board.to_fen()
            );
        }

        let mut half_moves = self.half_moves;
        let mut full_moves = self.full_moves;

        if m.is_capture() || m.piece_type() == PieceType::Pawn {
            half_moves = 0;
        } else {
            half_moves += 1;
        }

        if self.position.board.black_turn {
            full_moves += 1;
        }

        let recent_moves = [
            m,
            self.recent_moves[0],
            self.recent_moves[1],
            self.recent_moves[2],
            self.recent_moves[3],
            self.recent_moves[4],
        ];

        let result_state = result_state(half_moves, recent_moves, &new_position); // TODO: Might need to add in some extra logic here
        let subjective_eval = get_subjective_eval(&new_position);

        Self {
            position: new_position,
            half_moves,
            full_moves,
            recent_moves,
            result_state,
            entry_move: m,
            subjective_eval,
        }
    }

    pub fn after_position(&self, position: Position, m: Move) -> Option<GameState> {
        if !position.legal {
            return None;
        }

        let mut half_moves = self.half_moves;
        let mut full_moves = self.full_moves;

        if m.is_capture() || m.piece_type() == PieceType::Pawn {
            half_moves = 0;
        } else {
            half_moves += 1;
        }

        if self.position.board.black_turn {
            full_moves += 1;
        }

        let recent_moves = [
            m,
            self.recent_moves[0],
            self.recent_moves[1],
            self.recent_moves[2],
            self.recent_moves[3],
            self.recent_moves[4],
        ];

        let result_state = result_state(half_moves, recent_moves, &position); // TODO: Might need to add in some extra logic here
        let subjective_eval = get_subjective_eval(&position);

        Some(Self {
            position: position,
            half_moves,
            full_moves,
            recent_moves,
            result_state,
            entry_move: m,
            subjective_eval,
        })
    }

    pub fn to_fen(&self) -> String {
        let mut result = self.position.board.to_fen();

        result = format!("{result} {}", self.half_moves);
        result = format!("{result} {}", self.full_moves);

        result
    }

    pub fn move_from_uci(&self, move_uci: &str) -> Move {
        let from = index_from_coords(&move_uci[0..2]);
        let to = index_from_coords(&move_uci[2..4]);

        let promotion = if move_uci.len() == 5 {
            get_piece_type_from_char(move_uci.chars().nth(4).unwrap())
        } else {
            PieceType::None
        };

        let opponent_occupancy = if self.position.board.black_turn {
            self.position.board.white_occupancy
        } else {
            self.position.board.black_occupancy
        };

        let mut flags = 0;

        let piece_type = self.position.board.get_piece_type_at_index(from);

        let is_capture = if opponent_occupancy.occupied(to) {
            flags = MF_CAPTURE;
            true
        } else {
            false
        };

        match piece_type {
            PieceType::Pawn => {
                if from.abs_diff(to) == 16 {
                    flags = MF_DOUBLE_PAWN_PUSH;
                } else if self.position.board.ep_index == to {
                    flags = MF_EP_CAPTURE
                } else if promotion != PieceType::None {
                    flags = match (is_capture, promotion) {
                        (true, PieceType::Knight) => MF_KNIGHT_CAPTURE_PROMOTION,
                        (false, PieceType::Knight) => MF_KNIGHT_PROMOTION,
                        (true, PieceType::Bishop) => MF_BISHOP_CAPTURE_PROMOTION,
                        (false, PieceType::Bishop) => MF_BISHOP_PROMOTION,
                        (true, PieceType::Rook) => MF_ROOK_CAPTURE_PROMOTION,
                        (false, PieceType::Rook) => MF_ROOK_PROMOTION,
                        (true, PieceType::Queen) => MF_QUEEN_CAPTURE_PROMOTION,
                        (false, PieceType::Queen) => MF_QUEEN_PROMOTION,
                        _ => panic!(""),
                    }
                }
            }
            PieceType::King => {
                let dif = get_file(from) as i8 - get_file(to) as i8;
                if dif == -2 {
                    flags = MF_KING_CASTLING;
                } else if dif == 2 {
                    flags = MF_QUEEN_CASTLING;
                }
            }
            _ => {}
        }

        Move::new(
            from,
            to,
            flags,
            piece_type,
            self.position.board.black_turn,
            0,
        ) // See shouldnt be 0 but also theres no real need to calc it
    }
}

impl Debug for GameState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("GameState")
            .field(&self.to_fen())
            .field(&self.position.eval)
            .field(&self.position.board.zorb_key)
            .finish()
    }
}

fn result_state(half_moves: u8, recent_moves: [Move; 6], position: &Position) -> MatchResultState {
    if half_moves >= 50 {
        return MatchResultState::Draw;
    }

    if !recent_moves[0].is_empty()
        && !recent_moves[1].is_empty()
        && recent_moves[0] == recent_moves[2]
        && recent_moves[0] == recent_moves[4]
        && recent_moves[1] == recent_moves[3]
        && recent_moves[1] == recent_moves[5]
    {
        return MatchResultState::Draw;
    }

    // if position.moves.len() == 0 {
    //     if position.board.black_turn && position.black_in_check {
    //         return MatchResultState::WhiteVictory;
    //     }
    //     if !position.board.black_turn && position.white_in_check {
    //         return MatchResultState::BlackVictory;
    //     }

    //     return MatchResultState::Draw;
    // }
    MatchResultState::Active
}

// Gets the eval from the POV of the current player
pub fn get_subjective_eval(position: &Position) -> i16 {
    if position.board.black_turn {
        -position.eval as i16
    } else {
        position.eval as i16
    }
}

fn has_player_moves(moves: &Vec<Move>, is_black: bool) -> bool {
    for m in moves {
        if m.is_black() == is_black {
            return true;
        }
    }
    return false;
}

impl Default for GameState {
    fn default() -> Self {
        Self::new("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1".into())
    }
}
#[cfg(test)]
mod test {
    use crate::shared::constants::{MF_DOUBLE_PAWN_PUSH, MF_KING_CASTLING};

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
            0,
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
        game_state = game_state.make(Move::new(3, 1, MF_KING_CASTLING, PieceType::King, true, 0));
        assert_eq!(
            game_state.to_fen(),
            "rnbq1rk1/ppp2pbp/3p1np1/4p3/2PPP3/2N2N2/PP2BPPP/R1BQ1RK1 b - - 1 2"
        );
    }
}
