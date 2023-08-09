use super::state::BoardState;
use crate::{
    board::piece::*,
    shared::{binary_utils::BinaryUtils, BLACK_MASK, PAWN_INDEX, PIECE_MASK},
};

impl BoardState {
    pub fn apply_move(&self, m: u16) -> BoardState {
        let mut bitboard: u64 = self.bitboard;
        let mut pieces: u128 = self.pieces;
        let mut flags: u8 = self.flags;
        let mut half_moves: u8 = self.half_moves;

        let from_file = m >> 13;
        let from_rank = m >> 10 & 0b111;
        let to_file = m >> 7 & 0b111;
        let to_rank = m >> 4 & 0b111;

        let capture = is_capture(m);

        let from_index: usize = (((from_file) * 8) + (7 - from_rank)).try_into().unwrap();
        let to_index: usize = (((to_file) * 8) + (7 - to_rank)).try_into().unwrap();

        let (picked_up_piece, mut new_pieces) =
            pickup_piece(pieces, bitboard, from_file, from_rank);
        bitboard = bitboard ^ (1 << from_index);

        if capture {
            new_pieces = remove_piece(new_pieces, bitboard, to_file, to_rank);
        }

        pieces = place_piece(new_pieces, bitboard, to_file, to_rank, picked_up_piece);
        bitboard = bitboard | (1 << to_index);

        // Turn
        flags = flags ^ 0b1;

        // Double Pawn Push
        let piece_u8: u8 = picked_up_piece.try_into().unwrap();
        if is_double_pawn_push(piece_u8, from_file, to_file) {
            flags = flags & 0b00011111;
            flags = flags ^ ((from_rank as u8) << 5);
        }

        // Half moves
        if (piece_u8 & PIECE_MASK) == PAWN_INDEX {
            half_moves = 0;
        } else {
            half_moves = half_moves + 1;
        }

        // Full moves
        let full_moves: u32 = self.full_moves + if (flags & 1) == 1 { 1 } else { 0 };

        BoardState {
            bitboard,
            pieces,
            flags,
            half_moves,
            full_moves,
        }
    }
}

fn is_double_pawn_push(picked_up_piece: u8, from_file: u16, to_file: u16) -> bool {
    if (picked_up_piece & PIECE_MASK) != PAWN_INDEX {
        return false;
    }

    if picked_up_piece & BLACK_MASK > 0 {
        return from_file == 6 && to_file == 4;
    }

    return from_file == 1 && to_file == 3;
}

fn is_capture(m: u16) -> bool {
    m >> 2 & 0b1 > 0
}

fn pickup_piece(pieces: u128, bitboard: u64, file: u16, rank: u16) -> (u128, u128) {
    let pos: u32 = ((file * 8) + (7 - rank)).into();
    let bitboard_relevant = bitboard & (u64::pow(2, pos) - 1);
    let bitboard_pos: usize = bitboard_relevant.count_ones() as usize;
    let piece = pieces.copy_b(bitboard_pos * 4, 4);
    let board = pieces.remove_b(bitboard_pos * 4, 4);
    (piece, board)
}

fn remove_piece(pieces: u128, bitboard: u64, file: u16, rank: u16) -> u128 {
    let pos: u32 = ((file * 8) + (7 - rank)).into();
    let bitboard_relevant = bitboard & (u64::pow(2, pos) - 1);
    let bitboard_pos: usize = bitboard_relevant.count_ones() as usize;
    let board = pieces.remove_b(bitboard_pos * 4, 4);
    board
}

fn place_piece(pieces: u128, bitboard: u64, file: u16, rank: u16, piece: u128) -> u128 {
    let pos: u32 = ((file * 8) + (7 - rank)).into();
    let bitboard_relevant = bitboard & (u64::pow(2, pos) - 1);
    let bitboard_pos = (bitboard_relevant.count_ones()) as usize;
    pieces.insert_b(bitboard_pos * 4, piece, 4)
}

pub fn standard_notation_to_move(std_notation: &str) -> u16 {
    let capture = std_notation.chars().nth(2).unwrap() == 'x';

    let mut result: u16 = 0;

    let from_rank_char = std_notation.chars().nth(0).unwrap();
    result = result | (file_from_char(from_rank_char) << 10);
    let from_file_char: u16 = std_notation.chars().nth(1).unwrap().to_digit(8).unwrap() as u16;
    result = result | ((from_file_char - 1) << 13);

    let start_pos = if capture { 3 } else { 2 };
    let to_rank_char = std_notation.chars().nth(start_pos).unwrap();
    result = result | (file_from_char(to_rank_char) << 4);
    let to_file_char: u16 = std_notation
        .chars()
        .nth(start_pos + 1)
        .unwrap()
        .to_digit(8)
        .unwrap() as u16;
    result = result | ((to_file_char - 1) << 7);

    if capture {
        result = result | 0b100;
    }

    result
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    pub fn standard_notation_to_move_b1b2() {
        let r = standard_notation_to_move(&"b1b2".to_string());
        //   from file 000 from rank 001  to file 001 to rank 001 promotion 0 capture 0 specials 0 0 => 0000010010010000
        assert_eq!(r, 0b0000010010010000);
    }

    #[test]
    pub fn standard_notation_to_move_e2e4() {
        let r = standard_notation_to_move(&"e2e4".to_string());
        // from file 001 from rank 100 to file 011 to rank 100 promotion 0 capture 0 specials 0 0 => 0011000111000000
        assert_eq!(r, 0b0011000111000000);
    }
}
