use super::move_magic_bitboards::MagicTable;

pub const KING_CASTLING_CLEARANCE: u64 = 0b110;
pub const KING_CASTLING_CHECK: u64 = 0b1110;
pub const QUEEN_CASTLING_CLEARANCE: u64 = 0b1110000;
pub const QUEEN_CASTLING_CHECK: u64 = 0b111000;
pub const WHITE_PAWN_PROMOTION_RANK: u64 = 0b11111111 << 56;
pub const BLACK_PAWN_PROMOTION_RANK: u64 = 0b11111111;

pub fn horizontal_mask_generation() -> [u64; 64] {
    let mut result = [0; 64];
    for square in 0..64 {
        let rank = square / 8;
        let file = square % 8;

        let mut r = rank + 1;
        while r <= 7 {
            result[square] |= 1 << (file + r * 8);
            r += 1;
        }
        if rank > 0 {
            r = rank - 1;
            loop {
                result[square] |= 1 << (file + r * 8);
                if r == 0 {
                    break;
                }
                r -= 1;
            }
        }

        let mut f = file + 1;
        while f <= 7 {
            result[square] |= 1 << (f + rank * 8);
            f += 1;
        }

        if file > 0 {
            f = file - 1;
            while f >= 0 {
                result[square] |= 1 << (f + rank * 8);
                if f == 0 {
                    break;
                }
                f -= 1;
            }
        }
    }
    result
}

pub fn diagonal_mask_generation() -> [u64; 64] {
    let mut result = [0; 64];
    for square in 0..64 {
        let rank = square / 8;
        let file = square % 8;

        let mut r = rank + 1;
        let mut f = file + 1;
        while r <= 7 && f <= 7 {
            result[square] |= 1 << (f + (r * 8));
            r += 1;
            f += 1;
        }

        if file > 0 {
            r = rank + 1;
            f = file - 1;
            while r <= 7 {
                result[square] |= 1 << (f + (r * 8));
                r += 1;
                if f == 0 {
                    break;
                }
                f -= 1;
            }
        }
        if rank > 0 {
            r = rank - 1;
            f = file + 1;
            while f <= 7 {
                result[square] |= 1 << (f + (r * 8));
                if r == 0 {
                    break;
                }
                r -= 1;
                f += 1;
            }
        }
        if rank > 0 && file > 0 {
            r = rank - 1;
            f = file - 1;
            loop {
                result[square] |= 1 << (f + (r * 8));
                if r == 0 || f == 0 {
                    break;
                }
                r -= 1;
                f -= 1;
            }
        }
    }
    result
}

pub struct MoveData {
    pub magic_bitboard_table: MagicTable,
    pub white_pawn_moves: [u64; 64],
    pub black_pawn_moves: [u64; 64],
    pub white_pawn_captures: [u64; 64],
    pub black_pawn_captures: [u64; 64],
    pub knight_moves: [u64; 64],
    pub king_moves: [u64; 64],
    pub diagonal_threat_boards: [u64; 64],
    pub orthogonal_threat_board: [u64; 64],
}

impl MoveData {
    pub fn new() -> Self {
        let magic_bitboard_table = MagicTable::new();
        let (white_pawn_moves, black_pawn_moves) = generate_pawn_moves();
        let (white_pawn_captures, black_pawn_captures) = generate_pawn_captures();
        let knight_moves = generate_knight_moves();
        let king_moves = generate_king_moves();
        let diagonal_threat_boards = diagonal_mask_generation();
        let horizontal_threat_board = horizontal_mask_generation();

        Self {
            magic_bitboard_table,
            white_pawn_moves,
            white_pawn_captures,
            black_pawn_moves,
            black_pawn_captures,
            knight_moves,
            king_moves,
            diagonal_threat_boards,
            orthogonal_threat_board: horizontal_threat_board,
        }
    }
}

fn generate_pawn_moves() -> ([u64; 64], [u64; 64]) {
    let mut white_pawn_moves = [0; 64];
    let mut black_pawn_moves = [0; 64];

    for index in 8..56 {
        let rank = index / 8;
        white_pawn_moves[index] = 1 << (index + 8);
        if rank == 1 {
            white_pawn_moves[index] |= 1 << (index + 16);
        }
        black_pawn_moves[index] = 1 << (index - 8);

        if rank == 6 {
            black_pawn_moves[index] |= 1 << (index - 16);
        }
    }

    (white_pawn_moves, black_pawn_moves)
}

fn generate_pawn_captures() -> ([u64; 64], [u64; 64]) {
    let mut white_pawn_captures = [0; 64];
    let mut black_pawn_captures = [0; 64];

    for index in 8..56 {
        let file = index % 8;
        if file != 7 {
            white_pawn_captures[index] |= 1 << (index + 9);
            black_pawn_captures[index] |= 1 << (index - 7);
        }
        if file != 0 {
            white_pawn_captures[index] |= 1 << (index + 7);
            black_pawn_captures[index] |= 1 << (index - 9);
        }
    }

    (white_pawn_captures, black_pawn_captures)
}

fn generate_knight_moves() -> [u64; 64] {
    let mut knight_moves = [0; 64];

    for index in 0..64usize {
        let file = index % 8;
        if index <= 48 && file != 0 {
            let tar = index + 15;
            knight_moves[index] |= 1 << tar;
        }
        // U1R2 = +8-2 = 6
        if index <= 55 && file > 1 {
            let tar = index + 6;
            knight_moves[index] |= 1 << tar;
        }
        // D1R2 = -8-2 = -10
        if index >= 10 && file > 1 {
            let tar = index - 10;
            knight_moves[index] |= 1 << tar;
        }
        // D2R1 = -16-1 = -17
        if index >= 17 && file != 0 {
            let tar = index - 17;
            knight_moves[index] |= 1 << tar;
        }
        // D2L1 = -16+1 = -15
        if index >= 15 && file != 7 {
            let tar = index - 15;
            knight_moves[index] |= 1 << tar;
        }
        // D1L2 = -8+2 = -6
        if index >= 6 && file < 6 {
            let tar = index - 6;
            knight_moves[index] |= 1 << tar;
        }
        // U1L2 = 8+2 = 10
        if index <= 53 && file < 6 {
            let tar = index + 10;
            knight_moves[index] |= 1 << tar;
        }
        // U2L1 = 16+1 = 17
        if index <= 46 && file != 7 {
            let tar = index + 17;
            knight_moves[index] |= 1 << tar;
        }
    }

    knight_moves
}

fn generate_king_moves() -> [u64; 64] {
    let mut king_moves = [0; 64];

    for index in 0..64i32 {
        let file = index % 8;
        // Up Right
        let mut tar = index + 7;
        if tar < 64 && file != 0 {
            king_moves[index as usize] |= 1 << tar;
        }
        // Right
        tar = index - 1;
        if tar >= 0 && file != 0 {
            king_moves[index as usize] |= 1 << tar;
        }
        // Down Right
        tar = index - 9;
        if tar >= 0 && file != 0 {
            king_moves[index as usize] |= 1 << tar;
        }
        // Down
        tar = index - 8;
        if tar >= 0 {
            king_moves[index as usize] |= 1 << tar;
        }
        // Down Left
        tar = index - 7;
        if tar >= 0 && file != 7 {
            king_moves[index as usize] |= 1 << tar;
        }
        //  Left
        tar = index + 1;
        if tar < 64 && file != 7 {
            king_moves[index as usize] |= 1 << tar;
        }
        //  Left
        tar = index + 9;
        if tar < 64 && file != 7 {
            king_moves[index as usize] |= 1 << tar;
        }
        //  Up
        tar = index + 8;
        if tar < 64 {
            king_moves[index as usize] |= 1 << tar;
        }
    }

    king_moves
}
