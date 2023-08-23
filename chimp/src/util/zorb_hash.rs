use rand::Rng;

use crate::{
    board::{
        bitboard::BitboardExtensions,
        piece::Piece,
        r#move::Move,
        state::{BoardState, BoardStateFlagsTrait},
    },
    shared::*,
};

const WHITE_PAWN_ID: usize = 0;
const WHITE_KNIGHT_ID: usize = 1;
const WHITE_BISHOP_ID: usize = 2;
const WHITE_ROOK_ID: usize = 3;
const WHITE_QUEEN_ID: usize = 4;
const WHITE_KING_ID: usize = 5;
const BLACK_PAWN_ID: usize = 6;
const BLACK_KNIGHT_ID: usize = 7;
const BLACK_BISHOP_ID: usize = 8;
const BLACK_ROOK_ID: usize = 9;
const BLACK_QUEEN_ID: usize = 10;
const BLACK_KING_ID: usize = 11;
type PositionChange = (u8, usize);

fn get_piece_id(p: Piece) -> usize {
    match p.0 {
        PAWN_INDEX => WHITE_PAWN_ID,
        KNIGHT_INDEX => WHITE_KNIGHT_ID,
        BISHOP_INDEX => WHITE_BISHOP_ID,
        ROOK_INDEX => WHITE_ROOK_ID,
        QUEEN_INDEX => WHITE_QUEEN_ID,
        KING_INDEX => WHITE_KING_ID,
        BLACK_PAWN => BLACK_PAWN_ID,
        BLACK_KNIGHT => BLACK_KNIGHT_ID,
        BLACK_BISHOP => BLACK_BISHOP_ID,
        BLACK_ROOK => BLACK_ROOK_ID,
        BLACK_QUEEN => BLACK_QUEEN_ID,
        BLACK_KING => BLACK_KING_ID,
        _ => panic!("Unknown piece"),
    }
}

impl BoardState {
    pub fn get_position_changes(&self, m: Move) -> Vec<PositionChange> {
        let mut changes = Vec::new();
        let from_index: u8 = m.from();
        let to_index: u8 = m.to();
        if m.is_castling() {
            let (rook_from_index, rook_to_index) = if m.is_king_castling() {
                (from_index - 3, to_index + 1)
            } else {
                (from_index + 4, to_index - 1)
            };
            let king = self.pieces.get_by_position_index(self.bitboard, from_index);
            let rook = self
                .pieces
                .get_by_position_index(self.bitboard, rook_from_index);

            changes.push((from_index, get_piece_id(king))); // remove king;
            changes.push((rook_from_index, get_piece_id(rook))); // remove rook;
            changes.push((to_index, get_piece_id(king))); // place king
            changes.push((rook_to_index, get_piece_id(rook))); // place rook;
        } else if m.is_promotion() {
            let pawn = self.pieces.get_by_position_index(self.bitboard, from_index);
            let new_piece = Piece::new_coloured(
                match m.flags() {
                    8 | 12 => KNIGHT_INDEX.into(),
                    9 | 13 => BISHOP_INDEX.into(),
                    10 | 14 => ROOK_INDEX.into(),
                    11 | 15 => QUEEN_INDEX.into(),
                    _ => panic!("Unknown promotion"),
                },
                self.flags.is_black_turn(),
            );

            changes.push((from_index, get_piece_id(pawn))); // remove pawn;

            if m.is_capture() {
                let capture_piece = self.pieces.get_by_position_index(self.bitboard, to_index);

                changes.push((to_index, get_piece_id(capture_piece))); // remove captured piece
            }

            changes.push((to_index, get_piece_id(new_piece))); // remove new_piece;
        } else {
            let piece = self.pieces.get_by_position_index(self.bitboard, from_index);

            changes.push((from_index, get_piece_id(piece))); // remove piece;

            if m.is_capture() {
                let capture_piece = self.pieces.get_by_position_index(self.bitboard, to_index);

                changes.push((to_index, get_piece_id(capture_piece))); // remove captured piece
            }

            changes.push((to_index, get_piece_id(piece))); // place piece
        }
        changes
    }
}

#[derive(Clone, Copy)] // This doesn't feel right but is necessary right now
pub struct ZorbSet {
    table: [[u64; 12]; 64],
    black_turn: u64,
}

// https://en.wikipedia.org/wiki/Zobrist_hashing
// #TODO: Optimize by adding a 'modify' that applies a XOR to allow for move_application
impl ZorbSet {
    pub fn new() -> Self {
        let mut arr: [[u64; 12]; 64] = [[0u64; 12]; 64];
        let mut rng = rand::thread_rng();

        for i in 0..64 {
            for j in 0..12 {
                arr[i][j] = rng.gen();
            }
        }
        let black_turn = rng.gen();
        Self {
            table: arr,
            black_turn,
        }
    }

    pub fn hash(&self, board_state: BoardState) -> u64 {
        let mut r = 0;
        if board_state.flags.is_black_turn() {
            r ^= self.black_turn;
        }
        for position_index in 0..64 {
            if !board_state.bitboard.occupied(position_index as u8) {
                continue;
            }
            let piece = board_state
                .pieces
                .get_by_position_index(board_state.bitboard, position_index as u8);
            r ^= match piece.0 {
                PAWN_INDEX => self.table[position_index][WHITE_PAWN_ID],
                KNIGHT_INDEX => self.table[position_index][WHITE_KNIGHT_ID],
                BISHOP_INDEX => self.table[position_index][WHITE_BISHOP_ID],
                ROOK_INDEX => self.table[position_index][WHITE_ROOK_ID],
                QUEEN_INDEX => self.table[position_index][WHITE_QUEEN_ID],
                KING_INDEX => self.table[position_index][WHITE_KING_ID],
                BLACK_PAWN => self.table[position_index][BLACK_PAWN_ID],
                BLACK_KNIGHT => self.table[position_index][BLACK_KNIGHT_ID],
                BLACK_BISHOP => self.table[position_index][BLACK_BISHOP_ID],
                BLACK_ROOK => self.table[position_index][BLACK_ROOK_ID],
                BLACK_QUEEN => self.table[position_index][BLACK_QUEEN_ID],
                BLACK_KING => self.table[position_index][BLACK_KING_ID],
                _ => panic!("Unknown piece"),
            }
        }
        r
    }

    pub fn shift(&self, zorb: u64, position_change: PositionChange) -> u64 {
        zorb ^ self.table[position_change.0 as usize][position_change.1]
    }

    pub fn colour_shift(&self, zorb: u64) -> u64 {
        zorb ^ self.black_turn
    }
}
