use crate::evaluation::subcategories::pawn::utils::file_fill;

pub fn get_forts_on_rook_open_file(a_forts: u64, b_rooks: u64, open_files: u64) -> i16 {
    let rook_open_files = file_fill(b_rooks & open_files);
    (a_forts & rook_open_files).count_ones() as i16
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{
        board::board_rep::BoardRep,
        evaluation::subcategories::{
            pawn::{files::board_open_files, forts::get_forts}
        },
    };

    #[test]
    fn undefended_minor_piece_on_rook_file_not_considered_blocking() {
        let board = BoardRep::from_fen("k3r3/8/8/8/4NP2/8/PPPP2PP/K7 w - - 0 1".into());

        let blocking_outposts = get_forts_on_rook_open_file(
            get_forts(false, (board.knight_bitboard | board.bishop_bitboard) & board.white_occupancy, board.pawn_bitboard & board.white_occupancy),
            board.rook_bitboard & board.black_occupancy,
            board_open_files(board),
        );

        assert_eq!(blocking_outposts, 0);
    }

    #[test]
    fn pawn_defended_minor_piece_on_rook_file_considered_blocking() {
        let board = BoardRep::from_fen("k3r3/8/8/4N3/5P2/8/PPPP2PP/K7 w - - 0 1".into());

        let blocking_outposts = get_forts_on_rook_open_file(
            get_forts(false, (board.knight_bitboard | board.bishop_bitboard) & board.white_occupancy, board.pawn_bitboard & board.white_occupancy),
            board.rook_bitboard & board.black_occupancy,
            board_open_files(board),
        );

        assert_eq!(blocking_outposts, 1);
    }
}
