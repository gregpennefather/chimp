use log::info;

use crate::{
    board::{
        bitboard::Bitboard,
        board_rep::BoardRep,
        king_position_analysis::{KingPositionAnalysis, ThreatRaycastCollision, ThreatType},
    },
    r#move::{calculate_see, Move},
    shared::{
        board_utils::get_file,
        constants::MF_CAPTURE,
        piece_type::{self},
    },
    MOVE_DATA,
};

mod king;
mod knight;
mod pawn;
pub(crate) mod sliding;
mod tests;

#[derive(Clone, Default)]
pub struct MoveGenerationEvalMetrics {
    pub white_threatboard: u64,
    pub black_threatboard: u64,
    pub white_pinned: Vec<ThreatRaycastCollision>,
    pub black_pinned: Vec<ThreatRaycastCollision>,
}

fn generate_moves(
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

    let mut moves = king::generate_king_moves(
        king_pos,
        opponent_occupancy,
        board.occupancy,
        king_analysis.check,
        board.black_turn,
        king_side_castling,
        queen_side_castling,
        opponent_threat_board,
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
        piece_type::PieceType::Pawn => pawn::generate_pawn_moves(
            board,
            index,
            opponent_occupancy,
            king_analysis.threat_source,
            pin,
        ),
        piece_type::PieceType::Knight => match pin {
            Some(_) => vec![],
            None => knight::generate_knight_moves(
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
            sliding::bishop::generate_bishop_moves(
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
            sliding::rook::generate_rook_moves(
                index,
                board,
                opponent_occupancy,
                board.occupancy,
                king_analysis.threat_source,
                pin,
            )
        }
        piece_type::PieceType::Queen => sliding::queen::generate_queen_moves(
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
