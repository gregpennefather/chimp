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
        r += &format!(" {}", game_state.to_san(m));
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

        let from_file = char_from_file(get_file(m.from()));
        r = format!("{r}{from_file}");
        if moves_targeting_square.len() > 1 {
            let from_rank = get_rank(m.from()) + 1;
            r = format!("{r}{from_rank}");
        }

        if m.is_capture() {
            r = format!("{r}x");
        }

        if m.is_promotion() {
            return m.uci();
        }

        format!("{r}{}", get_coords_from_index(m.to()))
    }
}
