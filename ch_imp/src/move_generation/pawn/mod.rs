use crate::{
    board::{
        attack_and_defend_lookups::AttackAndDefendTable,
        bitboard::Bitboard,
        board_rep::BoardRep,
        king_position_analysis::{ThreatRaycastCollision, ThreatSource},
        see::{see_from_capture, piece_safety},
    },
    r#move::Move,
    shared::{
        board_utils::get_rank,
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

pub(super) mod legal_move;
mod tests;

pub(super) fn generate_pawn_moves(
    board: BoardRep,
    ad_table: &mut AttackAndDefendTable,
    index: u8,
    opponent_occupancy: u64,
    king_threat: Option<ThreatSource>,
    pin: Option<ThreatRaycastCollision>,
    reveal_attack: Option<ThreatRaycastCollision>
) -> Vec<Move> {
    if king_threat != None {
        let kt = king_threat.unwrap();
        return generate_pawn_moves_when_threatened(index, kt.from, kt.threat_ray_mask, board, ad_table);
    }

    if pin != None && pin.unwrap().reveal_attack == false {
        let pin = pin.unwrap();
        return generate_pawn_moves_when_threatened(index, pin.from, pin.threat_ray_mask, board, ad_table);
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
                get_see(ad_table, board, to, false),
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
                        get_see(ad_table, board, dpp, false),
                    ));
                }
            }
        } else {
            moves.extend(generate_pawn_promotion_moves(
                index,
                to,
                false,
                board.black_turn,
                get_see(ad_table, board, to, false),
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
            let see = get_see(ad_table, board, capture_a, true);
            if capture_a_rank == 0 || capture_a_rank == 7 {
                moves.extend(generate_pawn_promotion_moves(
                    index,
                    capture_a,
                    true,
                    board.black_turn,
                    see,
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
                let see = get_see(ad_table, board, capture_b, true);
                if capture_b_rank == 0 || capture_b_rank == 7 {
                    moves.extend(generate_pawn_promotion_moves(
                        index,
                        capture_b,
                        true,
                        board.black_turn,
                        see,
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
) -> Vec<Move> {
    let mut moves = Vec::new();
    let offset_file: i8 = if board.black_turn { -1 } else { 1 };
    let rank = get_rank(index);

    if threat_ray_mask != 0 {
        let to: u8 = (index as i8 + (8 * offset_file)) as u8;
        if (1 << to) & threat_ray_mask != 0 {
            let see = get_see(ad_table, board, to, false);
            if get_rank(to) != 0 && get_rank(to) != 7 {
                moves.push(Move::new(
                    index,
                    to,
                    0b0,
                    PieceType::Pawn,
                    board.black_turn,
                    0,
                ));
            } else {
                moves.extend(generate_pawn_promotion_moves(
                    index,
                    to,
                    false,
                    board.black_turn,
                    0,
                ));
            }
        }

        if !board.occupancy.occupied(to)
            && ((board.black_turn && rank == 6) || (!board.black_turn && rank == 1))
        {
            let dpp = (to as i8 + (8 * offset_file)) as u8;
            if (1 << dpp) & threat_ray_mask != 0 {
                let see = get_see(ad_table, board, dpp, false);
                moves.push(Move::new(
                    index,
                    dpp,
                    MF_DOUBLE_PAWN_PUSH,
                    PieceType::Pawn,
                    board.black_turn,
                    see,
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
            let see = get_see(ad_table, board, capture_a, true);
            if capture_a_rank == 0 || capture_a_rank == 7 {
                moves.extend(generate_pawn_promotion_moves(
                    index,
                    capture_a,
                    true,
                    board.black_turn,
                    see,
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
                let see = get_see(ad_table, board, capture_b, true);
                if capture_b_rank == 0 || capture_b_rank == 7 {
                    moves.extend(generate_pawn_promotion_moves(
                        index,
                        capture_b,
                        true,
                        board.black_turn,
                        see,
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
) -> Vec<Move> {
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
        ), // Queen
    ];
}

fn get_see(
    ad_table: &mut AttackAndDefendTable,
    board: BoardRep,
    index: u8,
    is_capture: bool,
) -> i8 {
    let attacked_piece_type = board.get_piece_type_at_index(index);
    let attacked_by = ad_table.get_attacked_by(index, board, board.black_turn);
    let defended_by = ad_table.get_attacked_by(index, board, !board.black_turn);
    if is_capture {
        see_from_capture(
            PieceType::Pawn,
            attacked_by,
            attacked_piece_type,
            defended_by,
        )
    } else {
        piece_safety(PieceType::Pawn, true, attacked_by, defended_by)
    }
}
