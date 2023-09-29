use std::{collections::HashMap, sync::RwLock};

use log::error;
use rand::RngCore;

use crate::{
    board::{bitboard::Bitboard, board_rep::BoardRep, position::Position},
    shared::board_utils::{get_file, get_rank, reverse_position_orientation},
};

use super::{shared::{BOARD_FILES, CENTER_FILES}, subcategories::{king::{pawn_distance::get_pawn_distance_penalty, pawnless_files::is_on_pawnless_file}, pawn::{utils::file_fill, files::open_files}}};





const PAWN_SHIELD_RANK_2_MASK: [u64; 8] = [57344, 57344, 57344, 0, 0, 1792, 1792, 1792];

const PAWN_SHIELD_RANK_3_MASK: [u64; 8] =
    [14680064, 14680064, 14680064, 0, 0, 458752, 458752, 458752];

const OPENING_DOUBLE_PENALTY: i16 = 10;
const ENDGAME_DOUBLE_PENALTY: i16 = 50;

const OPENING_ISOLATED_PENALTY: i16 = 10;
const ENDGAME_ISOLATED_PENALTY: i16 = 35;

const OPENING_BACKWARD_PENALTY: i16 = 10;
const ENDGAME_BACKWARD_PENALTY: i16 = 35;

const OPENING_STRAGGLERS_PENALTY: i16 = 5;
const ENDGAME_STRAGGLERS_PENALTY: i16 = 25;

const OPENING_OPEN_REWARD: i16 = 5;
const ENDGAME_OPEN_REWARD: i16 = 25;

const OPENING_PASSED_REWARDS: [i16; 8] = [0, 20, 30, 40, 60, 80, 100, 0];
const ENDGAME_PASSED_REWARDS: [i16; 8] = [0, 30, 55, 78, 92, 104, 137, 0];

const PAWN_SHIELD_REWARD: i16 = 200;

const PAWN_CENTER_ADVANTAGE_REWARD: i16 = 35;

pub const KING_PAWN_DISTANCE_PENALTY: i16 = 11;

const OPENING_KING_PAWNLESS_FILE_PENALTY: i16 = -25;
const ENDGAME_KING_PAWNLESS_FILE_PENALTY: i16 = -25;

#[derive(Clone, Copy, Debug, Default)]

pub struct PawnStructureEval {
    pub opening: i16,
    pub endgame: i16,
    pub p_count: u8,
}

pub struct PawnZorb {
    pawn_table: [[u64; 2]; 48],
    king_table: [[u64; 2]; 64],
}

impl PawnZorb {
    pub fn new() -> Self {
        let mut rng = rand::thread_rng();

        let mut pawn_table: [[u64; 2]; 48] = [[0; 2]; 48];
        let mut king_table: [[u64; 2]; 64] = [[0; 2]; 64];

        for i in 0..48 {
            pawn_table[i][0] = rng.next_u64();
            pawn_table[i][1] = rng.next_u64();
        }

        for i in 0..64 {
            king_table[i][0] = rng.next_u64();
            king_table[i][1] = rng.next_u64();
        }

        Self {
            pawn_table,
            king_table,
        }
    }

    pub fn hash(&self, board: BoardRep) -> u64 {
        let mut key = 0;
        let mut pawn_occupancy = board.pawn_bitboard;
        while pawn_occupancy != 0 {
            let pos = pawn_occupancy.trailing_zeros();
            assert!(pos >= 8 && pos < 56);
            let colour = if board.white_occupancy.occupied(pos as u8) {
                0
            } else {
                1
            };

            key ^= self.pawn_table[(pos - 8) as usize][colour];
            pawn_occupancy ^= 1 << pos;
        }
        key ^= self.king_table[board.white_king_position as usize][0];
        key ^= self.king_table[board.black_king_position as usize][1];
        key
    }

    pub fn shift(&self, key: u64, changed_pos_in_64_square_rep: u8, is_black: bool) -> u64 {
        if changed_pos_in_64_square_rep < 8 || changed_pos_in_64_square_rep >= 56 {
            panic!("cant pawn zorb for position {changed_pos_in_64_square_rep}")
        }
        let sq_in_48 = changed_pos_in_64_square_rep - 8;
        key ^ self.pawn_table[sq_in_48 as usize][if is_black { 1 } else { 0 }]
    }

    pub fn shift_king(&self, key: u64, pos: u8, is_black: bool) -> u64 {
        key ^ self.king_table[pos as usize][if is_black { 1 } else { 0 }]
    }
}

lazy_static! {
    static ref PAWN_STRUCTURE_EVAL_HASHMAP: RwLock<HashMap<u64, PawnStructureEval>> =
        RwLock::new(HashMap::new());
}

pub fn get_pawn_structure_eval(
    zorb_key: u64,
    w_pawns: u64,
    b_pawns: u64,
    w_king: u8,
    b_king: u8,
) -> PawnStructureEval {
    let p_count = w_pawns.count_ones() + b_pawns.count_ones();
    if p_count == 0 {
        return PawnStructureEval::default();
    }
    match lookup(zorb_key, p_count as u8) {
        Ok(option) => match option {
            Some(r) => r,
            None => build_and_store_pawn_structure_eval(
                zorb_key, w_pawns, b_pawns, w_king, b_king, p_count,
            ),
        },
        Err(r) => {
            error!("{r}");
            build_pawn_pawn_structure_eval(w_pawns, b_pawns, w_king, b_king, p_count)
        }
    }
}

fn build_and_store_pawn_structure_eval(
    zorb_key: u64,
    w_pawns: u64,
    b_pawns: u64,
    w_king: u8,
    b_king: u8,
    p_count: u32,
) -> PawnStructureEval {
    let eval = build_pawn_pawn_structure_eval(w_pawns, b_pawns, w_king, b_king, p_count);
    store(zorb_key, eval);
    eval
}

fn build_pawn_pawn_structure_eval(
    w_pawns: u64,
    b_pawns: u64,
    w_king: u8,
    b_king: u8,
    p_count: u32,
) -> PawnStructureEval {
    let mut opening = 0;
    let mut endgame = 0;

    let b_pawns_mirrored = b_pawns.flip_orientation();
    let b_king_mirrored = reverse_position_orientation(b_king);

    let w_frontspan = calculate_frontspan(w_pawns);
    let w_attack_frontspan = calculate_attack_frontspan(w_pawns);

    let b_frontspan = calculate_frontspan(b_pawns_mirrored);
    let b_attack_frontspan = calculate_attack_frontspan(b_pawns_mirrored);

    let w_filefill = file_fill(w_pawns);
    let b_filefill = file_fill(b_pawns);

    let open_files = open_files(w_filefill, b_filefill);

    // == Doubled ==
    let w_doubles = get_doubled(w_pawns, w_frontspan).count_ones() as i16;
    opening -= w_doubles * OPENING_DOUBLE_PENALTY;
    endgame -= w_doubles * ENDGAME_DOUBLE_PENALTY;

    let b_doubles = get_doubled(b_pawns_mirrored, b_frontspan).count_ones() as i16;
    opening += b_doubles * OPENING_DOUBLE_PENALTY;
    endgame += b_doubles * ENDGAME_DOUBLE_PENALTY;

    // == Isolated ==
    let w_isolated = get_isolated(w_pawns).count_ones() as i16;
    opening -= w_isolated * OPENING_ISOLATED_PENALTY;
    endgame -= w_isolated * ENDGAME_ISOLATED_PENALTY;

    let b_isolated = get_isolated(b_pawns_mirrored).count_ones() as i16;
    opening += b_isolated * OPENING_ISOLATED_PENALTY;
    endgame += b_isolated * ENDGAME_ISOLATED_PENALTY;

    // == Backward Pawns ==
    let w_backward_pawns = get_backward_pawns(
        w_pawns,
        w_attack_frontspan,
        b_attack_frontspan.flip_orientation(),
    );
    let w_backward_count = w_backward_pawns.count_ones() as i16;
    opening -= w_backward_count * OPENING_BACKWARD_PENALTY;
    endgame -= w_backward_count * ENDGAME_BACKWARD_PENALTY;

    let b_backward_pawns = get_backward_pawns(
        b_pawns,
        b_attack_frontspan,
        w_attack_frontspan.flip_orientation(),
    );
    let b_backward_count = b_backward_pawns.count_ones() as i16;
    opening += b_backward_count * OPENING_BACKWARD_PENALTY;
    endgame += b_backward_count * ENDGAME_BACKWARD_PENALTY;

    // == Open Pawns ==
    let w_open_pawns = get_open_pawns(w_pawns, b_frontspan.flip_orientation());
    let w_open_count = w_open_pawns.count_ones() as i16;
    opening += w_open_count * OPENING_OPEN_REWARD;
    endgame += w_open_count * ENDGAME_OPEN_REWARD;

    let b_open_pawns = get_open_pawns(b_pawns, w_frontspan);
    let b_open_count = b_open_pawns.count_ones() as i16;
    opening -= b_open_count * OPENING_OPEN_REWARD;
    endgame -= b_open_count * ENDGAME_OPEN_REWARD;

    // == Passed Pawns ==
    let w_passed_pawns = get_passed_pawns(
        w_pawns,
        b_frontspan.flip_orientation(),
        b_attack_frontspan.flip_orientation(),
    );
    opening += get_passed_reward(w_passed_pawns, OPENING_PASSED_REWARDS);
    endgame += get_passed_reward(w_passed_pawns, ENDGAME_PASSED_REWARDS);

    let b_passed_pawns = get_passed_pawns(
        b_pawns_mirrored,
        w_frontspan.flip_orientation(),
        w_attack_frontspan.flip_orientation(),
    );
    opening -= get_passed_reward(b_passed_pawns, OPENING_PASSED_REWARDS);
    endgame -= get_passed_reward(b_passed_pawns, ENDGAME_PASSED_REWARDS);

    // == Stragglers ==
    let w_stragglers = get_straggler_pawns(w_backward_pawns, w_open_pawns).count_ones() as i16;
    opening -= w_stragglers * OPENING_STRAGGLERS_PENALTY;
    endgame -= w_stragglers * ENDGAME_STRAGGLERS_PENALTY;

    let b_stragglers = get_straggler_pawns(b_backward_pawns, b_open_pawns).count_ones() as i16;
    opening -= b_stragglers * OPENING_STRAGGLERS_PENALTY;
    endgame -= b_stragglers * ENDGAME_STRAGGLERS_PENALTY;

    // == Pawn Shield ==
    opening += get_pawn_shield(w_pawns, w_king);
    opening -= get_pawn_shield(b_pawns_mirrored, b_king_mirrored);

    // == Central Pawn Balance
    opening += get_central_pawn_balance(w_open_pawns, b_open_pawns);

    // == King Distance To Friendly Pawn Penalty
    endgame += get_pawn_distance_penalty(w_king, w_pawns);
    endgame -= get_pawn_distance_penalty(b_king, b_pawns);

    // == King on pawnless files (king file + adjacents)
    if is_on_pawnless_file(w_king, open_files) {
        opening += OPENING_KING_PAWNLESS_FILE_PENALTY;
        endgame += ENDGAME_KING_PAWNLESS_FILE_PENALTY;
    }
    if is_on_pawnless_file(b_king, open_files) {
        opening -= OPENING_KING_PAWNLESS_FILE_PENALTY;
        endgame -= ENDGAME_KING_PAWNLESS_FILE_PENALTY;
    }

    PawnStructureEval {
        opening,
        endgame,
        p_count: p_count as u8,
    }
}

fn get_passed_reward(mut passed_pawns: u64, passed_rewards: [i16;8]) -> i16 {
    let mut r = 0;
    while passed_pawns != 0 {
        let lsb = passed_pawns.trailing_zeros() as u8;
        let rank = get_rank(lsb);
        r += passed_rewards[rank as usize];
        passed_pawns = passed_pawns.flip(lsb);
    }
    r
}

fn lookup(zorb_key: u64, p_count: u8) -> Result<Option<PawnStructureEval>, String> {
    let binding = PAWN_STRUCTURE_EVAL_HASHMAP.try_read().unwrap();
    let r = binding.get(&zorb_key);

    match r {
        Some(&result) => {
            if p_count != result.p_count {
                return Err(format!(
                    "Pawn Zorb_key conflict {zorb_key}. p_count {p_count}!={}",
                    result.p_count
                ));
            }
            Ok(Some(result))
        }
        None => Ok(None),
    }
}

fn store(zorb_key: u64, eval: PawnStructureEval) {
    PAWN_STRUCTURE_EVAL_HASHMAP
        .write()
        .unwrap()
        .insert(zorb_key, eval);
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

pub(super) fn calculate_attack_frontspan(mut pawn_occupancy: u64) -> u64 {
    let mut r = 0;
    while pawn_occupancy != 0 {
        let pos = pawn_occupancy.trailing_zeros() as u8;
        let rank = get_rank(pos);
        let file = get_file(pos);

        if file > 0 {
            r |= BOARD_FILES[file as usize - 1] << ((rank + 1) * 8)
        }

        if file < 7 {
            r |= BOARD_FILES[file as usize + 1] << ((rank + 1) * 8)
        }
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
        let l_mask = BOARD_FILES[file as usize - 1];
        if pawn_occupancy & l_mask != 0 {
            return false;
        }
    }
    if file != 7 {
        let r_mask = BOARD_FILES[file as usize + 1];
        if pawn_occupancy & r_mask != 0 {
            return false;
        }
    }
    return true;
}

fn get_pawn_shield(pawn_occupancy: u64, king_position: u8) -> i16 {
    let king_file = get_file(king_position);
    let king_rank = get_rank(king_position);
    if king_rank > 1 || !(king_file <= 2 || king_file >= 6) {
        return -5;
    }
    let rank_two_pawns = pawn_occupancy & PAWN_SHIELD_RANK_2_MASK[king_file as usize];
    let rank_three_pawns = pawn_occupancy & PAWN_SHIELD_RANK_3_MASK[king_file as usize];

    let score = rank_three_pawns.count_ones() + (rank_two_pawns.count_ones() * 2);
    PAWN_SHIELD_REWARD / 6 * (score as i16)
}

pub fn get_backward_pawns(a_pawns: u64, a_attack_frontspan: u64, b_attack_frontspan: u64) -> u64 {
    let stops = a_pawns << 8; // Move each pawn 1 push to its stop
    (stops & b_attack_frontspan & !a_attack_frontspan) >> 8
}

pub fn get_open_pawns(a_pawns: u64, b_front_span: u64) -> u64 {
    a_pawns & !b_front_span
}

pub fn get_straggler_pawns(backward_pawns: u64, open_pawns: u64) -> u64 {
    backward_pawns & open_pawns & 0xffff00 // rank 2,3
}

pub fn get_passed_pawns(a_pawns: u64, b_front_span: u64, b_attack_frontspan: u64) -> u64 {
    let b_control = b_front_span | b_attack_frontspan;
    a_pawns & !b_control
}

fn get_central_pawn_balance(w_open_pawns: u64, b_open_pawns: u64) -> i16 {
    let c_w = w_open_pawns & CENTER_FILES;
    let c_b = b_open_pawns & CENTER_FILES;
    i16::clamp(c_w.count_ones() as i16 - c_b.count_ones() as i16, -1, 1)
        * PAWN_CENTER_ADVANTAGE_REWARD
}
pub(super) fn get_open_files(pawns: u64) -> u8 {
    let mut r = 0;
    if pawns & BOARD_FILES[0] == 0 {
        r |= 1<<7;
    }
    if pawns & BOARD_FILES[1] == 0 {
        r |= 1<<6;
    }
    if pawns & BOARD_FILES[2] == 0 {
        r |= 1<<5;
    }
    if pawns & BOARD_FILES[3] == 0 {
        r |= 1<<4;
    }
    if pawns & BOARD_FILES[4] == 0 {
        r |= 1<<3;
    }
    if pawns & BOARD_FILES[5] == 0 {
        r |= 1<<2;
    }
    if pawns & BOARD_FILES[6] == 0 {
        r |= 1<<1;
    }
    if pawns & BOARD_FILES[7] == 0 {
        r |= 1;
    }
    r
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

    #[test]
    fn get_starting_position_eval() {
        let pos = Position::default();
        let eval = build_pawn_pawn_structure_eval(
            pos.board.white_occupancy & pos.board.pawn_bitboard,
            pos.board.black_occupancy & pos.board.pawn_bitboard,
            pos.board.white_king_position,
            pos.board.black_king_position,
            16,
        );
        assert_eq!(eval.opening, 0);
        assert_eq!(eval.endgame, 0);
    }

    #[test]
    fn white_pawn_advantage_in_center() {
        let w = 0.flip(index_from_coords("d2"));
        let b = 0;
        let eval = get_central_pawn_balance(w, b);
        assert_eq!(eval, PAWN_CENTER_ADVANTAGE_REWARD);
    }

    #[test]
    fn white_pawn_advantage_in_center_black_adv_outside_center() {
        let w = 0.flip(index_from_coords("d2"));
        let b = 0
            .flip(index_from_coords("b7"))
            .flip(index_from_coords("h6"));
        let eval = get_central_pawn_balance(w, b);
        assert_eq!(eval, PAWN_CENTER_ADVANTAGE_REWARD);
    }
}
