use log::info;

use crate::{
    board::{
        bitboard::Bitboard,
        board_rep::BoardRep,
        king_position_analysis::{KingPositionAnalysis, ThreatSource, ThreatType},
    },
    shared::{
        board_utils::get_direction_to_normalized,
        constants::{
            MF_BISHOP_CAPTURE_PROMOTION, MF_BISHOP_PROMOTION, MF_CAPTURE, MF_DOUBLE_PAWN_PUSH,
            MF_EP_CAPTURE, MF_KING_CASTLING, MF_KNIGHT_CAPTURE_PROMOTION, MF_KNIGHT_PROMOTION,
            MF_QUEEN_CAPTURE_PROMOTION, MF_QUEEN_CASTLING, MF_QUEEN_PROMOTION,
            MF_ROOK_CAPTURE_PROMOTION, MF_ROOK_PROMOTION,
        },
        piece_type,
    },
    MOVE_DATA,
};

use super::{
    move_data::{
        BLACK_PAWN_PROMOTION_RANK, KING_CASTLING_CHECK, KING_CASTLING_CLEARANCE,
        QUEEN_CASTLING_CHECK, QUEEN_CASTLING_CLEARANCE, WHITE_PAWN_PROMOTION_RANK,
    },
    Move,
};

type MoveList = [Move; 128];

pub fn generate_moves(king_analysis: &KingPositionAnalysis, board: BoardRep) -> MoveList {
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

    let threat_board = generate_threat_board(!board.black_turn, opponent_occupancy, board);

    let mut moves = generate_king_moves(
        king_pos,
        opponent_occupancy,
        board.occupancy,
        king_analysis.check,
        board.black_turn,
        king_side_castling,
        queen_side_castling,
        threat_board,
        king_analysis.threat_source,
    );

    // In the event of double king check we can only avoid check by moving the king
    if king_analysis.double_check {
        return to_move_list(moves);
    }

    while friendly_occupancy != 0 {
        let piece_position = friendly_occupancy.trailing_zeros() as u8;
        moves.extend(generate_position_moves(
            board,
            piece_position,
            board.black_turn,
            board.ep_index,
        ));
        friendly_occupancy ^= 1 << piece_position;
    }

    to_move_list(moves)
}

fn generate_position_moves(board: BoardRep, index: u8, is_black: bool, ep_index: u8) -> Vec<Move> {
    let piece_type = board.get_piece_type_at_index(index);
    let opponent_occupancy = if is_black {
        board.white_occupancy
    } else {
        board.black_occupancy
    };

    match piece_type {
        piece_type::PieceType::Pawn => {
            generate_pawn_moves(board, index, is_black, ep_index, opponent_occupancy)
        }
        piece_type::PieceType::Knight => {
            generate_knight_moves(index, opponent_occupancy, board.occupancy, is_black)
        }
        piece_type::PieceType::Bishop => {
            generate_bishop_moves(index, board, opponent_occupancy, board.occupancy, is_black)
        }
        piece_type::PieceType::Rook => {
            generate_rook_moves(index, board, opponent_occupancy, board.occupancy, is_black)
        }
        piece_type::PieceType::Queen => {
            generate_queen_moves(index, board, opponent_occupancy, board.occupancy, is_black)
        }
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

fn get_pawn_threatboard(piece_position: u8, is_black: bool) -> u64 {
    if is_black {
        let mut r = if piece_position > 8 {
            1 << piece_position - 9
        } else {
            0
        };
        if piece_position > 7 {
            r |= 1 << piece_position - 7
        };

        return r;
    } else {
        let mut r = if piece_position < 54 {
            1 << piece_position + 9
        } else {
            0
        };

        if piece_position < 55 {
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
    match threat {
        Some(threat) => {
            if threat.threat_type == ThreatType::DiagonalSlide
                || threat.threat_type == ThreatType::OrthogonalSlide
            {
                // TODO: use the known threat to reduce illegal move gen
                // let threat_normal = get_direction_to_normalized(index, threat.from);
                // moveboard ^= 1 << (index as i8) + threat_normal;
            }
        }
        None => {}
    }
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
) -> Vec<Move> {
    let moveboard = MOVE_DATA
        .magic_bitboard_table
        .get_bishop_attacks(index as usize, board.occupancy.into())
        | MOVE_DATA
            .magic_bitboard_table
            .get_rook_attacks(index as usize, board.occupancy.into());
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
) -> Vec<Move> {
    let moveboard = MOVE_DATA
        .magic_bitboard_table
        .get_rook_attacks(index as usize, board.occupancy.into());
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
) -> Vec<Move> {
    let moveboard = MOVE_DATA
        .magic_bitboard_table
        .get_bishop_attacks(index as usize, board.occupancy.into());
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
) -> Vec<Move> {
    moveboard_to_moves(
        index,
        piece_type::PieceType::Knight,
        MOVE_DATA.knight_moves[index as usize],
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
) -> Vec<Move> {
    let mut moves = Vec::new();
    let mut moveboard = if is_black {
        MOVE_DATA.black_pawn_moves[index as usize]
    } else {
        MOVE_DATA.white_pawn_moves[index as usize]
    };

    // Black and whites move u64s have different orientations that we need to parse out. For white its fairly simple, for black we need to see if a
    // double pawn push is possible or not before determining what the normal move to_index is
    let (to_index, to_index_dpp, promotion_rank) = if is_black {
        let to_index_dpp = moveboard.trailing_zeros() as u8;
        moveboard ^= 1 << to_index_dpp;
        let to_index = moveboard.trailing_zeros() as u8;
        if to_index == 64 {
            (to_index_dpp, 64, BLACK_PAWN_PROMOTION_RANK)
        } else {
            (to_index, to_index_dpp, BLACK_PAWN_PROMOTION_RANK)
        }
    } else {
        let to_index = moveboard.trailing_zeros() as u8;
        moveboard ^= 1 << to_index;
        let to_index_dpp = moveboard.trailing_zeros() as u8;
        (to_index, to_index_dpp, WHITE_PAWN_PROMOTION_RANK)
    };

    // Can we move one square
    if !board.occupancy.occupied(to_index) {
        // Does moving that one square lead to a promotion?
        if (1 << to_index) & promotion_rank != 0 {
            moves.extend(generate_pawn_promotion_moves(
                index, to_index, false, is_black,
            ));
        } else {
            moves.push(Move::new(
                index,
                to_index,
                0b0,
                piece_type::PieceType::Pawn,
                is_black,
            ));

            // Can we move a second square in a Double Pawn Push?
            if to_index_dpp != 64 {
                if !board.occupancy.occupied(to_index_dpp) {
                    moves.push(Move::new(
                        index,
                        to_index_dpp,
                        MF_DOUBLE_PAWN_PUSH,
                        piece_type::PieceType::Pawn,
                        is_black,
                    ));
                }
            }
        }
    }

    let mut capture_board = if is_black {
        MOVE_DATA.black_pawn_captures[index as usize]
    } else {
        MOVE_DATA.white_pawn_captures[index as usize]
    };

    // Can we capture right or EP capture right
    let first_capture_index = capture_board.trailing_zeros() as u8;
    if opponent_occupancy.occupied(first_capture_index) || first_capture_index == ep_index {
        // Does capturing right lead to a promotion?
        if (1 << first_capture_index) & promotion_rank != 0 {
            moves.extend(generate_pawn_promotion_moves(
                index,
                first_capture_index,
                true,
                is_black,
            ))
        } else {
            moves.push(Move::new(
                index,
                first_capture_index,
                if first_capture_index == ep_index {
                    MF_EP_CAPTURE
                } else {
                    MF_CAPTURE
                },
                piece_type::PieceType::Pawn,
                is_black,
            ));
        }
    }

    // Can we capture left or EP capture left
    capture_board ^= 1 << first_capture_index;
    let second_capture_index = capture_board.trailing_zeros() as u8;
    if second_capture_index != 64 {
        if opponent_occupancy.occupied(second_capture_index) || second_capture_index == ep_index {
            // Does capturing left lead to a promotion?
            if (1 << second_capture_index) & promotion_rank != 0 {
                moves.extend(generate_pawn_promotion_moves(
                    index,
                    second_capture_index,
                    true,
                    is_black,
                ))
            } else {
                moves.push(Move::new(
                    index,
                    second_capture_index,
                    if second_capture_index == ep_index {
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
    let mut to_index = 0;
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

fn to_move_list(vec: Vec<Move>) -> MoveList {
    let mut v = vec.clone();
    v.sort();
    if v.len() > 128 {
        info!("Throwing away {} moves", v.len() - 128)
    }
    let mut r: MoveList = [Move::default(); 128];
    for i in 0..v.len() {
        if i >= 128 {
            break;
        }
        r[i] = v[i];
    }
    r
}

#[cfg(test)]
mod test {

    use crate::board::king_position_analysis::analyze_king_position;

    use super::*;

    #[test]
    pub fn startpos_move_generation() {
        let board = BoardRep::default();
        let king_analysis = analyze_king_position(
            board.white_king_position,
            board.black_turn,
            board.occupancy,
            board.white_occupancy,
            board.black_occupancy,
            board.pawn_bitboard,
            board.knight_bitboard,
            board.bishop_bitboard,
            board.rook_bitboard,
            board.queen_bitboard,
        );
        let moves = generate_moves(&king_analysis, board);
        assert_eq!(moves.into_iter().position(|m| m.is_empty()).unwrap(), 20);
    }

    #[test]
    pub fn king_double_checked() {
        let board = BoardRep::from_fen(
            "rnbqk1nr/pppp1pNp/2Pb4/8/1B6/4Q3/PP1PPPPP/RN2KB1R b KQkq - 0 1".into(),
        );
        let king_analysis = analyze_king_position(
            board.black_king_position,
            board.black_turn,
            board.occupancy,
            board.black_occupancy,
            board.white_occupancy,
            board.pawn_bitboard,
            board.knight_bitboard,
            board.bishop_bitboard,
            board.rook_bitboard,
            board.queen_bitboard,
        );
        let moves: [Move; 128] = generate_moves(&king_analysis, board);
        assert!(moves.into_iter().position(|m| m.is_empty()).unwrap() <= 2);
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
