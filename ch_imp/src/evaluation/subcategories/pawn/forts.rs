use super::utils::get_pawn_attacks;

pub fn get_forts(is_black: bool, pieces: u64, a_pawns: u64) -> u64 {
    let p_attacks = get_pawn_attacks(is_black, a_pawns);
    pieces & p_attacks
}

#[cfg(test)]
mod test {
    use crate::{board::{board_rep::BoardRep, bitboard::Bitboard}, shared::board_utils::index_from_coords};

    use super::get_forts;

    #[test]
    fn fort_at_f3() {
        let board = BoardRep::from_fen("rnbqkbnr/pppppppp/8/8/8/5N2/PPPPPPPP/RNBQKB1R w KQkq - 0 1".into());

        let r = get_forts(false, board.knight_bitboard & board.white_occupancy, board.pawn_bitboard & board.white_occupancy);

        assert_eq!(r.count_ones(), 1);
        assert!(r.occupied(index_from_coords("f3")))
    }
}