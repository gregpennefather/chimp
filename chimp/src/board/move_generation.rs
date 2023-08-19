use crate::{
    board::board_utils::get_piece_from_position_index,
    shared::{
        BISHOP_CAPTURE_PROMOTION, BISHOP_INDEX, BISHOP_PROMOTION, BLACK_KING,
        BLACK_PAWN, CAPTURE_FLAG, DOUBLE_PAWN_FLAG, EP_CAPTURE_FLAG, KING_CASTLING_FLAG,
        KING_INDEX, KNIGHT_CAPTURE_PROMOTION, KNIGHT_INDEX, KNIGHT_PROMOTION, PAWN_INDEX,
        PIECE_MASK, QUEEN_CAPTURE_PROMOTION, QUEEN_CASTLING_FLAG, QUEEN_INDEX, QUEEN_PROMOTION,
        ROOK_CAPTURE_PROMOTION, ROOK_INDEX, ROOK_PROMOTION, bitboard_to_string,
    },
};

use super::{
    bitboard::BitboardExtensions,
    board_metrics::BoardMetrics,
    board_utils::{
        get_file, get_position_index_from_piece_index, get_rank, rank_and_file_to_index,
        rank_from_char,
    },
    move_utils::{build_move, get_available_slide_pos, is_castling, is_king_castling},
    piece_utils::{get_piece_code, is_piece_black},
    state::{BoardState, BoardStateFlagsTrait},
};

impl BoardState {
    pub fn generate_psudolegals(&self) -> Vec<u16> {
        let mut moves = Vec::new();
        let black_turn = self.flags.is_black_turn();
        let opponent_bitboard = if black_turn {
            self.white_bitboard
        } else {
            self.black_bitboard
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
            if is_piece_black(piece) == black_turn {
                friendly_piece_index_pairs[friendly_piece_count] = (position_index, piece);
                friendly_piece_count += 1;
            } else {
                opponent_piece_index_pairs[opponent_piece_count] = (position_index, piece);
                opponent_piece_count += 1;
            }
            piece_index += 1;
        }

        for pair in friendly_piece_index_pairs {
            let p_moves = generate_piece_moves(
                self.bitboard,
                black_turn,
                pair.0,
                pair.1,
                opponent_bitboard,
                self.flags,
                self.ep_rank,
            );
            moves.extend(&p_moves);
        }

        moves
    }

    pub fn generate_legal_moves(
        &self,
        psudolegal_moves: &Vec<u16>,
        current_metrics: &BoardMetrics,
    ) -> Vec<(u16, BoardState, BoardMetrics)> {
        let black_turn = self.flags.is_black_turn();
        let mut legal_moves_with_state_and_metrics = Vec::new();
        for &psudolegal_move in psudolegal_moves {
            let move_flags = (psudolegal_move & 0b1111) as u8;
            if is_castling(move_flags)
                && !is_legal_castling(psudolegal_move, move_flags, black_turn, &current_metrics)
            {
                continue;
            }

            let new_state = self.apply_move(psudolegal_move);
            let new_metrics = new_state.generate_metrics();

            if (!black_turn && !new_metrics.white_in_check)
                || (black_turn && !new_metrics.black_in_check)
            {
                legal_moves_with_state_and_metrics.push((psudolegal_move, new_state, new_metrics));
            }
        }
        legal_moves_with_state_and_metrics
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
                } else if self.ep_rank == to_rank
                    && to_rank != from_rank
                    && ((piece == PAWN_INDEX && to_file == 5)
                        || (piece == BLACK_PAWN && to_file == 2))
                {
                    flags = EP_CAPTURE_FLAG;
                } else {
                    let promotion_char = move_string.chars().nth(4);
                    match promotion_char {
                        Some(p) => {
                            flags = match p {
                                'n' => {
                                    if flags != CAPTURE_FLAG {
                                        KNIGHT_PROMOTION
                                    } else {
                                        KNIGHT_CAPTURE_PROMOTION
                                    }
                                }
                                'b' => {
                                    if flags != CAPTURE_FLAG {
                                        BISHOP_PROMOTION
                                    } else {
                                        BISHOP_CAPTURE_PROMOTION
                                    }
                                }
                                'r' => {
                                    if flags != CAPTURE_FLAG {
                                        ROOK_PROMOTION
                                    } else {
                                        ROOK_CAPTURE_PROMOTION
                                    }
                                }
                                'q' => {
                                    if flags != CAPTURE_FLAG {
                                        QUEEN_PROMOTION
                                    } else {
                                        QUEEN_CAPTURE_PROMOTION
                                    }
                                }
                                _ => 0,
                            }
                        }
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

fn is_legal_castling(
    psudolegal_move: u16,
    move_flags: u8,
    black_turn: bool,
    current_metrics: &BoardMetrics,
) -> bool {
    let from_index: u8 = (psudolegal_move >> 10).try_into().unwrap();
    if (!black_turn && current_metrics.white_in_check)
        || (black_turn && current_metrics.black_in_check)
    {
        return false;
    }
    let opponent_threat_board = if black_turn {
        current_metrics.white_threat_board
    } else {
        current_metrics.black_threat_board
    };
    if is_king_castling(move_flags) {
        return !opponent_threat_board.occupied(from_index - 1)
            && !opponent_threat_board.occupied(from_index - 2);
    } else {
        return !opponent_threat_board.occupied(from_index + 1)
            && !opponent_threat_board.occupied(from_index + 2);
    }
}

fn generate_piece_moves(
    bitboard: u64,
    is_black: bool,
    position_index: u8,
    piece: u8,
    opponent_bitboard: u64,
    flags: u8,
    ep_rank: u8,
) -> (Vec<u16>) {
    let piece_code = piece & PIECE_MASK;
    let castling_flags = if is_black {
        (flags >> 3) & 0b11
    } else {
        flags >> 1 & 0b11
    };
    match piece_code {
        PAWN_INDEX => generate_pawn_moves(
            bitboard,
            is_black,
            opponent_bitboard,
            position_index,
            ep_rank,
        ),
        KNIGHT_INDEX => generate_knight_moves(bitboard, opponent_bitboard, position_index),
        BISHOP_INDEX => generate_bishop_moves(bitboard, opponent_bitboard, position_index),
        ROOK_INDEX => generate_rook_moves(bitboard, opponent_bitboard, position_index),
        QUEEN_INDEX => generate_queen_moves(bitboard, opponent_bitboard, position_index),
        KING_INDEX => {
            generate_king_moves(bitboard, opponent_bitboard, position_index, castling_flags)
        }
        _ => vec![],
    }
}

fn generate_pawn_moves(
    bitboard: u64,
    is_black: bool,
    opponent_bitboard: u64,
    position_index: u8,
    ep_rank: u8,
) -> Vec<u16> {
    let mut results: Vec<_> = Vec::new();
    let file = get_file(position_index);
    let rank = get_rank(position_index);
    let is_ep = ep_rank != u8::MAX;

    if !is_black {
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
                        results.push(build_move(
                            position_index,
                            position_index + 16,
                            DOUBLE_PAWN_FLAG,
                        ));
                    }
                }
            }
        }

        // Capture Right
        if rank != 7 {
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
                        results.push(build_move(
                            position_index,
                            position_index - 16,
                            DOUBLE_PAWN_FLAG,
                        ));
                    }
                }
            }
        }

        // Capture right
        if rank != 7 {
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
    results
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

fn generate_knight_moves(bitboard: u64, opponent_bitboard: u64, position_index: u8) -> Vec<u16> {
    let mut results: Vec<_> = Vec::new();
    let rank = get_rank(position_index);
    // U2R1 = +16-1 = 15
    if position_index <= 48 && rank != 7 {
        let tar = position_index + 15;
        if opponent_bitboard.occupied(tar) {
            results.push(build_move(position_index, tar, CAPTURE_FLAG));
        } else if !bitboard.occupied(tar) {
            results.push(build_move(position_index, tar, 0b0));
        }
    }
    // U1R2 = +8-2 = 6
    if position_index <= 55 && rank < 6 {
        let tar = position_index + 6;
        if opponent_bitboard.occupied(tar) {
            results.push(build_move(position_index, tar, CAPTURE_FLAG));
        } else if !bitboard.occupied(tar) {
            results.push(build_move(position_index, tar, 0b0));
        }
    }
    // D1R2 = -8-2 = -10
    if position_index >= 10 && rank < 6 {
        let tar = position_index - 10;
        if opponent_bitboard.occupied(tar) {
            results.push(build_move(position_index, tar, CAPTURE_FLAG));
        } else if !bitboard.occupied(tar) {
            results.push(build_move(position_index, tar, 0b0));
        }
    }
    // D2R1 = -16-1 = -17
    if position_index >= 17 && rank != 7 {
        let tar = position_index - 17;
        if opponent_bitboard.occupied(tar) {
            results.push(build_move(position_index, tar, CAPTURE_FLAG));
        } else if !bitboard.occupied(tar) {
            results.push(build_move(position_index, tar, 0b0));
        }
    }
    // D2L1 = -16+1 = -15
    if position_index >= 15 && rank != 0 {
        let tar = position_index - 15;
        if opponent_bitboard.occupied(tar) {
            results.push(build_move(position_index, tar, CAPTURE_FLAG));
        } else if !bitboard.occupied(tar) {
            results.push(build_move(position_index, tar, 0b0));
        }
    }
    // D1L2 = -8+2 = -6
    if position_index >= 6 && rank > 1 {
        let tar = position_index - 6;
        if opponent_bitboard.occupied(tar) {
            results.push(build_move(position_index, tar, CAPTURE_FLAG));
        } else if !bitboard.occupied(tar) {
            results.push(build_move(position_index, tar, 0b0));
        }
    }
    // U1L2 = 8+2 = 10
    if position_index <= 53 && rank > 1 {
        let tar = position_index + 10;
        if opponent_bitboard.occupied(tar) {
            results.push(build_move(position_index, tar, CAPTURE_FLAG));
        } else if !bitboard.occupied(tar) {
            results.push(build_move(position_index, tar, 0b0));
        }
    }
    // U2L1 = 16+1 = 17
    if position_index <= 46 && rank != 0 {
        let tar = position_index + 17;
        if opponent_bitboard.occupied(tar) {
            results.push(build_move(position_index, tar, CAPTURE_FLAG));
        } else if !bitboard.occupied(tar) {
            results.push(build_move(position_index, tar, 0b0));
        }
    }
    results
}

fn generate_bishop_moves(bitboard: u64, opponent_bitboard: u64, position_index: u8) -> Vec<u16> {
    sliding_move_generator(
        bitboard,
        opponent_bitboard,
        position_index,
        true,
        false,
        false,
    )
}

fn generate_rook_moves(bitboard: u64, opponent_bitboard: u64, position_index: u8) -> Vec<u16> {
    sliding_move_generator(
        bitboard,
        opponent_bitboard,
        position_index,
        false,
        true,
        false,
    )
}

fn generate_queen_moves(bitboard: u64, opponent_bitboard: u64, position_index: u8) -> Vec<u16> {
    sliding_move_generator(
        bitboard,
        opponent_bitboard,
        position_index,
        true,
        true,
        false,
    )
}

fn generate_king_moves(
    bitboard: u64,
    opponent_bitboard: u64,
    position_index: u8,
    castling_flags: u8,
) -> Vec<u16> {
    let mut moves = sliding_move_generator(
        bitboard,
        opponent_bitboard,
        position_index,
        true,
        true,
        true,
    );

    if (castling_flags & 0b01) > 0 {
        // King side castling
        if (!bitboard.occupied(position_index - 1) && !bitboard.occupied(position_index - 2)) {
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
        {
            moves.push(build_move(
                position_index,
                position_index + 2,
                QUEEN_CASTLING_FLAG,
            ))
        }
    }
    moves
}

fn sliding_move_generator(
    bitboard: u64,
    opponent_bitboard: u64,
    pos: u8,
    diag: bool,
    straight: bool,
    king: bool,
) -> Vec<u16> {
    let mut moves: Vec<u16> = Vec::new();

    let depth = if king { 1 } else { 8 };

    if diag {
        let positions_d_l = get_available_slide_pos(bitboard, pos, -1, -1, depth);

        for i in 0..positions_d_l.len() {
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
            if (i == positions_d.len() - 1) && bitboard.occupied(positions_d[i]) {
                if opponent_bitboard.occupied(positions_d[i]) {
                    moves.push(build_move(pos, positions_d[i], CAPTURE_FLAG));
                }
                break;
            }
            moves.push(build_move(pos, positions_d[i], 0b0));
        }
    }

    moves
}

#[cfg(test)]
mod test {
    use crate::board::board_utils::rank_and_file_to_index;

    use super::*;

    #[test]
    pub fn generate_rook_moves_with_one_move() {
        let board = BoardState::from_fen(
            &"rnbqkbnr/ppppppp1/7p/8/8/7P/PPPPPPP1/RNBQKBNR w KQkq - 0 2".into(),
        );
        let rook_moves = generate_rook_moves(board.bitboard, 0b0, 0);

        assert_eq!(rook_moves.len(), 1);
    }

    #[test]
    pub fn generate_rook_moves_with_zero_moves() {
        let board = BoardState::from_fen(
            &"rnbqkbnr/ppppppp1/7p/8/8/7P/PPPPPPP1/RNBQKBNR w KQkq - 0 2".into(),
        );
        let rook_moves = generate_rook_moves(board.bitboard, 0b0, 7);
        assert_eq!(rook_moves.len(), 0);
    }

    #[test]
    pub fn generate_bishop_moves_b2_pawn_opening() {
        let board = BoardState::from_fen(
            &"r1bqkbnr/pppppppp/n7/8/1P6/8/P1PPPPPP/RNBQKBNR w KQkq - 0 1".into(),
        );
        let moves = generate_bishop_moves(board.bitboard, 0b0, 5);
        assert_eq!(moves.len(), 2, "{moves:?}");
    }

    #[test]
    pub fn generate_pawn_moves_en_passant_a5_white_taking_right() {
        let board = BoardState::from_fen(
            &"rnbqkbnr/2pppppp/p7/Pp6/8/8/1PPPPPPP/RNBQKBNR w KQkq b6 0 1".into(),
        );
        let moves = generate_pawn_moves(
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
    }

    #[test]
    pub fn generate_pawn_moves_en_passant_e5_white_taking_left() {
        let board = BoardState::from_fen(
            &"rnbqkbnr/pp2pppp/2p5/3pP3/8/8/PPPP1PPP/RNBQKBNR w KQkq d6 0 1".into(),
        );
        let pos = rank_and_file_to_index(4, 4);
        let moves = generate_pawn_moves(
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
    }

    #[test]
    pub fn generate_pawn_moves_en_passant_b5_white_taking_left_to_a_rank() {
        let board = BoardState::from_fen(
            &"rnbqkbnr/2pppppp/1p6/pP6/8/8/P1PPPPPP/RNBQKBNR w KQkq a6 0 1".into(),
        );
        let pos = rank_and_file_to_index(1, 4);
        let moves = generate_pawn_moves(
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
    }

    #[test]
    pub fn generate_pawn_moves_b5_white_taking_non_en_passant_in_a_rank() {
        let board = BoardState::from_fen(
            &"rnbqkbnr/2pppppp/1p6/pP6/8/6P1/P1PPPP1P/RNBQKBNR w KQkq - 0 1".into(),
        );
        let pos = rank_and_file_to_index(1, 4);

        let moves = generate_pawn_moves(
            board.bitboard,
            true,
            board.black_bitboard,
            pos,
            board.ep_rank,
        );

        assert_eq!(moves.len(), 0, "{moves:?}");
    }

    #[test]
    pub fn generate_pawn_moves_en_passant_black() {
        let board = BoardState::from_fen(
            &"rnbqkbnr/ppppppp1/8/8/6Pp/P6P/1PPPPP2/RNBQKBNR b KQkq g3 0 1".into(),
        );

        let moves = generate_pawn_moves(
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
    }

    #[test]
    pub fn generate_pawn_moves_no_en_passant_black_b_pawn_opening() {
        let board = BoardState::from_fen(
            &"rnbqkbnr/pppppppp/8/8/P7/8/1PPPPPPP/RNBQKBNR b KQkq a3 0 1".into(),
        );
        let expected_bitboard = 0b0u64
            .flip(rank_and_file_to_index(0, 5))
            .flip(rank_and_file_to_index(2, 5));
        let moves = generate_pawn_moves(
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
    }

    #[test]
    pub fn generate_knight_moves_case_1() {
        let board = BoardState::from_fen(&"8/8/4p3/1k6/3N4/8/8/8 w - - 0 1".into());

        let moves = generate_knight_moves(
            board.bitboard,
            board.black_bitboard,
            rank_and_file_to_index(3, 3),
        );

        assert_eq!(moves.len(), 8);
    }
}
