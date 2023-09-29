use crate::{shared::board_utils::chebyshev_distance, board::bitboard::Bitboard, evaluation::pawn_structure::KING_PAWN_DISTANCE_PENALTY};

pub fn get_pawn_distance_penalty(king_position:u8, mut pawn_occupancy:u64) -> i16 {
    let mut dist = 8;

    while pawn_occupancy != 0 {
        let index = pawn_occupancy.trailing_zeros() as u8;
        let a = chebyshev_distance(king_position as i8, index as i8);
        if a < dist {
            dist = a;
            if dist == 1 {
                break;
            }
        }
        pawn_occupancy = pawn_occupancy.flip(index);
    }

    KING_PAWN_DISTANCE_PENALTY * (dist as i16-1)
}

#[cfg(test)]
mod test {
    use crate::{board::board_rep::BoardRep, evaluation::{subcategories::king::pawn_distance::get_pawn_distance_penalty, pawn_structure::KING_PAWN_DISTANCE_PENALTY}};

    #[test]
    fn pawn_adjacent_to_king_no_penalty() {
        let board = BoardRep::from_fen("rnbq1rk1/1ppp1pp1/5n2/p1b1p3/P2P2P1/2NQBN2/1PP1PP1P/3R1BKR w - a6 0 4".into());
        let r = get_pawn_distance_penalty(board.white_king_position, board.pawn_bitboard & board.white_occupancy);
        assert_eq!(r,0);
    }


    #[test]
    fn no_pawns_max_penalty() {
        let board = BoardRep::from_fen("k7/8/8/8/8/8/8/4K3 w - - 0 1".into());
        let r = get_pawn_distance_penalty(board.white_king_position, board.pawn_bitboard & board.white_occupancy);
        assert_eq!(r,7*KING_PAWN_DISTANCE_PENALTY);
    }
}