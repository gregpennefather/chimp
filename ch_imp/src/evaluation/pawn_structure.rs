use std::{collections::HashMap, sync::RwLock};

use log::error;
use rand::RngCore;

use crate::{
    board::{bitboard::Bitboard, position::Position},
    shared::board_utils::{get_file, get_rank},
};

const BOARD_FILES: [u64; 8] = [
    9259542123273814144,
    4629771061636907072,
    2314885530818453536,
    1157442765409226768,
    578721382704613384,
    289360691352306692,
    144680345676153346,
    72340172838076673,
];

#[derive(Clone, Copy, Debug)]
pub struct PawnInfo {
    pub doubles: u8,
    pub isolated: u8,
    pub pawn_shield: i8,
    pub lsb: u8,
}

pub struct PawnZorb {
    pawn_table: [u32; 48],
    king_table: [u32; 64],
}

impl PawnZorb {
    pub fn new() -> Self {
        let mut rng = rand::thread_rng();

        let mut pawn_table: [u32; 48] = [0; 48];
        let mut king_table: [u32; 64] = [0; 64];

        for i in 0..48 {
            pawn_table[i] = rng.next_u32();
        }

        for i in 0..64 {
            king_table[i] = rng.next_u32();
        }

        Self { pawn_table, king_table }
    }

    pub fn hash(&self, mut pawn_occupancy: u64, king_pos: u8) -> u32 {
        let mut key = 0;
        while pawn_occupancy != 0 {
            let pos = pawn_occupancy.trailing_zeros();
            assert!(pos >= 8 && pos < 56);
            key ^= self.pawn_table[(pos - 8) as usize];
            pawn_occupancy ^= 1 << pos;
        }
        key ^= self.king_table[king_pos as usize];
        key
    }

    pub fn shift(&self, key: u32, changed_pos_in_64_square_rep: u8) -> u32 {
        if changed_pos_in_64_square_rep < 8 || changed_pos_in_64_square_rep >= 56 {
            panic!("cant pawn zorb for position {changed_pos_in_64_square_rep}")
        }
        let sq_in_48 = changed_pos_in_64_square_rep - 8;
        key ^ self.pawn_table[sq_in_48 as usize]
    }


    pub fn shift_king(&self, key: u32, pos: u8) -> u32 {
        key ^ self.king_table[pos as usize]
    }
}

lazy_static! {
    static ref PAWN_STRUCTURE_HASH: RwLock<HashMap<u32, PawnInfo>> = RwLock::new(HashMap::new());
}

pub fn get_pawn_structure_metrics(pawn_zorb: u32, pawn_occupancy: u64, king_position: u8) -> PawnInfo {
    let lsb = pawn_occupancy.trailing_zeros() as u8 - 8;
    match lookup(pawn_zorb, lsb) {
        Ok(option) => match option {
            Some(r) => r,
            None => build_and_store_metrics(pawn_zorb, pawn_occupancy, king_position),
        },
        Err(r) => {
            error!("{r}");
            build_pawn_metrics(pawn_occupancy, king_position)
        }
    }
}

fn build_and_store_metrics(pawn_zorb: u32, pawn_occupancy: u64, king_position: u8) -> PawnInfo {
    let metrics = build_pawn_metrics(pawn_occupancy, king_position);
    store(pawn_zorb, metrics);
    metrics
}

fn build_pawn_metrics(pawn_occupancy: u64, king_position: u8) -> PawnInfo {
    let lsb = pawn_occupancy.trailing_zeros() as u8 - 8;
    let frontspan = calculate_frontspan(pawn_occupancy);

    let doubles = get_doubled(pawn_occupancy, frontspan);
    let pawn_shield = get_pawn_shield(pawn_occupancy, king_position);
    let isolated = get_isolated(pawn_occupancy);
    println!("isolated: {isolated}");

    PawnInfo { doubles, lsb, isolated, pawn_shield }
}

fn lookup(zorb_key: u32, lsb: u8) -> Result<Option<PawnInfo>, String> {
    let binding = PAWN_STRUCTURE_HASH.try_read().unwrap();
    let r = binding.get(&zorb_key);

    match r {
        Some(&result) => {
            if lsb != result.lsb {
                return Err(format!(
                    "Pawn Zorb_key conflict {zorb_key}. LSB {lsb}!={}",
                    result.lsb
                ));
            }
            Ok(Some(result))
        }
        None => Ok(None),
    }
}

fn store(zorb_key: u32, metrics: PawnInfo) {
    PAWN_STRUCTURE_HASH
        .write()
        .unwrap()
        .insert(zorb_key, metrics);
}

pub fn calculate_frontspan(mut pawn_occupancy: u64) -> u64 {
    let mut r = 0;
    while pawn_occupancy != 0 {
        let pos = pawn_occupancy.trailing_zeros() as u8;
        let rank = get_rank(pos);
        let file = get_file(pos);

        let mask = BOARD_FILES[file as usize] << ((rank + 1) * 8);
        r |= mask;
        pawn_occupancy ^= 1 << pos;
    }
    r
}

fn get_doubled(mut pawn_occupancy: u64, frontspan: u64) -> u8 {
    let mut r = 0;
    while pawn_occupancy != 0 {
        let pos = pawn_occupancy.trailing_zeros() as u8;

        if frontspan.occupied(pos) {
            r += 1;
        }
        pawn_occupancy ^= 1 << pos;
    }
    r
}

fn get_isolated(pawn_occupancy: u64) -> u8 {
    let mut r = 0;
    let mut wip_occ = pawn_occupancy;
    while wip_occ != 0 {
        let pos = wip_occ.trailing_zeros() as u8;

        if isolated(pos, pawn_occupancy) {
            r += 1;
        }
        wip_occ ^= 1 << pos;
    }
    r
}

fn isolated(pos: u8, pawn_occupancy: u64) -> bool {
    let file = get_file(pos);
    if file != 0 {
        let l_mask = BOARD_FILES[file as usize-1];
        if pawn_occupancy & l_mask != 0 {
            return false;
        }
    }
    if file != 7 {
        let r_mask = BOARD_FILES[file as usize+1];
        if pawn_occupancy & r_mask != 0 {
            return false;
        }
    }
    return true;
}

fn get_pawn_shield(pawn_occupancy: u64, king_position: u8) -> i8 {
    let king_file = get_file(king_position);
    let king_rank = get_rank(king_position);
    println!("king at {king_position} r:{king_rank} f:{king_file}");
    if king_rank > 1 || king_file == 4 || king_file == 3  {
        return -10;
    }
    0
}

pub fn build_pawn_frontspan_board() {
    for sq in 8..9 {
        for offset_1 in 0..2 {
            let mut b: u64 = 1 << sq;
            for p_count in 1..8 {
                b |= 1 << sq + p_count + offset_1;
                println!("{}", b.to_board_format())
            }
        }
    }
}

#[cfg(test)]
mod test {
    use crate::{board::bitboard::Bitboard, shared::board_utils::index_from_coords};

    use super::*;

    #[test]
    fn calculate_frontspan_single_pawn() {
        let pawn_occupancy = 0.set(8);
        let r = calculate_frontspan(pawn_occupancy);

        let expected = 0.set(16).set(24).set(32).set(40).set(48).set(56);

        println!("r:\n{}", r.to_board_format());
        println!("e:\n{}", expected.to_board_format());
        assert_eq!(r, expected);
    }

    #[test]
    fn calculate_frontspan_three_pawns_two_doubles() {
        let pawn_occupancy = 0.set(12).set(28).set(18);
        let r = calculate_frontspan(pawn_occupancy);

        let expected = 0
            .set_file(2)
            .flip(2)
            .flip(10)
            .flip(18)
            .set_file(4)
            .flip(index_from_coords("d1"))
            .flip(index_from_coords("d2"));

        println!("r:\n{}", r.to_board_format());
        println!("e:\n{}", expected.to_board_format());
        assert_eq!(r, expected);
    }

    #[test]
    fn get_doubled_one_double_in_d_file() {
        let pawn_occupancy = 0.set(12).set(28).set(18);
        let span = calculate_frontspan(pawn_occupancy);

        let doubles = get_doubled(pawn_occupancy, span);
        assert_eq!(doubles, 1);
    }

    #[test]
    fn get_doubled_two_doubles_in_d_file() {
        let pawn_occupancy = 0.set(12).set(28).set(18).set(20);
        let span = calculate_frontspan(pawn_occupancy);

        let doubles = get_doubled(pawn_occupancy, span);
        assert_eq!(doubles, 2);
    }
}
