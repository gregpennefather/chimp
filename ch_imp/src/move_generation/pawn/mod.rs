use crate::{
    board::{
        attack_and_defend_lookups::{AttackAndDefendTable, AttackedBy},
        bitboard::Bitboard,
        board_rep::BoardRep,
        king_position_analysis::{ThreatRaycastCollision, ThreatSource},
        see::{piece_safety, see_from_capture},
    },
    r#move::Move,
    shared::{
        board_utils::{get_file, get_rank},
        constants::{
            MF_BISHOP_CAPTURE_PROMOTION, MF_BISHOP_PROMOTION, MF_CAPTURE, MF_DOUBLE_PAWN_PUSH,
            MF_EP_CAPTURE, MF_KNIGHT_CAPTURE_PROMOTION, MF_KNIGHT_PROMOTION,
            MF_QUEEN_CAPTURE_PROMOTION, MF_QUEEN_PROMOTION, MF_ROOK_CAPTURE_PROMOTION,
            MF_ROOK_PROMOTION,
        },
        piece_type::PieceType,
    },
    MOVE_DATA,
};

use super::square_delta;

pub(super) mod legal_move;
mod tests;

pub(super) fn generate_pawn_moves(
    board: BoardRep,
    ad_table: &mut AttackAndDefendTable,
    index: u8,
    opponent_occupancy: u64,
    king_threat: Option<ThreatSource>,
    pin: Option<ThreatRaycastCollision>,
    reveal_attack: Option<ThreatRaycastCollision>,
    phase: i16,
) -> Vec<Move> {
    if king_threat != None {
        let kt = king_threat.unwrap();
        return generate_pawn_moves_when_threatened(
            index,
            kt.from,
            kt.threat_ray_mask,
            board,
            ad_table,
            reveal_attack,
            phase,
        );
    }

    if pin != None && pin.unwrap().reveal_attack == false {
        let pin = pin.unwrap();
        return generate_pawn_moves_when_threatened(
            index,
            pin.from,
            pin.threat_ray_mask,
            board,
            ad_table,
            reveal_attack,
            phase,
        );
    }

    let mut moves = Vec::new();
    let offset_file: i8 = if board.black_turn { -1 } else { 1 };
    let rank = get_rank(index);

    let to = (index as i8 + (8 * offset_file)) as u8;
    if !board.occupancy.occupied(to) {
        if get_rank(to) != 0 && get_rank(to) != 7 {
            moves.push(Move::new(
                index,
                to,
                0b0,
                PieceType::Pawn,
                board.black_turn,
                get_see(ad_table, board, to, false, reveal_attack),
                square_delta(
                    index as usize,
                    to as usize,
                    board.black_turn,
                    PieceType::Pawn,
                    phase,
                ),
            ));

            if (board.black_turn && rank == 6) || (!board.black_turn && rank == 1) {
                let dpp = (to as i8 + (8 * offset_file)) as u8;
                if !board.occupancy.occupied(dpp) {
                    moves.push(Move::new(
                        index,
                        dpp,
                        MF_DOUBLE_PAWN_PUSH,
                        PieceType::Pawn,
                        board.black_turn,
                        get_see(ad_table, board, dpp, false, reveal_attack),
                        square_delta(
                            index as usize,
                            to as usize,
                            board.black_turn,
                            PieceType::Pawn,
                            phase,
                        ),
                    ));
                }
            }
        } else {
            moves.extend(generate_pawn_promotion_moves(
                index,
                to,
                false,
                board.black_turn,
                get_see(ad_table, board, to, false, reveal_attack),
                phase,
            ));
        }
    }

    let capture_a = (index as i8 + (offset_file * 8)) as u8 + 1;
    let capture_a_rank = get_rank(capture_a);
    if (rank as i8 + offset_file) as u8 == capture_a_rank
        && (capture_a == board.ep_index || opponent_occupancy.occupied(capture_a))
    {
        let is_ep_capture = capture_a == board.ep_index;
        let leads_to_ep_check =
            ep_leads_to_orthogonal_check(board, index, index + 1, opponent_occupancy);
        if !is_ep_capture || !leads_to_ep_check {
            let see = get_see(ad_table, board, capture_a, true, reveal_attack);
            if capture_a_rank == 0 || capture_a_rank == 7 {
                moves.extend(generate_pawn_promotion_moves(
                    index,
                    capture_a,
                    true,
                    board.black_turn,
                    see,
                    phase,
                ));
            } else {
                moves.push(Move::new(
                    index,
                    capture_a,
                    if capture_a == board.ep_index {
                        MF_EP_CAPTURE
                    } else {
                        MF_CAPTURE
                    },
                    PieceType::Pawn,
                    board.black_turn,
                    see,
                    square_delta(
                        index as usize,
                        capture_a as usize,
                        board.black_turn,
                        PieceType::Pawn,
                        phase,
                    ),
                ));
            }
        }
    }

    if index > 8 {
        let capture_b = (index as i8 + (offset_file * 8)) as u8 - 1;
        let capture_b_rank = get_rank(capture_b);
        if (rank as i8 + offset_file) as u8 == capture_b_rank
            && (capture_b == board.ep_index || opponent_occupancy.occupied(capture_b))
        {
            let is_ep_capture = capture_b == board.ep_index;
            let leads_to_ep_check =
                ep_leads_to_orthogonal_check(board, index, index - 1, opponent_occupancy);
            if !is_ep_capture || !leads_to_ep_check {
                let see = get_see(ad_table, board, capture_b, true, reveal_attack);
                if capture_b_rank == 0 || capture_b_rank == 7 {
                    moves.extend(generate_pawn_promotion_moves(
                        index,
                        capture_b,
                        true,
                        board.black_turn,
                        see,
                        phase,
                    ));
                } else {
                    moves.push(Move::new(
                        index,
                        capture_b,
                        if capture_b == board.ep_index {
                            MF_EP_CAPTURE
                        } else {
                            MF_CAPTURE
                        },
                        PieceType::Pawn,
                        board.black_turn,
                        see,
                        square_delta(
                            index as usize,
                            capture_b as usize,
                            board.black_turn,
                            PieceType::Pawn,
                            phase,
                        ),
                    ));
                }
            }
        }
    }

    moves
}

pub fn ep_leads_to_orthogonal_check(
    board: BoardRep,
    pawn_position: u8,
    captured_pawn_position: u8,
    opponent_occupancy: u64,
) -> bool {
    let threat_sources = (board.rook_bitboard | board.queen_bitboard) & opponent_occupancy;
    if threat_sources != 0 {
        let king_index = if board.black_turn {
            board.black_king_position
        } else {
            board.white_king_position
        };
        let occupancy_without_pawns =
            board.occupancy ^ (1 << pawn_position) ^ (1 << captured_pawn_position);
        let move_board = MOVE_DATA
            .magic_bitboard_table
            .get_rook_attacks(king_index as usize, occupancy_without_pawns);
        return move_board & threat_sources != 0;
    }
    return false;
}

fn generate_pawn_moves_when_threatened(
    index: u8,
    threat_source: u8,
    threat_ray_mask: u64,
    board: BoardRep,
    ad_table: &mut AttackAndDefendTable,
    reveal_attack: Option<ThreatRaycastCollision>,
    phase: i16,
) -> Vec<Move> {
    let mut moves = Vec::new();
    let offset_file: i8 = if board.black_turn { -1 } else { 1 };
    let rank = get_rank(index);

    if threat_ray_mask != 0 {
        let to: u8 = (index as i8 + (8 * offset_file)) as u8;
        if (1 << to) & threat_ray_mask != 0 {
            let see = get_see(ad_table, board, to, false, reveal_attack);
            if get_rank(to) != 0 && get_rank(to) != 7 {
                moves.push(Move::new(
                    index,
                    to,
                    0b0,
                    PieceType::Pawn,
                    board.black_turn,
                    0,
                    square_delta(
                        index as usize,
                        to as usize,
                        board.black_turn,
                        PieceType::Pawn,
                        phase,
                    ),
                ));
            } else {
                moves.extend(generate_pawn_promotion_moves(
                    index,
                    to,
                    false,
                    board.black_turn,
                    0,
                    phase,
                ));
            }
        }

        if !board.occupancy.occupied(to)
            && ((board.black_turn && rank == 6) || (!board.black_turn && rank == 1))
        {
            let dpp = (to as i8 + (8 * offset_file)) as u8;
            if (1 << dpp) & threat_ray_mask != 0 {
                let see = get_see(ad_table, board, dpp, false, reveal_attack);
                moves.push(Move::new(
                    index,
                    dpp,
                    MF_DOUBLE_PAWN_PUSH,
                    PieceType::Pawn,
                    board.black_turn,
                    see,
                    square_delta(
                        index as usize,
                        to as usize,
                        board.black_turn,
                        PieceType::Pawn,
                        phase,
                    ),
                ));
            }
        }
    }

    let capture_a = (index as i8 + (offset_file * 8)) as u8 + 1;
    if threat_source == capture_a
        || (capture_a == board.ep_index
            && threat_source == (board.ep_index as i8 - (offset_file * 8)) as u8)
    {
        let capture_a_rank = get_rank(capture_a);
        if (rank as i8 + offset_file) as u8 == capture_a_rank {
            let see = get_see(ad_table, board, capture_a, true, reveal_attack);
            if capture_a_rank == 0 || capture_a_rank == 7 {
                moves.extend(generate_pawn_promotion_moves(
                    index,
                    capture_a,
                    true,
                    board.black_turn,
                    see,
                    phase,
                ));
            } else {
                moves.push(Move::new(
                    index,
                    capture_a,
                    if capture_a == board.ep_index {
                        MF_EP_CAPTURE
                    } else {
                        MF_CAPTURE
                    },
                    PieceType::Pawn,
                    board.black_turn,
                    see,
                    square_delta(
                        index as usize,
                        capture_a as usize,
                        board.black_turn,
                        PieceType::Pawn,
                        phase,
                    ),
                ));
            }
        }
    }
    if index > 8 {
        let capture_b = (index as i8 + (offset_file * 8)) as u8 - 1;
        if threat_source == capture_b
            || (capture_b == board.ep_index
                && threat_source == (board.ep_index as i8 - (offset_file * 8)) as u8)
        {
            let capture_b_rank = get_rank(capture_b);
            if (rank as i8 + offset_file) as u8 == capture_b_rank {
                let see = get_see(ad_table, board, capture_b, true, reveal_attack);
                if capture_b_rank == 0 || capture_b_rank == 7 {
                    moves.extend(generate_pawn_promotion_moves(
                        index,
                        capture_b,
                        true,
                        board.black_turn,
                        see,
                        phase,
                    ));
                } else {
                    moves.push(Move::new(
                        index,
                        capture_b,
                        if capture_b == board.ep_index {
                            MF_EP_CAPTURE
                        } else {
                            MF_CAPTURE
                        },
                        PieceType::Pawn,
                        board.black_turn,
                        see,
                        square_delta(
                            index as usize,
                            capture_b as usize,
                            board.black_turn,
                            PieceType::Pawn,
                            phase,
                        ),
                    ));
                }
            }
        }
    }
    moves
}

fn generate_pawn_promotion_moves(
    from_index: u8,
    to_index: u8,
    is_capture: bool,
    is_black: bool,
    see: i8,
    phase: i16,
) -> Vec<Move> {
    let square_delta = square_delta(
        from_index as usize,
        to_index as usize,
        is_black,
        PieceType::Pawn,
        phase,
    );
    return vec![
        Move::new(
            from_index,
            to_index,
            if !is_capture {
                MF_KNIGHT_PROMOTION
            } else {
                MF_KNIGHT_CAPTURE_PROMOTION
            },
            PieceType::Pawn,
            is_black,
            see,
            square_delta,
        ), // Knight
        Move::new(
            from_index,
            to_index,
            if !is_capture {
                MF_BISHOP_PROMOTION
            } else {
                MF_BISHOP_CAPTURE_PROMOTION
            },
            PieceType::Pawn,
            is_black,
            see,
            square_delta,
        ), // Bishop
        Move::new(
            from_index,
            to_index,
            if !is_capture {
                MF_ROOK_PROMOTION
            } else {
                MF_ROOK_CAPTURE_PROMOTION
            },
            PieceType::Pawn,
            is_black,
            see,
            square_delta,
        ), // Rook
        Move::new(
            from_index,
            to_index,
            if !is_capture {
                MF_QUEEN_PROMOTION
            } else {
                MF_QUEEN_CAPTURE_PROMOTION
            },
            PieceType::Pawn,
            is_black,
            see,
            square_delta,
        ), // Queen
    ];
}

fn get_see(
    ad_table: &mut AttackAndDefendTable,
    board: BoardRep,
    index: u8,
    is_capture: bool,
    reveal_attack: Option<ThreatRaycastCollision>,
) -> i8 {
    let attacked_piece_type = board.get_piece_type_at_index(index);
    let friendly = ad_table.get_attacked_by(index, board, board.black_turn);
    let opponent = match reveal_attack {
        None => ad_table.get_attacked_by(index, board, !board.black_turn),
        Some(ra) => {
            if !ra.threat_ray_mask.occupied(index) {
                AttackedBy::default()
            } else {
                ad_table.get_attacked_by(index, board, !board.black_turn)
            }
        }
    };

    if is_capture {
        see_from_capture(PieceType::Pawn, friendly, attacked_piece_type, opponent)
    } else {
        piece_safety(PieceType::Pawn, true, opponent, friendly)
    }
}

pub(crate) fn get_pawn_threat_positions(index: u8, is_black: bool) -> u64 {
    let file = get_file(index);
    let offset_file: i8 = if is_black { -1 } else { 1 };
    let mut r = 0;
    if file != 0 {
        r |= 1 << ((index as i8 + (offset_file * 8)) + 1)
    }
    if file != 7 {
        r |= 1 << ((index as i8 + (offset_file * 8)) - 1)
    }
    r
}
