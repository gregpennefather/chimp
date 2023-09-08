use crate::{
    match_state::game_state::GameState,
    r#move::Move,
    shared::{
        board_utils::{char_from_file, get_coords_from_index, get_file, get_rank},
        piece_type::{get_piece_char, PieceType},
    },
};

pub fn build_san(moves: Vec<Move>) -> String {
    let mut r = String::default();

    let mut game_state = GameState::default();

    for m in moves {
        r += &format!(" {}", game_state.to_san_lichess(m));
        game_state = game_state.make(m).unwrap();
    }

    r
}

impl GameState {
    pub fn to_san(&self, m: Move) -> String {
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
        for c_m in &self.position.moves {
            let cm_to = c_m.to();
            let cm_from = c_m.from();
            let cm_piece = self.position.board.get_piece_type_at_index(cm_from);
            if cm_to == m.to() && (cm_piece == piece_type || piece_type == PieceType::Pawn) {
                moves_targeting_square.push(c_m);
            }
        }

        // let from_file = char_from_file(get_file(m.from()));
        // r = format!("{r}{from_file}");
        if moves_targeting_square.len() > 1 && piece_type != PieceType::Pawn{
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

        let ngs = self.make(m).unwrap();
        if (m.is_black() && ngs.position.white_in_check) || (!m.is_black() && ngs.position.black_in_check) {
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
        for c_m in &self.position.moves {
            let cm_to = c_m.to();
            let cm_from = c_m.from();
            let cm_piece = self.position.board.get_piece_type_at_index(cm_from);
            if cm_to == m.to() && (cm_piece == piece_type || piece_type == PieceType::Pawn) {
                moves_targeting_square.push(c_m);
            }
        }

        let from_file = char_from_file(get_file(m.from()));
        r = format!("{r}{from_file}");
        if moves_targeting_square.len() > 1 && piece_type != PieceType::Pawn{
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

        let ngs = self.make(m).unwrap();
        if (m.is_black() && ngs.position.white_in_check) || (!m.is_black() && ngs.position.black_in_check) {
            r = format!("{r}+");
        }
        r
    }
}



#[cfg(test)]
mod test {
    use crate::{match_state::game_state, shared::{board_utils::index_from_coords, constants::{MF_DOUBLE_PAWN_PUSH, MF_CAPTURE}}};

    use super::*;

    #[test]
    fn simple_situation_startpos_e4() {
        let game_state = GameState::default();
        let m = Move::new(index_from_coords("e2"), index_from_coords("e4"), MF_DOUBLE_PAWN_PUSH, PieceType::Pawn, false);

        assert_eq!(game_state.to_san(m), "e4");
    }

    #[test]
    fn simple_situation_startpos_knight_to_f3() {
        let game_state = GameState::default();
        let m = Move::new(index_from_coords("g1"), index_from_coords("f3"), 0b0, PieceType::Knight, false);

        assert_eq!(game_state.to_san(m), "Nf3");
    }

    #[test]
    fn capture_with_multiple_possible_attack_pieces() {
        let game_state = GameState::new("3r2k1/p2r1p1p/1p2p1p1/q4n2/3P4/PQ5P/1P1RNPP1/3R2K1 b - -".into());
        let m = Move::new(index_from_coords("f5"), index_from_coords("d4"), MF_CAPTURE, PieceType::Knight, true);
        assert_eq!(game_state.to_san(m), "Nxd4");
    }

    #[test]
    fn show_the_move_is_check() {
        let game_state = GameState::new("1k1r4/pp1b1R2/3q2pp/4p3/2B5/4Q3/PPP2B2/2K5 b - - 0 1".into());
        let m = Move::new(index_from_coords("d6"), index_from_coords("d1"), 0b0, PieceType::Queen, true);
        assert_eq!(game_state.to_san(m), "Qd1+");
    }

    #[test]
    fn do_not_include_from_rank_if_piece_is_pawn() {
        let game_state = GameState::new("2q1rr1k/3bbnnp/p2p1pp1/2pPp3/PpP1P1P1/1P2BNNP/2BQ1PRK/7R b - -".into());
        let m = Move::new(index_from_coords("f6"), index_from_coords("f5"), 0b0, PieceType::Pawn, true);
        assert_eq!(game_state.to_san(m), "f5");
    }


}