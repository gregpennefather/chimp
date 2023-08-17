use crate::{
    board::{
        board_utils::{get_friendly_name_for_index, get_piece_from_position_index, board_to_string},
        move_utils::get_move_uci,
    },
    shared::{
        bitboard_to_string, BISHOP_CAPTURE_PROMOTION, BISHOP_INDEX, BISHOP_PROMOTION, BLACK_KING,
        BLACK_MASK, BLACK_PAWN, CAPTURE_FLAG, EP_CAPTURE_FLAG, KING_CASTLING_FLAG, KING_INDEX,
        KNIGHT_CAPTURE_PROMOTION, KNIGHT_INDEX, KNIGHT_PROMOTION, PAWN_INDEX, PIECE_MASK,
        QUEEN_CAPTURE_PROMOTION, QUEEN_CASTLING_FLAG, QUEEN_INDEX, QUEEN_PROMOTION,
        ROOK_CAPTURE_PROMOTION, ROOK_INDEX, ROOK_PROMOTION, DOUBLE_PAWN_FLAG,
    },
};

use super::{
    bitboard::BitboardExtensions,
    board_metrics::BoardMetrics,
    board_utils::{
        get_file, get_position_index_from_piece_index, get_rank, get_rank_i8,
        rank_and_file_to_index, rank_from_char,
    },
    move_utils::build_move,
    piece_utils::{get_piece_code, is_white},
    state::BoardState,
};

impl BoardState {
    pub fn generate_psudolegals(&self) -> BoardMetrics {
        let mut moves = Vec::new();
        let white_turn = self.flags & 0b1 > 0;
        let mut friendly_threat_bitboard = 0;
        let mut opponent_threat_bitboard = 0;
        let opponent_bitboard = if white_turn {
            self.black_bitboard
        } else {
            self.white_bitboard
        };

        let mut friendly_piece_index_pairs: [(u8, u8); 32] = [(u8::MAX, u8::MAX); 32];
        let mut friendly_piece_count = 0;
        let mut opponent_piece_index_pairs: [(u8, u8); 32] = [(u8::MAX, u8::MAX); 32];
        let mut opponent_piece_count = 0;
        let mut piece_index = 0;
        while piece_index < self.piece_count {
            let position_index =
                get_position_index_from_piece_index(self.bitboard, 0, 0, piece_index);
            let piece = get_piece_code(&self.pieces, piece_index);
            if is_white(piece) == white_turn {
                friendly_piece_index_pairs[friendly_piece_count] = (position_index, piece);
                friendly_piece_count += 1;
            } else {
                opponent_piece_index_pairs[opponent_piece_count] = (position_index, piece);
                opponent_piece_count += 1;
            }
            piece_index += 1;
        }

        for pair in opponent_piece_index_pairs {
            opponent_threat_bitboard |=
                generate_threat_bitboard(self.bitboard, pair.0, pair.1, !white_turn)
        }

        for pair in friendly_piece_index_pairs {
            let (p_threat_bitboard, p_moves) = generate_piece_moves(
                self.bitboard,
                white_turn,
                pair.0,
                pair.1,
                opponent_bitboard,
                self.flags,
                self.ep_rank,
                opponent_threat_bitboard,
            );
            friendly_threat_bitboard |= p_threat_bitboard;
            moves.extend(&p_moves);
        }

        let (white_threat_bitboard, black_threat_bitboard) = if white_turn {
            (friendly_threat_bitboard, opponent_threat_bitboard)
        } else {
            (opponent_threat_bitboard, friendly_threat_bitboard)
        };

        BoardMetrics {
            psudolegal_moves: moves,
            white_threat_bitboard,
            black_threat_bitboard,
        }
    }

    pub fn generate_legal_moves(&self, metrics: BoardMetrics) -> Vec<(u16, BoardState,BoardMetrics)> {
        let white_turn = self.flags & 0b1 > 0;
        let mut moves = Vec::new();
        for psudolegal_move in metrics.psudolegal_moves {
            let new_state = self.apply_move(psudolegal_move);
            let new_metrics = new_state.generate_psudolegals();
            if psudolegal_move == 9246 {
                println!("{}", get_move_uci(psudolegal_move));
                println!("{} vs {}", self.piece_count, new_state.piece_count);
                println!("{}", board_to_string(self.bitboard, self.pieces));
                println!("{}", board_to_string(new_state.bitboard, new_state.pieces));
            }
            if white_turn {
                if !new_metrics
                    .black_threat_bitboard
                    .occupied(new_state.white_king_index)
                {
                    moves.push((psudolegal_move, new_state, new_metrics));
                }
            } else {
                if !new_metrics
                    .white_threat_bitboard
                    .occupied(new_state.black_king_index)
                {
                    moves.push((psudolegal_move, new_state, new_metrics));
                }
            }
        }
        moves
    }

    pub fn move_from_string(&self, move_string: &str) -> u16 {
        let from_rank_char = move_string.chars().nth(0).unwrap();
        let from_rank = rank_from_char(from_rank_char);
        let from_file: u8 = (move_string.chars().nth(1).unwrap().to_digit(16).unwrap() - 1) as u8;

        let from_index = rank_and_file_to_index(from_rank, from_file);

        let to_rank_char = move_string.chars().nth(2).unwrap();
        let to_rank = rank_from_char(to_rank_char);
        let to_file: u8 = (move_string.chars().nth(3).unwrap().to_digit(16).unwrap() - 1) as u8;

        let to_index = rank_and_file_to_index(to_rank, to_file);

        let mut flags = 0;

        if self.bitboard.occupied(to_index) {
            flags = CAPTURE_FLAG;
        }

        let piece = get_piece_from_position_index(self.bitboard, self.pieces, from_index);
        match piece {
            PAWN_INDEX | BLACK_PAWN => {
                if piece == PAWN_INDEX && from_file == 1 && to_file == 3 {
                    flags = DOUBLE_PAWN_FLAG;
                } else if piece == BLACK_PAWN && from_file == 6 && to_file == 4 {
                    flags = DOUBLE_PAWN_FLAG;
                } else if self.ep_rank == to_rank && to_rank != from_rank && ((piece == PAWN_INDEX && to_file == 5) || (piece == BLACK_PAWN && to_file == 2)) {
                    flags = EP_CAPTURE_FLAG;
                } else {
                    let promotion_char = move_string.chars().nth(4);
                    match promotion_char {
                        Some(p) => {
                            flags = match p {
                                'n' => if flags != CAPTURE_FLAG {KNIGHT_PROMOTION} else {KNIGHT_CAPTURE_PROMOTION},
                                'b' => if flags != CAPTURE_FLAG {BISHOP_PROMOTION} else {BISHOP_CAPTURE_PROMOTION},
                                'r' => if flags != CAPTURE_FLAG {ROOK_PROMOTION} else {ROOK_CAPTURE_PROMOTION},
                                'q' => if flags != CAPTURE_FLAG {QUEEN_PROMOTION} else {QUEEN_CAPTURE_PROMOTION},
                                _ => 0
                            }
                        },
                        None => {}
                    }
                }
            }
            KING_INDEX | BLACK_KING => {
                if from_rank == 4 {
                    if to_rank == 2 {
                        flags = QUEEN_CASTLING_FLAG
                    } else if to_rank == 6 {
                        flags = KING_CASTLING_FLAG
                    }
                }
            }
            _ => {}
        }

        build_move(from_index, to_index, flags)
    }
}

fn generate_piece_moves(
    bitboard: u64,
    is_white: bool,
    position_index: u8,
    piece: u8,
    opponent_bitboard: u64,
    flags: u8,
    ep_rank: u8,
    opponent_threat_bitboard: u64,
) -> (u64, Vec<u16>) {
    let piece_code = piece & PIECE_MASK;
    let castling_flags = if is_white {
        flags >> 1 & 0b11
    } else {
        (flags >> 3) & 0b11
    };
    match piece_code {
        PAWN_INDEX => generate_pawn_moves(
            bitboard,
            is_white,
            opponent_bitboard,
            position_index,
            ep_rank,
        ),
        KNIGHT_INDEX => generate_knight_moves(bitboard, opponent_bitboard, position_index),
        BISHOP_INDEX => generate_bishop_moves(bitboard, opponent_bitboard, position_index),
        ROOK_INDEX => generate_rook_moves(bitboard, opponent_bitboard, position_index),
        QUEEN_INDEX => generate_queen_moves(bitboard, opponent_bitboard, position_index),
        KING_INDEX => generate_king_moves(
            bitboard,
            opponent_bitboard,
            position_index,
            castling_flags,
            opponent_threat_bitboard,
        ),
        _ => (0, vec![]),
    }
}

fn generate_threat_bitboard(bitboard: u64, position_index: u8, piece: u8, is_white: bool) -> u64 {
    let piece_code = piece & PIECE_MASK;
    match piece_code {
        PAWN_INDEX => generate_pawn_threat_board(is_white, position_index),
        KNIGHT_INDEX => generate_knight_threat_board(position_index),
        BISHOP_INDEX => generate_bishop_threat_board(bitboard, position_index),
        ROOK_INDEX => generate_rook_threat_board(bitboard, position_index),
        QUEEN_INDEX => generate_queen_threat_board(bitboard, position_index),
        KING_INDEX => generate_king_threat_board(bitboard, position_index),
        _ => 0,
    }
}

fn generate_pawn_moves(
    bitboard: u64,
    is_white: bool,
    opponent_bitboard: u64,
    position_index: u8,
    ep_rank: u8,
) -> (u64, Vec<u16>) {
    let mut results: Vec<_> = Vec::new();
    let mut threat_bitboard: u64 = 0;
    let file = get_file(position_index);
    let rank = get_rank(position_index);
    let is_ep = ep_rank != u8::MAX;

    if is_white {
        if !bitboard.occupied(position_index + 8) {
            // Promotion
            if file == 6 {
                results.extend(build_promotion_moves(
                    position_index,
                    position_index + 8,
                    false,
                ));
            } else {
                // Move
                results.push(build_move(position_index, position_index + 8, 0b0));
                if file == 1 {
                    if !bitboard.occupied(position_index + 16) {
                        results.push(build_move(position_index, position_index + 16, DOUBLE_PAWN_FLAG));
                    }
                }
            }
        }

        // Capture Right
        if rank != 7 {
            threat_bitboard |= 1 << (position_index + 7);
            if opponent_bitboard.occupied(position_index + 7) {
                // Promotion
                if file == 6 {
                    results.extend(build_promotion_moves(
                        position_index,
                        position_index + 7,
                        true,
                    ));
                } else {
                    // Double push
                    results.push(build_move(position_index, position_index + 7, CAPTURE_FLAG));
                }
            }
        }

        // Capture left
        if rank != 0 {
            threat_bitboard |= 1 << (position_index + 9);
            if opponent_bitboard.occupied(position_index + 9) {
                // Promotion
                if file == 6 {
                    results.extend(build_promotion_moves(
                        position_index,
                        position_index + 9,
                        true,
                    ));
                } else {
                    results.push(build_move(position_index, position_index + 9, CAPTURE_FLAG));
                }
            }
        }

        if is_ep && file == 4 {
            if rank != 0 && ep_rank == rank - 1 {
                results.push(build_move(
                    position_index,
                    position_index + 9,
                    EP_CAPTURE_FLAG,
                ));
            } else if rank != 7 && ep_rank == rank + 1 {
                results.push(build_move(
                    position_index,
                    position_index + 7,
                    EP_CAPTURE_FLAG,
                ));
            }
        }
    } else {
        if !bitboard.occupied(position_index - 8) {
            // Promotion
            if file == 1 {
                results.extend(build_promotion_moves(
                    position_index,
                    position_index - 8,
                    false,
                ));
            } else {
                // Move
                results.push(build_move(position_index, position_index - 8, 0b0));

                // Double push
                if file == 6 {
                    if !bitboard.occupied(position_index - 16) {
                        results.push(build_move(position_index, position_index - 16, DOUBLE_PAWN_FLAG));
                    }
                }
            }
        }

        // Capture right
        if rank != 7 {
            threat_bitboard |= 1 << (position_index - 9);
            if opponent_bitboard.occupied(position_index - 9) {
                // Promotion
                if file == 1 {
                    results.extend(build_promotion_moves(
                        position_index,
                        position_index - 9,
                        true,
                    ));
                } else {
                    results.push(build_move(position_index, position_index - 9, CAPTURE_FLAG));
                }
            }
        }

        // Capture left
        if rank != 0 {
            threat_bitboard |= 1 << (position_index - 7);
            if opponent_bitboard.occupied(position_index - 7) {
                // Promotion
                if file == 1 {
                    results.extend(build_promotion_moves(
                        position_index,
                        position_index - 7,
                        true,
                    ));
                } else {
                    results.push(build_move(position_index, position_index - 7, CAPTURE_FLAG));
                }
            }
        }

        if is_ep && file == 3 {
            if rank != 0 && ep_rank == rank - 1 {
                results.push(build_move(
                    position_index,
                    position_index - 7,
                    EP_CAPTURE_FLAG,
                ));
            } else if rank != 7 && ep_rank == rank + 1 {
                results.push(build_move(
                    position_index,
                    position_index - 9,
                    EP_CAPTURE_FLAG,
                ));
            }
        }
    }
    (threat_bitboard, results)
}

fn build_promotion_moves(from_index: u8, to_index: u8, capture: bool) -> Vec<u16> {
    return vec![
        build_move(
            from_index,
            to_index,
            if !capture {
                KNIGHT_PROMOTION
            } else {
                KNIGHT_CAPTURE_PROMOTION
            },
        ), // Knight
        build_move(
            from_index,
            to_index,
            if !capture {
                BISHOP_PROMOTION
            } else {
                BISHOP_CAPTURE_PROMOTION
            },
        ), // Bishop
        build_move(
            from_index,
            to_index,
            if !capture {
                ROOK_PROMOTION
            } else {
                ROOK_CAPTURE_PROMOTION
            },
        ), // Rook
        build_move(
            from_index,
            to_index,
            if !capture {
                QUEEN_PROMOTION
            } else {
                QUEEN_CAPTURE_PROMOTION
            },
        ), // Queen
    ];
}

fn generate_pawn_threat_board(is_white: bool, position_index: u8) -> u64 {
    let mut threat_bitboard: u64 = 0;
    let rank = get_rank(position_index);
    if is_white {
        if rank != 7 {
            threat_bitboard |= 1 << (position_index + 7);
        }

        if rank != 0 {
            threat_bitboard |= 1 << (position_index + 9);
        }
    } else {
        if rank != 7 {
            threat_bitboard |= 1 << (position_index - 9);
        }

        if rank != 0 {
            threat_bitboard |= 1 << (position_index - 7);
        }
    }
    threat_bitboard
}

fn generate_knight_moves(
    bitboard: u64,
    opponent_bitboard: u64,
    position_index: u8,
) -> (u64, Vec<u16>) {
    let mut results: Vec<_> = Vec::new();
    let rank = get_rank(position_index);
    let mut threat_bitboard: u64 = 0;
    // U2R1 = +16-1 = 15
    if position_index <= 48 && rank != 7 {
        let tar = position_index + 15;
        threat_bitboard = threat_bitboard | (1 << tar);
        if opponent_bitboard.occupied(tar) {
            results.push(build_move(position_index, tar, CAPTURE_FLAG));
        } else if !bitboard.occupied(tar) {
            results.push(build_move(position_index, tar, 0b0));
        }
    }
    // U1R2 = +8-2 = 6
    if position_index <= 55 && rank < 6 {
        let tar = position_index + 6;
        threat_bitboard = threat_bitboard | (1 << tar);
        if opponent_bitboard.occupied(tar) {
            results.push(build_move(position_index, tar, CAPTURE_FLAG));
        } else if !bitboard.occupied(tar) {
            results.push(build_move(position_index, tar, 0b0));
        }
    }
    // D1R2 = -8-2 = -10
    if position_index >= 10 && rank < 6 {
        let tar = position_index - 10;
        threat_bitboard = threat_bitboard | (1 << tar);
        if opponent_bitboard.occupied(tar) {
            results.push(build_move(position_index, tar, CAPTURE_FLAG));
        } else if !bitboard.occupied(tar) {
            results.push(build_move(position_index, tar, 0b0));
        }
    }
    // D2R1 = -16-1 = -17
    if position_index >= 17 && rank != 7 {
        let tar = position_index - 17;
        threat_bitboard = threat_bitboard | (1 << tar);
        if opponent_bitboard.occupied(tar) {
            results.push(build_move(position_index, tar, CAPTURE_FLAG));
        } else if !bitboard.occupied(tar) {
            results.push(build_move(position_index, tar, 0b0));
        }
    }
    // D2L1 = -16+1 = -15
    if position_index >= 15 && rank != 0 {
        let tar = position_index - 15;
        threat_bitboard = threat_bitboard | (1 << tar);
        if opponent_bitboard.occupied(tar) {
            results.push(build_move(position_index, tar, CAPTURE_FLAG));
        } else if !bitboard.occupied(tar) {
            results.push(build_move(position_index, tar, 0b0));
        }
    }
    // D1L2 = -8+2 = -6
    if position_index >= 6 && rank > 1 {
        let tar = position_index - 6;
        threat_bitboard = threat_bitboard | (1 << tar);
        if opponent_bitboard.occupied(tar) {
            results.push(build_move(position_index, tar, CAPTURE_FLAG));
        } else if !bitboard.occupied(tar) {
            results.push(build_move(position_index, tar, 0b0));
        }
    }
    // U1L2 = 8+2 = 10
    if position_index <= 53 && rank > 1 {
        let tar = position_index + 10;
        threat_bitboard = threat_bitboard | (1 << tar);
        if opponent_bitboard.occupied(tar) {
            results.push(build_move(position_index, tar, CAPTURE_FLAG));
        } else if !bitboard.occupied(tar) {
            results.push(build_move(position_index, tar, 0b0));
        }
    }
    // U2L1 = 16+1 = 17
    if position_index <= 46 && rank != 0 {
        let tar = position_index + 17;
        threat_bitboard = threat_bitboard | (1 << tar);
        if opponent_bitboard.occupied(tar) {
            results.push(build_move(position_index, tar, CAPTURE_FLAG));
        } else if !bitboard.occupied(tar) {
            results.push(build_move(position_index, tar, 0b0));
        }
    }
    (threat_bitboard, results)
}

fn generate_knight_threat_board(position_index: u8) -> u64 {
    let mut threat_bitboard: u64 = 0;
    let rank = get_rank(position_index);

    // U2R1 = +16-1 = 15
    if position_index <= 48 && rank != 0 {
        let tar = position_index + 15;
        threat_bitboard = threat_bitboard | (1 << tar);
    }
    // U1R2 = +8-2 = 6
    if position_index <= 55 && rank > 1 {
        let tar = position_index + 6;
        threat_bitboard = threat_bitboard | (1 << tar);
    }
    // D1R2 = -8-2 = -10
    if position_index >= 10 && rank > 1 {
        let tar = position_index - 10;
        threat_bitboard = threat_bitboard | (1 << tar);
    }
    // D2R1 = -16-1 = -17
    if position_index >= 17 && rank != 0 {
        let tar = position_index - 17;
        threat_bitboard = threat_bitboard | (1 << tar);
    }
    // D2L1 = -16+1 = -15
    if position_index >= 15 && rank != 7 {
        let tar = position_index - 15;
        threat_bitboard = threat_bitboard | (1 << tar);
    }
    // D1L2 = -8+2 = -6
    if position_index >= 6 && rank < 6 {
        let tar = position_index - 6;
        threat_bitboard = threat_bitboard | (1 << tar);
    }
    // U1L2 = 8+2 = 10
    if position_index <= 53 && rank < 6 {
        let tar = position_index + 10;
        threat_bitboard = threat_bitboard | (1 << tar);
    }
    // U2L1 = 16+1 = 17
    if position_index <= 46 && rank != 7 {
        let tar = position_index + 17;
        threat_bitboard = threat_bitboard | (1 << tar);
    }
    threat_bitboard
}

fn generate_bishop_moves(
    bitboard: u64,
    opponent_bitboard: u64,
    position_index: u8,
) -> (u64, Vec<u16>) {
    sliding_move_generator(
        bitboard,
        opponent_bitboard,
        position_index,
        true,
        false,
        false,
    )
}

fn generate_bishop_threat_board(bitboard: u64, position_index: u8) -> u64 {
    sliding_threat_generator(bitboard, position_index, true, false, false)
}

fn generate_rook_moves(
    bitboard: u64,
    opponent_bitboard: u64,
    position_index: u8,
) -> (u64, Vec<u16>) {
    sliding_move_generator(
        bitboard,
        opponent_bitboard,
        position_index,
        false,
        true,
        false,
    )
}

fn generate_rook_threat_board(bitboard: u64, position_index: u8) -> u64 {
    sliding_threat_generator(bitboard, position_index, false, true, false)
}

fn generate_queen_moves(
    bitboard: u64,
    opponent_bitboard: u64,
    position_index: u8,
) -> (u64, Vec<u16>) {
    sliding_move_generator(
        bitboard,
        opponent_bitboard,
        position_index,
        true,
        true,
        false,
    )
}

fn generate_queen_threat_board(bitboard: u64, position_index: u8) -> u64 {
    sliding_threat_generator(bitboard, position_index, true, true, false)
}

fn generate_king_moves(
    bitboard: u64,
    opponent_bitboard: u64,
    position_index: u8,
    castling_flags: u8,
    opponent_threat_bitboard: u64,
) -> (u64, Vec<u16>) {
    let (threat_board, mut moves) = sliding_move_generator(
        bitboard,
        opponent_bitboard,
        position_index,
        true,
        true,
        true,
    );

    if (castling_flags & 0b01) > 0 {
        // King side castling
        if (!bitboard.occupied(position_index - 1) && !bitboard.occupied(position_index - 2))
            && (!opponent_threat_bitboard.occupied(position_index)
                && !opponent_threat_bitboard.occupied(position_index - 1)
                && !opponent_threat_bitboard.occupied(position_index - 2))
        {
            moves.push(build_move(
                position_index,
                position_index - 2,
                KING_CASTLING_FLAG,
            ))
        }
    }
    if (castling_flags & 0b10) > 0 {
        // Queen side castling

        // Multiple square checking could probably be done more efficiently with a | flag and a greater than check
        if !bitboard.occupied(position_index + 1)
            && !bitboard.occupied(position_index + 2)
            && !bitboard.occupied(position_index + 3)
            && !opponent_threat_bitboard.occupied(position_index)
            && !opponent_threat_bitboard.occupied(position_index + 1)
            && !opponent_threat_bitboard.occupied(position_index + 2)
        {
            moves.push(build_move(
                position_index,
                position_index + 2,
                QUEEN_CASTLING_FLAG,
            ))
        }
    }
    (threat_board, moves)
}

fn generate_king_threat_board(bitboard: u64, position_index: u8) -> u64 {
    sliding_threat_generator(bitboard, position_index, true, true, true)
}

fn sliding_move_generator(
    bitboard: u64,
    opponent_bitboard: u64,
    pos: u8,
    diag: bool,
    straight: bool,
    king: bool,
) -> (u64, Vec<u16>) {
    let mut moves: Vec<u16> = Vec::new();
    let mut threat_bitboard = 0;

    let depth = if king { 1 } else { 8 };

    if diag {
        let positions_d_l = get_available_slide_pos(bitboard, pos, -1, -1, depth);

        for i in 0..positions_d_l.len() {
            threat_bitboard |= 1 << positions_d_l[i];
            if (i == positions_d_l.len() - 1) && bitboard.occupied(positions_d_l[i]) {
                if opponent_bitboard.occupied(positions_d_l[i]) {
                    moves.push(build_move(pos, positions_d_l[i], CAPTURE_FLAG));
                }
                break;
            }
            moves.push(build_move(pos, positions_d_l[i], 0b0));
        }

        let positions_u_l = get_available_slide_pos(bitboard, pos, 1, -1, depth);

        for i in 0..positions_u_l.len() {
            threat_bitboard |= 1 << positions_u_l[i];
            if (i == positions_u_l.len() - 1) && bitboard.occupied(positions_u_l[i]) {
                if opponent_bitboard.occupied(positions_u_l[i]) {
                    moves.push(build_move(pos, positions_u_l[i], CAPTURE_FLAG));
                }
                break;
            }
            moves.push(build_move(pos, positions_u_l[i], 0b0));
        }

        let positions_u_r = get_available_slide_pos(bitboard, pos, 1, 1, depth);
        for i in 0..positions_u_r.len() {
            threat_bitboard |= 1 << positions_u_r[i];
            if (i == positions_u_r.len() - 1) && bitboard.occupied(positions_u_r[i]) {
                if opponent_bitboard.occupied(positions_u_r[i]) {
                    moves.push(build_move(pos, positions_u_r[i], CAPTURE_FLAG));
                }
                break;
            }
            moves.push(build_move(pos, positions_u_r[i], 0b0));
        }

        let positions_d_r = get_available_slide_pos(bitboard, pos, -1, 1, depth);

        for i in 0..positions_d_r.len() {
            threat_bitboard |= 1 << positions_d_r[i];
            if (i == positions_d_r.len() - 1) && bitboard.occupied(positions_d_r[i]) {
                if opponent_bitboard.occupied(positions_d_r[i]) {
                    moves.push(build_move(pos, positions_d_r[i], CAPTURE_FLAG));
                }
                break;
            }
            moves.push(build_move(pos, positions_d_r[i], 0b0));
        }
    }

    if straight {
        let positions_r = get_available_slide_pos(bitboard, pos, 0, 1, depth);

        for i in 0..positions_r.len() {
            threat_bitboard |= 1 << positions_r[i];
            if (i == positions_r.len() - 1) && bitboard.occupied(positions_r[i]) {
                if opponent_bitboard.occupied(positions_r[i]) {
                    moves.push(build_move(pos, positions_r[i], CAPTURE_FLAG));
                }
                break;
            }
            moves.push(build_move(pos, positions_r[i], 0b0));
        }

        let positions_l = get_available_slide_pos(bitboard, pos, 0, -1, depth);

        for i in 0..positions_l.len() {
            threat_bitboard |= 1 << positions_l[i];
            if (i == positions_l.len() - 1) && bitboard.occupied(positions_l[i]) {
                if opponent_bitboard.occupied(positions_l[i]) {
                    moves.push(build_move(pos, positions_l[i], CAPTURE_FLAG));
                }
                break;
            }
            moves.push(build_move(pos, positions_l[i], 0b0));
        }

        let positions_u = get_available_slide_pos(bitboard, pos, 1, 0, depth);

        for i in 0..positions_u.len() {
            threat_bitboard |= 1 << positions_u[i];
            if (i == positions_u.len() - 1) && bitboard.occupied(positions_u[i]) {
                if opponent_bitboard.occupied(positions_u[i]) {
                    moves.push(build_move(pos, positions_u[i], CAPTURE_FLAG));
                }
                break;
            }
            moves.push(build_move(pos, positions_u[i], 0b0));
        }

        let positions_d = get_available_slide_pos(bitboard, pos, -1, 0, depth);

        for i in 0..positions_d.len() {
            threat_bitboard |= 1 << positions_d[i];
            if (i == positions_d.len() - 1) && bitboard.occupied(positions_d[i]) {
                if opponent_bitboard.occupied(positions_d[i]) {
                    moves.push(build_move(pos, positions_d[i], CAPTURE_FLAG));
                }
                break;
            }
            moves.push(build_move(pos, positions_d[i], 0b0));
        }
    }

    (threat_bitboard, moves)
}

fn sliding_threat_generator(
    bitboard: u64,
    position_index: u8,
    diag: bool,
    straight: bool,
    king: bool,
) -> u64 {
    let mut threat_bitboard = 0;
    let depth = if king { 1 } else { 8 };
    if diag {
        let positions_d_l = get_available_slide_pos(bitboard, position_index, -1, -1, depth);

        for i in 0..positions_d_l.len() {
            threat_bitboard |= 1 << positions_d_l[i];
        }

        let positions_u_l = get_available_slide_pos(bitboard, position_index, 1, -1, depth);

        for i in 0..positions_u_l.len() {
            threat_bitboard |= 1 << positions_u_l[i];
        }

        let positions_u_r = get_available_slide_pos(bitboard, position_index, 1, 1, depth);
        for i in 0..positions_u_r.len() {
            threat_bitboard |= 1 << positions_u_r[i];
        }

        let positions_d_r = get_available_slide_pos(bitboard, position_index, -1, 1, depth);

        for i in 0..positions_d_r.len() {
            threat_bitboard |= 1 << positions_d_r[i];
        }
    }

    if straight {
        let positions_r = get_available_slide_pos(bitboard, position_index, 0, 1, depth);

        for i in 0..positions_r.len() {
            threat_bitboard |= 1 << positions_r[i];
        }

        let positions_l = get_available_slide_pos(bitboard, position_index, 0, -1, depth);

        for i in 0..positions_l.len() {
            threat_bitboard |= 1 << positions_l[i];
        }

        let positions_u = get_available_slide_pos(bitboard, position_index, 1, 0, depth);

        for i in 0..positions_u.len() {
            threat_bitboard |= 1 << positions_u[i];
        }

        let positions_d = get_available_slide_pos(bitboard, position_index, -1, 0, depth);

        for i in 0..positions_d.len() {
            threat_bitboard |= 1 << positions_d[i];
        }
    }

    threat_bitboard
}

fn get_available_slide_pos(
    bitboard: u64,
    pos: u8,
    file_delta: i8,
    rank_delta: i8,
    max_depth: i32,
) -> Vec<u8> {
    let mut results = Vec::new();
    let delta = (file_delta * 8) + (-1 * rank_delta);
    let mut check_pos = pos as i8 + delta;
    let mut check_file = get_file(pos);
    let check_rank = get_rank(pos);
    while check_pos > -1 && check_pos < 64 {
        let cur_rank = get_rank_i8(check_pos);

        if (rank_delta > 0 && cur_rank < check_rank) || (rank_delta < 0 && cur_rank > check_rank) {
            break;
        }

        results.push(check_pos.try_into().unwrap());
        if bitboard.occupied_i8(check_pos) {
            break;
        }
        check_pos += delta;
        if file_delta == 0 && check_file != get_file(check_pos as u8) {
            break;
        }

        check_file = get_file(check_pos as u8);

        if max_depth == 1 {
            break;
        }
    }
    results
}

#[cfg(test)]
mod test {
    use crate::{board::board_utils::rank_and_file_to_index, shared::bitboard_to_string};

    use super::*;

    #[test]
    pub fn get_available_slide_pos_e4_diag_down_right() {
        let bitboard = 0b0u64;
        let result = get_available_slide_pos(bitboard, rank_and_file_to_index(4, 3), -1, 1, 8);
        assert_eq!(result.len(), 3);
        assert_eq!(result.get(0).unwrap(), &18);
        assert_eq!(result.get(1).unwrap(), &9);
        assert_eq!(result.get(2).unwrap(), &0);
    }

    #[test]
    pub fn get_available_slide_pos_c1_diag_up_left() {
        let bitboard = 0b0u64;
        let result = get_available_slide_pos(bitboard, rank_and_file_to_index(2, 0), 1, -1, 8);
        assert_eq!(result.len(), 2);
        assert_eq!(
            result.get(0).unwrap(),
            &rank_and_file_to_index(1, 1),
            "1,1 issue"
        );
        assert_eq!(
            result.get(1).unwrap(),
            &rank_and_file_to_index(0, 2),
            "0,2 issue"
        );
    }

    #[test]
    pub fn get_available_slide_pos_a3_diag_up_right_blocked_at_d6() {
        let bitboard = 0b0u64.flip(rank_and_file_to_index(3, 5));
        let result = get_available_slide_pos(bitboard, rank_and_file_to_index(0, 2), 1, 1, 8);
        assert_eq!(result.len(), 3);
        assert_eq!(result.get(0).unwrap(), &rank_and_file_to_index(1, 3));
        assert_eq!(result.get(1).unwrap(), &rank_and_file_to_index(2, 4));
        assert_eq!(result.get(2).unwrap(), &rank_and_file_to_index(3, 5));
    }

    #[test]
    pub fn get_available_slide_pos_rook_d7_left_unblocked() {
        let bitboard = 0b0u64;
        let result = get_available_slide_pos(bitboard, rank_and_file_to_index(3, 6), 0, -1, 8);
        assert_eq!(result.len(), 3);
        assert_eq!(result.get(0).unwrap(), &rank_and_file_to_index(2, 6));
        assert_eq!(result.get(1).unwrap(), &rank_and_file_to_index(1, 6));
        assert_eq!(result.get(2).unwrap(), &rank_and_file_to_index(0, 6));
    }

    #[test]
    pub fn get_available_slide_pos_rook_b3_right_unblocked() {
        let bitboard = 0b0u64;
        let result = get_available_slide_pos(bitboard, rank_and_file_to_index(1, 2), 0, 1, 8);
        assert_eq!(result.len(), 6);
    }

    #[test]
    pub fn get_available_slide_pos_rook_h1_blocked_in() {
        let bitboard = 0b1111111110u64;
        let result = get_available_slide_pos(bitboard, rank_and_file_to_index(7, 0), 1, 0, 8);
        assert_eq!(result.len(), 1); // blocked in at h2
    }

    #[test]
    pub fn generate_rook_moves_with_one_move() {
        let board = BoardState::from_fen(
            &"rnbqkbnr/ppppppp1/7p/8/8/7P/PPPPPPP1/RNBQKBNR w KQkq - 0 2".into(),
        );
        let expected_bitboard = 0b0u64
            .flip(rank_and_file_to_index(7, 1))
            .flip(rank_and_file_to_index(7, 2))
            .flip(rank_and_file_to_index(6, 0));
        let (threat_bitboard, rook_moves) = generate_rook_moves(board.bitboard, 0b0, 0);

        assert_eq!(rook_moves.len(), 1);
        assert_eq!(threat_bitboard, expected_bitboard);
    }

    #[test]
    pub fn generate_rook_moves_with_zero_moves() {
        let board = BoardState::from_fen(
            &"rnbqkbnr/ppppppp1/7p/8/8/7P/PPPPPPP1/RNBQKBNR w KQkq - 0 2".into(),
        );
        let expected_bitboard = 0b0u64
            .flip(rank_and_file_to_index(0, 1))
            .flip(rank_and_file_to_index(1, 0));
        let (threat_bitboard, rook_moves) = generate_rook_moves(board.bitboard, 0b0, 7);
        assert_eq!(rook_moves.len(), 0);
        assert_eq!(threat_bitboard, expected_bitboard);
    }

    #[test]
    pub fn generate_bishop_moves_b2_pawn_opening() {
        let board = BoardState::from_fen(
            &"r1bqkbnr/pppppppp/n7/8/1P6/8/P1PPPPPP/RNBQKBNR w KQkq - 0 1".into(),
        );
        let expected_bitboard = 0b0u64
            .flip(rank_and_file_to_index(0, 2))
            .flip(rank_and_file_to_index(1, 1))
            .flip(rank_and_file_to_index(3, 1));
        let (threat_bitboard, moves) = generate_bishop_moves(board.bitboard, 0b0, 5);
        assert_eq!(moves.len(), 2, "{moves:?}");
        assert_eq!(threat_bitboard, expected_bitboard);
    }

    #[test]
    pub fn get_available_slide_pos_bishop_moves_d2_pawn_opening() {
        let board = BoardState::from_fen(
            &"rnbqkb1r/pppppppp/5n2/8/8/3P4/PPP1PPPP/RNBQKBNR w KQkq - 0 1".into(),
        );
        let r = get_available_slide_pos(board.bitboard, 5, 1, 1, 8);
        assert_eq!(r.len(), 5, "{r:?}");
        assert_eq!(r.get(0).unwrap(), &rank_and_file_to_index(3, 1));
        assert_eq!(r.get(1).unwrap(), &rank_and_file_to_index(4, 2));
        assert_eq!(r.get(2).unwrap(), &rank_and_file_to_index(5, 3));
        assert_eq!(r.get(3).unwrap(), &rank_and_file_to_index(6, 4));
        assert_eq!(r.get(4).unwrap(), &rank_and_file_to_index(7, 5));
    }

    #[test]
    pub fn generate_pawn_moves_en_passant_a5_white_taking_right() {
        let board = BoardState::from_fen(
            &"rnbqkbnr/2pppppp/p7/Pp6/8/8/1PPPPPPP/RNBQKBNR w KQkq b6 0 1".into(),
        );
        let expected_bitboard = 0b0u64.flip(rank_and_file_to_index(1, 5));
        let (threat_bitboard, moves) = generate_pawn_moves(
            board.bitboard,
            true,
            board.black_bitboard,
            rank_and_file_to_index(0, 4),
            board.ep_rank,
        );

        assert_eq!(moves.len(), 1);
        assert_eq!(
            moves.get(0).unwrap(),
            &build_move(39, rank_and_file_to_index(1, 5), EP_CAPTURE_FLAG)
        );
        assert_eq!(threat_bitboard, expected_bitboard);
    }

    #[test]
    pub fn generate_pawn_moves_en_passant_e5_white_taking_left() {
        let board = BoardState::from_fen(
            &"rnbqkbnr/pp2pppp/2p5/3pP3/8/8/PPPP1PPP/RNBQKBNR w KQkq d6 0 1".into(),
        );
        let pos = rank_and_file_to_index(4, 4);
        let expected_bitboard = 0b0u64
            .flip(rank_and_file_to_index(3, 5))
            .flip(rank_and_file_to_index(5, 5));
        let (threat_bitboard, moves) = generate_pawn_moves(
            board.bitboard,
            true,
            board.black_bitboard,
            pos,
            board.ep_rank,
        );

        assert_eq!(moves.len(), 2, "{moves:?}");
        assert_eq!(
            moves.get(0).unwrap(),
            &build_move(pos, rank_and_file_to_index(4, 5), 0b0)
        );
        assert_eq!(
            moves.get(1).unwrap(),
            &build_move(pos, rank_and_file_to_index(3, 5), EP_CAPTURE_FLAG)
        );
        assert_eq!(threat_bitboard, expected_bitboard);
    }

    #[test]
    pub fn generate_pawn_moves_en_passant_b5_white_taking_left_to_a_rank() {
        let board = BoardState::from_fen(
            &"rnbqkbnr/2pppppp/1p6/pP6/8/8/P1PPPPPP/RNBQKBNR w KQkq a6 0 1".into(),
        );
        let pos = rank_and_file_to_index(1, 4);
        let expected_bitboard = 0b0u64
            .flip(rank_and_file_to_index(0, 5))
            .flip(rank_and_file_to_index(2, 5));
        let (threat_bitboard, moves) = generate_pawn_moves(
            board.bitboard,
            true,
            board.black_bitboard,
            pos,
            board.ep_rank,
        );

        assert_eq!(moves.len(), 1, "{moves:?}");
        assert_eq!(
            moves.get(0).unwrap(),
            &build_move(pos, rank_and_file_to_index(0, 5), EP_CAPTURE_FLAG)
        );
        assert_eq!(threat_bitboard, expected_bitboard);
    }

    #[test]
    pub fn generate_pawn_moves_b5_white_taking_non_en_passant_in_a_rank() {
        let board = BoardState::from_fen(
            &"rnbqkbnr/2pppppp/1p6/pP6/8/6P1/P1PPPP1P/RNBQKBNR w KQkq - 0 1".into(),
        );
        let pos = rank_and_file_to_index(1, 4);

        let expected_bitboard = 0b0u64
            .flip(rank_and_file_to_index(0, 5))
            .flip(rank_and_file_to_index(2, 5));
        let (threat_bitboard, moves) = generate_pawn_moves(
            board.bitboard,
            true,
            board.black_bitboard,
            pos,
            board.ep_rank,
        );

        assert_eq!(moves.len(), 0, "{moves:?}");
        assert_eq!(threat_bitboard, expected_bitboard);
    }

    #[test]
    pub fn generate_pawn_moves_en_passant_black() {
        let board = BoardState::from_fen(
            &"rnbqkbnr/ppppppp1/8/8/6Pp/P6P/1PPPPP2/RNBQKBNR b KQkq g3 0 1".into(),
        );

        let expected_bitboard = 0b0u64.flip(rank_and_file_to_index(6, 2));
        let (threat_bitboard, moves) = generate_pawn_moves(
            board.bitboard,
            false,
            board.white_bitboard,
            rank_and_file_to_index(7, 3),
            board.ep_rank,
        );

        assert_eq!(moves.len(), 1);
        assert_eq!(
            moves.get(0).unwrap(),
            &build_move(
                rank_and_file_to_index(7, 3),
                rank_and_file_to_index(6, 2),
                EP_CAPTURE_FLAG
            )
        );
        assert_eq!(threat_bitboard, expected_bitboard);
    }

    #[test]
    pub fn generate_pawn_moves_no_en_passant_black_b_pawn_opening() {
        let board = BoardState::from_fen(
            &"rnbqkbnr/pppppppp/8/8/P7/8/1PPPPPPP/RNBQKBNR b KQkq a3 0 1".into(),
        );
        let expected_bitboard = 0b0u64
            .flip(rank_and_file_to_index(0, 5))
            .flip(rank_and_file_to_index(2, 5));
        let (threat_bitboard, moves) = generate_pawn_moves(
            board.bitboard,
            false,
            board.white_bitboard,
            rank_and_file_to_index(1, 6),
            board.ep_rank,
        );

        assert_eq!(moves.len(), 2);
        assert_eq!(
            moves.get(0).unwrap(),
            &build_move(
                rank_and_file_to_index(1, 6),
                rank_and_file_to_index(1, 5),
                0b0
            )
        );
        assert_eq!(
            moves.get(1).unwrap(),
            &build_move(
                rank_and_file_to_index(1, 6),
                rank_and_file_to_index(1, 4),
                0b0
            )
        );
        assert_eq!(threat_bitboard, expected_bitboard);
    }

    #[test]
    pub fn generate_knight_moves_case_1() {
        let board = BoardState::from_fen(&"8/8/4p3/1k6/3N4/8/8/8 w - - 0 1".into());
        let expected_bitboard = 0b0u64
            .flip(rank_and_file_to_index(2, 1))
            .flip(rank_and_file_to_index(1, 2))
            .flip(rank_and_file_to_index(1, 4))
            .flip(rank_and_file_to_index(2, 5))
            .flip(rank_and_file_to_index(4, 5))
            .flip(rank_and_file_to_index(5, 4))
            .flip(rank_and_file_to_index(5, 2))
            .flip(rank_and_file_to_index(4, 1));

        let (threat_bitboard, moves) = generate_knight_moves(
            board.bitboard,
            board.black_bitboard,
            rank_and_file_to_index(3, 3),
        );

        assert_eq!(moves.len(), 8);
        assert_eq!(threat_bitboard, expected_bitboard);
    }
}
