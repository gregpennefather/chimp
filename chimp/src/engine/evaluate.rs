use crate::{
    board::{
        bitboard::BitboardExtensions,
        board_metrics::BoardMetrics,
        state::{BoardState, BoardStateFlagsTrait}, piece::Piece,
    },
    shared::{
        BISHOP_INDEX, BLACK_BISHOP, BLACK_KNIGHT, BLACK_PAWN, BLACK_QUEEN, BLACK_ROOK,
        KNIGHT_INDEX, PAWN_INDEX, QUEEN_INDEX, ROOK_INDEX,
    },
};

impl BoardState {
    pub fn evaluate(&self, metrics: &BoardMetrics) -> f32 {
        let mut score: f32 = 0.0;
        let black_turn = self.flags.is_black_turn();
        for piece_index in 0..self.piece_count {
            let piece = self.pieces.get(piece_index);
            score += if piece.is_black() == black_turn { 1.0 } else { -1.0 } * eval_piece(piece);
        }

        score += metrics.white_mobility_board.count_occupied() as f32 * 0.1;
        score -= metrics.black_mobility_board.count_occupied() as f32 * 0.1;

        score
    }
}

fn eval_piece(piece: Piece) -> f32 {
    match piece.0 {
        PAWN_INDEX => 1.0,
        BLACK_PAWN => -1.0,
        KNIGHT_INDEX | BISHOP_INDEX => 3.0,
        BLACK_KNIGHT | BLACK_BISHOP => -3.0,
        ROOK_INDEX => 5.0,
        BLACK_ROOK => -5.0,
        QUEEN_INDEX => 9.0,
        BLACK_QUEEN => -9.0,
        _ => 0.0,
    }
}
