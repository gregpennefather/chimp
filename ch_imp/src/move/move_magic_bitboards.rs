use crate::board::bitboard::Bitboard;

fn rook_mask_generation(square: i64) -> u64 {
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

const ROOK_MAGIC: [u64; 64] = [1189513251583494144, 35186553394450, 2594530790793084928, 9799832823518007392, 2341876205352927232, 1155173304831582224, 146367537647997960, 73183504750346272, 2377910019967992064, 1297184027526561792, 1495370762813441, 9149362945917568, 1189514351090925569, 72339069098856656, 1172066649912639488, 4612005153824841728, 8075660018877767840, 9230804739485335584, 2305878262333833248, 72110473675292672, 1748557982670585920, 1008947603779682432, 10376295809242628768, 288305417853992963, 74555688759918848, 1152949550743298560, 11962893252748773380, 2344123614642045312, 148095422320085504, 180708038855426113, 4761167923331366948, 2888003630416806464, 70405394006016, 68757225984, 9305562730591715368, 435160460040388608, 11529215046106742784, 4535619813416, 828662560177129478, 9295431864342806528, 10385372277960409088, 9011597368365066, 2307250452963655685, 1125899906845260, 175922968002576, 4510265433399296, 4612390292134494304, 67840004874305555, 9223380841538035778, 2317383494000450064, 10415139298008367232, 4303880577, 396319515991875618, 4756373095367901376, 2199027593216, 9223378161750769664, 297519600307552272, 45600633210930568, 4611686847356110984, 140741787601441, 145311456997606400, 4665729265529577472, 9223934987881939077, 9223864732421783556];

const ROOK_LEFT_SHIFT_BITS: [usize; 64] = [52, 53, 53, 53, 53, 53, 53, 52, 53, 54, 54, 54, 54, 54, 54, 53, 53, 54, 54, 54, 54, 54, 54, 53, 53, 54, 54, 54, 54, 54, 54, 53, 53, 54, 54, 54, 54, 54, 54, 53, 53, 54, 54, 54, 54, 54, 54,
53, 53, 54, 54, 54, 54, 54, 54, 53, 52, 53, 53, 53, 53, 53, 53, 52];

const BISHOP_MAGIC: [u64; 64] = [651050470220160, 2315413777178183816, 73201245384492064, 14150873666413477920, 9511602761033056320, 1452490044803710984, 5764608693414135817, 18718360831344832, 6917546682105348098, 140943663648402, 17749088157697, 5764618519361094720, 358162104384, 4508014887764004, 2305878468667114016, 864694770631442438, 288371114177332304, 9313479351246819888, 883269097608118400, 576461456694846464, 13916404324356851337, 4947819712512, 2549037943276736640, 9511992328500547618, 2317265973037776896, 9011599616540722, 5075347873726592, 38847953422798464, 2449958197369241600, 4683745847995797250, 13871654206743513856, 9439546124644941984, 18015875995013128, 1152991899657967616, 5476942306948957200, 2305844108729534512, 655273750094303234, 4612319345717020672, 4917930875767497000, 2595060952999133194,
154034740663368, 2343595840668565520, 2377900603260016656, 5189343039937577024, 3146496, 648519452364636228, 283716949639192, 4828212912005326602, 5764680099395928208, 175922665947136, 147498389666357266, 1229485240965268737, 4612249519479095368, 36028831659999266, 18015533459858752, 869480738548877312, 18036947658216448, 4574071519182852, 649107718933615136, 576481780475887808, 576465150358421632,
721703318294364180, 13835076747334320160, 9232451838238883858];

const BISHOP_LEFT_SHIFT_BITS: [usize; 64] = [58, 59, 59, 59, 59, 59, 59, 58, 59, 59, 59, 59, 59, 59, 59, 59, 59, 59, 57, 57, 57, 57, 59, 59, 59, 59, 57, 55, 55, 57, 59, 59, 59, 59, 57, 55, 55, 57, 59, 59, 59, 59, 57, 57, 57, 57, 59,
59, 59, 59, 59, 59, 59, 59, 59, 59, 58, 59, 59, 59, 59, 59, 59, 58];

fn build_rook_attack_table(square: i64, left_shift: usize, mask: u64, magic: u64) -> Vec<Bitboard> {

    let num_bits = 64 - left_shift;
    let table_lookup_size = 1 << num_bits;

    let mut table = Vec::with_capacity(table_lookup_size);
    unsafe {
        table.set_len(table_lookup_size);
    }

    let blocker_patterns: Vec<u64> = generate_blocker_patterns(mask);
    for pattern in blocker_patterns {
        let index = (pattern.overflowing_mul(magic).0 >> left_shift) as usize;
        let moves = generate_legal_rook_moves(square, Bitboard::new(pattern));
        table[index] = moves;
    }

    table
}

fn build_bishop_attack_table(square: i64, left_shift: usize, mask: u64, magic: u64) -> Vec<Bitboard> {
    let num_bits = 64 - left_shift;
    let table_lookup_size = 1 << num_bits;

    let mut table = Vec::with_capacity(table_lookup_size);
    unsafe {
        table.set_len(table_lookup_size);
    }

    let blocker_patterns: Vec<u64> = generate_blocker_patterns(mask);
    for pattern in blocker_patterns {
        let index = (pattern.overflowing_mul(magic).0 >> left_shift) as usize;
        let moves = generate_legal_bishop_moves(square, Bitboard::new(pattern));
        table[index] = moves;
    }

    table
}

fn generate_legal_rook_moves(square: i64, blocker_bitboard: Bitboard) -> Bitboard {
    let mut legal_moves = Bitboard::new(0);

    // Left
    let rank = square / 8;
    for dist in 1..8 {
        let pos = square + (dist * 1);
        if (pos / 8) != rank || pos > 63 {
            break;
        }
        legal_moves = legal_moves.set(pos as u8);
        if blocker_bitboard.occupied(pos as u8) {
            break;
        }
    }

    // Right
    for dist in 1..8 {
        let pos = square - (dist * 1);
        if (pos / 8) != rank || pos < 0 {
            break;
        }
        legal_moves = legal_moves.set(pos as u8);
        if blocker_bitboard.occupied(pos as u8) {
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
        legal_moves = legal_moves.set(pos as u8);
        if blocker_bitboard.occupied(pos as u8) {
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
        legal_moves = legal_moves.set(pos as u8);
        if blocker_bitboard.occupied(pos as u8) {
            break;
        }
    }

    legal_moves
}

pub fn generate_legal_bishop_moves(square: i64, blocker_bitboard: Bitboard) -> Bitboard {
    let mut legal_moves = Bitboard::new(0);

    let origin_file = square % 8;

    // Left-Down
    let delta = 1 - 8;
    for dist in 1..8 {
        let pos = square + (delta * dist);
        let pos_file = pos % 8;
        if (pos < 0) || (pos_file <= origin_file) {
            break;
        }
        legal_moves = legal_moves.set(pos as u8);
        if blocker_bitboard.occupied(pos as u8) {
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
        legal_moves = legal_moves.set(pos as u8);
        if blocker_bitboard.occupied(pos as u8) {
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
        legal_moves = legal_moves.set(pos as u8);
        if blocker_bitboard.occupied(pos as u8) {
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
        legal_moves = legal_moves.set(pos as u8);
        if blocker_bitboard.occupied(pos as u8) {
            break;
        }
    }

    legal_moves
}

fn generate_blocker_patterns(mask: u64) -> Vec<u64> {
    let mut move_indicies = Vec::new();
    for i in 0..64 {
        if (mask >> i) & 1 == 1 {
            move_indicies.push(i);
        }
    }

    let num_patterns = 1 << move_indicies.len();
    let mut blocker_bitboards = Vec::with_capacity(num_patterns);
    unsafe {
        blocker_bitboards.set_len(num_patterns);

        for pattern_index in 0..num_patterns {
            for bit_index in 0..move_indicies.len() {
                let bit = ((pattern_index >> bit_index) & 1) as u64;
                let r = bit << move_indicies[bit_index];
                blocker_bitboards[pattern_index] |= r;
            }
        }
    }

    blocker_bitboards
}

#[derive(Copy,Clone)]
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
    rook_attack_table: Vec<Vec<Bitboard>>,
    bishop_table: [TableEntry; 64],
    bishop_attack_table:  Vec<Vec<Bitboard>>,
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
            rook_attack_table[i] = build_rook_attack_table(i as i64, ROOK_LEFT_SHIFT_BITS[i], rook_table[i].mask, rook_table[i].magic);
            bishop_attack_table[i] = build_bishop_attack_table(i as i64, BISHOP_LEFT_SHIFT_BITS[i], bishop_table[i].mask, bishop_table[i].magic);
        }

        MagicTable { rook_table, rook_attack_table, bishop_table, bishop_attack_table }
    }

    pub fn get_rook_attacks(&self, index: usize, occupancy: u64) -> Bitboard {
        let mut occ = occupancy & self.rook_table[index].mask;
        occ = occ.overflowing_mul(self.rook_table[index].magic).0;
        occ = occ >> ROOK_LEFT_SHIFT_BITS[index];

        self.rook_attack_table[index][occ as usize]
    }


    pub fn get_bishop_attacks(&self, index: usize, occupancy: u64) -> Bitboard {
        let mut occ = occupancy & self.bishop_table[index].mask;
        occ = occ.overflowing_mul(self.bishop_table[index].magic).0;
        occ = occ >> BISHOP_LEFT_SHIFT_BITS[index];

        self.bishop_attack_table[index][occ as usize]
    }
}