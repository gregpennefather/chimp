use crate::board::bitboard::Bitboard;
use rand::Rng;

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
        result |= 1 << (f + (r * 8));
        r += 1;
        f += 1;
    }

    r = rank + 1;
    f = file - 1;
    while r <= 6 && f >= 1 {
        result |= 1 << (f + (r * 8));
        r += 1;
        f -= 1;
    }
    r = rank - 1;
    f = file + 1;
    while r >= 1 && f <= 6 {
        result |= 1 << (f + (r * 8));
        r -= 1;
        f += 1;
    }
    r = rank - 1;
    f = file - 1;
    while r >= 1 && f >= 1 {
        result |= 1 << (f + (r * 8));
        r -= 1;
        f -= 1;
    }
    result
}

const ROOK_MAGIC: [u64; 64] = [648536213958983684, 9313444305622294532, 4647724161832062980, 936754220186468384, 144150441200976904, 396338757466333696, 36029346808332544, 2449958336880251396, 5770376609021952, 1152991873887916036, 703756188516480, 9518639322052567296, 108649478583751168, 3940719601516548, 3171097399027565056, 9147938630534400, 6932970644717569, 306254121047441536, 2918404027328762117, 9369748920617467938, 11541601048851384320, 37718196702151296, 18168330274275856, 14987981758917542164, 1225050571197677568, 144150377816674304, 729591936804982784, 4688827758529283080, 2251838469440768, 16286142704381984772, 612498431319085105, 72198471114309888, 36028934629883968, 4756153187684061192, 369312763786305540, 3170674943885051904, 2251836329296897, 9223935124314521616, 613333993596649984,
13925200967436861700, 9224568359195541504, 141012383498274, 1157442696957165696, 4543250782289928, 585473449250586640, 306246973751591040, 5332279559717126145, 1191202805818458113, 2350949376379072640, 1153062516975206528, 288247973169627520, 10175348019712, 576469550544191616, 306246975832195200, 7036883007963392, 108122142503141888, 4648595253821210625, 145410983203074, 18298356514162705, 8933615865857, 844493685784653, 155655696782983217, 18760695154708, 74319031842144290];

pub const ROOK_LEFT_SHIFT_BITS: [usize; 64] = [
    52, 53, 53, 53, 53, 53, 53, 52, 53, 54, 54, 54, 54, 54, 54, 53, 53, 54, 54, 54, 54, 54, 54, 53,
    53, 54, 54, 54, 54, 54, 54, 53, 53, 54, 54, 54, 54, 54, 54, 53, 53, 54, 54, 54, 54, 54, 54, 53,
    53, 54, 54, 54, 54, 54, 54, 53, 52, 53, 53, 53, 53, 53, 53, 52,
];

const BISHOP_MAGIC: [u64; 64] = [580628315668993, 144687213916585986, 1130315808636936, 649715723639227904, 576760094411326468, 2386644091582021636, 16141754500303044736, 637748017246209, 74326995039159424, 3100184094183538820, 2332873437447464064, 16158919878442221824, 22610512269279234, 288235392746930176, 4755801765124801568, 9225753587781536000, 1162773713042606088, 9268412439832101424, 9948452833158365248, 1127137947951361, 2306406388673556490, 142146372108803, 307097996892840961, 38319088338736256, 9016081515677728, 878492748700188952, 582092455222977539, 9293072282419328, 1801721394728288258, 142936528404992, 600126674948109320, 12385998799962928, 9225872360657133572, 2308660104042385412, 19210873319064577, 5764642741767702544, 144397779745243650, 9015999776981132, 282591672476680, 72631599237189696, 218433395267490180, 867508086372303458, 45054758973624320, 10448492011903256834, 567419168883712, 18094115300312192, 6764230975986768, 10134009912754240, 4620772674646312065, 4613093702616039554, 4647715099460501568, 2214854656, 306279993946734592, 7512074564512915528, 5917765111926259712, 1768102675582296066, 293931378310906944, 4503745791296576, 9295712209744503808, 1152923291317568002, 144116498309454336, 2305844521582264898, 24844601014092304, 54045411798712336];

pub const BISHOP_LEFT_SHIFT_BITS: [usize; 64] = [
    58, 59, 59, 59, 59, 59, 59, 58, 59, 59, 59, 59, 59, 59, 59, 59, 59, 59, 57, 57, 57, 57, 59, 59,
    59, 59, 57, 55, 55, 57, 59, 59, 59, 59, 57, 55, 55, 57, 59, 59, 59, 59, 57, 57, 57, 57, 59, 59,
    59, 59, 59, 59, 59, 59, 59, 59, 58, 59, 59, 59, 59, 59, 59, 58,
];

fn build_rook_attack_table(square: i64, left_shift: usize, mask: u64, magic: u64) -> Vec<u64> {
    let num_bits = 64 - left_shift;
    let table_lookup_size = 1 << num_bits;

    let mut table = Vec::with_capacity(table_lookup_size);
    unsafe {
        table.set_len(table_lookup_size);
    }

    let blocker_patterns: Vec<u64> = generate_blocker_patterns(mask);
    for pattern in blocker_patterns {
        let index = (pattern.overflowing_mul(magic).0 >> left_shift) as usize;
        let moves = generate_legal_rook_moves(square, pattern);
        // if table[index] != 0 && table[index] != moves {
        //     println!("dup: {index}:{pattern}");
        // }
        table[index] = moves;
    }

    table
}

fn build_bishop_attack_table(square: i64, left_shift: usize, mask: u64, magic: u64) -> Vec<u64> {
    let num_bits = 64 - left_shift;
    let table_lookup_size = 1 << num_bits;

    let mut table = Vec::with_capacity(table_lookup_size);
    unsafe {
        table.set_len(table_lookup_size);
    }

    let blocker_patterns: Vec<u64> = generate_blocker_patterns(mask);
    for pattern in blocker_patterns {
        let index = (pattern.overflowing_mul(magic).0 >> left_shift) as usize;
        let moves = generate_legal_bishop_moves(square, pattern);
        // if index == 33 && square == 26 {
        //     // This proves we're generating crap magics
        //     println!("mask:\n{}", Bitboard::new(mask));
        //     println!("pattern:\n{}", Bitboard::new(pattern));
        //     println!("moves:\n{}", Bitboard::new(moves));
        // }

        // if table[index] != 0 && table[index] != moves {
        // println!("duplicate: {index}:{pattern}");
        // }

        table[index] = moves;
    }

    table
}

fn occupied(bb: u64, pos: i64) -> bool {
    bb >> pos & 0b1 > 0
}

fn set(bb: u64, pos: i64) -> u64 {
    bb | (1 << pos)
}

fn generate_legal_rook_moves(square: i64, blocker_bitboard: u64) -> u64 {
    let mut legal_moves = 0;

    // Left
    let rank = square / 8;
    for dist in 1..8 {
        let pos = square + (dist * 1);
        if (pos / 8) != rank || pos > 63 {
            break;
        }
        legal_moves = set(legal_moves, pos);
        if occupied(blocker_bitboard, pos) {
            break;
        }
    }

    // Right
    for dist in 1..8 {
        let pos = square - (dist * 1);
        if (pos / 8) != rank || pos < 0 {
            break;
        }
        legal_moves = set(legal_moves, pos);
        if occupied(blocker_bitboard, pos) {
            break;
        }
    }

    // Up
    let file = square % 8;
    for dist in 1..8 {
        let pos = square + (dist * 8);
        if (pos % 8) != file || pos > 63 {
            break;
        }
        legal_moves = set(legal_moves, pos);
        if occupied(blocker_bitboard, pos) {
            break;
        }
    }

    // Down
    let file = square % 8;
    for dist in 1..8 {
        let pos = square - (dist * 8);
        if (pos % 8) != file || pos < 0 {
            break;
        }
        legal_moves = set(legal_moves, pos);
        if occupied(blocker_bitboard, pos) {
            break;
        }
    }

    legal_moves
}

pub fn generate_legal_bishop_moves(square: i64, blocker_bitboard: u64) -> u64 {
    let mut legal_moves = 0;

    let origin_file = square % 8;

    // Left-Down
    let delta = 1 - 8;
    for dist in 1..8 {
        let pos = square + (delta * dist);
        let pos_file = pos % 8;
        if (pos < 0) || (pos_file <= origin_file) {
            break;
        }
        legal_moves = set(legal_moves, pos);
        if occupied(blocker_bitboard, pos) {
            break;
        }
    }

    // Left+Up
    let delta = 1 + 8;
    for dist in 1..8 {
        let pos = square + (delta * dist);
        let pos_file = pos % 8;
        if (pos > 63) || pos_file <= origin_file {
            break;
        }
        legal_moves = set(legal_moves, pos);
        if occupied(blocker_bitboard, pos) {
            break;
        }
    }

    // Right+Up
    let delta = -1 + 8;
    for dist in 1..8 {
        let pos = square + (delta * dist);
        let pos_file = pos % 8;
        if (pos > 63) || pos_file >= origin_file {
            break;
        }
        legal_moves = set(legal_moves, pos);
        if occupied(blocker_bitboard, pos) {
            break;
        }
    }

    // Right+Down
    let delta = -1 - 8;
    for dist in 1..8 {
        let pos = square + (delta * dist);
        let pos_file = pos % 8;
        if (pos < 0) || pos_file >= origin_file {
            break;
        }
        legal_moves = set(legal_moves, pos);
        if occupied(blocker_bitboard, pos) {
            break;
        }
    }

    legal_moves
}

pub fn generate_blocker_patterns(mask: u64) -> Vec<u64> {
    let mut move_indicies = Vec::new();
    for i in 0..64 {
        if ((mask >> i) & 1) == 1 {
            move_indicies.push(i);
        }
    }

    let num_patterns = 1 << move_indicies.len();
    let mut blocker_bitboards = Vec::with_capacity(num_patterns);

    for pattern_index in 0..num_patterns {
        blocker_bitboards.push(0);
        for bit_index in 0..move_indicies.len() {
            let bit = ((pattern_index >> bit_index) & 1) as u64;
            let r = bit << move_indicies[bit_index];
            blocker_bitboards[pattern_index] |= r;
        }
    }

    blocker_bitboards
}

#[derive(Copy, Clone)]
struct TableEntry {
    mask: u64,
    magic: u64,
}

impl Default for TableEntry {
    fn default() -> Self {
        Self {
            mask: Default::default(),
            magic: Default::default(),
        }
    }
}

pub struct MagicTable {
    rook_table: [TableEntry; 64],
    rook_attack_table: Vec<Vec<u64>>,
    bishop_table: [TableEntry; 64],
    bishop_attack_table: Vec<Vec<u64>>,
}

impl MagicTable {
    pub fn new() -> Self {
        let mut rook_table = [TableEntry::default(); 64];
        let mut bishop_table = [TableEntry::default(); 64];

        for i in 0..64usize {
            let rook_mask = rook_mask_generation(i as i64);
            let rook_magic = ROOK_MAGIC[i];
            rook_table[i] = TableEntry {
                mask: rook_mask,
                magic: rook_magic,
            };
            let bishop_mask = bishop_mask_generation(i as i64);
            let bishop_magic = BISHOP_MAGIC[i];
            bishop_table[i] = TableEntry {
                mask: bishop_mask,
                magic: bishop_magic,
            };
        }

        let mut rook_attack_table = Vec::with_capacity(64);
        let mut bishop_attack_table = Vec::with_capacity(64);
        for i in 0..64 {
            rook_attack_table.push(Vec::new());
            bishop_attack_table.push(Vec::new());
        }
        for i in 0..64usize {
            rook_attack_table[i] = build_rook_attack_table(
                i as i64,
                ROOK_LEFT_SHIFT_BITS[i],
                rook_table[i].mask,
                rook_table[i].magic,
            );
            bishop_attack_table[i] = build_bishop_attack_table(
                i as i64,
                BISHOP_LEFT_SHIFT_BITS[i],
                bishop_table[i].mask,
                bishop_table[i].magic,
            );
        }

        MagicTable {
            rook_table,
            rook_attack_table,
            bishop_table,
            bishop_attack_table,
        }
    }

    pub fn get_rook_attacks(&self, index: usize, occupancy: u64) -> u64 {
        let mut occ = occupancy & self.rook_table[index].mask;
        occ = occ.overflowing_mul(self.rook_table[index].magic).0;
        occ = occ >> ROOK_LEFT_SHIFT_BITS[index];

        self.rook_attack_table[index][occ as usize]
    }

    pub fn get_bishop_attacks(&self, index: usize, occupancy: u64) -> u64 {
        let mut occ = occupancy & self.bishop_table[index].mask;
        occ = occ.overflowing_mul(self.bishop_table[index].magic).0;
        occ = occ >> BISHOP_LEFT_SHIFT_BITS[index];

        self.bishop_attack_table[index][occ as usize]
    }
}

//https://stackoverflow.com/questions/24798499/chess-bitscanning
const DEBRUIJN: u64 = 0x03f79d71b4cb0a89u64;

fn bit_scan_forward(bitboard: u64) -> u64 {
    assert!(bitboard != 0);
    bitboard
        .overflowing_mul(bitboard - 1)
        .0
        .overflowing_mul(DEBRUIJN)
        .0
        >> 58
}

fn transform(b: u64, magic: u64, bits: usize) -> usize {
    return ((b.overflowing_mul(magic)).0 >> bits) as usize;
}

pub fn find_rook_magics(square: i64, bits: usize) -> u64 {
    let mut rng = rand::thread_rng();
    let mask = rook_mask_generation(square);
    let n = mask.count_ones() as usize;

    let mut move_indicies = Vec::new();
    for i in 0..64 {
        if ((mask >> i) & 1) == 1 {
            move_indicies.push(i);
        }
    }

    let mut a = [0; 4096];
    let mut b = [0; 4096];
    let count = 1 << n;
    // for i in 0..count {
    //     for bit_index in 0..move_indicies.len() {
    //         let bit = ((i >> bit_index) & 1) as u64;
    //         let r = bit << move_indicies[bit_index];
    //         b[i] |= r;
    //     }
    //     a[i] = generate_legal_rook_moves(square, b[i]);
    // }

    let patterns = generate_blocker_patterns(mask);
    for i in 0..patterns.len() {
        b[i] = patterns[i];
        a[i] = generate_legal_rook_moves(square, b[i]);
    }

    for k in 0..100000000 {
        let magic = rng.gen::<u64>() & rng.gen::<u64>() & rng.gen::<u64>();
        if (mask.overflowing_mul(magic).0 & 0xFF00000000000000).count_ones() < 6 {
            continue;
        }
        let mut fail = false;
        let mut used = [0; 4096];
        for i in 0..count {
            let j = transform(b[i], magic, bits);
            if used[j] == 0 {
                used[j] = a[i];
            } else if used[j] != a[i] {
                fail = true;
                break;
            }
        }
        if !fail {
            return magic;
        }
    }
    println!("FAILED!");
    0
}

pub fn find_bishop_magics(square: i64, bits: usize) -> u64 {
    let mut rng = rand::thread_rng();
    let mask = bishop_mask_generation(square);
    let n = mask.count_ones() as usize;

    let mut move_indicies = Vec::new();
    for i in 0..64 {
        if ((mask >> i) & 1) == 1 {
            move_indicies.push(i);
        }
    }

    let mut a = [0; 4096];
    let mut b = [0; 4096];
    let count = 1 << n;
    // for i in 0..count {
    //     for bit_index in 0..move_indicies.len() {
    //         let bit = ((i >> bit_index) & 1) as u64;
    //         let r = bit << move_indicies[bit_index];
    //         b[i] |= r;
    //     }
    //     a[i] = generate_legal_rook_moves(square, b[i]);
    // }

    let patterns = generate_blocker_patterns(mask);
    for i in 0..patterns.len() {
        b[i] = patterns[i];
        a[i] = generate_legal_bishop_moves(square, b[i]);
    }

    for k in 0..100000000 {
        let magic = rng.gen::<u64>() & rng.gen::<u64>() & rng.gen::<u64>();
        if (mask.overflowing_mul(magic).0 & 0xFF00000000000000).count_ones() < 6 {
            continue;
        }
        let mut fail = false;
        let mut used = [0; 4096];
        for i in 0..count {
            let j = transform(b[i], magic, bits);
            if used[j] == 0 {
                used[j] = a[i];
            } else if used[j] != a[i] {
                fail = true;
                break;
            }
        }
        if !fail {
            return magic;
        }
    }
    println!("FAILED!");
    0
}
