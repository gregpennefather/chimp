pub fn check_board_position(bitboard: u64, rank: u8, file: u8) -> bool {
    let index = (file * 8) + rank;
    let check_result = bitboard & (1 << index);
    check_result > 0
}