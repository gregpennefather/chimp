use crate::{
    board::{
        attack_and_defend_lookups::AttackAndDefendTable,
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

use super::{moveboard_to_moves, square_delta};

pub(super) fn generate_king_moves(
    index: u8,
    opponent_occupancy: u64,
    occupancy: u64,
    king_analysis: &KingPositionAnalysis,
    is_black: bool,
    king_side_castling: bool,
    queen_side_castling: bool,
    board: BoardRep,
    ad_table: &mut AttackAndDefendTable,
    phase: i16
) -> Vec<Move> {
    let moveboard = get_legal_moveboard(index, ad_table, board, is_black);
    let mut moves = moveboard_to_moves(
        index,
        PieceType::King,
        moveboard,
        opponent_occupancy,
        occupancy,
        board,
        ad_table,
        None,
        phase
    );

    if !king_analysis.check {
        if king_side_castling {
            match generate_king_castling_move(
                index,
                index - 2,
                MF_KING_CASTLING,
                is_black,
                KING_CASTLING_CLEARANCE << (index - 3),
                KING_CASTLING_CHECK << (index - 3),
                board,
                ad_table,
                phase
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
                QUEEN_CASTLING_CHECK << (index - 3),
                board,
                ad_table,
                phase
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

fn get_legal_moveboard(
    index: u8,
    ad_table: &AttackAndDefendTable,
    board: BoardRep,
    is_black: bool,
) -> u64 {
    let mut moveboard = MOVE_DATA.king_moves[index as usize];
    let mut r = moveboard;

    while moveboard != 0 {
        let lsb = moveboard.trailing_zeros();
        if ad_table.has_at_least_one_attacker(lsb as u8, !is_black, true, board) {
            r ^= 1 << lsb;
        }
        moveboard ^= 1 << lsb;
    }
    r
}

fn generate_king_castling_move(
    from_index: u8,
    to_index: u8,
    castling_flag: u16,
    is_black: bool,
    castling_clearance_board: u64,
    mut castling_check_board: u64,
    board: BoardRep,
    ad_table: &AttackAndDefendTable,
    phase: i16
) -> Option<Move> {
    if castling_clearance_board & board.occupancy != 0 {
        return None;
    }

    while castling_check_board != 0 {
        let lsb = castling_check_board.trailing_zeros();
        if ad_table.has_at_least_one_attacker(lsb as u8, !is_black, true, board) {
            return None;
        }
        castling_check_board = castling_check_board.flip(lsb as u8);
    }

    let m = Move::new(
        from_index,
        to_index,
        castling_flag,
        PieceType::King,
        is_black,
        0,
        square_delta(from_index as usize, to_index as usize, is_black, PieceType::King, phase)
    );
    return Some(m);
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
    !is_in_check(m.to(), board, king_position_analysis)
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
    if is_in_check(m.from(), board, king_position_analysis) {
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

    !any_position_in_check(check_mask, board)
}

fn is_in_check(to: u8, board: BoardRep, king_position_analysis: &KingPositionAnalysis) -> bool {
    if position_is_in_check(to, board) {
        return true;
    }
    match king_position_analysis.threat_source {
        Some(t) => t.threat_ray_mask.occupied(to),
        None => false,
    }
}

fn any_position_in_check(mut mask: u64, board: BoardRep) -> bool {
    while mask != 0 {
        let lsb = mask.trailing_zeros() as u8;
        if position_is_in_check(lsb, board) {
            return true;
        }
        mask = mask.flip(lsb);
    }
    false
}

fn position_is_in_check(index: u8, board: BoardRep) -> bool {
    board.has_at_least_one_attacker(index, !board.black_turn, true)
}
