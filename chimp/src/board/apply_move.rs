use super::{
    bitboard::{Bitboard, BitboardExtensions},
    board_utils::{get_file, get_rank, rank_and_file_to_index},
    r#move::{Move, MoveFunctions},
    state::{BoardState, BoardStateFlags, BoardStateFlagsTrait},
};
use crate::shared::{
    binary_utils::BinaryUtils, BISHOP_INDEX, BLACK_MASK, KING_INDEX, KNIGHT_INDEX, PAWN_INDEX,
    PIECE_MASK, QUEEN_INDEX, ROOK_INDEX,
};

impl BoardState {
    pub fn apply_move(&self, m: Move) -> BoardState {
        let mut bitboard = self.bitboard;
        let mut pieces: u128 = self.pieces;
        let mut flags: BoardStateFlags = self.flags;
        let mut ep_rank: u8 = u8::MAX;
        let mut half_moves: u8 = self.half_moves;
        let mut white_king_index: u8 = self.white_king_index;
        let mut black_king_index: u8 = self.black_king_index;

        let black_move: bool = self.flags.is_black_turn();

        let (mut friendly_bitboard, mut opponent_bitboard) = if black_move {
            (self.black_bitboard, self.white_bitboard)
        } else {
            (self.white_bitboard, self.black_bitboard)
        };

        let from_index: u8 = m.from();
        let to_index: u8 = m.to();

        let capture = m.is_capture();
        let mut primary_piece: u128 = 0;

        if m.is_castling() {
            let (rook_from_index, rook_to_index) = if m.is_king_castling() {
                (from_index - 3, to_index + 1)
            } else {
                (from_index + 4, to_index - 1)
            };
            // pick-up king
            let (king_piece, new_pieces) = pickup_piece(pieces, bitboard, from_index);
            primary_piece = king_piece;
            bitboard = bitboard.flip(from_index);
            friendly_bitboard = friendly_bitboard.flip(from_index);

            // place king
            pieces = place_piece(new_pieces, bitboard, to_index, king_piece);
            bitboard = bitboard.set(to_index);
            friendly_bitboard = friendly_bitboard.set(to_index);

            // pickup rook
            let (rook_piece, new_pieces) = pickup_piece(pieces, bitboard, rook_from_index);
            bitboard = bitboard.flip(rook_from_index);
            friendly_bitboard = friendly_bitboard.flip(rook_from_index);

            // place rook
            pieces = place_piece(new_pieces, bitboard, rook_to_index, rook_piece);
            bitboard = bitboard.set(rook_to_index);
            friendly_bitboard = friendly_bitboard.set(rook_to_index);

            // Clear castling rights
            if black_move {
                flags = flags & 0b10_0111;
            } else {
                flags = flags & 0b11_1001;
            }

            // Increment half-moves
            half_moves += 1;
        } else if m.is_promotion() {
            let (removed_piece, mut new_pieces) = pickup_piece(pieces, bitboard, from_index);
            bitboard = bitboard.flip(from_index);
            friendly_bitboard = friendly_bitboard.flip(from_index);

            primary_piece = match m.flags() {
                8 | 12 => KNIGHT_INDEX.into(),
                9 | 13 => BISHOP_INDEX.into(),
                10 | 14 => ROOK_INDEX.into(),
                11 | 15 => QUEEN_INDEX.into(),
                _ => panic!("Unknown promotion"),
            };

            if black_move {
                primary_piece ^= BLACK_MASK as u128;
            }

            if capture {
                new_pieces = remove_piece(new_pieces, bitboard, to_index);
                bitboard = bitboard.flip(to_index);
                opponent_bitboard = opponent_bitboard.flip(to_index);
            }

            pieces = place_piece(new_pieces, bitboard, to_index, primary_piece);
            bitboard = bitboard.set(to_index);
            friendly_bitboard = friendly_bitboard.set(to_index);

            // a pawn promotion is always a half_moves reset
            half_moves = 0;
        } else {
            let (picked_up_piece, mut new_pieces) = pickup_piece(pieces, bitboard, from_index);
            primary_piece = picked_up_piece;
            bitboard = bitboard.flip(from_index);
            friendly_bitboard = friendly_bitboard.flip(from_index);

            if m.is_ep_capture() {
                let ep_capture_index = rank_and_file_to_index(self.ep_rank, get_file(from_index));
                new_pieces = remove_piece(new_pieces, bitboard, ep_capture_index);
                bitboard = bitboard.flip(ep_capture_index);
                opponent_bitboard = opponent_bitboard.flip(ep_capture_index);
            } else if capture {
                new_pieces = remove_piece(new_pieces, bitboard, to_index);
                bitboard = bitboard.flip(to_index);
                opponent_bitboard = opponent_bitboard.flip(to_index);
            }

            pieces = place_piece(new_pieces, bitboard, to_index, picked_up_piece);
            bitboard = bitboard.set(to_index);
            friendly_bitboard = friendly_bitboard.set(to_index);

            // Double Pawn Push
            let piece_u8: u8 = picked_up_piece.try_into().unwrap();
            if m.is_double_pawn_push() {
                ep_rank = get_rank(from_index);
            }

            // Half moves
            if (piece_u8 & PIECE_MASK) == PAWN_INDEX || capture {
                half_moves = 0;
            } else {
                half_moves = half_moves + 1;
            }

            // King move castling clear
            if (primary_piece as u8 & PIECE_MASK) == KING_INDEX {
                if black_move {
                    flags = flags & 0b1110_0111;
                } else {
                    flags = flags & 0b1111_1001;
                }
            }

            // Rook move castling clear
            if (primary_piece as u8 & PIECE_MASK) == ROOK_INDEX {
                if black_move {
                    if from_index == 63 {
                        flags = flags & 0b1110_1111;
                    } else if from_index == 56 {
                        flags = flags & 0b1111_0111;
                    }
                } else {
                    if from_index == 7 {
                        flags = flags & 0b1111_1011;
                    } else if from_index == 0 {
                        flags = flags & 0b1111_1101;
                    };
                }
            }
        }

        // Rook taken clear castling
        if capture {
            match to_index {
                63 => flags = flags & 0b111_01_11_1, // Clear black queenside castling,
                56 => flags = flags & 0b111_10_11_1, // Clear black kingside castling,
                7 => flags = flags & 0b111_11_01_1,  // Clear white queenside castling,
                0 => flags = flags & 0b111_11_10_1,  // Clear white kingside castling
                _ => {}
            }
        }

        // Turn
        flags.alternate_turn();

        // Full moves
        let full_moves: u32 = self.full_moves + if !flags.is_black_turn() { 1 } else { 0 };

        // Piece Count
        let piece_count = bitboard.count_ones() as u8;

        if piece_count > 32 {
            panic!(
                "Move {} leading to >32 pieces ({piece_count}) (isCastling:{}) fen: {}",
                m.uci(),
                m.is_castling(),
                &self.to_fen()
            );
        }

        // King Position
        if (primary_piece as u8 & PIECE_MASK) == KING_INDEX {
            if black_move {
                black_king_index = to_index;
            } else {
                white_king_index = to_index;
            }
        }

        let (white_bitboard, black_bitboard) = if black_move {
            (opponent_bitboard, friendly_bitboard)
        } else {
            (friendly_bitboard, opponent_bitboard)
        };

        BoardState {
            bitboard,
            white_bitboard,
            black_bitboard,
            pieces,
            flags,
            ep_rank,
            half_moves,
            full_moves,
            piece_count,
            white_king_index,
            black_king_index,
        }
    }
}

fn pickup_piece(pieces: u128, bitboard: Bitboard, position_index: u8) -> (u128, u128) {
    let piece_index = bitboard.position_to_piece_index(position_index);
    let piece = pieces.copy_b(piece_index * 4, 4);
    let board = pieces.remove_b(piece_index * 4, 4);
    (piece, board)
}

fn remove_piece(pieces: u128, bitboard: u64, position_index: u8) -> u128 {
    let piece_index = bitboard.position_to_piece_index(position_index);
    let board = pieces.remove_b(piece_index * 4, 4);
    board
}

fn place_piece(pieces: u128, bitboard: u64, position_index: u8, piece: u128) -> u128 {
    let piece_index = bitboard.position_to_piece_index(position_index);
    pieces.insert_b(piece_index * 4, piece, 4)
}
