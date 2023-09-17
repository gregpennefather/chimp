use crate::{
    board::{board_rep::BoardRep},
    r#move::{
        move_data::{
            KING_CASTLING_CHECK, KING_CASTLING_CLEARANCE, QUEEN_CASTLING_CHECK,
            QUEEN_CASTLING_CLEARANCE,
        },
        Move,
    },
    shared::{
        constants::{MF_KING_CASTLING, MF_QUEEN_CASTLING},
        piece_type::PieceType,
    },
    MOVE_DATA,
};

use super::moveboard_to_moves;

pub(super) fn generate_king_moves(
    index: u8,
    opponent_occupancy: u64,
    occupancy: u64,
    in_check: bool,
    is_black: bool,
    king_side_castling: bool,
    queen_side_castling: bool,
    threat_board: u64,
    board: BoardRep,
) -> Vec<Move> {
    let mut moveboard = MOVE_DATA.king_moves[index as usize];
    moveboard = moveboard & !threat_board;
    let mut moves = moveboard_to_moves(
        index,
        PieceType::King,
        moveboard,
        opponent_occupancy,
        occupancy,
        board,
    );

    if !in_check {
        if king_side_castling {
            match generate_king_castling_move(
                index,
                index - 2,
                MF_KING_CASTLING,
                is_black,
                KING_CASTLING_CLEARANCE << (index - 3),
                occupancy,
                KING_CASTLING_CHECK << (index - 3),
                threat_board,
            ) {
                Some(generated_move) => {
                    moves.push(generated_move);
                }
                None => {}
            }
        }
        if queen_side_castling {
            match generate_king_castling_move(
                index,
                index + 2,
                MF_QUEEN_CASTLING,
                is_black,
                QUEEN_CASTLING_CLEARANCE << (index - 3),
                occupancy,
                QUEEN_CASTLING_CHECK << (index - 3),
                threat_board,
            ) {
                Some(generated_move) => {
                    moves.push(generated_move);
                }
                None => {}
            }
        }
    }

    moves
}

fn generate_king_castling_move(
    from_index: u8,
    to_index: u8,
    castling_flag: u16,
    is_black: bool,
    castling_clearance_board: u64,
    occupancy: u64,
    castling_check_board: u64,
    threat_board: u64,
) -> Option<Move> {
    if (castling_clearance_board & occupancy == 0) & (castling_check_board & threat_board == 0) {
        let m = Move::new(
            from_index,
            to_index,
            castling_flag,
            PieceType::King,
            is_black,
            0,
        );
        return Some(m);
    }
    None
}

pub fn is_legal_king_move(m: Move, board: BoardRep) -> bool {
    return false;
}