use super::{
    board_utils::get_rank,
    move_utils::{is_capture, is_double_pawn_push},
    state::BoardState,
};
use crate::shared::{binary_utils::BinaryUtils, BLACK_MASK, KING_INDEX, PAWN_INDEX, PIECE_MASK};

impl BoardState {
    pub fn apply_move(&self, m: u16) -> BoardState {
        let mut bitboard: u64 = self.bitboard;
        let mut white_bitboard: u64 = self.white_bitboard;
        let mut black_bitboard: u64 = self.black_bitboard;
        let mut pieces: u128 = self.pieces;
        let mut flags = self.flags;
        let mut ep_rank: u8 = 0;
        let mut half_moves: u8 = self.half_moves;
        let mut white_king_index: u8 = self.white_king_index;
        let mut black_king_index: u8 = self.black_king_index;

        let from_index: u8 = (m >> 10).try_into().unwrap();
        let to_index: u8 = (m >> 4 & 0b111111).try_into().unwrap();

        let capture = is_capture(m);

        let (picked_up_piece, mut new_pieces) = pickup_piece(pieces, bitboard, from_index);
        let black_move = (picked_up_piece as u8) & BLACK_MASK > 0;
        bitboard = bitboard ^ (1 << from_index);
        if black_move {
            black_bitboard = black_bitboard ^ (1 << from_index);
        } else {
            white_bitboard = white_bitboard ^ (1 << from_index);
        }

        if capture {
            new_pieces = remove_piece(new_pieces, bitboard, to_index);
        }

        pieces = place_piece(new_pieces, bitboard, to_index, picked_up_piece);
        bitboard = bitboard | (1 << to_index);
        if black_move {
            black_bitboard = black_bitboard | (1 << to_index);
        } else {
            white_bitboard = white_bitboard | (1 << to_index);
        }

        // Turn
        flags = flags ^ 0b1;

        // Double Pawn Push
        let piece_u8: u8 = picked_up_piece.try_into().unwrap();
        flags = flags & 0b11111; // reset ep flag
        if is_double_pawn_push(piece_u8, from_index, to_index) {
            flags = flags | 0b100000;
            ep_rank = get_rank(from_index);
        }

        // Half moves
        if (piece_u8 & PIECE_MASK) == PAWN_INDEX {
            half_moves = 0;
        } else {
            half_moves = half_moves + 1;
        }

        // Full moves
        let full_moves: u32 = self.full_moves + if (flags & 1) == 1 { 1 } else { 0 };

        // Piece Count
        let piece_count = bitboard.count_ones() as u8;

        // King Position
        if (piece_u8 & PIECE_MASK) == KING_INDEX {
            if black_move {
                black_king_index = to_index;
            } else {
                white_king_index = to_index;
            }
        }

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

fn pickup_piece(pieces: u128, bitboard: u64, index: u8) -> (u128, u128) {
    let bitboard_relevant = bitboard & (u64::pow(2, index.into()) - 1);
    let bitboard_pos: usize = bitboard_relevant.count_ones() as usize;
    let piece = pieces.copy_b(bitboard_pos * 4, 4);
    let board = pieces.remove_b(bitboard_pos * 4, 4);
    (piece, board)
}

fn remove_piece(pieces: u128, bitboard: u64, index: u8) -> u128 {
    let bitboard_relevant = bitboard & (u64::pow(2, index.into()) - 1);
    let bitboard_pos: usize = bitboard_relevant.count_ones() as usize;
    let board = pieces.remove_b(bitboard_pos * 4, 4);
    board
}

fn place_piece(pieces: u128, bitboard: u64, index: u8, piece: u128) -> u128 {
    let bitboard_relevant = bitboard & (u64::pow(2, index.into()) - 1);
    let bitboard_pos = (bitboard_relevant.count_ones()) as usize;
    pieces.insert_b(bitboard_pos * 4, piece, 4)
}
