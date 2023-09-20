use std::mem::size_of;

use crate::r#move::Move;

const TRANSPOSITION_TABLE_MB_SIZE: usize = 64;

const TRANSPOSITION_TABLE_SIZE: usize =
    (TRANSPOSITION_TABLE_MB_SIZE * 1204 * 1204) / size_of::<Option<TransTableEntry>>();

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum NodeType {
    PVNode,
    CutNode,
    AllNode,
}

#[derive(Clone, Copy, Debug)]
pub struct TransTableEntry {
    zorb_key: u64,
    depth: u8,
    value: i16,
    t: NodeType,
    m: Option<Move>,
}

pub struct TranspositionTable {
    table: Box<[Option<TransTableEntry>; TRANSPOSITION_TABLE_SIZE]>,
}

impl TranspositionTable {
    pub fn new() -> Self {
        // https://stackoverflow.com/a/56426372
        let table = {
            let mut v: Vec<Option<TransTableEntry>> = Vec::with_capacity(TRANSPOSITION_TABLE_SIZE);
            unsafe {
                v.set_len(TRANSPOSITION_TABLE_SIZE);
            };
            let mut slice = v.into_boxed_slice();
            for i in &mut slice[..] {
                *i = None;
            }
            let raw_slice = Box::into_raw(slice);
            // Using `from_raw` is safe as long as the pointer is
            // retrieved using `into_raw`.
            unsafe {
                Box::from_raw(raw_slice as *mut [Option<TransTableEntry>; TRANSPOSITION_TABLE_SIZE])
            }
        };
        Self { table }
    }

    pub fn lookup(&self, zorb_key: u64, depth: u8, alpha: i16, beta: i16) -> Option<(i16, Move)> {
        let index = (zorb_key as usize) % TRANSPOSITION_TABLE_SIZE;
        let option = self.table[index];
        match option {
            Some(entry) => {
                if entry.zorb_key == zorb_key {
                    if entry.depth >= depth {
                        match entry.t {
                            // Exact value known - return the value + move
                            NodeType::PVNode => return Some((entry.value, entry.m.unwrap())),
                            // This was a fail low node - this score is the upper bound of all searched nodes and the
                            // real value may be less. As a result if the upper bound is less than current alpha we know that none of these
                            // nodes are going to improve our alpha and thus aren't worth considering. Returning alpha allows us to prune this search branch
                            NodeType::AllNode => {
                                if entry.value <= alpha {
                                    return Some((alpha, Move::default()));
                                }
                            }
                            // This was a fail high node - this score is the lower bound of the searched nodes and the real value may be
                            // higher. As a result if the lower bound is higher than current beta we know that we would trim all these
                            // nodes due to beta cutoff. Returning beta here allows us to prune this search branch
                            NodeType::CutNode => {
                                if entry.value >= beta {
                                    return Some((beta, Move::default()));
                                }
                            }
                        }
                    }
                }
                return None;
            }
            None => None,
        }
    }

    pub fn get_move(&self, zorb_key: u64) -> Option<Move> {
        let index = (zorb_key as usize) % TRANSPOSITION_TABLE_SIZE;
        let option = self.table[index];
        match option {
            Some(entry) => {
                if entry.zorb_key == zorb_key {
                    entry.m
                } else {
                    None
                }
            }
            None => None,
        }
    }

    pub fn record(&mut self, zorb_key: u64, depth: u8, value: i16, t: NodeType, m: Option<Move>) {
        let index = (zorb_key as usize) % TRANSPOSITION_TABLE_SIZE;
        self.table[index] = Some(TransTableEntry {
            zorb_key,
            depth,
            value,
            t,
            m,
        })
    }
}
