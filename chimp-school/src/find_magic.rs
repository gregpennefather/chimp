use rand::Rng;

const ROOK_BITS: [usize; 64] = [
    12, 11, 11, 11, 11, 11, 11, 12, 11, 10, 10, 10, 10, 10, 10, 11, 11, 10, 10, 10, 10, 10, 10, 11,
    11, 10, 10, 10, 10, 10, 10, 11, 11, 10, 10, 10, 10, 10, 10, 11, 11, 10, 10, 10, 10, 10, 10, 11,
    11, 10, 10, 10, 10, 10, 10, 11, 12, 11, 11, 11, 11, 11, 11, 12,
];

const BISHOP_BITS: [usize; 64] = [
    6, 5, 5, 5, 5, 5, 5, 6, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 7, 7, 7, 7, 5, 5, 5, 5, 7, 9, 9, 7, 5, 5,
    5, 5, 7, 9, 9, 7, 5, 5, 5, 5, 7, 7, 7, 7, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 6, 5, 5, 5, 5, 5, 5, 6,
];

fn count_1s(b: u64) -> usize {
    let mut r = 0;
    let mut bb = b;
    while bb > 0 {
        bb &= bb - 1;
        r += 1;
    }
    r
}

//https://stackoverflow.com/questions/24798499/chess-bitscanning
const DEBRUIJN: u64 = 0x03f79d71b4cb0a89u64;

fn bit_scan_forward(bitboard: u64) -> u64 {
    assert!(bitboard!=0);
    bitboard.overflowing_mul(bitboard - 1).0.overflowing_mul(DEBRUIJN).0 >> 58
}

pub fn rook_mask_generation(square: i64) -> u64 {
    let mut result = 0u64;
    let rank = square / 8;
    let file = square % 8;

    let mut r = rank + 1;
    while r <= 6 {
        result |= 1 << (file + r * 8);
        r += 1;
    }
    r = rank - 1;
    while r >= 1 {
        result |= 1 << (file + r * 8);
        r -= 1;
    }

    let mut f = file + 1;
    while f <= 6 {
        result |= 1 << (f + rank * 8);
        f += 1;
    }

    f = file - 1;
    while f >= 1 {
        result |= 1 << (f + rank * 8);
        f -= 1;
    }
    result
}

pub fn bishop_mask_generation(square: i64) -> u64 {
    let mut result = 0u64;
    let rank = square / 8;
    let file = square % 8;

    let mut r = rank + 1;
    let mut f = file + 1;
    while r <= 6 && f <= 6 {
        result |= 1 << (f + r * 8);
        r += 1;
        f += 1;
    }

    r = rank + 1;
    f = file - 1;
    while r <= 6 && f >= 1 {
        result |= 1 << (f + r * 8);
        r += 1;
        f -= 1;
    }
    r = rank - 1;
    f = file + 1;
    while r >= 1 && f <= 6 {
        result |= 1 << (f + r * 8);
        r -= 1;
        f += 1;
    }
    r = rank - 1;
    f = file - 1;
    while r >= 1 && f >= 1 {
        result |= 1 << (f + r * 8);
        r -= 1;
        f -= 1;
    }
    result
}

fn ratt(square: i64, block: u64) -> u64 {
    let mut result = 0u64;
    let rank = square / 8;
    let file = square % 8;

    let mut r = rank + 1;
    while r <= 7 {
        result |= 1 << (file + r * 8);
        if (block & (1u64 << (file + r * 8))) > 0 {
            break;
        }
        r += 1;
    }

    r = rank - 1;
    while r >= 0 {
        result |= 1 << (file + r * 8);
        if (block & (1u64 << (file + r * 8))) > 0 {
            break;
        }
        r -= 1;
    }

    let mut f = file + 1;
    while f <= 7 {
        result |= 1 << (f + rank * 8);
        if (block & (1u64 << (f + rank * 8))) > 0 {
            break;
        }
        f += 1;
    }

    f = file - 1;
    while f >= 0 {
        result |= 1 << (f + rank * 8);
        if (block & (1u64 << (f + rank * 8))) > 0 {
            break;
        }
        f -= 1;
    }

    result
}

fn batt(square: i64, block: u64) -> u64 {
    let mut result = 0u64;
    let rank = square / 8;
    let file = square % 8;

    let mut r = rank + 1;
    let mut f = file + 1;
    while r <= 7 && f <= 7 {
        result |= 1 << (f + r * 8);
        if (block & (1u64 << (f + r * 8))) > 0 {
            break;
        }
        r += 1;
        f += 1;
    }

    r = rank + 1;
    f = file - 1;
    while r <= 7 && f >= 1 {
        result |= 1 << (f + r * 8);
        if (block & (1u64 << (f + r * 8))) > 0 {
            break;
        }
        r += 1;
        f -= 1;
    }

    r = rank - 1;
    f = file + 1;
    while r >= 1 && f <= 7 {
        result |= 1 << (f + r * 8);
        if (block & (1u64 << (f + r * 8))) > 0 {
            break;
        }
        r -= 1;
        f += 1;
    }

    r = rank - 1;
    f = file - 1;
    while r >= 1 && f >= 1 {
        result |= 1 << (f + r * 8);
        if (block & (1u64 << (f + r * 8))) > 0 {
            break;
        }
        r -= 1;
        f -= 1;
    }

    result
}

fn transform(b: u64, magic: u64, bits: usize) -> usize {
    return ((b * magic) >> (64 - bits)) as usize;
}

fn find_magic(square: i64, m: usize, bishop: bool) -> u64 {
    let mut rng = rand::thread_rng();

    let mask = if bishop {
        bishop_mask_generation(square)
    } else {
        rook_mask_generation(square)
    };
    let n = mask.count_ones() as usize; // Potentially this should do something different

    let k = count_1s(mask);
    assert_eq!(n, k);

    let mut a = [0; 4096];
    let mut b = [0; 4096];
    let mut used = [0; 4096];

    let mut sub_mask = mask;
    for i in 0..(1 << n) {
        if sub_mask == 0 {
            break;
        }
        b[i] = bit_scan_forward(sub_mask);
        sub_mask &= sub_mask - 1;
        a[i] = if bishop {
            batt(square, b[i])
        } else {
            ratt(square, b[i])
        }
    }
    for k in 0..100000000 {
        let magic: u64 = rng.gen::<u64>() & rng.gen::<u64>() & rng.gen::<u64>();
        if (mask.overflowing_mul(magic).0 & 0xFF00000000000000).count_ones() < 6 {
            continue;
        }
        used = [0; 4096];
        let mut i = 0;
        let mut fail = false;
        while !fail && i < (i << n) {
            let j = transform(b[i], magic, m);
            if used[j] == 0 {
                used[j] = a[i];
            } else if used[j] != a[i] {
                fail = true;
            }
            i += 1;
        }
        if !fail {
            return magic;
        }
    }
    println!("*** FAILED ***");
    0
}

pub fn find_main() {
    println!("finding rook magic number");
    let mut rook_magic = [0u64; 64];
    let mut rook_shift = [0;64];
    for square in 0..64usize {
        rook_magic[square] = find_magic(square as i64, ROOK_BITS[square], false);
        rook_shift[square] = 64-ROOK_BITS[square];
    }
    println!("Magics: {:?}", rook_magic);
    println!("Bit-shifts: {:?}", rook_shift);


    println!("finding bishop magic number");
    let mut bishop_magic = [0u64; 64];
    let mut bishop_shift = [0;64];
    for square in 0..64usize {
        bishop_magic[square] = find_magic(square as i64, BISHOP_BITS[square], true);
        bishop_shift[square] = 64-BISHOP_BITS[square];
    }
    println!("Magics: {:?}", bishop_magic);
    println!("Bit-shifts: {:?}", bishop_shift);
}
