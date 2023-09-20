use crate::{r#move::Move, board::{board_rep::BoardRep, bitboard::Bitboard}, shared::{constants::{MF_CAPTURE, MF_EP_CAPTURE, MF_DOUBLE_PAWN_PUSH, MF_KNIGHT_PROMOTION, MF_BISHOP_PROMOTION, MF_ROOK_PROMOTION, MF_QUEEN_PROMOTION, MF_KNIGHT_CAPTURE_PROMOTION, MF_BISHOP_CAPTURE_PROMOTION, MF_ROOK_CAPTURE_PROMOTION, MF_QUEEN_CAPTURE_PROMOTION}, board_utils::get_rank}};

pub fn is_legal_pawn_move(m: Move, board: BoardRep) -> bool {
    match m.flags() {
        MF_CAPTURE => is_legal_capture(m, board),
        MF_EP_CAPTURE => is_legal_ep_capture(m, board),
        MF_DOUBLE_PAWN_PUSH => is_legal_ddp(m, board),
        MF_KNIGHT_PROMOTION | MF_BISHOP_PROMOTION | MF_ROOK_PROMOTION | MF_QUEEN_PROMOTION => is_legal_promotion(m, board),
        MF_KNIGHT_CAPTURE_PROMOTION | MF_BISHOP_CAPTURE_PROMOTION | MF_ROOK_CAPTURE_PROMOTION | MF_QUEEN_CAPTURE_PROMOTION => is_legal_capture_promotion(m, board),
        0 => is_legal_move(m, board),
        _ => false
    }
}

fn is_legal_move(m: Move, board: BoardRep) -> bool {
    let offset_file: i8 = if board.black_turn { -1 } else { 1 };
    let push_forward_square = (m.from() as i8 + (offset_file*8)) as u8;
    push_forward_square == m.to() && !board.occupancy.occupied(push_forward_square)
}

fn is_legal_capture(m: Move, board: BoardRep) -> bool {
    let offset_file: i8 = if board.black_turn { -1 } else { 1 };
    let push_forward_square = (m.from() as i8 + (offset_file*8)) as u8;
    (m.to() == push_forward_square + 1 || m.to() == push_forward_square - 1) && board.get_opponent_occupancy().occupied(m.to())
}

fn is_legal_ddp(m: Move, board: BoardRep) -> bool {
    let rank = get_rank(m.from());
    if (!m.is_black() && rank != 1) || (m.is_black() && rank != 6) {
        return false
    }
    let offset_file: i8 = if board.black_turn { -1 } else { 1 };
    let mask = (1 << (m.from() as i8 + (offset_file*8))) | (1<< (m.from() as i8 + (offset_file*8*2)));
    board.occupancy & mask == 0
}

fn is_legal_ep_capture(m: Move, board: BoardRep) -> bool {
    m.to() == board.ep_index
}


fn is_legal_capture_promotion(m: Move, board: BoardRep) -> bool {
    let rank = get_rank(m.to());
    if (m.is_black() && rank != 0) || (!m.is_black() && rank != 7) {
        return false
    }
    !board.occupancy.occupied(m.to())
}

fn is_legal_promotion(m: Move, board: BoardRep) -> bool {
    let rank = get_rank(m.to());
    if (m.is_black() && rank != 0) || (!m.is_black() && rank != 7) {
        return false
    }
    is_legal_capture(m, board)
}
