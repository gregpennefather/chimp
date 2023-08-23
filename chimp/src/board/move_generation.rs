use crate::util::t_table::MoveTableLookup;

use super::{
    bitboard::{Bitboard, BitboardExtensions},
    board_metrics::BoardMetrics,
    board_utils::{file_and_rank_to_index, file_from_char, get_file, get_rank},
    move_utils::get_available_slide_pos,
    piece::{Piece, PieceType},
    r#move::{
        Move, BISHOP_CAPTURE_PROMOTION, BISHOP_PROMOTION, CAPTURE, DOUBLE_PAWN_PUSH, EP_CAPTURE,
        KING_CASTLING, KNIGHT_CAPTURE_PROMOTION, KNIGHT_PROMOTION, QUEEN_CAPTURE_PROMOTION,
        QUEEN_CASTLING, QUEEN_PROMOTION, ROOK_CAPTURE_PROMOTION, ROOK_PROMOTION,
    },
    state::{BoardState, BoardStateFlagsTrait},
};

impl BoardState {
    pub fn generate_psudolegals(&self) -> Vec<Move> {
        let mut moves = Vec::new();
        let black_turn = self.flags.is_black_turn();
        let opponent_bitboard = if black_turn {
            self.white_bitboard
        } else {
            self.black_bitboard
        };

        let mut piece_index = 0;
        while piece_index < self.piece_count {
            let position_index = self.bitboard.get_nth_piece_position_index(piece_index);
            let piece = self.pieces.get(piece_index);
            if piece.is_black() == black_turn {
                let p_moves = generate_piece_moves(
                    self.bitboard,
                    black_turn,
                    position_index,
                    piece,
                    opponent_bitboard,
                    self.flags,
                    self.ep_file,
                );
                moves.extend(p_moves);
            }
            piece_index += 1;
        }

        moves
    }

    pub fn move_from_string(&self, move_string: &str) -> Move {
        let from_file_char = move_string.chars().nth(0).unwrap();
        let from_file = file_from_char(from_file_char);
        let from_rank: u8 = (move_string.chars().nth(1).unwrap().to_digit(16).unwrap() - 1) as u8;

        let from_index = file_and_rank_to_index(from_file, from_rank);

        let to_file_char = move_string.chars().nth(2).unwrap();
        let to_file = file_from_char(to_file_char);
        let to_rank: u8 = (move_string.chars().nth(3).unwrap().to_digit(16).unwrap() - 1) as u8;

        let to_index = file_and_rank_to_index(to_file, to_rank);

        let mut flags = 0;

        if self.bitboard.occupied(to_index) {
            flags = CAPTURE;
        }

        let piece = self.pieces.get_by_position_index(self.bitboard, from_index);
        match piece.without_colour() {
            PieceType::Pawn => {
                if piece.is_white() && from_rank == 1 && to_rank == 3 {
                    flags = DOUBLE_PAWN_PUSH;
                } else if piece.is_black() && from_rank == 6 && to_rank == 4 {
                    flags = DOUBLE_PAWN_PUSH;
                } else if self.ep_file == to_file
                    && to_file != from_file
                    && ((piece.is_white() && to_rank == 5) || (piece.is_black() && to_rank == 2))
                {
                    flags = EP_CAPTURE;
                } else {
                    let promotion_char = move_string.chars().nth(4);
                    match promotion_char {
                        Some(p) => {
                            flags = match p {
                                'n' => {
                                    if flags != CAPTURE {
                                        KNIGHT_PROMOTION
                                    } else {
                                        KNIGHT_CAPTURE_PROMOTION
                                    }
                                }
                                'b' => {
                                    if flags != CAPTURE {
                                        BISHOP_PROMOTION
                                    } else {
                                        BISHOP_CAPTURE_PROMOTION
                                    }
                                }
                                'r' => {
                                    if flags != CAPTURE {
                                        ROOK_PROMOTION
                                    } else {
                                        ROOK_CAPTURE_PROMOTION
                                    }
                                }
                                'q' => {
                                    if flags != CAPTURE {
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
            PieceType::King => {
                if from_file == 4 {
                    if to_file == 2 {
                        flags = QUEEN_CASTLING
                    } else if to_file == 6 {
                        flags = KING_CASTLING
                    }
                }
            }
            _ => {}
        }

        Move::new(from_index, to_index, flags)
    }
}

pub fn is_legal_castling(
    psudolegal_move: Move,
    black_turn: bool,
    current_metrics: &BoardMetrics,
) -> bool {
    let from_index = psudolegal_move.from();
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
    if psudolegal_move.is_king_castling() {
        return !opponent_threat_board.occupied(from_index - 1)
            && !opponent_threat_board.occupied(from_index - 2);
    } else {
        return !opponent_threat_board.occupied(from_index + 1)
            && !opponent_threat_board.occupied(from_index + 2);
    }
}

fn generate_piece_moves(
    bitboard: Bitboard,
    is_black: bool,
    position_index: u8,
    piece: Piece,
    opponent_bitboard: Bitboard,
    flags: u8,
    ep_file: u8,
) -> Vec<Move> {
    let castling_flags = if is_black {
        (flags >> 3) & 0b11
    } else {
        flags >> 1 & 0b11
    };
    match piece.without_colour() {
        PieceType::Pawn => generate_pawn_moves(
            bitboard,
            is_black,
            opponent_bitboard,
            position_index,
            ep_file,
        ),
        PieceType::Knight => generate_knight_moves(bitboard, opponent_bitboard, position_index),
        PieceType::Bishop => generate_bishop_moves(bitboard, opponent_bitboard, position_index),
        PieceType::Rook => generate_rook_moves(bitboard, opponent_bitboard, position_index),
        PieceType::Queen => generate_queen_moves(bitboard, opponent_bitboard, position_index),
        PieceType::King => {
            generate_king_moves(bitboard, opponent_bitboard, position_index, castling_flags)
        }
    }
}

fn generate_pawn_moves(
    bitboard: Bitboard,
    is_black: bool,
    opponent_bitboard: Bitboard,
    position_index: u8,
    ep_file: u8,
) -> Vec<Move> {
    let mut results = Vec::new();
    let rank = get_rank(position_index);
    let file = get_file(position_index);
    let is_ep = ep_file != u8::MAX;

    if !is_black {
        if !bitboard.occupied(position_index + 8) {
            // Promotion
            if rank == 6 {
                results.extend(build_promotion_moves(
                    position_index,
                    position_index + 8,
                    false,
                ));
            } else {
                // Move
                results.push(Move::new(position_index, position_index + 8, 0b0));
                if rank == 1 {
                    if !bitboard.occupied(position_index + 16) {
                        results.push(Move::new(
                            position_index,
                            position_index + 16,
                            DOUBLE_PAWN_PUSH,
                        ));
                    }
                }
            }
        }

        // Capture Right
        if file != 7 {
            if opponent_bitboard.occupied(position_index + 7) {
                // Promotion
                if rank == 6 {
                    results.extend(build_promotion_moves(
                        position_index,
                        position_index + 7,
                        true,
                    ));
                } else {
                    // Double push
                    results.push(Move::new(position_index, position_index + 7, CAPTURE));
                }
            }
        }

        // Capture left
        if file != 0 {
            if opponent_bitboard.occupied(position_index + 9) {
                // Promotion
                if rank == 6 {
                    results.extend(build_promotion_moves(
                        position_index,
                        position_index + 9,
                        true,
                    ));
                } else {
                    results.push(Move::new(position_index, position_index + 9, CAPTURE));
                }
            }
        }

        if is_ep && rank == 4 {
            if file != 0 && ep_file == file - 1 {
                results.push(Move::new(position_index, position_index + 9, EP_CAPTURE));
            } else if file != 7 && ep_file == file + 1 {
                results.push(Move::new(position_index, position_index + 7, EP_CAPTURE));
            }
        }
    } else {
        if !bitboard.occupied(position_index - 8) {
            // Promotion
            if rank == 1 {
                results.extend(build_promotion_moves(
                    position_index,
                    position_index - 8,
                    false,
                ));
            } else {
                // Move
                results.push(Move::new(position_index, position_index - 8, 0b0));

                // Double push
                if rank == 6 {
                    if !bitboard.occupied(position_index - 16) {
                        results.push(Move::new(
                            position_index,
                            position_index - 16,
                            DOUBLE_PAWN_PUSH,
                        ));
                    }
                }
            }
        }

        // Capture right
        if file != 7 {
            if opponent_bitboard.occupied(position_index - 9) {
                // Promotion
                if rank == 1 {
                    results.extend(build_promotion_moves(
                        position_index,
                        position_index - 9,
                        true,
                    ));
                } else {
                    results.push(Move::new(position_index, position_index - 9, CAPTURE));
                }
            }
        }

        // Capture left
        if file != 0 {
            if opponent_bitboard.occupied(position_index - 7) {
                // Promotion
                if rank == 1 {
                    results.extend(build_promotion_moves(
                        position_index,
                        position_index - 7,
                        true,
                    ));
                } else {
                    results.push(Move::new(position_index, position_index - 7, CAPTURE));
                }
            }
        }

        if is_ep && rank == 3 {
            if file != 0 && ep_file == file - 1 {
                results.push(Move::new(position_index, position_index - 7, EP_CAPTURE));
            } else if file != 7 && ep_file == file + 1 {
                results.push(Move::new(position_index, position_index - 9, EP_CAPTURE));
            }
        }
    }
    results
}

fn build_promotion_moves(from_index: u8, to_index: u8, capture: bool) -> Vec<Move> {
    return vec![
        Move::new(
            from_index,
            to_index,
            if !capture {
                KNIGHT_PROMOTION
            } else {
                KNIGHT_CAPTURE_PROMOTION
            },
        ), // Knight
        Move::new(
            from_index,
            to_index,
            if !capture {
                BISHOP_PROMOTION
            } else {
                BISHOP_CAPTURE_PROMOTION
            },
        ), // Bishop
        Move::new(
            from_index,
            to_index,
            if !capture {
                ROOK_PROMOTION
            } else {
                ROOK_CAPTURE_PROMOTION
            },
        ), // Rook
        Move::new(
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

fn generate_knight_moves(
    bitboard: Bitboard,
    opponent_bitboard: Bitboard,
    position_index: u8,
) -> Vec<Move> {
    let mut results: Vec<_> = Vec::new();
    let file = get_file(position_index);
    // U2R1 = +16-1 = 15
    if position_index <= 48 && file != 7 {
        let tar = position_index + 15;
        if opponent_bitboard.occupied(tar) {
            results.push(Move::new(position_index, tar, CAPTURE));
        } else if !bitboard.occupied(tar) {
            results.push(Move::new(position_index, tar, 0b0));
        }
    }
    // U1R2 = +8-2 = 6
    if position_index <= 55 && file < 6 {
        let tar = position_index + 6;
        if opponent_bitboard.occupied(tar) {
            results.push(Move::new(position_index, tar, CAPTURE));
        } else if !bitboard.occupied(tar) {
            results.push(Move::new(position_index, tar, 0b0));
        }
    }
    // D1R2 = -8-2 = -10
    if position_index >= 10 && file < 6 {
        let tar = position_index - 10;
        if opponent_bitboard.occupied(tar) {
            results.push(Move::new(position_index, tar, CAPTURE));
        } else if !bitboard.occupied(tar) {
            results.push(Move::new(position_index, tar, 0b0));
        }
    }
    // D2R1 = -16-1 = -17
    if position_index >= 17 && file != 7 {
        let tar = position_index - 17;
        if opponent_bitboard.occupied(tar) {
            results.push(Move::new(position_index, tar, CAPTURE));
        } else if !bitboard.occupied(tar) {
            results.push(Move::new(position_index, tar, 0b0));
        }
    }
    // D2L1 = -16+1 = -15
    if position_index >= 15 && file != 0 {
        let tar = position_index - 15;
        if opponent_bitboard.occupied(tar) {
            results.push(Move::new(position_index, tar, CAPTURE));
        } else if !bitboard.occupied(tar) {
            results.push(Move::new(position_index, tar, 0b0));
        }
    }
    // D1L2 = -8+2 = -6
    if position_index >= 6 && file > 1 {
        let tar = position_index - 6;
        if opponent_bitboard.occupied(tar) {
            results.push(Move::new(position_index, tar, CAPTURE));
        } else if !bitboard.occupied(tar) {
            results.push(Move::new(position_index, tar, 0b0));
        }
    }
    // U1L2 = 8+2 = 10
    if position_index <= 53 && file > 1 {
        let tar = position_index + 10;
        if opponent_bitboard.occupied(tar) {
            results.push(Move::new(position_index, tar, CAPTURE));
        } else if !bitboard.occupied(tar) {
            results.push(Move::new(position_index, tar, 0b0));
        }
    }
    // U2L1 = 16+1 = 17
    if position_index <= 46 && file != 0 {
        let tar = position_index + 17;
        if opponent_bitboard.occupied(tar) {
            results.push(Move::new(position_index, tar, CAPTURE));
        } else if !bitboard.occupied(tar) {
            results.push(Move::new(position_index, tar, 0b0));
        }
    }
    results
}

fn generate_bishop_moves(
    bitboard: Bitboard,
    opponent_bitboard: Bitboard,
    position_index: u8,
) -> Vec<Move> {
    sliding_move_generator(
        bitboard,
        opponent_bitboard,
        position_index,
        true,
        false,
        false,
    )
}

fn generate_rook_moves(
    bitboard: Bitboard,
    opponent_bitboard: Bitboard,
    position_index: u8,
) -> Vec<Move> {
    sliding_move_generator(
        bitboard,
        opponent_bitboard,
        position_index,
        false,
        true,
        false,
    )
}

fn generate_queen_moves(
    bitboard: Bitboard,
    opponent_bitboard: Bitboard,
    position_index: u8,
) -> Vec<Move> {
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
    bitboard: Bitboard,
    opponent_bitboard: Bitboard,
    position_index: u8,
    castling_flags: u8,
) -> Vec<Move> {
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
        if !bitboard.occupied(position_index - 1) && !bitboard.occupied(position_index - 2) {
            moves.push(Move::new(position_index, position_index - 2, KING_CASTLING))
        }
    }
    if (castling_flags & 0b10) > 0 {
        // Queen side castling

        // Multiple square checking could probably be done more efficiently with a | flag and a greater than check
        if !bitboard.occupied(position_index + 1)
            && !bitboard.occupied(position_index + 2)
            && !bitboard.occupied(position_index + 3)
        {
            moves.push(Move::new(
                position_index,
                position_index + 2,
                QUEEN_CASTLING,
            ))
        }
    }
    moves
}

fn sliding_move_generator(
    bitboard: Bitboard,
    opponent_bitboard: Bitboard,
    pos: u8,
    diag: bool,
    straight: bool,
    king: bool,
) -> Vec<Move> {
    let mut moves = Vec::new();

    let depth = if king { 1 } else { 8 };

    if diag {
        let positions_d_l = get_available_slide_pos(bitboard, pos, -1, -1, depth);

        for i in 0..positions_d_l.len() {
            if (i == positions_d_l.len() - 1) && bitboard.occupied(positions_d_l[i]) {
                if opponent_bitboard.occupied(positions_d_l[i]) {
                    moves.push(Move::new(pos, positions_d_l[i], CAPTURE));
                }
                break;
            }
            moves.push(Move::new(pos, positions_d_l[i], 0b0));
        }

        let positions_u_l = get_available_slide_pos(bitboard, pos, 1, -1, depth);

        for i in 0..positions_u_l.len() {
            if (i == positions_u_l.len() - 1) && bitboard.occupied(positions_u_l[i]) {
                if opponent_bitboard.occupied(positions_u_l[i]) {
                    moves.push(Move::new(pos, positions_u_l[i], CAPTURE));
                }
                break;
            }
            moves.push(Move::new(pos, positions_u_l[i], 0b0));
        }

        let positions_u_r = get_available_slide_pos(bitboard, pos, 1, 1, depth);
        for i in 0..positions_u_r.len() {
            if (i == positions_u_r.len() - 1) && bitboard.occupied(positions_u_r[i]) {
                if opponent_bitboard.occupied(positions_u_r[i]) {
                    moves.push(Move::new(pos, positions_u_r[i], CAPTURE));
                }
                break;
            }
            moves.push(Move::new(pos, positions_u_r[i], 0b0));
        }

        let positions_d_r = get_available_slide_pos(bitboard, pos, -1, 1, depth);

        for i in 0..positions_d_r.len() {
            if (i == positions_d_r.len() - 1) && bitboard.occupied(positions_d_r[i]) {
                if opponent_bitboard.occupied(positions_d_r[i]) {
                    moves.push(Move::new(pos, positions_d_r[i], CAPTURE));
                }
                break;
            }
            moves.push(Move::new(pos, positions_d_r[i], 0b0));
        }
    }

    if straight {
        let positions_r = get_available_slide_pos(bitboard, pos, 0, 1, depth);

        for i in 0..positions_r.len() {
            if (i == positions_r.len() - 1) && bitboard.occupied(positions_r[i]) {
                if opponent_bitboard.occupied(positions_r[i]) {
                    moves.push(Move::new(pos, positions_r[i], CAPTURE));
                }
                break;
            }
            moves.push(Move::new(pos, positions_r[i], 0b0));
        }

        let positions_l = get_available_slide_pos(bitboard, pos, 0, -1, depth);

        for i in 0..positions_l.len() {
            if (i == positions_l.len() - 1) && bitboard.occupied(positions_l[i]) {
                if opponent_bitboard.occupied(positions_l[i]) {
                    moves.push(Move::new(pos, positions_l[i], CAPTURE));
                }
                break;
            }
            moves.push(Move::new(pos, positions_l[i], 0b0));
        }

        let positions_u = get_available_slide_pos(bitboard, pos, 1, 0, depth);

        for i in 0..positions_u.len() {
            if (i == positions_u.len() - 1) && bitboard.occupied(positions_u[i]) {
                if opponent_bitboard.occupied(positions_u[i]) {
                    moves.push(Move::new(pos, positions_u[i], CAPTURE));
                }
                break;
            }
            moves.push(Move::new(pos, positions_u[i], 0b0));
        }

        let positions_d = get_available_slide_pos(bitboard, pos, -1, 0, depth);

        for i in 0..positions_d.len() {
            if (i == positions_d.len() - 1) && bitboard.occupied(positions_d[i]) {
                if opponent_bitboard.occupied(positions_d[i]) {
                    moves.push(Move::new(pos, positions_d[i], CAPTURE));
                }
                break;
            }
            moves.push(Move::new(pos, positions_d[i], 0b0));
        }
    }

    moves
}

#[cfg(test)]
mod test {
    use std::default;

    use crate::board::board_utils::file_and_rank_to_index;

    use super::*;

    #[test]
    pub fn generate_rook_moves_with_one_move() {
        let board = BoardState::from_fen(
            &"rnbqkbnr/ppppppp1/7p/8/8/7P/PPPPPPP1/RNBQKBNR w KQkq - 0 2".into(),
        );
        let rook_moves = generate_rook_moves(board.bitboard, Bitboard::default(), 0);

        assert_eq!(rook_moves.len(), 1);
    }

    #[test]
    pub fn generate_rook_moves_with_zero_moves() {
        let board = BoardState::from_fen(
            &"rnbqkbnr/ppppppp1/7p/8/8/7P/PPPPPPP1/RNBQKBNR w KQkq - 0 2".into(),
        );
        let rook_moves = generate_rook_moves(board.bitboard, Bitboard::default(), 7);
        assert_eq!(rook_moves.len(), 0);
    }

    #[test]
    pub fn generate_bishop_moves_b2_pawn_opening() {
        let board = BoardState::from_fen(
            &"r1bqkbnr/pppppppp/n7/8/1P6/8/P1PPPPPP/RNBQKBNR w KQkq - 0 1".into(),
        );
        let moves = generate_bishop_moves(board.bitboard, Bitboard::default(), 5);
        assert_eq!(moves.len(), 2, "{moves:?}");
    }

    #[test]
    pub fn generate_pawn_moves_en_passant_a5_white_taking_right() {
        let board = BoardState::from_fen(
            &"rnbqkbnr/2pppppp/p7/Pp6/8/8/1PPPPPPP/RNBQKBNR w KQkq b6 0 1".into(),
        );
        let moves = generate_pawn_moves(
            board.bitboard,
            false,
            board.black_bitboard,
            file_and_rank_to_index(0, 4),
            board.ep_file,
        );

        assert_eq!(moves.len(), 1);
        assert_eq!(
            moves.get(0).unwrap(),
            &Move::new(39, file_and_rank_to_index(1, 5), EP_CAPTURE)
        );
    }

    #[test]
    pub fn generate_pawn_moves_en_passant_e5_white_taking_left() {
        let board = BoardState::from_fen(
            &"rnbqkbnr/pp2pppp/2p5/3pP3/8/8/PPPP1PPP/RNBQKBNR w KQkq d6 0 1".into(),
        );
        let pos = file_and_rank_to_index(4, 4);
        let moves = generate_pawn_moves(
            board.bitboard,
            false,
            board.black_bitboard,
            pos,
            board.ep_file,
        );

        assert_eq!(moves.len(), 2, "{moves:?}");
        assert_eq!(
            moves.get(0).unwrap(),
            &Move::new(pos, file_and_rank_to_index(4, 5), 0b0)
        );
        assert_eq!(
            moves.get(1).unwrap(),
            &Move::new(pos, file_and_rank_to_index(3, 5), EP_CAPTURE)
        );
    }

    #[test]
    pub fn generate_pawn_moves_en_passant_b5_white_taking_left_to_a_file() {
        let board = BoardState::from_fen(
            &"rnbqkbnr/2pppppp/1p6/pP6/8/8/P1PPPPPP/RNBQKBNR w KQkq a6 0 1".into(),
        );
        let pos = file_and_rank_to_index(1, 4);
        let moves = generate_pawn_moves(
            board.bitboard,
            false,
            board.black_bitboard,
            pos,
            board.ep_file,
        );

        assert_eq!(moves.len(), 1, "{moves:?}");
        assert_eq!(
            moves.get(0).unwrap(),
            &Move::new(pos, file_and_rank_to_index(0, 5), EP_CAPTURE)
        );
    }

    #[test]
    pub fn generate_pawn_moves_b5_white_taking_non_en_passant_in_a_file() {
        let board = BoardState::from_fen(
            &"rnbqkbnr/2pppppp/1p6/pP6/8/6P1/P1PPPP1P/RNBQKBNR w KQkq - 0 1".into(),
        );
        let pos = file_and_rank_to_index(1, 4);

        let moves = generate_pawn_moves(
            board.bitboard,
            false,
            board.black_bitboard,
            pos,
            board.ep_file,
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
            true,
            board.white_bitboard,
            file_and_rank_to_index(7, 3),
            board.ep_file,
        );

        assert_eq!(moves.len(), 1);
        assert_eq!(
            moves.get(0).unwrap(),
            &Move::new(
                file_and_rank_to_index(7, 3),
                file_and_rank_to_index(6, 2),
                EP_CAPTURE
            )
        );
    }

    #[test]
    pub fn generate_pawn_moves_no_en_passant_black_b_pawn_opening() {
        let board = BoardState::from_fen(
            &"rnbqkbnr/pppppppp/8/8/P7/8/1PPPPPPP/RNBQKBNR b KQkq a3 0 1".into(),
        );
        let moves = generate_pawn_moves(
            board.bitboard,
            true,
            board.white_bitboard,
            file_and_rank_to_index(1, 6),
            board.ep_file,
        );

        assert_eq!(moves.len(), 2);
        assert_eq!(
            moves.get(0).unwrap(),
            &Move::new(
                file_and_rank_to_index(1, 6),
                file_and_rank_to_index(1, 5),
                0b0
            )
        );
        assert_eq!(
            moves.get(1).unwrap(),
            &Move::new(
                file_and_rank_to_index(1, 6),
                file_and_rank_to_index(1, 4),
                DOUBLE_PAWN_PUSH
            )
        );
    }

    #[test]
    pub fn generate_knight_moves_case_1() {
        let board = BoardState::from_fen(&"8/8/4p3/1k6/3N4/8/8/8 w - - 0 1".into());

        let moves = generate_knight_moves(
            board.bitboard,
            board.black_bitboard,
            file_and_rank_to_index(3, 3),
        );

        assert_eq!(moves.len(), 8);
    }
}
