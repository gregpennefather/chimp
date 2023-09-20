use crate::{
    board::{
        bitboard::Bitboard,
        board_rep::BoardRep,
        king_position_analysis::{self, KingPositionAnalysis},
    },
    r#move::{
        move_data::{
            KING_CASTLING_CHECK, KING_CASTLING_CLEARANCE, QUEEN_CASTLING_CHECK,
            QUEEN_CASTLING_CLEARANCE,
        },
        Move,
    },
    shared::{
        board_utils::chebyshev_distance,
        constants::{MF_CAPTURE, MF_KING_CASTLING, MF_QUEEN_CASTLING},
        piece_type::PieceType,
    },
    MOVE_DATA,
};

mod tests;

use super::{generate_threat_board, moveboard_to_moves};

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

pub fn is_legal_king_move(
    m: Move,
    board: BoardRep,
    king_position_analysis: &KingPositionAnalysis,
) -> bool {
    match m.flags() {
        0 => is_legal_move(m, board, king_position_analysis),
        MF_CAPTURE => is_legal_capture(m, board, king_position_analysis),
        MF_KING_CASTLING => is_legal_castling(true, m, board, king_position_analysis),
        MF_QUEEN_CASTLING => is_legal_castling(false, m, board, king_position_analysis),
        _ => false,
    }
}

fn is_legal_move(m: Move, board: BoardRep, king_position_analysis: &KingPositionAnalysis) -> bool {
    if chebyshev_distance(m.from() as i8, m.to() as i8) != 1 {
        return false;
    }
    let opponent_threatboard =
        generate_threat_board(!m.is_black(), board.get_opponent_occupancy(), board);
    !is_in_check(m.to(), opponent_threatboard, king_position_analysis)
}

fn is_legal_capture(
    m: Move,
    board: BoardRep,
    king_position_analysis: &KingPositionAnalysis,
) -> bool {
    if !is_legal_move(m, board, king_position_analysis) {
        return false;
    }
    board.get_opponent_occupancy().occupied(m.to())
}

pub(super) fn is_legal_castling(
    king_side: bool,
    m: Move,
    board: BoardRep,
    king_position_analysis: &KingPositionAnalysis,
) -> bool {
    let opponent_threatboard =
        generate_threat_board(!m.is_black(), board.get_opponent_occupancy(), board);
    if is_in_check(m.from(), opponent_threatboard, king_position_analysis) {
        return false;
    }

    if !match (board.black_turn, king_side) {
        (true, true) => board.black_king_side_castling,
        (true, false) => board.black_queen_side_castling,
        (false, true) => board.white_king_side_castling,
        (false, false) => board.white_queen_side_castling,
    } {
        return false;
    }

    let clearance_mask = if king_side {
        KING_CASTLING_CLEARANCE
    } else {
        QUEEN_CASTLING_CLEARANCE
    } << (m.from() as i8 - 3);

    if clearance_mask & board.occupancy != 0 {
        return false;
    }

    let check_mask = if king_side {
        KING_CASTLING_CHECK
    } else {
        QUEEN_CASTLING_CHECK
    } << (m.from() as i8 - 3);

    opponent_threatboard & check_mask == 0
}

fn is_in_check(to: u8, threat_board: u64, king_position_analysis: &KingPositionAnalysis) -> bool {
    if threat_board.occupied(to) {
        return true;
    }
    match king_position_analysis.threat_source {
        Some(t) => t.threat_ray_mask.occupied(to),
        None => false,
    }
}
