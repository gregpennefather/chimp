use crate::{
    board::{bitboard::Bitboard, position::Position},
    r#move,
    shared::{
        board_utils::get_file,
        constants::{MF_CAPTURE, MF_DOUBLE_PAWN_PUSH, MF_EP_CAPTURE},
        piece_type,
    },
};

use super::{move_data::MoveData, Move};

pub struct GeneratedMoves {
    moves: Vec<Move>,
    threat_board: u64,
    mobility_board: u64,
}

impl MoveData {
    pub fn generate_moves(&self, position: Position) -> Vec<Move> {
        let mut psudolegal_moves = Vec::new();
        let mut white_threatboard = 0;
        let mut black_threatboard = 0;
        let mut white_mobility = 0;
        let mut black_mobility = 0;
        // let mut legal_moves = Vec::new();

        for index in 0..64 {
            if position.occupancy.occupied(index) {
                let is_black = position.black_bitboard.occupied(index);
                let generated_moves = self.generate_position_moves(
                    position,
                    index,
                    is_black,
                    position.ep_index,
                    position.white_king_side_castling,
                    position.white_queen_side_castling,
                    position.black_king_side_castling,
                    position.black_queen_side_castling,
                );
                if is_black {
                    black_threatboard |= generated_moves.threat_board;
                    black_mobility |= generated_moves.mobility_board;
                } else {
                    white_threatboard |= generated_moves.threat_board;
                    white_mobility |= generated_moves.mobility_board;
                }
                psudolegal_moves.extend(generated_moves.moves);
            }
        }

        // #TODO
        // for m in psudolegal_moves {
        //     if (m.is_black() == position.black_turn) {
        //         let move_segments = position.generate_move_segments(&m);
        //         let n_p = position.apply_segments(move_segments);
        //         if (position.black_turn && !n_p.black_in_check)
        //             || (!position.black_turn && !n_p.white_in_check)
        //         {
        //             legal_moves.push(m);
        //         }
        //     }
        // }

        psudolegal_moves
    }

    pub fn generate_position_moves(
        &self,
        position: Position,
        index: u8,
        is_black: bool,
        ep_index: u8,
        wkc: bool,
        wqc: bool,
        bkc: bool,
        bqc: bool,
    ) -> GeneratedMoves {
        let piece_type = position.get_piece_type_at_index(index);
        let opponent_occupancy = if is_black {
            position.white_bitboard
        } else {
            position.black_bitboard
        };

        match piece_type {
            piece_type::PieceType::None => panic!("Unknown piece"),
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
            piece_type::PieceType::King => self.generate_king_moves(
                position,
                index,
                opponent_occupancy,
                position.occupancy,
                is_black,
                wkc,
                wqc,
                bkc,
                bqc,
            ),
        }
    }

    fn generate_king_moves(
        &self,
        position: Position,
        index: u8,
        opponent_occupancy: Bitboard,
        occupancy: Bitboard,
        is_black: bool,
        wkc: bool,
        wqc: bool,
        bkc: bool,
        bqc: bool,
    ) -> GeneratedMoves {
        moveboard_to_moves(
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
        opponent_occupancy: Bitboard,
        occupancy: Bitboard,
        is_black: bool,
    ) -> GeneratedMoves {
        let moveboard = self
            .magic_bitboard_table
            .get_bishop_attacks(index as usize, position.occupancy.into())
            | self
                .magic_bitboard_table
                .get_rook_attacks(index as usize, position.occupancy.into());
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
        &self,
        index: u8,
        position: Position,
        opponent_occupancy: Bitboard,
        occupancy: Bitboard,
        is_black: bool,
    ) -> GeneratedMoves {
        let moveboard = self
            .magic_bitboard_table
            .get_rook_attacks(index as usize, position.occupancy.into());
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
        &self,
        index: u8,
        position: Position,
        opponent_occupancy: Bitboard,
        occupancy: Bitboard,
        is_black: bool,
    ) -> GeneratedMoves {
        let moveboard = self
            .magic_bitboard_table
            .get_bishop_attacks(index as usize, position.occupancy.into());
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
        &self,
        index: u8,
        opponent_occupancy: Bitboard,
        occupancy: Bitboard,
        is_black: bool,
    ) -> GeneratedMoves {
        moveboard_to_moves(
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
        opponent_occupancy: Bitboard,
    ) -> GeneratedMoves {
        let mut moves = Vec::new();
        let mut threat_board = 0;
        let mut mobility_board = 0;
        let mut moveboard = if is_black {
            self.black_pawn_moves[index as usize]
        } else {
            self.white_pawn_moves[index as usize]
        };

        let (to_index, to_index_dpp) = if is_black {
            let to_index_dpp = moveboard.trailing_zeros() as u8;
            moveboard ^= 1 << to_index_dpp;
            let to_index = moveboard.trailing_zeros() as u8;
            if to_index == 64 {
                (to_index_dpp, 64)
            } else {
                (to_index, to_index_dpp)
            }
        } else {
            let to_index = moveboard.trailing_zeros() as u8;
            moveboard ^= 1 << to_index;
            let to_index_dpp = moveboard.trailing_zeros() as u8;
            (to_index, to_index_dpp)
        };

        mobility_board |= 1 << to_index;
        if !position.occupancy.occupied(to_index) {
            moves.push(Move::new(
                index,
                to_index,
                0b0,
                piece_type::PieceType::Pawn,
                is_black,
            ));
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

        let mut capture_board = if is_black {
            self.black_pawn_captures[index as usize]
        } else {
            self.white_pawn_captures[index as usize]
        };

        let first_capture_index = capture_board.trailing_zeros() as u8;
        threat_board |= 1 << first_capture_index;
        if opponent_occupancy.occupied(first_capture_index) || first_capture_index == ep_index {
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

        capture_board ^= 1 << first_capture_index;
        let second_capture_index = capture_board.trailing_zeros() as u8;
        if second_capture_index != 64 {
            threat_board |= 1 << second_capture_index;
            if (opponent_occupancy.occupied(second_capture_index)
                || second_capture_index == ep_index)
            {
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

        GeneratedMoves {
            moves: moves,
            threat_board,
            mobility_board,
        }
    }
}

fn moveboard_to_moves(
    from_index: u8,
    piece_type: piece_type::PieceType,
    moveboard: u64,
    opponent_occupancy: Bitboard,
    occupancy: Bitboard,
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

#[cfg(test)]
mod test {

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
