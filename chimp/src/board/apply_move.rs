use super::{
    bitboard::BitboardExtensions,
    board_utils::{get_rank, get_file, file_and_rank_to_index},
    piece::{Piece, PieceType},
    r#move::{Move, MoveFunctions},
    state::{BoardState, BoardStateFlags, BoardStateFlagsTrait},
};
use crate::shared::{
    BISHOP_INDEX, BLACK_MASK, KING_INDEX, KNIGHT_INDEX, PAWN_INDEX,
    PIECE_MASK, QUEEN_INDEX, ROOK_INDEX,
};

impl BoardState {
    pub fn apply_move(&self, m: Move) -> BoardState {
        let mut bitboard = self.bitboard;
        let mut pieces = self.pieces;
        let mut flags: BoardStateFlags = self.flags;
        let mut ep_file: u8 = u8::MAX;
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
        let mut primary_piece: Piece = Piece::default();

        if m.is_castling() {
            let (rook_from_index, rook_to_index) = if m.is_king_castling() {
                (from_index - 3, to_index + 1)
            } else {
                (from_index + 4, to_index - 1)
            };
            // pick-up king
            let (king_piece, new_pieces) = pieces.pickup(bitboard, from_index);
            primary_piece = king_piece;
            bitboard = bitboard.flip(from_index);
            friendly_bitboard = friendly_bitboard.flip(from_index);

            // place king
            pieces = new_pieces.place(bitboard, to_index, king_piece);
            bitboard = bitboard.set(to_index);
            friendly_bitboard = friendly_bitboard.set(to_index);

            // pickup rook
            let (rook_piece, new_pieces) = pieces.pickup(bitboard, rook_from_index);
            bitboard = bitboard.flip(rook_from_index);
            friendly_bitboard = friendly_bitboard.flip(rook_from_index);

            // place rook
            pieces = new_pieces.place(bitboard, rook_to_index, rook_piece);
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
            let (removed_piece, mut new_pieces) = pieces.pickup(bitboard, from_index);
            bitboard = bitboard.flip(from_index);
            friendly_bitboard = friendly_bitboard.flip(from_index);

            primary_piece = Piece::new_coloured(match m.flags() {
                8 | 12 => KNIGHT_INDEX.into(),
                9 | 13 => BISHOP_INDEX.into(),
                10 | 14 => ROOK_INDEX.into(),
                11 | 15 => QUEEN_INDEX.into(),
                _ => panic!("Unknown promotion"),
            }, black_move);

            if capture {
                new_pieces = new_pieces.remove(bitboard, to_index);
                bitboard = bitboard.flip(to_index);
                opponent_bitboard = opponent_bitboard.flip(to_index);
            }

            pieces = new_pieces.place(bitboard, to_index, primary_piece);
            bitboard = bitboard.set(to_index);
            friendly_bitboard = friendly_bitboard.set(to_index);

            // a pawn promotion is always a half_moves reset
            half_moves = 0;
        } else {
            let (picked_up_piece, mut new_pieces) = pieces.pickup(bitboard, from_index);
            primary_piece = picked_up_piece;
            bitboard = bitboard.flip(from_index);
            friendly_bitboard = friendly_bitboard.flip(from_index);

            if m.is_ep_capture() {
                let ep_capture_index = file_and_rank_to_index(self.ep_file, get_rank(from_index));
                new_pieces = new_pieces.remove(bitboard, ep_capture_index);
                bitboard = bitboard.flip(ep_capture_index);
                opponent_bitboard = opponent_bitboard.flip(ep_capture_index);
            } else if capture {
                new_pieces = new_pieces.remove(bitboard, to_index);
                bitboard = bitboard.flip(to_index);
                opponent_bitboard = opponent_bitboard.flip(to_index);
            }

            pieces = new_pieces.place(bitboard, to_index, picked_up_piece);
            bitboard = bitboard.set(to_index);
            friendly_bitboard = friendly_bitboard.set(to_index);

            // Double Pawn Push
            if m.is_double_pawn_push() {
                ep_file = get_file(from_index);
            }

            // Half moves
            if primary_piece.is(PieceType::Pawn) || capture {
                half_moves = 0;
            } else {
                half_moves = half_moves + 1;
            }

            // King move castling clear
            if primary_piece.is(PieceType::King) {
                if black_move {
                    flags = flags & 0b1110_0111;
                } else {
                    flags = flags & 0b1111_1001;
                }
            }

            // Rook move castling clear
            if primary_piece.is(PieceType::Rook) {
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
        let piece_count = bitboard.count_occupied();

        if piece_count > 32 {
            panic!(
                "Move {} leading to >32 pieces ({piece_count}) (isCastling:{}) fen: {}",
                m.uci(),
                m.is_castling(),
                &self.to_fen()
            );
        }

        // King Position
        if primary_piece.is(PieceType::King) {
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
            ep_file,
            half_moves,
            full_moves,
            piece_count,
            white_king_index,
            black_king_index,
        }
    }
}
