use log::info;

use crate::{
    board::{
        bitboard::Bitboard,
        board_rep::BoardRep,
        king_position_analysis::{KingPositionAnalysis, ThreatSource, ThreatType, ThreatRaycastCollision},
    },
    shared::{
        board_utils::{get_direction_to_normalized, get_file, get_rank},
        constants::{
            MF_BISHOP_CAPTURE_PROMOTION, MF_BISHOP_PROMOTION, MF_CAPTURE, MF_DOUBLE_PAWN_PUSH,
            MF_EP_CAPTURE, MF_KING_CASTLING, MF_KNIGHT_CAPTURE_PROMOTION, MF_KNIGHT_PROMOTION,
            MF_QUEEN_CAPTURE_PROMOTION, MF_QUEEN_CASTLING, MF_QUEEN_PROMOTION,
            MF_ROOK_CAPTURE_PROMOTION, MF_ROOK_PROMOTION,
        },
        piece_type::{self, PieceType},
    },
    MOVE_DATA,
};

use super::{
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
    pub black_pinned: Vec<ThreatRaycastCollision>
}

pub fn generate_moves(king_analysis: &KingPositionAnalysis, opponent_king_analysis: &KingPositionAnalysis, board: BoardRep) -> (Vec<Move>, MoveGenerationEvalMetrics) {
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
    );

    // In the event of double king check we can only avoid check by moving the king
    if king_analysis.double_check {
        let metrics =  build_move_generation_eval_metrics(board.black_turn, king_analysis, opponent_king_analysis, friendly_threat_board, opponent_threat_board);
        return (moves,metrics);
    }

    while friendly_occupancy != 0 {
        let piece_position = friendly_occupancy.trailing_zeros() as u8;
        moves.extend(generate_position_moves(
            board,
            piece_position,
            board.black_turn,
            board.ep_index,
            king_analysis.threat_source,
        ));
        friendly_occupancy ^= 1 << piece_position;
    }

    let metrics =  build_move_generation_eval_metrics(board.black_turn, king_analysis, opponent_king_analysis, friendly_threat_board, opponent_threat_board);
    (moves,metrics)
}

fn generate_position_moves(
    board: BoardRep,
    index: u8,
    is_black: bool,
    ep_index: u8,
    king_threat: Option<ThreatSource>,
) -> Vec<Move> {
    let piece_type = board.get_piece_type_at_index(index);
    let opponent_occupancy = if is_black {
        board.white_occupancy
    } else {
        board.black_occupancy
    };

    match piece_type {
        piece_type::PieceType::Pawn => generate_pawn_moves(
            board,
            index,
            is_black,
            ep_index,
            opponent_occupancy,
            king_threat,
        ),
        piece_type::PieceType::Knight => generate_knight_moves(
            index,
            opponent_occupancy,
            board.occupancy,
            is_black,
            king_threat,
        ),
        piece_type::PieceType::Bishop => generate_bishop_moves(
            index,
            board,
            opponent_occupancy,
            board.occupancy,
            is_black,
            king_threat,
        ),
        piece_type::PieceType::Rook => generate_rook_moves(
            index,
            board,
            opponent_occupancy,
            board.occupancy,
            is_black,
            king_threat,
        ),
        piece_type::PieceType::Queen => generate_queen_moves(
            index,
            board,
            opponent_occupancy,
            board.occupancy,
            is_black,
            king_threat,
        ),
        piece_type::PieceType::King => Vec::new(),
        _ => panic!(
            "Unexpected piece {piece_type:?} at position {index} : {}",
            board.to_fen()
        ),
    }
}

fn generate_threat_board(is_black: bool, mut piece_occupancy: u64, board: BoardRep) -> u64 {
    let mut r: u64 = 0;
    while piece_occupancy != 0 {
        let piece_position = piece_occupancy.trailing_zeros() as u8;
        r |= match board.get_piece_type_at_index(piece_position) {
            piece_type::PieceType::Pawn => get_pawn_threatboard(piece_position, is_black),
            piece_type::PieceType::Knight => MOVE_DATA.knight_moves[piece_position as usize],
            piece_type::PieceType::Bishop => MOVE_DATA
                .magic_bitboard_table
                .get_bishop_attacks(piece_position as usize, board.occupancy.into()),
            piece_type::PieceType::Rook => MOVE_DATA
                .magic_bitboard_table
                .get_rook_attacks(piece_position as usize, board.occupancy.into()),
            piece_type::PieceType::Queen => {
                MOVE_DATA
                    .magic_bitboard_table
                    .get_bishop_attacks(piece_position as usize, board.occupancy.into())
                    | MOVE_DATA
                        .magic_bitboard_table
                        .get_rook_attacks(piece_position as usize, board.occupancy.into())
            }
            piece_type::PieceType::King => MOVE_DATA.king_moves[piece_position as usize],
            _ => 0,
        };

        piece_occupancy ^= 1 << piece_position;
    }
    r
}

fn build_move_generation_eval_metrics(is_black: bool, friendly_king_analysis: &KingPositionAnalysis, opponent_king_analysis: &KingPositionAnalysis, friendly_threatboard: u64, opponent_threatboard: u64) -> MoveGenerationEvalMetrics {
    if is_black {
        MoveGenerationEvalMetrics {
            white_threatboard: opponent_threatboard,
            black_threatboard: friendly_threatboard,
            black_pinned: friendly_king_analysis.pins.clone(),
            white_pinned: opponent_king_analysis.pins.clone()
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
) -> Vec<Move> {
    let mut moveboard = MOVE_DATA.king_moves[index as usize];
    moveboard = moveboard & !threat_board;
    let mut moves = moveboard_to_moves(
        index,
        piece_type::PieceType::King,
        moveboard,
        opponent_occupancy,
        occupancy,
        is_black,
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

fn generate_queen_moves(
    index: u8,
    board: BoardRep,
    opponent_occupancy: u64,
    occupancy: u64,
    is_black: bool,
    king_threat: Option<ThreatSource>,
) -> Vec<Move> {
    let mut moveboard = MOVE_DATA
        .magic_bitboard_table
        .get_bishop_attacks(index as usize, board.occupancy.into())
        | MOVE_DATA
            .magic_bitboard_table
            .get_rook_attacks(index as usize, board.occupancy.into());

    if king_threat != None {
        let threat = king_threat.unwrap();
        moveboard &= threat.threat_path_mask | (1 << threat.from);
    }

    moveboard_to_moves(
        index,
        piece_type::PieceType::Queen,
        moveboard,
        opponent_occupancy,
        occupancy,
        is_black,
    )
}

fn generate_rook_moves(
    index: u8,
    board: BoardRep,
    opponent_occupancy: u64,
    occupancy: u64,
    is_black: bool,
    king_threat: Option<ThreatSource>,
) -> Vec<Move> {
    let mut moveboard = MOVE_DATA
        .magic_bitboard_table
        .get_rook_attacks(index as usize, board.occupancy.into());

    if king_threat != None {
        let threat = king_threat.unwrap();
        moveboard &= threat.threat_path_mask | (1 << threat.from);
    }

    moveboard_to_moves(
        index,
        piece_type::PieceType::Rook,
        moveboard,
        opponent_occupancy,
        occupancy,
        is_black,
    )
}
fn generate_bishop_moves(
    index: u8,
    board: BoardRep,
    opponent_occupancy: u64,
    occupancy: u64,
    is_black: bool,
    king_threat: Option<ThreatSource>,
) -> Vec<Move> {
    let mut moveboard = MOVE_DATA
        .magic_bitboard_table
        .get_bishop_attacks(index as usize, board.occupancy.into());

    if king_threat != None {
        let threat = king_threat.unwrap();
        moveboard &= threat.threat_path_mask | (1 << threat.from);
    }

    moveboard_to_moves(
        index,
        piece_type::PieceType::Bishop,
        moveboard,
        opponent_occupancy,
        occupancy,
        is_black,
    )
}

fn generate_knight_moves(
    index: u8,
    opponent_occupancy: u64,
    occupancy: u64,
    is_black: bool,
    king_threat: Option<ThreatSource>,
) -> Vec<Move> {
    let mut moveboard = MOVE_DATA.knight_moves[index as usize];

    if king_threat != None {
        let threat = king_threat.unwrap();
        moveboard &= threat.threat_path_mask | (1 << threat.from);
    }

    moveboard_to_moves(
        index,
        piece_type::PieceType::Knight,
        moveboard,
        opponent_occupancy,
        occupancy,
        is_black,
    )
}

fn generate_pawn_moves(
    board: BoardRep,
    index: u8,
    is_black: bool,
    ep_index: u8,
    opponent_occupancy: u64,
    king_threat: Option<ThreatSource>,
) -> Vec<Move> {
    if king_threat != None {
        return generate_pawn_moves_when_threatened(
            index,
            is_black,
            ep_index,
            king_threat.unwrap(),
            board.occupancy,
        );
    }

    let mut moves = Vec::new();
    let offset_file: i8 = if is_black { -1 } else { 1 };
    let rank = get_rank(index);

    let to = (index as i8 + (8 * offset_file)) as u8;
    if !board.occupancy.occupied(to) {
        if get_rank(to) != 0 && get_rank(to) != 7 {
            moves.push(Move::new(
                index,
                to,
                0b0,
                piece_type::PieceType::Pawn,
                is_black,
            ));

            if (is_black && rank == 6) || (!is_black && rank == 1) {
                let dpp = (to as i8 + (8 * offset_file)) as u8;
                if !board.occupancy.occupied(dpp) {
                    moves.push(Move::new(
                        index,
                        dpp,
                        MF_DOUBLE_PAWN_PUSH,
                        piece_type::PieceType::Pawn,
                        is_black,
                    ));
                }
            }
        } else {
            moves.extend(generate_pawn_promotion_moves(index, to, false, is_black));
        }
    }

    let capture_a = (index as i8 + (offset_file * 8)) as u8 + 1;
    let capture_a_rank = get_rank(capture_a);
    if (rank as i8 + offset_file) as u8 == capture_a_rank
        && (capture_a == ep_index || opponent_occupancy.occupied(capture_a))
    {
        if capture_a_rank == 0 || capture_a_rank == 7 {
            moves.extend(generate_pawn_promotion_moves(
                index, capture_a, true, is_black,
            ));
        } else {
            moves.push(Move::new(
                index,
                capture_a,
                if capture_a == ep_index {
                    MF_EP_CAPTURE
                } else {
                    MF_CAPTURE
                },
                piece_type::PieceType::Pawn,
                is_black,
            ));
        }
    }

    if index > 8 {
        let capture_b = (index as i8 + (offset_file * 8)) as u8 - 1;
        let capture_b_rank = get_rank(capture_b);
        if (rank as i8 + offset_file) as u8 == capture_b_rank
            && (capture_b == ep_index || opponent_occupancy.occupied(capture_b))
        {
            if capture_b_rank == 0 || capture_b_rank == 7 {
                moves.extend(generate_pawn_promotion_moves(
                    index, capture_b, true, is_black,
                ));
            } else {
                moves.push(Move::new(
                    index,
                    capture_b,
                    if capture_b == ep_index {
                        MF_EP_CAPTURE
                    } else {
                        MF_CAPTURE
                    },
                    piece_type::PieceType::Pawn,
                    is_black,
                ));
            }
        }
    }

    moves
}

fn generate_pawn_moves_when_threatened(
    index: u8,
    is_black: bool,
    ep_index: u8,
    threat: ThreatSource,
    occupancy: u64,
) -> Vec<Move> {
    let mut moves = Vec::new();
    let offset_file: i8 = if is_black { -1 } else { 1 };
    let rank = get_rank(index);

    if threat.threat_path_mask != 0 {
        let to: u8 = (index as i8 + (8 * offset_file)) as u8;
        if (1 << to) & threat.threat_path_mask != 0 {
            if get_rank(to) != 0 && get_rank(to) != 7 {
                moves.push(Move::new(
                    index,
                    to,
                    0b0,
                    piece_type::PieceType::Pawn,
                    is_black,
                ));
            } else {
                moves.extend(generate_pawn_promotion_moves(index, to, false, is_black));
            }
        }

        if !occupancy.occupied(to) && ((is_black && rank == 6) || (!is_black && rank == 1)) {
            let dpp = (to as i8 + (8 * offset_file)) as u8;
            if (1 << dpp) & threat.threat_path_mask != 0 {
                moves.push(Move::new(
                    index,
                    dpp,
                    MF_DOUBLE_PAWN_PUSH,
                    piece_type::PieceType::Pawn,
                    is_black,
                ));
            }
        }
    }

    let capture_a = (index as i8 + (offset_file * 8)) as u8 + 1;
    if threat.from == capture_a || (capture_a == ep_index && threat.threat_type == ThreatType::Pawn)
    {
        let capture_a_rank = get_rank(capture_a);
        if (rank as i8 + offset_file) as u8 == capture_a_rank {
            if capture_a_rank == 0 || capture_a_rank == 7 {
                moves.extend(generate_pawn_promotion_moves(
                    index, capture_a, true, is_black,
                ));
            } else {
                moves.push(Move::new(
                    index,
                    capture_a,
                    if capture_a == ep_index {
                        MF_EP_CAPTURE
                    } else {
                        MF_CAPTURE
                    },
                    piece_type::PieceType::Pawn,
                    is_black,
                ));
            }
        }
    }
    if index > 8 {
        let capture_b = (index as i8 + (offset_file * 8)) as u8 - 1;
        if threat.from == capture_b
            || (capture_b == ep_index && threat.threat_type == ThreatType::Pawn)
        {
            let capture_b_rank = get_rank(capture_b);
            if (rank as i8 + offset_file) as u8 == capture_b_rank {
                if capture_b_rank == 0 || capture_b_rank == 7 {
                    moves.extend(generate_pawn_promotion_moves(
                        index, capture_b, true, is_black,
                    ));
                } else {
                    moves.push(Move::new(
                        index,
                        capture_b,
                        if capture_b == ep_index {
                            MF_EP_CAPTURE
                        } else {
                            MF_CAPTURE
                        },
                        piece_type::PieceType::Pawn,
                        is_black,
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
    is_black: bool,
) -> Vec<Move> {
    let mut generated_moves = Vec::new();
    let mut m_b = moveboard;
    while m_b != 0 {
        let lsb = m_b.trailing_zeros() as u8;
        if opponent_occupancy.occupied(lsb) {
            generated_moves.push(Move::new(from_index, lsb, MF_CAPTURE, piece_type, is_black));
        } else if !occupancy.occupied(lsb) {
            generated_moves.push(Move::new(from_index, lsb, 0b0, piece_type, is_black));
        };
        m_b ^= 1 << lsb;
    }

    generated_moves
}

#[cfg(test)]
mod test {

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
            board.black_turn
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
            !board.black_turn
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
            board.black_turn
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
            !board.black_turn
        );
        let moves = generate_moves(&black_king_analysis, &white_king_analysis, board);
        assert!(moves.0.len() <= 2);
    }

    #[test]
    pub fn pawn_moves_scenario_0() {
        let board = BoardRep::from_fen(
            "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1".into(),
        );
        let moves = generate_pawn_moves(board, 9, false, u8::MAX, board.black_occupancy, None);
        assert_eq!(moves.len(), 3);
    }

    #[test]
    pub fn pawn_moves_scenario_1() {
        let board = BoardRep::from_fen(
            "r3k2r/p1ppqpb1/bn2pnp1/1N1PN3/1p2P3/5Q2/PPPBBPpP/R3K2R w KQkq - 0 2".into(),
        );
        let moves = generate_pawn_moves(board, 9, true, u8::MAX, board.white_occupancy, None);
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
            board.black_turn
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
            !board.black_turn
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
            board.black_turn
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
            !board.black_turn
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
            board.black_turn
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
            !board.black_turn
        );
        let moves = generate_moves(&black_king_analysis,&white_king_analysis, board);

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
            board.black_turn
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
            !board.black_turn
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
            board.black_turn
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
            !board.black_turn
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
            board.black_turn
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
            !board.black_turn
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
            board.black_turn
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
            !board.black_turn
        );

        let moves = generate_moves(&black_king_analysis, &white_king_analysis, board);

        assert_eq!(moves.0.len(), 5); // Should be 4 once we can stop retreating kings (king moving back along the line they're being checked from)
    }

    #[test]
    pub fn pawn_move_gen_threatened_block_with_double_pawn_push() {
        let board = BoardRep::from_fen(
            "r2q1rk1/pP1p2pp/Q4n2/bbp1p3/Np6/1B3NBn/pPPP1PPP/R3K2R b KQ - 0 1".into(),
        );
        let king_position_analysis = analyze_king_position(
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
            false
        );

        let moves = generate_pawn_moves_when_threatened(
            index_from_coords("d7"),
            true,
            u8::MAX,
            king_position_analysis.threat_source.unwrap(),
            board.occupancy,
        );
        println!("{:?}", moves);
        assert_eq!(moves.len(), 1);
    }

    #[test]
    pub fn pawn_move_gen_threatened_take_ep() {
        let board = BoardRep::from_fen("8/8/8/1Ppp3r/1KR2p1k/8/4P1P1/8 w - c6 0 3".into());
        let king_position_analysis = analyze_king_position(
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
            false
        );

        let moves = generate_pawn_moves_when_threatened(
            index_from_coords("b5"),
            false,
            board.ep_index,
            king_position_analysis.threat_source.unwrap(),
            board.occupancy,
        );
        println!("{:?}", moves);
        assert_eq!(moves.len(), 1);
    }

    #[test]
    pub fn pawn_move_gen_threatened_take_threat() {
        let board = BoardRep::from_fen("8/2p5/3p4/KPR3kr/5p2/8/4P1P1/8 b - - 3 2".into());
        let king_position_analysis = analyze_king_position(
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
            false
        );

        let moves = generate_pawn_moves_when_threatened(
            index_from_coords("d6"),
            true,
            board.ep_index,
            king_position_analysis.threat_source.unwrap(),
            board.occupancy,
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
            board.black_turn
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
            !board.black_turn
        );

        let moves = generate_moves(&black_king_analysis, &white_king_analysis, board);
        assert!(moves.0.contains(&Move::new(
            index_from_coords("g8"),
            index_from_coords("h7"),
            0b0,
            piece_type::PieceType::King,
            true
        )));
    }

    #[test]
    pub fn get_pawn_threatboard_no_wrap_around() {
        let r = get_pawn_threatboard(index_from_coords("a5"), false);
        println!("{}", r.to_board_format());
        assert_eq!(r, 1 << index_from_coords("b6"));
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
