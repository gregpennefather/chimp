use crate::board::bitboard::Bitboard;

const NOT_H_FILE :u64 = 18374403900871474942;
const NOT_A_FILE :u64 = 9187201950435737471;

pub fn south_file_fill(mut pawns: u64) -> u64 {
    pawns |= pawns >> 8;
    pawns |= pawns >> 16;
    pawns |= pawns >> 24;
    pawns
}

pub fn north_file_fill(mut pawns: u64) -> u64 {
    pawns |= pawns << 8;
    pawns |= pawns << 16;
    pawns |= pawns << 24;
    pawns
}

pub fn file_fill(mut pawns: u64) -> u64 {
    north_file_fill(pawns) | south_file_fill(pawns)
}

pub fn get_pawn_attacks(is_black:bool, pawns: u64) -> u64 {
    if is_black {
        b_pawn_east_attacks(pawns) | b_pawn_west_attacks(pawns)
    } else {
        w_pawn_east_attacks(pawns) | w_pawn_west_attacks(pawns)
    }
}

fn w_pawn_east_attacks(pawns: u64) -> u64 {
    pawns << 9 & NOT_H_FILE
}

fn w_pawn_west_attacks(pawns: u64) -> u64 {
    pawns << 7 & NOT_A_FILE
}

fn b_pawn_east_attacks(pawns: u64) -> u64 {
    pawns >> 7 & NOT_H_FILE
}

fn b_pawn_west_attacks(pawns: u64) -> u64 {
    pawns >> 9 & NOT_A_FILE
}

#[cfg(test)]
mod test {
    use crate::{board::bitboard::Bitboard, shared::board_utils::index_from_coords, evaluation::subcategories::pawn::utils::north_file_fill};

    use super::south_file_fill;

    #[test]
    pub fn south_file_fill_pawns_on_a_b() {
        let pawns = (0 as u64).flip(index_from_coords("a2")).flip(index_from_coords("b3"));
        let r = south_file_fill(pawns);
        println!("{}", r.to_board_format());
        assert_eq!(r, (1<<index_from_coords("a1")).flip(index_from_coords("a2")).flip(index_from_coords("b1")).flip(index_from_coords("b2")).flip(index_from_coords("b3")));
    }

    #[test]
    pub fn north_file_fill_starting_white_pos() {
        let pawns = (0 as u64).set_rank(1);
        let r = north_file_fill(pawns);
        println!("{}", r.to_board_format());
        assert_eq!(r, !0.set_rank(0));
    }
}