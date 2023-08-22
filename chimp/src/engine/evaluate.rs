use log::info;

use crate::{
    board::{
        bitboard::BitboardExtensions,
        board_metrics::BoardMetrics,
        piece::Piece,
        state::{BoardState, BoardStateFlagsTrait},
    },
    shared::{
        BISHOP_INDEX, BLACK_BISHOP, BLACK_KNIGHT, BLACK_PAWN, BLACK_QUEEN, BLACK_ROOK,
        KNIGHT_INDEX, PAWN_INDEX, QUEEN_INDEX, ROOK_INDEX,
    },
};

impl BoardState {
    pub fn evaluate(&self, metrics: &BoardMetrics) -> f32 {
        let mut score: f32 = 0.0;
        for piece_index in 0..self.piece_count {
            let piece = self.pieces.get(piece_index);
            score += eval_piece(piece);
        }

        score += metrics.white_threat_board.count_occupied() as f32 * 0.005;
        score -= metrics.black_threat_board.count_occupied() as f32 * 0.005;
        score += metrics.white_mobility_board.count_occupied() as f32 * 0.01;
        score -= metrics.black_mobility_board.count_occupied() as f32 * 0.01;

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

#[cfg(test)]
mod test {
    use crate::board::state::BoardState;

    #[test]
    fn eval_starting_position_white() {
        let board_state = BoardState::default();
        let metrics = board_state.generate_metrics();
        let r = board_state.evaluate(&metrics);
        assert_eq!(r,0.0, "Starting position shouldn't advantage anyone")
    }

    #[test]
    fn eval_starting_position_white_e4_pawn_slightly_improves_whites_position() {
        let board_state = BoardState::from_fen(&"rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1".into());
        let metrics = board_state.generate_metrics();
        let r = board_state.evaluate(&metrics);
        assert!(r>0.0, "Moving a pawn forward should increase whites position (>0)")
    }

    #[test]
    fn eval_white_taking_piece_should_improve_whites_position() {
         let b1 = BoardState::from_fen(&"rnbqkbnr/ppppp1pp/8/5p2/4P3/8/PPPP1PPP/RNBQKBNR w KQkq f6 0 2".into());
        let m1 = b1.generate_metrics();
        let r1 = b1.evaluate(&m1);

        let b2 = BoardState::from_fen(&"rnbqkbnr/ppppp1pp/8/5P2/8/8/PPPP1PPP/RNBQKBNR b KQkq - 0 2".into());
        let m2 = b2.generate_metrics();
        let r2 = b2.evaluate(&m2);

        assert!(r2> r1);
    }

    #[test]
    fn eval_black_taking_piece_should_improve_blacks_position() {
         let b1 = BoardState::from_fen(&"rnbqkbnr/pppppp1p/8/8/6p1/7N/PPPPPPPP/RNBQKB1R b Qkq - 1 3".into());
        let m1 = b1.generate_metrics();
        let r1 = b1.evaluate(&m1);

        let b2 = BoardState::from_fen(&"rnbqkbnr/pppppp1p/8/8/8/7p/PPPPPPPP/RNBQKB1R w Qkq - 0 4".into());
        let m2 = b2.generate_metrics();
        let r2 = b2.evaluate(&m2);

        assert!(r2 < r1, "{r2} >= {r1} but it should be less as black improves by lowering the value");
    }

    #[test]
    fn eval_opening_white_e_pawn_better_than_black_b_pawn() {
         let b1 = BoardState::from_fen(&"rnbqkbnr/p1pppppp/1p6/8/3P4/8/PPP1PPPP/RNBQKBNR w KQkq - 0 2".into());
        let m1 = b1.generate_metrics();
        let r1 = b1.evaluate(&m1);

        assert!(r1>0.0, "{r1} should be better for white (>0)");
    }

    #[test]
    fn eval_black_in_worse_opening_position() {
         let b1 = BoardState::from_fen(&"rnbqkbnr/p1pppppp/1p6/8/3P4/5N2/PPP1PPPP/RNBQKB1R b KQkq - 1 2".into());
        let m1 = b1.generate_metrics();
        let r1 = b1.evaluate(&m1);

        assert!(r1>0.0, "{r1} should be > 0 (black position better when <0");
    }

}