use log::info;

use crate::{
    board::{
        bitboard::Bitboard,
        board_rep::BoardRep,
        king_position_analysis::{
            analyze_king_position, KingPositionAnalysis, ThreatRaycastCollision, ThreatSource,
            ThreatType,
        },
        position::Position,
    },
    shared::{
        board_utils::{get_direction_to_normalized, get_file, get_rank},
        constants::{
            MF_BISHOP_CAPTURE_PROMOTION, MF_BISHOP_PROMOTION, MF_CAPTURE, MF_DOUBLE_PAWN_PUSH,
            MF_EP_CAPTURE, MF_KING_CASTLING, MF_KNIGHT_CAPTURE_PROMOTION, MF_KNIGHT_PROMOTION,
            MF_QUEEN_CAPTURE_PROMOTION, MF_QUEEN_CASTLING, MF_QUEEN_PROMOTION,
            MF_ROOK_CAPTURE_PROMOTION, MF_ROOK_PROMOTION,
        },
        piece_type::{self, PieceType, PIECE_TYPE_EXCHANGE_VALUE},
    },
    MOVE_DATA,
};

use super::{
    calculate_see,
    move_data::{
        KING_CASTLING_CHECK, KING_CASTLING_CLEARANCE, QUEEN_CASTLING_CHECK,
        QUEEN_CASTLING_CLEARANCE,
    },
    Move,
};

#[derive(Clone, Default)]
pub struct MoveGenerationEvalMetrics {
    pub white_threatboard: u64,
    pub black_threatboard: u64,
    pub white_pinned: Vec<ThreatRaycastCollision>,
    pub black_pinned: Vec<ThreatRaycastCollision>,
}

pub fn generate_moves(
    king_analysis: &KingPositionAnalysis,
    opponent_king_analysis: &KingPositionAnalysis,
    board: BoardRep,
) -> (Vec<Move>, MoveGenerationEvalMetrics) {
    let mut friendly_occupancy = if board.black_turn {
        board.black_occupancy
    } else {
        board.white_occupancy
    };
    let opponent_occupancy = if !board.black_turn {
        board.black_occupancy
    } else {
        board.white_occupancy
    };

    let (king_pos, king_side_castling, queen_side_castling) = if board.black_turn {
        (
            board.black_king_position,
            board.black_king_side_castling,
            board.black_queen_side_castling,
        )
    } else {
        (
            board.white_king_position,
            board.white_king_side_castling,
            board.white_queen_side_castling,
        )
    };

    let friendly_threat_board = generate_threat_board(board.black_turn, friendly_occupancy, board);
    let opponent_threat_board = generate_threat_board(!board.black_turn, opponent_occupancy, board);

    let mut moves = generate_king_moves(
        king_pos,
        opponent_occupancy,
        board.occupancy,
        king_analysis.check,
        board.black_turn,
        king_side_castling,
        queen_side_castling,
        opponent_threat_board,
        king_analysis.threat_source,
        board,
    );

    // In the event of double king check we can only avoid check by moving the king
    if king_analysis.double_check {
        let metrics = build_move_generation_eval_metrics(
            board.black_turn,
            king_analysis,
            opponent_king_analysis,
            friendly_threat_board,
            opponent_threat_board,
        );
        return (moves, metrics);
    }

    while friendly_occupancy != 0 {
        let piece_position = friendly_occupancy.trailing_zeros() as u8;
        moves.extend(generate_index_moves(board, piece_position, king_analysis));
        friendly_occupancy ^= 1 << piece_position;
    }

    let metrics = build_move_generation_eval_metrics(
        board.black_turn,
        king_analysis,
        opponent_king_analysis,
        friendly_threat_board,
        opponent_threat_board,
    );
    moves.sort();
    (moves, metrics)
}

pub fn generate_moves_for_board(board: BoardRep) -> Vec<Move> {
    let white_king_analysis = board.get_white_king_analysis();

    let black_king_analysis = board.get_black_king_analysis();

    let (king_analysis, opponent_king_analysis) = if board.black_turn {
        (black_king_analysis, white_king_analysis)
    } else {
        (white_king_analysis, black_king_analysis)
    };

    generate_moves(&king_analysis, &opponent_king_analysis, board).0
}

fn generate_index_moves(
    board: BoardRep,
    index: u8,
    king_analysis: &KingPositionAnalysis,
) -> Vec<Move> {
    let piece_type = board.get_piece_type_at_index(index);
    let opponent_occupancy = if board.black_turn {
        board.white_occupancy
    } else {
        board.black_occupancy
    };
    let pin = Option::<&ThreatRaycastCollision>::copied(
        king_analysis.pins.iter().find(|p| p.at == index),
    );

    // If we're pinned but the king is also threatened we can't help
    if pin != None && king_analysis.threat_source != None {
        return vec![];
    }

    match piece_type {
        piece_type::PieceType::Pawn => generate_pawn_moves(
            board,
            index,
            opponent_occupancy,
            king_analysis.threat_source,
            pin,
        ),
        piece_type::PieceType::Knight => match pin {
            Some(_) => vec![],
            None => generate_knight_moves(
                index,
                opponent_occupancy,
                board.occupancy,
                king_analysis.threat_source,
                board,
            ),
        },
        piece_type::PieceType::Bishop => {
            match pin {
                Some(pin) => {
                    if pin.threat_type != ThreatType::DiagonalSlide {
                        return vec![];
                    } else {
                    }
                }
                None => {}
            }
            generate_bishop_moves(
                index,
                board,
                opponent_occupancy,
                board.occupancy,
                king_analysis.threat_source,
                pin,
            )
        }
        piece_type::PieceType::Rook => {
            match pin {
                Some(pin) => {
                    if pin.threat_type != ThreatType::OrthogonalSlide {
                        return vec![];
                    } else {
                    }
                }
                None => {}
            }
            generate_rook_moves(
                index,
                board,
                opponent_occupancy,
                board.occupancy,
                king_analysis.threat_source,
                pin,
            )
        }
        piece_type::PieceType::Queen => generate_queen_moves(
            index,
            board,
            opponent_occupancy,
            board.occupancy,
            king_analysis.threat_source,
            pin,
        ),
        piece_type::PieceType::King => Vec::new(),
        _ => panic!(
            "Unexpected piece {piece_type:?} at position {index} : {}",
            board.to_fen()
        ),
    }
}

pub fn generate_threat_board(is_black: bool, mut piece_occupancy: u64, board: BoardRep) -> u64 {
    let mut r: u64 = 0;
    // Ignore the opponents king when generating threat board - we primarily use threatboard for checking king safety
    // TODO: Replace this with a 'generate_king_area_threatboard' that is potentially faster and generates less useless info
    let occ_without_opponent_king = if is_black {
        board.occupancy ^ (1 << board.white_king_position)
    } else {
        board.occupancy ^ (1 << board.black_king_position)
    };
    while piece_occupancy != 0 {
        let piece_position = piece_occupancy.trailing_zeros() as u8;
        r |= match board.get_piece_type_at_index(piece_position) {
            piece_type::PieceType::Pawn => get_pawn_threatboard(piece_position, is_black),
            piece_type::PieceType::Knight => MOVE_DATA.knight_moves[piece_position as usize],
            piece_type::PieceType::Bishop => MOVE_DATA
                .magic_bitboard_table
                .get_bishop_attacks(piece_position as usize, occ_without_opponent_king),
            piece_type::PieceType::Rook => MOVE_DATA
                .magic_bitboard_table
                .get_rook_attacks(piece_position as usize, occ_without_opponent_king),
            piece_type::PieceType::Queen => {
                MOVE_DATA
                    .magic_bitboard_table
                    .get_bishop_attacks(piece_position as usize, occ_without_opponent_king)
                    | MOVE_DATA
                        .magic_bitboard_table
                        .get_rook_attacks(piece_position as usize, occ_without_opponent_king)
            }
            piece_type::PieceType::King => MOVE_DATA.king_moves[piece_position as usize],
            _ => 0,
        };

        piece_occupancy ^= 1 << piece_position;
    }
    r
}

fn build_move_generation_eval_metrics(
    is_black: bool,
    friendly_king_analysis: &KingPositionAnalysis,
    opponent_king_analysis: &KingPositionAnalysis,
    friendly_threatboard: u64,
    opponent_threatboard: u64,
) -> MoveGenerationEvalMetrics {
    if is_black {
        MoveGenerationEvalMetrics {
            white_threatboard: opponent_threatboard,
            black_threatboard: friendly_threatboard,
            black_pinned: friendly_king_analysis.pins.clone(),
            white_pinned: opponent_king_analysis.pins.clone(),
        }
    } else {
        MoveGenerationEvalMetrics {
            white_threatboard: friendly_threatboard,
            black_threatboard: opponent_threatboard,
            black_pinned: opponent_king_analysis.pins.clone(),
            white_pinned: friendly_king_analysis.pins.clone(),
        }
    }
}

fn get_pawn_threatboard(piece_position: u8, is_black: bool) -> u64 {
    let file = get_file(piece_position);
    if is_black {
        let mut r = if piece_position > 8 && file != 7 {
            1 << piece_position - 9
        } else {
            0
        };
        if piece_position > 7 && file != 0 {
            r |= 1 << piece_position - 7
        };

        return r;
    } else {
        let mut r = if piece_position < 54 && file != 0 {
            1 << piece_position + 9
        } else {
            0
        };

        if piece_position < 55 && file != 7 {
            r |= 1 << piece_position + 7
        }

        return r;
    }
}

fn generate_king_moves(
    index: u8,
    opponent_occupancy: u64,
    occupancy: u64,
    in_check: bool,
    is_black: bool,
    king_side_castling: bool,
    queen_side_castling: bool,
    threat_board: u64,
    threat: Option<ThreatSource>,
    board: BoardRep,
) -> Vec<Move> {
    let mut moveboard = MOVE_DATA.king_moves[index as usize];
    moveboard = moveboard & !threat_board;
    let mut moves = moveboard_to_moves(
        index,
        piece_type::PieceType::King,
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

pub(crate) fn generate_queen_moves(
    index: u8,
    board: BoardRep,
    opponent_occupancy: u64,
    occupancy: u64,
    king_threat: Option<ThreatSource>,
    pin: Option<ThreatRaycastCollision>,
) -> Vec<Move> {
    let mut moveboard = match pin {
        Some(p) => p.threat_ray_mask | (1 << p.from),
        None => {
            MOVE_DATA
                .magic_bitboard_table
                .get_bishop_attacks(index as usize, board.occupancy.into())
                | MOVE_DATA
                    .magic_bitboard_table
                    .get_rook_attacks(index as usize, board.occupancy.into())
        }
    };

    if king_threat != None {
        let threat = king_threat.unwrap();
        moveboard &= threat.threat_ray_mask | (1 << threat.from);
    }

    moveboard_to_moves(
        index,
        piece_type::PieceType::Queen,
        moveboard,
        opponent_occupancy,
        occupancy,
        board,
    )
}

fn generate_rook_moves(
    index: u8,
    board: BoardRep,
    opponent_occupancy: u64,
    occupancy: u64,
    king_threat: Option<ThreatSource>,
    pin: Option<ThreatRaycastCollision>,
) -> Vec<Move> {
    let mut moveboard = match pin {
        Some(p) => p.threat_ray_mask | (1 << p.from),
        None => MOVE_DATA
            .magic_bitboard_table
            .get_rook_attacks(index as usize, board.occupancy.into()),
    };

    if king_threat != None {
        let threat = king_threat.unwrap();
        moveboard &= threat.threat_ray_mask | (1 << threat.from);
    }

    moveboard_to_moves(
        index,
        piece_type::PieceType::Rook,
        moveboard,
        opponent_occupancy,
        occupancy,
        board,
    )
}
fn generate_bishop_moves(
    index: u8,
    board: BoardRep,
    opponent_occupancy: u64,
    occupancy: u64,
    king_threat: Option<ThreatSource>,
    pin: Option<ThreatRaycastCollision>,
) -> Vec<Move> {
    let mut moveboard = match pin {
        Some(p) => p.threat_ray_mask | (1 << p.from),
        None => MOVE_DATA
            .magic_bitboard_table
            .get_bishop_attacks(index as usize, board.occupancy.into()),
    };

    if king_threat != None {
        let threat = king_threat.unwrap();
        moveboard &= threat.threat_ray_mask | (1 << threat.from);
    }

    moveboard_to_moves(
        index,
        piece_type::PieceType::Bishop,
        moveboard,
        opponent_occupancy,
        occupancy,
        board,
    )
}

fn generate_knight_moves(
    index: u8,
    opponent_occupancy: u64,
    occupancy: u64,
    king_threat: Option<ThreatSource>,
    board: BoardRep,
) -> Vec<Move> {
    let mut moveboard = MOVE_DATA.knight_moves[index as usize];

    if king_threat != None {
        let threat = king_threat.unwrap();
        moveboard &= threat.threat_ray_mask | (1 << threat.from);
    }

    moveboard_to_moves(
        index,
        piece_type::PieceType::Knight,
        moveboard,
        opponent_occupancy,
        occupancy,
        board,
    )
}

fn generate_pawn_moves(
    board: BoardRep,
    index: u8,
    opponent_occupancy: u64,
    king_threat: Option<ThreatSource>,
    pin: Option<ThreatRaycastCollision>,
) -> Vec<Move> {
    if king_threat != None {
        let kt = king_threat.unwrap();
        return generate_pawn_moves_when_threatened(index, kt.from, kt.threat_ray_mask, board);
    }

    if pin != None && pin.unwrap().reveal_attack == false {
        let pin = pin.unwrap();
        return generate_pawn_moves_when_threatened(index, pin.from, pin.threat_ray_mask, board);
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
                piece_type::PieceType::Pawn,
                board.black_turn,
                0,
            ));

            if (board.black_turn && rank == 6) || (!board.black_turn && rank == 1) {
                let dpp = (to as i8 + (8 * offset_file)) as u8;
                if !board.occupancy.occupied(dpp) {
                    moves.push(Move::new(
                        index,
                        dpp,
                        MF_DOUBLE_PAWN_PUSH,
                        piece_type::PieceType::Pawn,
                        board.black_turn,
                        0,
                    ));
                }
            }
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

    let capture_a = (index as i8 + (offset_file * 8)) as u8 + 1;
    let capture_a_rank = get_rank(capture_a);
    if (rank as i8 + offset_file) as u8 == capture_a_rank
        && (capture_a == board.ep_index || opponent_occupancy.occupied(capture_a))
    {
        let is_ep_capture = capture_a == board.ep_index;
        let leads_to_ep_check =
            ep_leads_to_orthogonal_check(board, index, index + 1, opponent_occupancy);
        if !is_ep_capture || !leads_to_ep_check {
            let see = calculate_see(PieceType::Pawn, board.get_piece_type_at_index(capture_a));
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
                    piece_type::PieceType::Pawn,
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
                let see = calculate_see(PieceType::Pawn, board.get_piece_type_at_index(capture_b));
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
                        piece_type::PieceType::Pawn,
                        board.black_turn,
                        see,
                    ));
                }
            }
        }
    }

    moves
}

fn ep_leads_to_orthogonal_check(
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
) -> Vec<Move> {
    let mut moves = Vec::new();
    let offset_file: i8 = if board.black_turn { -1 } else { 1 };
    let rank = get_rank(index);

    if threat_ray_mask != 0 {
        let to: u8 = (index as i8 + (8 * offset_file)) as u8;
        if (1 << to) & threat_ray_mask != 0 {
            if get_rank(to) != 0 && get_rank(to) != 7 {
                moves.push(Move::new(
                    index,
                    to,
                    0b0,
                    piece_type::PieceType::Pawn,
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
                moves.push(Move::new(
                    index,
                    dpp,
                    MF_DOUBLE_PAWN_PUSH,
                    piece_type::PieceType::Pawn,
                    board.black_turn,
                    0,
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
            let see = calculate_see(PieceType::Pawn, board.get_piece_type_at_index(capture_a));
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
                    piece_type::PieceType::Pawn,
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
                let see = calculate_see(PieceType::Pawn, board.get_piece_type_at_index(capture_b));
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
                        piece_type::PieceType::Pawn,
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
            piece_type::PieceType::Pawn,
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
            piece_type::PieceType::Pawn,
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
            piece_type::PieceType::Pawn,
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
            piece_type::PieceType::Pawn,
            is_black,
            see,
        ), // Queen
    ];
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
            piece_type::PieceType::King,
            is_black,
            0,
        );
        return Some(m);
    }
    None
}

fn moveboard_to_moves(
    from_index: u8,
    piece_type: piece_type::PieceType,
    moveboard: u64,
    opponent_occupancy: u64,
    occupancy: u64,
    board: BoardRep,
) -> Vec<Move> {
    let mut generated_moves = Vec::new();
    let mut m_b = moveboard;
    while m_b != 0 {
        let lsb = m_b.trailing_zeros() as u8;
        if opponent_occupancy.occupied(lsb) {
            let see = calculate_see(piece_type, board.get_piece_type_at_index(lsb));
            generated_moves.push(Move::new(
                from_index,
                lsb,
                MF_CAPTURE,
                piece_type,
                board.black_turn,
                see,
            ));
        } else if !occupancy.occupied(lsb) {
            generated_moves.push(Move::new(
                from_index,
                lsb,
                0b0,
                piece_type,
                board.black_turn,
                0,
            ));
        };
        m_b ^= 1 << lsb;
    }

    generated_moves
}

#[cfg(test)]
mod test {

    use rand::seq::index;

    use crate::{
        board::king_position_analysis::analyze_king_position,
        shared::board_utils::index_from_coords,
    };

    use super::*;

    #[test]
    pub fn startpos_move_generation() {
        let board = BoardRep::default();
        let white_king_analysis = analyze_king_position(
            board.white_king_position,
            false,
            board.occupancy,
            board.white_occupancy,
            board.black_occupancy,
            board.pawn_bitboard,
            board.knight_bitboard,
            board.bishop_bitboard,
            board.rook_bitboard,
            board.queen_bitboard,
            board.black_turn,
        );

        let black_king_analysis = analyze_king_position(
            board.black_king_position,
            true,
            board.occupancy,
            board.black_occupancy,
            board.white_occupancy,
            board.pawn_bitboard,
            board.knight_bitboard,
            board.bishop_bitboard,
            board.rook_bitboard,
            board.queen_bitboard,
            !board.black_turn,
        );
        let moves = generate_moves(&white_king_analysis, &black_king_analysis, board);
        assert_eq!(moves.0.len(), 20);
    }

    #[test]
    pub fn king_double_checked() {
        let board = BoardRep::from_fen(
            "rnbqk1nr/pppp1pNp/2Pb4/8/1B6/4Q3/PP1PPPPP/RN2KB1R b KQkq - 0 1".into(),
        );
        let white_king_analysis = analyze_king_position(
            board.white_king_position,
            false,
            board.occupancy,
            board.white_occupancy,
            board.black_occupancy,
            board.pawn_bitboard,
            board.knight_bitboard,
            board.bishop_bitboard,
            board.rook_bitboard,
            board.queen_bitboard,
            board.black_turn,
        );

        let black_king_analysis = analyze_king_position(
            board.black_king_position,
            true,
            board.occupancy,
            board.black_occupancy,
            board.white_occupancy,
            board.pawn_bitboard,
            board.knight_bitboard,
            board.bishop_bitboard,
            board.rook_bitboard,
            board.queen_bitboard,
            !board.black_turn,
        );
        let moves = generate_moves(&black_king_analysis, &white_king_analysis, board);
        assert!(moves.0.len() <= 2);
    }

    #[test]
    pub fn pawn_moves_scenario_0() {
        let board = BoardRep::from_fen(
            "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1".into(),
        );
        let moves = generate_pawn_moves(board, 9, board.black_occupancy, None, None);
        assert_eq!(moves.len(), 3);
    }

    #[test]
    pub fn pawn_moves_scenario_1() {
        let board = BoardRep::from_fen(
            "r3k2r/p1ppqpb1/bn2pnp1/1N1PN3/1p2P3/5Q2/PPPBBPpP/R3K2R b KQkq - 0 2".into(),
        );
        let moves = generate_pawn_moves(board, 9, board.white_occupancy, None, None);
        assert_eq!(moves.len(), 8);
    }

    #[test]
    pub fn move_generation_capture_the_threat_with_knight_or_move_the_king() {
        let board = BoardRep::from_fen(
            "r3k2r/p1Np1pb1/b3pnpq/1n1PN3/1p2P3/5Q1p/PPPBBPPP/R3K2R b KQkq - 0 2".into(),
        );
        let white_king_analysis = analyze_king_position(
            board.white_king_position,
            false,
            board.occupancy,
            board.white_occupancy,
            board.black_occupancy,
            board.pawn_bitboard,
            board.knight_bitboard,
            board.bishop_bitboard,
            board.rook_bitboard,
            board.queen_bitboard,
            board.black_turn,
        );

        let black_king_analysis = analyze_king_position(
            board.black_king_position,
            true,
            board.occupancy,
            board.black_occupancy,
            board.white_occupancy,
            board.pawn_bitboard,
            board.knight_bitboard,
            board.bishop_bitboard,
            board.rook_bitboard,
            board.queen_bitboard,
            !board.black_turn,
        );
        let moves = generate_moves(&black_king_analysis, &white_king_analysis, board);

        assert_eq!(moves.0.len(), 4);
    }

    #[test]
    pub fn move_generation_capture_the_threat_with_bishop_or_move_the_king() {
        let board = BoardRep::from_fen(
            "r3kb1r/p1Npqp2/1b2pnp1/n2PN3/1p2P3/5Q1p/PPPBBPPP/R3K2R b KQkq - 0 2".into(),
        );
        let white_king_analysis = analyze_king_position(
            board.white_king_position,
            false,
            board.occupancy,
            board.white_occupancy,
            board.black_occupancy,
            board.pawn_bitboard,
            board.knight_bitboard,
            board.bishop_bitboard,
            board.rook_bitboard,
            board.queen_bitboard,
            board.black_turn,
        );

        let black_king_analysis = analyze_king_position(
            board.black_king_position,
            true,
            board.occupancy,
            board.black_occupancy,
            board.white_occupancy,
            board.pawn_bitboard,
            board.knight_bitboard,
            board.bishop_bitboard,
            board.rook_bitboard,
            board.queen_bitboard,
            !board.black_turn,
        );
        let moves = generate_moves(&black_king_analysis, &white_king_analysis, board);

        assert_eq!(moves.0.len(), 2);
    }

    #[test]
    pub fn move_generation_capture_the_threat_with_rook_to_avoid_smother() {
        let board = BoardRep::from_fen(
            "3nkb1r/p1Npqp2/4pnp1/1b1PN3/1p2P3/5Q1p/PPrBBPPP/R3K2R b KQk - 0 2".into(),
        );
        let white_king_analysis = analyze_king_position(
            board.white_king_position,
            false,
            board.occupancy,
            board.white_occupancy,
            board.black_occupancy,
            board.pawn_bitboard,
            board.knight_bitboard,
            board.bishop_bitboard,
            board.rook_bitboard,
            board.queen_bitboard,
            board.black_turn,
        );

        let black_king_analysis = analyze_king_position(
            board.black_king_position,
            true,
            board.occupancy,
            board.black_occupancy,
            board.white_occupancy,
            board.pawn_bitboard,
            board.knight_bitboard,
            board.bishop_bitboard,
            board.rook_bitboard,
            board.queen_bitboard,
            !board.black_turn,
        );
        let moves = generate_moves(&black_king_analysis, &white_king_analysis, board);

        assert_eq!(moves.0.len(), 1);
    }

    #[test]
    pub fn move_generation_block_threat_with_bishop() {
        let board = BoardRep::from_fen(
            "r3kb2/pp3ppp/2n2n1r/1Bpp4/4b3/2N1PP2/PPPP2PP/R1B1q1KR w q - 0 11".into(),
        );
        let white_king_analysis = analyze_king_position(
            board.white_king_position,
            false,
            board.occupancy,
            board.white_occupancy,
            board.black_occupancy,
            board.pawn_bitboard,
            board.knight_bitboard,
            board.bishop_bitboard,
            board.rook_bitboard,
            board.queen_bitboard,
            board.black_turn,
        );

        let black_king_analysis = analyze_king_position(
            board.black_king_position,
            true,
            board.occupancy,
            board.black_occupancy,
            board.white_occupancy,
            board.pawn_bitboard,
            board.knight_bitboard,
            board.bishop_bitboard,
            board.rook_bitboard,
            board.queen_bitboard,
            !board.black_turn,
        );
        let moves = generate_moves(&white_king_analysis, &black_king_analysis, board);

        assert_eq!(moves.0.len(), 1);
    }

    #[test]
    pub fn move_generation_capture_the_threat_with_pawn_to_avoid_smother() {
        let board = BoardRep::from_fen(
            "3nkb1r/p1pbnp2/3Np1p1/q3N3/1p2P3/2q2Q1p/PPPBBPPP/R3K2R b KQk - 0 2".into(),
        );
        let white_king_analysis = analyze_king_position(
            board.white_king_position,
            false,
            board.occupancy,
            board.white_occupancy,
            board.black_occupancy,
            board.pawn_bitboard,
            board.knight_bitboard,
            board.bishop_bitboard,
            board.rook_bitboard,
            board.queen_bitboard,
            board.black_turn,
        );

        let black_king_analysis = analyze_king_position(
            board.black_king_position,
            true,
            board.occupancy,
            board.black_occupancy,
            board.white_occupancy,
            board.pawn_bitboard,
            board.knight_bitboard,
            board.bishop_bitboard,
            board.rook_bitboard,
            board.queen_bitboard,
            !board.black_turn,
        );
        let moves = generate_moves(&black_king_analysis, &white_king_analysis, board);

        assert_eq!(moves.0.len(), 1);
    }

    #[test]
    pub fn move_generation_block_with_pawn_or_move_king() {
        let board = BoardRep::from_fen(
            "r3kb2/pp3ppp/2n2n1r/1Bpp4/3qb3/2N2P2/PPPPP1PP/R1B3K1 w q - 0 11".into(),
        );
        let white_king_analysis = analyze_king_position(
            board.white_king_position,
            false,
            board.occupancy,
            board.white_occupancy,
            board.black_occupancy,
            board.pawn_bitboard,
            board.knight_bitboard,
            board.bishop_bitboard,
            board.rook_bitboard,
            board.queen_bitboard,
            board.black_turn,
        );

        let black_king_analysis = analyze_king_position(
            board.black_king_position,
            true,
            board.occupancy,
            board.black_occupancy,
            board.white_occupancy,
            board.pawn_bitboard,
            board.knight_bitboard,
            board.bishop_bitboard,
            board.rook_bitboard,
            board.queen_bitboard,
            !board.black_turn,
        );
        let moves = generate_moves(&white_king_analysis, &black_king_analysis, board);
        assert_eq!(moves.0.len(), 3);
    }

    #[test]
    pub fn move_generation_block_or_capture_with_bishop() {
        let board = BoardRep::from_fen(
            "r3k2Q/p1ppqpb1/bn2pn2/3PN1p1/1p2P3/2N5/PPPBBPPP/R3K2R b KQq - 0 2".into(),
        );
        let white_king_analysis = analyze_king_position(
            board.white_king_position,
            false,
            board.occupancy,
            board.white_occupancy,
            board.black_occupancy,
            board.pawn_bitboard,
            board.knight_bitboard,
            board.bishop_bitboard,
            board.rook_bitboard,
            board.queen_bitboard,
            board.black_turn,
        );

        let black_king_analysis = analyze_king_position(
            board.black_king_position,
            true,
            board.occupancy,
            board.black_occupancy,
            board.white_occupancy,
            board.pawn_bitboard,
            board.knight_bitboard,
            board.bishop_bitboard,
            board.rook_bitboard,
            board.queen_bitboard,
            !board.black_turn,
        );

        let moves = generate_moves(&black_king_analysis, &white_king_analysis, board);

        assert_eq!(moves.0.len(), 4);
    }

    #[test]
    pub fn pawn_move_gen_threatened_block_with_double_pawn_push() {
        let board = BoardRep::from_fen(
            "r2q1rk1/pP1p2pp/Q4n2/bbp1p3/Np6/1B3NBn/pPPP1PPP/R3K2R b KQ - 0 1".into(),
        );

        let moves = generate_index_moves(
            board,
            index_from_coords("d7"),
            &board.get_black_king_analysis(),
        );
        println!("{:?}", moves);
        assert_eq!(moves.len(), 1);
    }

    #[test]
    pub fn pawn_move_gen_threatened_take_ep() {
        let board = BoardRep::from_fen("8/8/8/1Ppp3r/1KR2p1k/8/4P1P1/8 w - c6 0 3".into());

        let moves = generate_index_moves(
            board,
            index_from_coords("b5"),
            &board.get_white_king_analysis(),
        );
        println!("{:?}", moves);
        assert_eq!(moves.len(), 1);
    }

    #[test]
    pub fn pawn_move_gen_threatened_take_threat() {
        let board = BoardRep::from_fen("8/2p5/3p4/KPR3kr/5p2/8/4P1P1/8 b - - 3 2".into());

        let moves = generate_index_moves(
            board,
            index_from_coords("d6"),
            &board.get_black_king_analysis(),
        );
        println!("{:?}", moves);
        assert_eq!(moves.len(), 2);
    }

    #[test]
    pub fn move_generation_scenario_pawn_wrap_around_king_threat() {
        let board = BoardRep::from_fen(
            "r4rk1/p1ppqpb1/bn2pnp1/P2PN3/1p2P3/2N2Q1p/1PPBBPPP/R3K2R b KQ - 0 2".into(),
        );

        let white_king_analysis = analyze_king_position(
            board.white_king_position,
            false,
            board.occupancy,
            board.white_occupancy,
            board.black_occupancy,
            board.pawn_bitboard,
            board.knight_bitboard,
            board.bishop_bitboard,
            board.rook_bitboard,
            board.queen_bitboard,
            board.black_turn,
        );

        let black_king_analysis = analyze_king_position(
            board.black_king_position,
            true,
            board.occupancy,
            board.black_occupancy,
            board.white_occupancy,
            board.pawn_bitboard,
            board.knight_bitboard,
            board.bishop_bitboard,
            board.rook_bitboard,
            board.queen_bitboard,
            !board.black_turn,
        );

        let moves = generate_moves(&black_king_analysis, &white_king_analysis, board);
        assert!(moves.0.contains(&Move::new(
            index_from_coords("g8"),
            index_from_coords("h7"),
            0b0,
            piece_type::PieceType::King,
            true,
            0
        )));
    }

    #[test]
    pub fn generate_moves_queen_check_bishop_can_capture() {
        let board = BoardRep::from_fen(
            "rnbqkbnr/pp4pp/2p1Qp2/3pp3/2B1P2P/8/PPPP1PP1/RNB1K1NR b KQkq - 1 5".into(),
        );

        let moves = generate_moves_for_board(board);
        assert_eq!(moves.len(), 4);
        assert!(moves.contains(&Move::new(
            index_from_coords("c8"),
            index_from_coords("e6"),
            MF_CAPTURE,
            piece_type::PieceType::Bishop,
            true,
            calculate_see(PieceType::Bishop, PieceType::Queen)
        )));
    }

    #[test]
    pub fn get_pawn_threatboard_no_wrap_around() {
        let r = get_pawn_threatboard(index_from_coords("a5"), false);
        println!("{}", r.to_board_format());
        assert_eq!(r, 1 << index_from_coords("b6"));
    }

    #[test]
    pub fn generate_pawn_moves_when_pinned_bishop() {
        let board = BoardRep::from_fen(
            "rnbqkbnr/pppppp2/8/1B4pp/4P3/2N5/PPPP1PPP/R1BQK1NR b KQkq -".into(),
        );

        let pawn_moves = generate_index_moves(
            board,
            index_from_coords("d7"),
            &board.get_black_king_analysis(),
        );
        assert_eq!(pawn_moves.len(), 0);
    }

    #[test]
    pub fn generate_pawn_moves_when_pinned_bishop_that_can_be_captured() {
        let board = BoardRep::from_fen(
            "rnbqkbnr/pppppp2/2B5/6pp/4P3/2N5/PPPP1PPP/R1BQK1NR b KQkq - 0 1".into(),
        );

        let pawn_moves = generate_index_moves(
            board,
            index_from_coords("d7"),
            &board.get_black_king_analysis(),
        );
        assert_eq!(pawn_moves.len(), 1);
        assert_eq!(pawn_moves[0].to(), index_from_coords("c6"));
    }

    #[test]
    pub fn generate_pawn_moves_when_pinned_on_e_file() {
        let board = BoardRep::from_fen(
            "rnbqkbnr/pppppp2/2B5/6pp/4P3/2N5/PPPP1PPP/R1BQK1NR b KQkq - 0 1".into(),
        );

        let pawn_moves = generate_index_moves(
            board,
            index_from_coords("e7"),
            &board.get_black_king_analysis(),
        );
        assert_eq!(pawn_moves.len(), 2);
    }

    #[test]
    pub fn generate_pawn_moves_pinned_and_threatened_where_threat_captureable_no_legal_moves() {
        let board = BoardRep::from_fen(
            "rnbqkbnr/1ppp1p1p/p3Q3/1B2p1p1/4P3/8/PPPP1PPP/RNB1K1NR b KQkq - 0 1".into(),
        );

        let pawn_moves = generate_index_moves(
            board,
            index_from_coords("d7"),
            &board.get_black_king_analysis(),
        );
        assert_eq!(pawn_moves.len(), 0);
    }

    #[test]
    pub fn generate_pawn_moves_pinned_and_right_ep_capture_available_disallow_the_capture() {
        let board = BoardRep::from_fen("8/2p5/3p4/KP5r/1R3pPk/8/4P3/8 b - g3".into());
        let pawn_moves = generate_index_moves(
            board,
            index_from_coords("f4"),
            &board.get_black_king_analysis(),
        );
        println!("{pawn_moves:?}");
        assert_eq!(pawn_moves.len(), 1);
    }

    #[test]
    pub fn generate_pawn_moves_pinned_and_left_ep_capture_available_disallow_the_capture() {
        let board = BoardRep::from_fen("8/2p5/3p4/KP5r/1R2Pp1k/8/6P1/8 b - e3 0 1".into());
        let pawn_moves = generate_index_moves(
            board,
            index_from_coords("f4"),
            &board.get_black_king_analysis(),
        );
        println!("{pawn_moves:?}");
        assert_eq!(pawn_moves.len(), 1);
    }

    #[test]
    pub fn generate_knight_moves_when_king_threatened_can_not_block_or_capture() {
        let board =
            BoardRep::from_fen("rnbq1bnr/pppppkpp/8/5p1Q/4P2P/8/PPPP1PP1/RNB1KBNR b KQ -".into());

        let knight_moves = generate_index_moves(
            board,
            index_from_coords("g8"),
            &board.get_black_king_analysis(),
        );
        assert_eq!(knight_moves.len(), 0);
    }

    #[test]
    pub fn generate_knight_moves_when_king_threatened_can_block_or_capture() {
        let board = BoardRep::from_fen(
            "rnbq1b1r/pppppkpp/8/5p1Q/4Pn1P/8/PPPP1PP1/RNB1KBNR b KQ - 0 1".into(),
        );

        let knight_moves = generate_index_moves(
            board,
            index_from_coords("f4"),
            &board.get_black_king_analysis(),
        );
        assert_eq!(knight_moves.len(), 2);
    }

    #[test]
    pub fn generate_knight_moves_when_king_threatened_can_only_block() {
        let board = BoardRep::from_fen(
            "rnbq1b1r/pppppkpp/8/5pn1/4P2P/1Q6/PPPP1PP1/RNB1KBNR b KQ - 0 1".into(),
        );

        let knight_moves = generate_index_moves(
            board,
            index_from_coords("g5"),
            &board.get_black_king_analysis(),
        );
        assert_eq!(knight_moves.len(), 1);
    }

    #[test]
    pub fn generate_knight_moves_when_knight_pinned_no_moves() {
        let board = BoardRep::from_fen(
            "rnbq1b1r/pppppkpp/8/3n1p2/4P2P/1Q6/PPPP1PP1/RNB1KBNR b KQ - 0 1".into(),
        );

        let moves = generate_index_moves(
            board,
            index_from_coords("d5"),
            &board.get_black_king_analysis(),
        );
        println!("{moves:?}");
        assert_eq!(moves.len(), 0);
    }

    #[test]
    pub fn generate_bishop_moves_when_bishop_pinned_diagonally_should_include_capture_and_full_ray()
    {
        let board = BoardRep::from_fen(
            "rnbq3r/pppppkpp/4b3/5p2/4P2P/1Q6/PPPP1PP1/RNB1KBNR b KQ - 0 1".into(),
        );

        let moves = generate_index_moves(
            board,
            index_from_coords("e6"),
            &board.get_black_king_analysis(),
        );
        println!("{moves:?}");
        assert_eq!(moves.len(), 3);
    }

    #[test]
    pub fn generate_bishop_moves_when_bishop_pinned_diagonally_should_include_capture_and_full_ray_including_retreat(
    ) {
        let board = BoardRep::from_fen(
            "rnbq2kr/ppppp1pp/8/3b1p2/4P2P/1Q6/PPPP1PP1/RNB1KBNR b KQ - 0 1".into(),
        );

        let moves = generate_index_moves(
            board,
            index_from_coords("d5"),
            &board.get_black_king_analysis(),
        );
        println!("{moves:?}");
        assert_eq!(moves.len(), 4);
    }

    #[test]
    pub fn generate_bishop_moves_when_bishop_pinned_orthogonally_should_return_0() {
        let board = BoardRep::from_fen(
            "rnbq3r/pppppkpp/8/5b2/1Q2P2P/5R2/PPPP1PP1/RNB1KBN1 b Q - 0 1".into(),
        );

        let moves = generate_index_moves(
            board,
            index_from_coords("f5"),
            &board.get_black_king_analysis(),
        );
        println!("{moves:?}");
        assert_eq!(moves.len(), 0);
    }

    #[test]
    pub fn generate_rook_moves_when_rook_pinned_orthogonally_should_include_capture_and_full_ray_including_retreat(
    ) {
        let board = BoardRep::from_fen(
            "rnbqk3/pppp2pp/3b3p/4rp2/7P/4Q3/PPPP1PP1/RNB1KBNR b KQ - 0 1".into(),
        );

        let moves = generate_index_moves(
            board,
            index_from_coords("e5"),
            &board.get_black_king_analysis(),
        );
        println!("{moves:?}");
        assert_eq!(moves.len(), 4);
    }

    #[test]
    pub fn generate_rook_moves_when_rook_pinned_diagonally_should_return_none() {
        let board = BoardRep::from_fen(
            "rnbqk3/pppp1rpp/3b3p/5p1Q/7P/8/PPPP1PP1/RNB1KBNR b KQ - 0 1".into(),
        );

        let moves = generate_index_moves(
            board,
            index_from_coords("f7"),
            &board.get_black_king_analysis(),
        );
        println!("{moves:?}");
        assert_eq!(moves.len(), 0);
    }

    #[test]
    pub fn generate_queen_moves_when_pinned_orthogonally() {
        let board = BoardRep::from_fen(
            "rnbqk3/ppppr1pp/3b3p/5p2/7P/4Q3/PPPP1PP1/RNB1KBNR w KQ - 0 1".into(),
        );

        let moves = generate_index_moves(
            board,
            index_from_coords("e3"),
            &&board.get_white_king_analysis(),
        );
        println!("{moves:?}");
        assert_eq!(moves.len(), 5);
    }

    #[test]
    pub fn generate_queen_moves_when_pinned_and_king_threatened_by_capturable_piece() {
        let board = BoardRep::from_fen(
            "rnb1kbnr/1ppq1p1p/p3Q3/1B2p1p1/4P3/8/PPPP1PPP/RNB1K1NR b KQkq - 0 1".into(),
        );

        let moves = generate_index_moves(
            board,
            index_from_coords("d2"),
            &board.get_black_king_analysis(),
        );
        println!("{moves:?}");
        assert_eq!(moves.len(), 0);
    }

    #[test]
    pub fn generate_queen_moves_when_pinned_orthogonally_and_forked_with_knight() {
        let board = BoardRep::from_fen(
            "rnbqk3/ppppr1pp/3b3p/5p2/7P/4Q3/PPPP1Pn1/RNB1KBNR w KQ - 0 1".into(),
        );

        let moves = generate_index_moves(
            board,
            index_from_coords("e3"),
            &&board.get_white_king_analysis(),
        );
        println!("{moves:?}");
        assert_eq!(moves.len(), 0);
    }

    #[test]
    pub fn generate_queen_moves_when_pinned_diagonally() {
        let board =
            BoardRep::from_fen("rnbqk3/ppppr1pp/3b3p/5p2/5Q1P/8/PPPP1P1K/RNB2BNR w - - 0 1".into());

        let moves = generate_index_moves(
            board,
            index_from_coords("f4"),
            &&board.get_white_king_analysis(),
        );
        println!("{moves:?}");
        assert_eq!(moves.len(), 3);
    }

    #[test]
    pub fn generate_moves_king_in_check_no_blockers_only_retreat_to_f3() {
        let board =
            BoardRep::from_fen("1nb1kbnr/pp1rpppp/8/2p5/4PP2/8/PPPqK1PP/R4BNR w k - 0 1".into());

        let moves = generate_moves_for_board(board);
        assert_eq!(moves.len(), 1);
        assert_eq!(moves[0].to(), index_from_coords("f3"));
    }

    #[test]
    pub fn ep_leads_to_orthogonal_check_right_true() {
        let board = BoardRep::from_fen("8/2p5/3p4/KP5r/1R3pPk/8/4P3/8 b - g3 0 1".into());
        let leads_to_check = ep_leads_to_orthogonal_check(
            board,
            index_from_coords("f4"),
            index_from_coords("f4") - 1,
            board.white_occupancy,
        );
        assert!(leads_to_check);
    }

    #[test]
    pub fn ep_leads_to_orthogonal_check_left_true() {
        let board = BoardRep::from_fen("8/2p5/3p4/KP5r/1R2Pp1k/8/6P1/8 b - e3 0 1".into());
        let leads_to_check = ep_leads_to_orthogonal_check(
            board,
            index_from_coords("f4"),
            index_from_coords("f4") + 1,
            board.white_occupancy,
        );
        assert!(leads_to_check);
    }

    // #[test]
    // pub fn generate_knight_moves_e4() {
    //     let position = Position::new("k7/8/8/8/4N3/8/8/7K".into());
    //     let moves = generate_knight_moves(position, index_from_coords("e4"), false);
    //     assert_eq!(moves.len(), 8);
    // }

    // #[test]
    // pub fn generate_knight_moves_g7_capture_on_f5() {
    //     let position = Position::new("k7/6N1/8/5p2/8/8/8/7K".into());
    //     let moves = generate_knight_moves(position, index_from_coords("g7"), false);

    //     assert_eq!(moves.len(), 4);
    //     let capture_move = Move::new(
    //         index_from_coords("g7"),
    //         index_from_coords("f5"),
    //         MF_CAPTURE,
    //         PieceType::Knight,
    //     );
    //     assert!(moves.contains(&capture_move))
    // }
}
