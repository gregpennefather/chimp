use crate::{board::position::Position, utils::board_utils::position_from_coords};

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
}

#[cfg(test)]
mod test {
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
}
