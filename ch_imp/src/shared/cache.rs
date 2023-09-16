use std::mem::size_of;

use log::info;

use crate::{board::position::Position, r#move::Move};

const POSITION_CACHE_MB_SIZE: usize = 256;
const MOVES_CACHE_MB_SIZE: usize = 64;

const POSITION_CACHE_SIZE: usize =
    (POSITION_CACHE_MB_SIZE * 1204 * 1204) / size_of::<Option<Position>>();
const MOVES_CACHE_SIZE: usize =
    (MOVES_CACHE_MB_SIZE * 1204 * 1204) / size_of::<Option<MoveCacheEntry>>();

#[derive(PartialEq, Clone)]
pub struct MoveCacheEntry {
    zorb_key: u64,
    moves: Vec<Move>,
}

pub struct PositionCache {
    table: Box<[Option<Position>; POSITION_CACHE_SIZE]>,
    pub hits: usize,
    pub misses: usize,
}

impl PositionCache {
    pub fn new() -> Self {
        // https://stackoverflow.com/a/56426372
        let table = {
            let mut v: Vec<Option<Position>> = Vec::with_capacity(POSITION_CACHE_SIZE);
            unsafe {
                v.set_len(POSITION_CACHE_SIZE);
            };
            let mut slice = v.into_boxed_slice();
            for i in &mut slice[..] {
                *i = None;
            }
            let raw_slice = Box::into_raw(slice);
            // Using `from_raw` is safe as long as the pointer is
            // retrieved using `into_raw`.
            unsafe { Box::from_raw(raw_slice as *mut [Option<Position>; POSITION_CACHE_SIZE]) }
        };
        Self {
            table,
            hits: 0,
            misses: 0,
        }
    }

    pub fn lookup(&mut self, zorb_key: u64) -> Option<Position> {
        let index = (zorb_key as usize) % POSITION_CACHE_SIZE;
        let e = self.table[index];
        if e != None && e.unwrap().board.zorb_key == zorb_key {
            self.hits += 1;
            return e;
        }
        self.misses += 1;
        None
    }

    pub fn record(&mut self, zorb_key: u64, e: Position) {
        let index = (zorb_key as usize) % POSITION_CACHE_SIZE;
        self.table[index] = Some(e)
    }
}

pub struct MovesCache {
    table: Box<[Option<MoveCacheEntry>; MOVES_CACHE_SIZE]>,
    pub hits: usize,
    pub misses: usize,
}
impl MovesCache {
    pub fn new() -> Self {
        // https://stackoverflow.com/a/56426372
        let table = {
            let mut v: Vec<Option<MoveCacheEntry>> = Vec::with_capacity(MOVES_CACHE_SIZE);
            unsafe {
                v.set_len(MOVES_CACHE_SIZE);
            };
            let mut slice = v.into_boxed_slice();
            for i in &mut slice[..] {
                *i = None;
            }
            let raw_slice = Box::into_raw(slice);
            // Using `from_raw` is safe as long as the pointer is
            // retrieved using `into_raw`.
            unsafe { Box::from_raw(raw_slice as *mut [Option<MoveCacheEntry>; MOVES_CACHE_SIZE]) }
        };
        Self {
            table,
            hits: 0,
            misses: 0,
        }
    }

    pub fn lookup(&mut self, zorb_key: u64) -> Option<Vec<Move>> {
        let index = (zorb_key as usize) % MOVES_CACHE_SIZE;
        let options = &self.table[index];
        match options {
            Some(cache_entry) => {
                if cache_entry.zorb_key == zorb_key {
                    self.hits += 1;
                    return Some(cache_entry.moves.clone())
                }
            },
            None => {}
        }
        self.misses += 1;
        None
    }
    pub fn record(&mut self, zorb_key: u64, e: Vec<Move>) {
        let index = (zorb_key as usize) % MOVES_CACHE_SIZE;
        self.table[index] = Some(MoveCacheEntry { zorb_key, moves: e })
    }
}
