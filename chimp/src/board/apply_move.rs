use super::{bitboard::BitboardExtensions, state::BoardState};
use crate::{
    board::piece::*,
    shared::{binary_utils::BinaryUtils, BLACK_MASK, KNIGHT_INDEX, PAWN_INDEX, PIECE_MASK},
};

impl BoardState {
    pub fn apply_move(&self, m: u16) -> BoardState {
        let mut bitboard: u64 = self.bitboard;
        let mut pieces: u128 = self.pieces;
        let mut flags: u8 = self.flags;
        let mut half_moves: u8 = self.half_moves;

        let from_index: u8 = (m >> 10).try_into().unwrap();
        let to_index: u8 = (m >> 4 & 0b111111).try_into().unwrap();

        let capture = is_capture(m);

        let (picked_up_piece, mut new_pieces) = pickup_piece(pieces, bitboard, from_index);
        bitboard = bitboard ^ (1 << from_index);

        if capture {
            new_pieces = remove_piece(new_pieces, bitboard, to_index);
        }

        pieces = place_piece(new_pieces, bitboard, to_index, picked_up_piece);
        bitboard = bitboard | (1 << to_index);

        // Turn
        flags = flags ^ 0b1;

        // Double Pawn Push
        let piece_u8: u8 = picked_up_piece.try_into().unwrap();
        if is_double_pawn_push(piece_u8, from_index, to_index) {
            flags = flags & 0b00011111;
            flags = flags ^ (((from_index % 8 as u8) + 1) << 5);
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

        BoardState {
            bitboard,
            pieces,
            flags,
            half_moves,
            full_moves,
            piece_count,
        }
    }

    pub fn generate_moves(&self) -> Vec<u16> {
        let mut moves = Vec::new();
        let white_turn = self.flags & 0b1 > 0;
        let mut piece_index = 0;
        let mut next_start_pos = 0;
        let mut start_count = 0;
        while piece_index < self.piece_count {
            let piece = get_piece_code(&self.pieces, piece_index);
            if is_white(piece) == white_turn {
                let position_index = get_position_index_from_piece_index(self.bitboard, 0, 0, piece_index);
                next_start_pos = position_index + 1;
                start_count = piece_index;
                moves.extend(self.generate_piece_moves(position_index, piece));
            }
            piece_index += 1;
        }

        moves
    }

    pub fn generate_piece_moves(&self, position_index: u8, piece: u8) -> Vec<u16> {
        let piece_code = piece & PIECE_MASK;
        match piece_code {
            PAWN_INDEX => generate_pawn_moves(self.bitboard, position_index, piece),
            KNIGHT_INDEX => generate_knight_moves(self.bitboard, position_index, piece),
            _ => vec![],
        }
    }
}

fn generate_knight_moves(bitboard: u64, pos: u8, piece: u8) -> Vec<u16> {
    let mut vec: Vec<_> = Vec::new();
    let rank = pos % 8;
    // U2R1 = +16-1 = 15
    if pos <= 48 && rank != 0 && !bitboard.occupied(pos + 15) {
        vec.push(build_move(pos, pos + 15, 0b0));
    }
    // U1R2 = +8-2 = 6
    if pos <= 55 && rank > 1 && !bitboard.occupied(pos + 6) {
        vec.push(build_move(pos, pos + 6, 0b0));
    }
    // D1R2 = -8-2 = -10
    if pos >= 10 && rank > 1 && !bitboard.occupied(pos - 10) {
        vec.push(build_move(pos, pos - 10, 0b0));
    }
    // D2R1 = -16-1 = -17
    if pos >= 17 && rank != 0 && !bitboard.occupied(pos - 17) {
        vec.push(build_move(pos, pos - 17, 0b0));
    }
    // D2L1 = -16+1 = -15
    if pos >= 15 && rank != 7 && !bitboard.occupied(pos - 15) {
        vec.push(build_move(pos, pos - 15, 0b0));
    }
    // D1L2 = -8+2 = -6
    if pos >= 6 && rank < 6 && !bitboard.occupied(pos - 6) {
        vec.push(build_move(pos, pos - 6, 0b0));
    }
    // U1L2 = 8+2 = 10
    if pos <= 53 && rank < 6 && !bitboard.occupied(pos + 10) {
        vec.push(build_move(pos, pos + 10, 0b0));
    }
    // U2L1 = 16+1 = 17
    if pos <= 46 && rank != 7 && !bitboard.occupied(pos + 17) {
        vec.push(build_move(pos, pos + 17, 0b0));
    }
    vec
}

fn generate_pawn_moves(bitboard: u64, position_index: u8, piece: u8) -> Vec<u16> {
    let mut vec: Vec<_> = Vec::new();
    let is_white = piece & BLACK_MASK == 0;

    if is_white {
        if !bitboard.occupied(position_index + 8) {
            vec.push(build_move(position_index, position_index + 8, 0b0));

            if position_index / 8 == 1 {
                if !bitboard.occupied(position_index + 16) {
                    vec.push(build_move(position_index, position_index + 16, 0b0));
                }
            }
        }
    } else {
        if !bitboard.occupied(position_index - 8) {
            vec.push(build_move(position_index, position_index - 8, 0b0));

            if position_index / 8 == 6 {
                if !bitboard.occupied(position_index - 16) {
                    vec.push(build_move(position_index, position_index - 16, 0b0));
                }
            }
        }
    }
    vec
}

fn build_move(from_index: u8, to_index: u8, flags: u16) -> u16 {
    let f: u16 = from_index.into();
    let t: u16 = to_index.into();
    let m: u16 = f << 10 | t << 4 | flags;
    m
}

fn is_white(piece: u8) -> bool {
    piece & BLACK_MASK == 0
}

fn get_position_index_from_piece_index(bitboard: u64, start_index: u8, start_count: u8, search_index: u8) -> u8 {
    let mut pos: u32 = start_index as u32;
    let mut count = start_count;

    while pos < 64 {
        if bitboard & u64::pow(2, pos) > 0 {
            count += 1;
            if count > search_index.into() {
                break;
            }
        }
        pos += 1;
    }
    pos.try_into().unwrap()
}

fn is_double_pawn_push(picked_up_piece: u8, from_index: u8, to_index: u8) -> bool {
    if (picked_up_piece & PIECE_MASK) != PAWN_INDEX {
        return false;
    }

    if picked_up_piece & BLACK_MASK > 0 {
        return from_index >= 48 && from_index <= 55 && to_index >= 32 && to_index <= 39;
    }

    return from_index >= 8 && from_index <= 15 && to_index >= 24 && to_index <= 31;
}

fn is_capture(m: u16) -> bool {
    m >> 2 & 0b1 > 0
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

pub fn standard_notation_to_move(std_notation: &str) -> u16 {
    let capture = std_notation.chars().nth(2).unwrap() == 'x';

    let mut result: u16 = 0;

    let from_rank_char = std_notation.chars().nth(0).unwrap();
    let from_rank = rank_from_char(from_rank_char);
    let from_file: u8 = std_notation.chars().nth(1).unwrap().to_digit(8).unwrap() as u8;

    let from_index = rank_and_file_to_index(from_rank, from_file - 1) as u16;
    result = result | (from_index << 10);

    let start_pos = if capture { 3 } else { 2 };
    let to_rank_char = std_notation.chars().nth(start_pos).unwrap();
    let to_rank = rank_from_char(to_rank_char);
    let to_file: u8 = std_notation
        .chars()
        .nth(start_pos + 1)
        .unwrap()
        .to_digit(8)
        .unwrap() as u8;

    let to_index = rank_and_file_to_index(to_rank, to_file - 1) as u16;
    result = result | (to_index << 4);

    if capture {
        result = result | 0b100;
    }

    result
}

fn rank_and_file_to_index(rank: u8, file: u8) -> u8 {
    ((file) * 8) + (7 - rank)
}

pub fn get_move_uci(m: u16) -> String {
    let from = (m >> 10) as u8;
    let to = (m >> 4 & 0b111111) as u8;
    format!("{}{}", get_friendly_name_for_index(from), get_friendly_name_for_index(to))
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    pub fn standard_notation_to_move_b1b2() {
        let r = standard_notation_to_move(&"b1b2".to_string());
        // b1 = 6th pos aka 0b000110
        // b2 = 14th pos ob001110
        //  promotion 0 capture 0 specials 0 0 = 0000
        //  => 0001100011100000
        println!("{r:#018b}");
        assert_eq!(r, 0b0001100011100000);
    }

    #[test]
    pub fn standard_notation_to_move_e2e4() {
        let r = standard_notation_to_move(&"e2e4".to_string());
        // e2 = 11th pos aka 001011
        // e4 = 27th pos aka 011011
        //  promotion 0 capture 0 specials 1 0 = 0000
        //  => 0010110110110000
        println!("{r:#018b}");
        assert_eq!(r, 0b0010110110110000);
    }

    #[test]
    pub fn build_move_e1_e2_pawn_push() {
        let from_index = 11; // 001011
        let to_index = 19; // 010011
        let r = build_move(from_index, to_index, 0b0u16);
        println!("{r:#018b}");
        assert_eq!(r, 0b0010110100110000);
    }

    #[test]
    pub fn build_move_a7_a8_pawn_push() {
        let from_index = 63; // 111111
        let to_index = 55; // 110111
        let r = build_move(from_index, to_index, 0b0u16);
        println!("{r:#018b}");
        assert_eq!(r, 0b1111111101110000);
    }
}
