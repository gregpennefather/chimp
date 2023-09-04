use crate::{
    board::{
        bitboard::Bitboard,
        king_position_analysis::{analyze_active_king_position, ThreatSource, ThreatType},
        position::Position,
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
};

use super::{
    move_data::{
        MoveData, BLACK_KING_CASTLING_CHECK, BLACK_KING_CASTLING_CLEARANCE,
        BLACK_PAWN_PROMOTION_RANK, BLACK_QUEEN_CASTLING_CHECK, BLACK_QUEEN_CASTLING_CLEARANCE,
        WHITE_KING_CASTLING_CHECK, WHITE_KING_CASTLING_CLEARANCE, WHITE_PAWN_PROMOTION_RANK,
        WHITE_QUEEN_CASTLING_CHECK, WHITE_QUEEN_CASTLING_CLEARANCE,
    },
    Move,
};

pub struct GeneratedMoves {
    moves: Vec<Move>,
    threat_board: u64,
    mobility_board: u64,
}

impl MoveData {
    pub fn generate_moves(&self, position: Position) -> Vec<Move> {
        let mut friendly_occupancy = if position.black_turn {
            position.black_bitboard
        } else {
            position.white_bitboard
        };
        let opponent_occupancy = if !position.black_turn {
            position.black_bitboard
        } else {
            position.white_bitboard
        };

        let king_analysis = analyze_active_king_position(position);

        println!("{:?}", king_analysis);

        let king_pos = if position.black_turn {
            position.black_king_position
        } else {
            position.white_king_position
        };

        let mut moves = self.generate_king_moves(
            king_pos,
            opponent_occupancy,
            position.occupancy,
            king_analysis.check,
            position.black_turn,
            king_analysis.threat_source,
        );

        println!("king_moves: {moves:?}");

        // In the event of double king check we can only avoid check by moving the king
        if king_analysis.double_check {
            return moves;
        }

        while friendly_occupancy != 0 {
            let piece_position = friendly_occupancy.trailing_zeros() as u8;
            moves.extend(
                self.generate_position_moves(
                    position,
                    piece_position,
                    position.black_turn,
                    position.ep_index,
                )
                .moves,
            );
            friendly_occupancy ^= 1 << piece_position;
        }

        moves
    }

    pub fn generate_moves_old(
        &self,
        position: Position,
    ) -> (Vec<Move>, Vec<Move>, u64, u64, u64, u64) {
        let friendly_occupancy = if position.black_turn {
            position.black_bitboard
        } else {
            position.white_bitboard
        };
        let opponent_occupancy = if !position.black_turn {
            position.black_bitboard
        } else {
            position.white_bitboard
        };

        let mut white_moves = Vec::new();
        let mut black_moves = Vec::new();
        let mut white_threatboard = 0;
        let mut black_threatboard = 0;
        let mut white_mobility = 0;
        let mut black_mobility = 0;

        for index in 0..64 {
            if position.occupancy.occupied(index) {
                let is_black = position.black_bitboard.occupied(index);
                let generated_moves =
                    self.generate_position_moves(position, index, is_black, position.ep_index);
                if is_black {
                    black_moves.extend(generated_moves.moves);
                    black_threatboard |= generated_moves.threat_board;
                    black_mobility |= generated_moves.mobility_board;
                } else {
                    white_moves.extend(generated_moves.moves);
                    white_threatboard |= generated_moves.threat_board;
                    white_mobility |= generated_moves.mobility_board;
                }
            }
        }

        if position.white_king_side_castling {
            match generate_king_castling_move(
                3,
                1,
                MF_KING_CASTLING,
                false,
                WHITE_KING_CASTLING_CLEARANCE,
                position.occupancy,
                WHITE_KING_CASTLING_CHECK,
                black_threatboard,
            ) {
                Some(generated_move) => {
                    white_moves.extend(generated_move.moves);
                    white_mobility |= generated_move.mobility_board;
                }
                None => {}
            }
        }
        if position.white_queen_side_castling {
            match generate_king_castling_move(
                3,
                5,
                MF_QUEEN_CASTLING,
                false,
                WHITE_QUEEN_CASTLING_CLEARANCE,
                position.occupancy,
                WHITE_QUEEN_CASTLING_CHECK,
                black_threatboard,
            ) {
                Some(generated_move) => {
                    white_moves.extend(generated_move.moves);
                    white_mobility |= generated_move.mobility_board;
                }
                None => {}
            }
        }

        if position.black_king_side_castling {
            match generate_king_castling_move(
                59,
                57,
                MF_KING_CASTLING,
                true,
                BLACK_KING_CASTLING_CLEARANCE,
                position.occupancy,
                BLACK_KING_CASTLING_CHECK,
                white_threatboard,
            ) {
                Some(generated_move) => {
                    black_moves.extend(generated_move.moves);
                    black_mobility |= generated_move.mobility_board;
                }
                None => {}
            }
        }

        if position.black_queen_side_castling {
            match generate_king_castling_move(
                59,
                61,
                MF_QUEEN_CASTLING,
                true,
                BLACK_QUEEN_CASTLING_CLEARANCE,
                position.occupancy,
                BLACK_QUEEN_CASTLING_CHECK,
                white_threatboard,
            ) {
                Some(generated_move) => {
                    black_moves.extend(generated_move.moves);
                    black_mobility |= generated_move.mobility_board;
                }
                None => {}
            }
        }

        (
            white_moves,
            black_moves,
            white_threatboard,
            black_threatboard,
            white_mobility,
            black_mobility,
        )
    }

    pub fn generate_position_moves(
        &self,
        position: Position,
        index: u8,
        is_black: bool,
        ep_index: u8,
    ) -> GeneratedMoves {
        let piece_type = position.get_piece_type_at_index(index);
        let opponent_occupancy = if is_black {
            position.white_bitboard
        } else {
            position.black_bitboard
        };

        match piece_type {
            piece_type::PieceType::None => panic!(
                "Unknown piece {piece_type:?} at position {index} : {}",
                position.to_fen()
            ),
            piece_type::PieceType::Pawn => {
                self.generate_pawn_moves(position, index, is_black, ep_index, opponent_occupancy)
            }
            piece_type::PieceType::Knight => {
                self.generate_knight_moves(index, opponent_occupancy, position.occupancy, is_black)
            }
            piece_type::PieceType::Bishop => self.generate_bishop_moves(
                index,
                position,
                opponent_occupancy,
                position.occupancy,
                is_black,
            ),
            piece_type::PieceType::Rook => self.generate_rook_moves(
                index,
                position,
                opponent_occupancy,
                position.occupancy,
                is_black,
            ),
            piece_type::PieceType::Queen => self.generate_queen_moves(
                index,
                position,
                opponent_occupancy,
                position.occupancy,
                is_black,
            ),
            piece_type::PieceType::King => self.generate_king_moves_old(
                index,
                opponent_occupancy,
                position.occupancy,
                is_black,
            ),
        }
    }

    fn generate_king_moves(
        &self,
        index: u8,
        opponent_occupancy: u64,
        occupancy: u64,
        in_check: bool,
        is_black: bool,
        threat: Option<ThreatSource>,
    ) -> Vec<Move> {
        println!("threat: {threat:?}");
        let mut moveboard = self.king_moves[index as usize];
        match threat {
            Some(threat) => {
                if (threat.threat_type == ThreatType::DiagonalSlide
                    || threat.threat_type == ThreatType::OrthogonalSlide)
                {
                    let threat_normal = get_direction_to_normalized(index, threat.from);
                    println!(
                        "flipping {} ({},{})",
                        (index as i8) + threat_normal,
                        index,
                        threat_normal
                    );
                    moveboard ^= 1 << (index as i8) + threat_normal;
                }
            }
            None => {}
        }
        moveboard_to_moves(
            index,
            piece_type::PieceType::King,
            moveboard,
            opponent_occupancy,
            occupancy,
            is_black,
        )
    }

    fn generate_king_moves_old(
        &self,
        index: u8,
        opponent_occupancy: u64,
        occupancy: u64,
        is_black: bool,
    ) -> GeneratedMoves {
        moveboard_to_moves_old(
            index,
            piece_type::PieceType::King,
            self.king_moves[index as usize],
            opponent_occupancy,
            occupancy,
            is_black,
        )
    }

    fn generate_queen_moves(
        &self,
        index: u8,
        position: Position,
        opponent_occupancy: u64,
        occupancy: u64,
        is_black: bool,
    ) -> GeneratedMoves {
        let moveboard = self
            .magic_bitboard_table
            .get_bishop_attacks(index as usize, position.occupancy.into())
            | self
                .magic_bitboard_table
                .get_rook_attacks(index as usize, position.occupancy.into());
        moveboard_to_moves_old(
            index,
            piece_type::PieceType::Queen,
            moveboard,
            opponent_occupancy,
            occupancy,
            is_black,
        )
    }

    fn generate_rook_moves(
        &self,
        index: u8,
        position: Position,
        opponent_occupancy: u64,
        occupancy: u64,
        is_black: bool,
    ) -> GeneratedMoves {
        let moveboard = self
            .magic_bitboard_table
            .get_rook_attacks(index as usize, position.occupancy.into());
        moveboard_to_moves_old(
            index,
            piece_type::PieceType::Rook,
            moveboard,
            opponent_occupancy,
            occupancy,
            is_black,
        )
    }

    fn generate_bishop_moves(
        &self,
        index: u8,
        position: Position,
        opponent_occupancy: u64,
        occupancy: u64,
        is_black: bool,
    ) -> GeneratedMoves {
        let moveboard = self
            .magic_bitboard_table
            .get_bishop_attacks(index as usize, position.occupancy.into());
        moveboard_to_moves_old(
            index,
            piece_type::PieceType::Bishop,
            moveboard,
            opponent_occupancy,
            occupancy,
            is_black,
        )
    }

    fn generate_knight_moves(
        &self,
        index: u8,
        opponent_occupancy: u64,
        occupancy: u64,
        is_black: bool,
    ) -> GeneratedMoves {
        moveboard_to_moves_old(
            index,
            piece_type::PieceType::Knight,
            self.knight_moves[index as usize],
            opponent_occupancy,
            occupancy,
            is_black,
        )
    }

    fn generate_pawn_moves(
        &self,
        position: Position,
        index: u8,
        is_black: bool,
        ep_index: u8,
        opponent_occupancy: u64,
    ) -> GeneratedMoves {
        let mut moves = Vec::new();
        let mut threat_board = 0;
        let mut mobility_board = 0;
        let mut moveboard = if is_black {
            self.black_pawn_moves[index as usize]
        } else {
            self.white_pawn_moves[index as usize]
        };

        // Black and whites move u64s have different orientations that we need to parse out. For white its fairly simple, for black we need to see if a
        // double pawn push is possible or not before determining what the normal move to_index is
        let (to_index, to_index_dpp, promotion_rank) = if is_black {
            let to_index_dpp = moveboard.trailing_zeros() as u8;
            if to_index_dpp >= 64 {
                println!("{}", position.to_fen());
            }
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
        mobility_board |= 1 << to_index;
        if !position.occupancy.occupied(to_index) {
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
                    mobility_board |= 1 << to_index_dpp;
                    if !position.occupancy.occupied(to_index_dpp) {
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
            self.black_pawn_captures[index as usize]
        } else {
            self.white_pawn_captures[index as usize]
        };

        // Can we capture right or EP capture right
        let first_capture_index = capture_board.trailing_zeros() as u8;
        threat_board |= 1 << first_capture_index;
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
            threat_board |= 1 << second_capture_index;
            if opponent_occupancy.occupied(second_capture_index) || second_capture_index == ep_index
            {
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

        GeneratedMoves {
            moves: moves,
            threat_board,
            mobility_board,
        }
    }
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
    opponent_threat_board: u64,
) -> Option<GeneratedMoves> {
    if (castling_clearance_board & occupancy == 0)
        && (castling_check_board & opponent_threat_board == 0)
    {
        let m = Move::new(
            from_index,
            to_index,
            castling_flag,
            piece_type::PieceType::King,
            is_black,
        );
        return Some(GeneratedMoves {
            moves: vec![m],
            threat_board: 0,
            mobility_board: 1 << to_index,
        });
    }
    None
}

fn moveboard_to_moves_old(
    from_index: u8,
    piece_type: piece_type::PieceType,
    moveboard: u64,
    opponent_occupancy: u64,
    occupancy: u64,
    is_black: bool,
) -> GeneratedMoves {
    let mut generated_moves = Vec::new();
    let mut m_b = moveboard;
    let mut to_index = 0;
    let mut threat_board = 0;
    let mut mobility_board = 0;
    while m_b != 0 {
        let lsb = m_b.trailing_zeros() as u8;
        to_index += lsb;
        threat_board |= 1 << to_index;
        if opponent_occupancy.occupied(to_index) {
            generated_moves.push(Move::new(
                from_index, to_index, MF_CAPTURE, piece_type, is_black,
            ));
        } else if !occupancy.occupied(to_index) {
            mobility_board |= 1 << to_index;
            generated_moves.push(Move::new(from_index, to_index, 0b0, piece_type, is_black));
        };
        to_index += 1; // Account for the move we just added
        m_b >>= lsb + 1;
    }

    GeneratedMoves {
        moves: generated_moves,
        threat_board,
        mobility_board,
    }
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

#[cfg(test)]
mod test {
    use crate::{board::position::Position, r#move::move_data::MoveData, MOVE_DATA};

    #[test]
    pub fn startpos_move_generation() {
        let position = Position::default();
        let moves = MOVE_DATA.generate_moves(position);
        assert_eq!(moves.len(), 20);
    }

    #[test]
    pub fn king_double_checked() {
        let position = Position::from_fen(
            "rnbqk1nr/pppp1pNp/2Pb4/8/1B6/4Q3/PP1PPPPP/RN2KB1R b KQkq - 0 1".into(),
        );
        let moves = MOVE_DATA.generate_moves(position);
        assert!(moves.len() <= 2);
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
