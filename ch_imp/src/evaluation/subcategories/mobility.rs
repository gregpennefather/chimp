use crate::{
    board::{bitboard::Bitboard, board_rep::BoardRep},
    evaluation::shared::get_pawn_controlled_squares,
    shared::{board_utils::get_coords_from_index, piece_type::PieceType},
    MOVE_DATA,
};

pub fn get_mobility(is_black: bool, board: BoardRep) -> u32 {
    let mut mobility = 0;
    // A permissable square is:
    // - Not defended by an opponents pawn
    // - Not occupied by our king
    let permissable_squared = !get_pawn_controlled_squares(!is_black, board)
        ^ 1 << if is_black {
            board.black_king_position
        } else {
            board.white_king_position
        };

    let mut friendly_occupancy = !board.pawn_bitboard & if is_black {
        board.black_occupancy
    } else {
        board.white_occupancy
    };

    while friendly_occupancy != 0 {
        let lsb = friendly_occupancy.trailing_zeros() as u8;
        let l = get_piece_mobility(lsb, board, permissable_squared);
        mobility += l;
        friendly_occupancy = friendly_occupancy.flip(lsb);
    }

    mobility
}

fn get_piece_mobility(index: u8, board: BoardRep, safe_squares: u64) -> u32 {
    match board.get_piece_type_at_index(index) {
        PieceType::Pawn | PieceType::None | PieceType::King => 0,
        PieceType::Knight => {
            let move_board = MOVE_DATA.knight_moves[index as usize] & safe_squares;
            move_board.count_ones()
        }
        PieceType::Bishop => {
            let move_board = MOVE_DATA
                .magic_bitboard_table
                .get_bishop_attacks(index as usize, board.occupancy)
                & safe_squares;
            move_board.count_ones()
        }
        PieceType::Rook => {
            let move_board = MOVE_DATA
                .magic_bitboard_table
                .get_rook_attacks(index as usize, board.occupancy)
                & safe_squares;
            move_board.count_ones()
        }
        PieceType::Queen => {
            let move_board = (MOVE_DATA
                .magic_bitboard_table
                .get_rook_attacks(index as usize, board.occupancy)
                | MOVE_DATA
                    .magic_bitboard_table
                    .get_bishop_attacks(index as usize, board.occupancy))
                & safe_squares;
            move_board.count_ones()
        }
    }
}

#[cfg(test)]
mod test {
    use crate::{board::board_rep::BoardRep, evaluation::subcategories::mobility::get_mobility};

    #[test]
    fn get_mobility_area_white_starting_position() {
        let board =
            BoardRep::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1".into());

        let r = get_mobility(false, board);

        assert_eq!(r, 18)
    }
}
