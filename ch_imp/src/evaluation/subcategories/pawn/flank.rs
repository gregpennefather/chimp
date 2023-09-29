use crate::{board::board_rep::BoardRep, evaluation::shared::{ABC_FLANK, FGH_FLANK}};

pub fn get_flank_score(abc:bool, board: BoardRep) -> i16 {
    let pawns = board.pawn_bitboard & if abc { ABC_FLANK } else { FGH_FLANK };



    0
}