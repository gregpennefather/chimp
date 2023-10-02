use crate::{board::bitboard::Bitboard, shared::board_utils::get_file};

pub fn is_on_pawnless_file(king_pos: u8, open_files: u64) -> bool {
    let file = get_file(king_pos);
    let mut mask = 1 << king_pos;
    mask |= if file != 0 {
        1 << (king_pos + 1)
    } else {
        1 << (king_pos - 2)
    };
    mask |= if file != 7 {
        1 << (king_pos - 1)
    } else {
        1 << (king_pos + 2)
    };
    (mask & open_files).count_ones() == 3
}

#[cfg(test)]
mod test {
    use crate::{
        board::{board_rep::BoardRep, bitboard::Bitboard},
        evaluation::{
            pawn_structure::get_open_files,
            subcategories::pawn::{files::open_files, utils::file_fill},
        },
    };

    use super::is_on_pawnless_file;

    #[test]
    fn starting_position_white() {
        let board = BoardRep::default();
        let w_filefill = file_fill(board.pawn_bitboard & board.white_occupancy);
        let b_filefill = file_fill(board.pawn_bitboard & board.black_occupancy);
        assert_eq!(
            is_on_pawnless_file(
                board.white_king_position,
                open_files(w_filefill, b_filefill)
            ),
            false
        );
    }

    #[test]
    fn case_0() {
        let board =
            BoardRep::from_fen("rnbq1r2/1p3k2/5n2/p1b5/8/2NQBN2/6K1/3R1B1R b - - 1 4".into());
        let w_filefill = file_fill(board.pawn_bitboard & board.white_occupancy);
        let b_filefill = file_fill(board.pawn_bitboard & board.black_occupancy);
        assert_eq!(
            is_on_pawnless_file(
                board.white_king_position,
                open_files(w_filefill, b_filefill)
            ),
            true
        );
    }

    #[test]
    fn case_1() {
        let board =
            BoardRep::from_fen("rnbq1r2/1p6/k4n2/p1b5/8/2NQBN2/6K1/3R1B1R b - - 1 4".into());
        let w_filefill = file_fill(board.pawn_bitboard & board.white_occupancy);
        let b_filefill = file_fill(board.pawn_bitboard & board.black_occupancy);
        assert_eq!(
            is_on_pawnless_file(
                board.black_king_position,
                open_files(w_filefill, b_filefill)
            ),
            false
        );
    }


}
