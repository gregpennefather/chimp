use super::utils::file_fill;

pub fn closed_files(white_file_fill: u64, black_file_fill: u64) -> u64 {
    white_file_fill & black_file_fill
}

pub fn open_files(white_file_fill: u64, black_file_fill: u64) -> u64 {
    !(white_file_fill | black_file_fill)
}

pub fn half_open_or_open_files(pawns: u64) -> u64 {
    !file_fill(pawns)
}

pub fn half_open_files(a_file_fill: u64, b_file_fill: u64) -> u64 {
    !a_file_fill ^ open_files(a_file_fill, b_file_fill)
}

#[cfg(test)]
mod test {
    use crate::{board::{board_rep::BoardRep, bitboard::Bitboard}, evaluation::subcategories::pawn::utils::file_fill};

    use super::half_open_files;

    #[test]
    fn black_half_open_file() {
        let board = BoardRep::from_fen("rnbqkbnr/ppp1pppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1".into());

        let w_filefill = file_fill(board.pawn_bitboard & board.white_occupancy);
        let b_filefill = file_fill(board.pawn_bitboard & board.black_occupancy);

        let w_half_open = half_open_files(w_filefill, b_filefill);
        println!("{}", w_half_open.to_board_format());
        assert_eq!(w_half_open, 0);


        let b_half_open = half_open_files(b_filefill, w_filefill);
        assert_eq!(b_half_open, 0.set_file(4));
    }

    #[test]
    fn case_0() {
        let board = BoardRep::from_fen("k7/1p3pp1/p1p4p/3p4/P4P2/2P5/1PP3PP/4K3 w - - 0 1".into());

        let w_filefill = file_fill(board.pawn_bitboard & board.white_occupancy);
        let b_filefill = file_fill(board.pawn_bitboard & board.black_occupancy);

        let w_half_open = half_open_files(w_filefill, b_filefill);
        println!("{}", w_half_open.to_board_format());
        assert_eq!(w_half_open, 0.set_file(4));


        let b_half_open = half_open_files(b_filefill, w_filefill);
        assert_eq!(b_half_open, 0);
    }

}