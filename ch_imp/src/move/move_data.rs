use crate::shared::board_utils::{get_file, get_rank};

use super::move_magic_bitboards::MagicTable;

pub const KING_CASTLING_CLEARANCE: u64 = 0b110;
pub const KING_CASTLING_CHECK: u64 = 0b110;
pub const QUEEN_CASTLING_CLEARANCE: u64 = 0b1110000;
pub const QUEEN_CASTLING_CHECK: u64 = 0b110000;
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
    pub slide_inbetween: [u64; 64 * 65 / 2],
    pub slide_legal: [(bool, bool); 64 * 65 / 2],
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
        let (slide_inbetween, slide_legal) = generate_slide_data();
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
            slide_inbetween,
            slide_legal,
        }
    }

    pub fn is_slide_legal(&self, from: u8, to: u8) -> (bool, bool) {
        let index = calculate_triangular_index(from.into(), to.into());
        self.slide_legal[index]
    }

    pub fn get_slide_inbetween(&self, from: u8, to: u8) -> u64 {
        let index = calculate_triangular_index(from.into(), to.into());
        self.slide_inbetween[index]
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

fn calculate_triangular_index(mut a: i32, mut b: i32) -> usize {
    let mut d = a - b;
    d &= d.wrapping_shr(31);
    b += d;
    a -= d;
    b *= b ^ 127;
    return ((b >> 1) + a) as usize;
}

fn generate_slide_data() -> ([u64; 2080], [(bool, bool); 2080]) {
    let mut slide_data = [0; 2080];
    let mut slide_psudolegal = [(false, false); 2080];

    for from in 0..64 {
        for to in 0..64 {
            let index = calculate_triangular_index(from, to);
            slide_psudolegal[index] = get_slide_psudolegal(from, to);
            slide_data[index] = match slide_psudolegal[index] {
                (true, false) => get_slide_data(from, to),
                (false, true) => get_slide_data(from, to),
                _ => 0,
            }
            // println!("{from}->{to}={index} {:?}", slide_psudolegal[index]);
        }
    }

    (slide_data, slide_psudolegal)
}

fn get_slide_data(from: i32, to: i32) -> u64 {
    let mut r = 0u64;
    let fr = get_rank(from as u8) as i8;
    let ff = get_file(from as u8) as i8;
    let tr = get_rank(to as u8) as i8;
    let tf = get_file(to as u8) as i8;
    let mut dr = tr - fr;
    let mut df = ff - tf;

    // println!("fr:{fr},ff:{ff},tr:{tr},tf:{tf}");

    dr = if dr != 0 { dr / i8::abs(dr) } else { 0 };
    df = if df != 0 { df / i8::abs(df) } else { 0 };

    // println!("dr:{dr},df:{df}");

    let mut i = from as i8 + (dr * 8) + df;
    while i != to as i8 && i < 64 {
        r |= 1 << i;
        i += (dr * 8) + df;
    }

    r
}

fn get_slide_psudolegal(from: i32, to: i32) -> (bool, bool) {
    let fr = get_rank(from as u8);
    let ff = get_file(from as u8);
    let tr = get_rank(to as u8);
    let tf = get_file(to as u8);
    let orthag = fr == tr || ff == tf;
    if orthag {
        return (orthag, false);
    }

    let diag = u8::abs_diff(fr, tr) == u8::abs_diff(ff, tf);

    (false, diag)
}
