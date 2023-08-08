use super::state::BoardState;
use crate::board::piece::*;

impl BoardState {
    pub fn apply_move(&self, m: u16) -> BoardState {
        let mut bitboard: u64 = self.bitboard;
        let mut pieces: u128 = self.pieces;
        let mut flags: u8 = self.flags;
        let half_moves: u8 = self.half_moves;

        let from_file = m >> 13;
        let from_rank = m >> 10 & 0b111;
        let to_file = m >> 7 & 0b111;
        let to_rank = m >> 4 & 0b111;

        let capture = false;
        let to_piece_index = 0b0u16;
        let from_piece_index = 0b0u16;

        // if capture {
        //     pieces = remove_piece_from_pieces_list(&pieces, to_piece_index);
        // }

        // let piece = get_piece_from_piece_list(&pieces, from_piece_index);
        // pieces = remove_piece_from_piece_list(&pieces, from_piece_index);

        // pieces = insert_piece_in_piece_list_at_position(&pieces, to_piece_index);

        let from_index: usize = (((from_file) * 8) + (7 - from_rank)).try_into().unwrap();
        let to_index: usize = (((to_file) * 8) + (7 - to_rank)).try_into().unwrap();

        println!("from_file {from_file} from_rank {from_rank} {from_index} t_f {to_file} t_r {to_rank} {to_index}");

        bitboard = bitboard ^ (1 << from_index);
        bitboard = bitboard | (1 << to_index);

        // Turn
        flags = flags ^ 0b1;

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
