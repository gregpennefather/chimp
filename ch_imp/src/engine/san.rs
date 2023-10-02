use std::f32::consts::E;

use crate::{
    match_state::game_state::GameState,
    move_generation::generate_moves_for_board,
    r#move::Move,
    shared::{
        board_utils::{char_from_file, get_coords_from_index, get_file, get_rank},
        piece_type::{get_piece_char, PieceType},
    },
};

pub fn build_san(moves: Vec<Move>, starting_fen: String) -> String {
    let mut r = String::default();

    let mut game_state = GameState::new(starting_fen);

    for m in moves {
        r += &format!(" {}", game_state.to_san_lichess(m));
        game_state = game_state.make(m);
    }

    r
}

impl GameState {
    pub fn to_san(&self, m: Move) -> String {
        let piece_type = m.piece_type();
        let piece_letter = get_piece_char(m.piece_type(), false);
        let from_file = get_file(m.from());
        let from_rank = get_rank(m.from()) + 1;

        let mut r = if !piece_letter.eq(&'P') {
            format!("{}", piece_letter)
        } else {
            "".into()
        };

        if m.is_castling() {
            if m.is_king_castling() {
                return "O-O".into();
            } else {
                return "O-O-O".into();
            }
        }

        let mut duplicate_piece_type = false;
        let mut duplicate_files = false;
        let mut duplicate_pawns = false;

        for c_m in generate_moves_for_board(self.position.board) {
            let cm_to = c_m.to();
            if cm_to == m.to() {
                let cm_from = c_m.from();
                if cm_from == m.from() {
                    continue;
                }
                if cm_from == from_file {
                    duplicate_files = true;
                }
                let cm_piece = self.position.board.get_piece_type_at_index(cm_from);
                if cm_piece == piece_type {
                    duplicate_piece_type = true;
                    if piece_type == PieceType::Pawn {
                        duplicate_pawns = true;
                    }
                }
            }
        }

        // println!("df : {duplicate_files}\ndp : {duplicate_pawns}\ndpt : {duplicate_piece_type}\n");

        match (duplicate_files, duplicate_pawns, duplicate_piece_type) {
            (true, false, true) => r = format!("{r}{}", char_from_file(from_file)),
            (false, true, true) => r = format!("{r}{}", char_from_file(from_file)),
            (false, false, true) => r = format!("{r}{}", char_from_file(from_file)),
            _ => {}
        };

        if m.is_capture() {
            r = format!("{r}x");
        }

        if m.is_promotion() {
            return m.uci();
        }

        r = format!("{r}{}", get_coords_from_index(m.to()));

        let ngs = self.make(m);
        if (m.is_black() && ngs.position.white_in_check)
            || (!m.is_black() && ngs.position.black_in_check)
        {
            r = format!("{r}+");
        }
        r
    }

    pub fn to_san_lichess(&self, m: Move) -> String {
        let piece_type = m.piece_type();
        let piece_letter = get_piece_char(m.piece_type(), false);

        let mut r = if !piece_letter.eq(&'P') {
            format!("{}", piece_letter)
        } else {
            "".into()
        };

        if m.is_castling() {
            if m.is_king_castling() {
                return "O-O".into();
            } else {
                return "O-O-O".into();
            }
        }

        let mut moves_targeting_square = Vec::new();
        for c_m in generate_moves_for_board(self.position.board) {
            let cm_to = c_m.to();
            let cm_from = c_m.from();
            let cm_piece = self.position.board.get_piece_type_at_index(cm_from);
            if cm_to == m.to() && (cm_piece == piece_type || piece_type == PieceType::Pawn) {
                moves_targeting_square.push(c_m);
            }
        }

        let from_file = char_from_file(get_file(m.from()));
        r = format!("{r}{from_file}");
        if moves_targeting_square.len() > 1 && piece_type != PieceType::Pawn {
            let from_rank = get_rank(m.from()) + 1;
            r = format!("{r}{from_rank}");
        }

        if m.is_capture() {
            r = format!("{r}x");
        }

        if m.is_promotion() {
            return m.uci();
        }

        r = format!("{r}{}", get_coords_from_index(m.to()));

        let ngs = self.make(m);
        if (m.is_black() && ngs.position.white_in_check)
            || (!m.is_black() && ngs.position.black_in_check)
        {
            r = format!("{r}+");
        }
        r
    }
}

#[cfg(test)]
mod test {
    use crate::{
        match_state::game_state,
        shared::{
            board_utils::index_from_coords,
            constants::{MF_CAPTURE, MF_DOUBLE_PAWN_PUSH},
        },
    };

    use super::*;

    #[test]
    fn simple_situation_startpos_e4() {
        let game_state = GameState::default();
        let m = Move::new(
            index_from_coords("e2"),
            index_from_coords("e4"),
            MF_DOUBLE_PAWN_PUSH,
            PieceType::Pawn,
            false,
            0,
            0
        );

        assert_eq!(game_state.to_san(m), "e4");
    }

    #[test]
    fn simple_situation_startpos_knight_to_f3() {
        let game_state = GameState::default();
        let m = Move::new(
            index_from_coords("g1"),
            index_from_coords("f3"),
            0b0,
            PieceType::Knight,
            false,
            0,
            0
        );

        assert_eq!(game_state.to_san(m), "Nf3");
    }

    #[test]
    fn capture_with_multiple_possible_attack_pieces() {
        let game_state =
            GameState::new("3r2k1/p2r1p1p/1p2p1p1/q4n2/3P4/PQ5P/1P1RNPP1/3R2K1 b - -".into());
        let m = Move::new(
            index_from_coords("f5"),
            index_from_coords("d4"),
            MF_CAPTURE,
            PieceType::Knight,
            true,
            0,
            0
        );
        assert_eq!(game_state.to_san(m), "Nxd4");
    }

    #[test]
    fn show_the_move_is_check() {
        let game_state =
            GameState::new("1k1r4/pp1b1R2/3q2pp/4p3/2B5/4Q3/PPP2B2/2K5 b - - 0 1".into());
        let m = Move::new(
            index_from_coords("d6"),
            index_from_coords("d1"),
            0b0,
            PieceType::Queen,
            true,
            0,
            0
        );
        assert_eq!(game_state.to_san(m), "Qd1+");
    }

    #[test]
    fn do_not_include_from_rank_if_piece_is_pawn() {
        let game_state =
            GameState::new("2q1rr1k/3bbnnp/p2p1pp1/2pPp3/PpP1P1P1/1P2BNNP/2BQ1PRK/7R b - -".into());
        let m = Move::new(
            index_from_coords("f6"),
            index_from_coords("f5"),
            0b0,
            PieceType::Pawn,
            true,
            0,
            0
        );
        assert_eq!(game_state.to_san(m), "f5");
    }

    #[test]
    fn case_0() {
        let game_state =
            GameState::new("1k1r1r2/p1p5/Bpnbb3/3p2pp/3P4/P1N1NPP1/1PP4P/2KR1R2 w - - 0 1".into());
        let m = Move::new(
            index_from_coords("c3"),
            index_from_coords("d5"),
            MF_CAPTURE,
            PieceType::Knight,
            false,
            0,
            0
        );
        assert_eq!(game_state.to_san(m), "Ncxd5");
    }

    #[test]
    fn case_1() {
        let game_state =
            GameState::new("1qrr3k/6p1/1p1pp2p/pNn5/Pn1bP1PP/5Q2/1PP1N3/1K1R2R1 w - -".into());
        let m = Move::new(
            index_from_coords("e2"),
            index_from_coords("d4"),
            MF_CAPTURE,
            PieceType::Knight,
            false,
            0,
            0
        );
        assert_eq!(game_state.to_san(m), "Nexd4");
    }

    #[test]
    fn case_2() {
        let game_state =
            GameState::new("r3r1k1/pp2q3/2b1pp2/6pN/Pn1P4/6R1/1P3PP1/3QRBK1 w - -".into());
        let m = Move::new(
            index_from_coords("f2"),
            index_from_coords("f4"),
            MF_DOUBLE_PAWN_PUSH,
            PieceType::Pawn,
            false,
            0,
            0
        );
        assert_eq!(game_state.to_san(m), "f4");
    }
}
