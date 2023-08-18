use crate::{
    board::{board_metrics::BoardMetrics, state::BoardState, piece_utils::get_piece_code},
    shared::{
        BISHOP_INDEX, BLACK_BISHOP, BLACK_KNIGHT, BLACK_PAWN, BLACK_QUEEN, BLACK_ROOK,
        KNIGHT_INDEX, PAWN_INDEX, QUEEN_INDEX, ROOK_INDEX, BLACK_MASK,
    },
};

impl BoardState {
    pub fn evaluate(&self, metrics: &BoardMetrics) -> i32 {
        let mut score: i32 = 0;
        let black_turn = &self.flags & 0b1 == 1;
        for piece_index in 0..self.piece_count {
            let piece_code = get_piece_code(&self.pieces, piece_index);
            let black_piece = piece_code & BLACK_MASK > 0;
            score += if black_piece == black_turn { 1 } else { -1 } * eval_piece(piece_code);
        }
        score
    }
}

fn eval_piece(piece_code: u8) -> i32 {
    match piece_code {
        PAWN_INDEX | BLACK_PAWN => 1,
        KNIGHT_INDEX | BLACK_KNIGHT => 3,
        BISHOP_INDEX | BLACK_BISHOP => 3,
        ROOK_INDEX | BLACK_ROOK => 5,
        QUEEN_INDEX | BLACK_QUEEN => 9,
        _ => 0,
    }
}
