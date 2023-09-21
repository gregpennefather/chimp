use crate::{shared::piece_type::PieceType, MOVE_DATA};

use super::{
    bitboard::Bitboard, board_rep::BoardRep, king_position_analysis::get_pawn_threat_source,
};

pub struct AttackedBy {
    pub pawns: u8,
    pub knights: u8,
    pub bishop: bool,
    pub rooks: u8,
    pub queen: bool,
}
impl AttackedBy {
    pub(crate) fn pop_least_valuable(&mut self) -> PieceType {
        if self.pawns > 0 {
            self.pawns -= 1;
            return PieceType::Pawn;
        }

        if self.knights > 0 {
            self.knights -= 1;
            return PieceType::Knight;
        }

        if self.bishop {
            self.bishop = false;
            return PieceType::Bishop;
        }

        if self.rooks > 0 {
            self.rooks -= 1;
            return PieceType::Rook;
        }

        if self.queen {
            self.queen = false;
            return PieceType::Queen;
        }

        PieceType::None
    }

    pub(crate) fn remove(&mut self, piece_type: PieceType) {
        match piece_type {
            PieceType::Pawn => {
                assert_ne!(self.pawns, 0);
                self.pawns -= 1;
            },PieceType::Knight => {
                assert_ne!(self.knights, 0);
                self.knights -= 1;
            },PieceType::Rook => {
                assert_ne!(self.rooks, 0);
                self.rooks -= 1;
            },PieceType::Bishop => {
                assert_ne!(self.bishop, false);
                self.bishop = false;
            },PieceType::Queen => {
                assert_ne!(self.queen, false);
                self.queen = false;
            },
            _ => panic!("Unexpected piece type {piece_type:?}")
        }
    }
}

impl BoardRep {
    pub fn get_attacked_by(&self, index: u8, attacker_is_black: bool) -> AttackedBy {
        let attacker_occupancy = if attacker_is_black {
            self.black_occupancy
        } else {
            self.white_occupancy
        };
        let pawns = get_pawn_threats(
            index,
            attacker_occupancy & self.pawn_bitboard,
            self.black_turn,
        );
        let knights = get_knight_threat_count(index, attacker_occupancy & self.knight_bitboard);
        let bishop = has_bishop_threat(
            index,
            attacker_occupancy & self.bishop_bitboard,
            self.occupancy,
        );
        let rooks = get_rook_threat_count(
            index,
            attacker_occupancy & self.rook_bitboard,
            self.occupancy,
        );
        let queen = has_queen_threat(
            index,
            attacker_occupancy & self.queen_bitboard,
            self.occupancy,
        );
        AttackedBy {
            pawns,
            knights,
            bishop,
            rooks,
            queen,
        }
    }
}

fn get_pawn_threats(index: u8, att_p_occ: u64, black_turn: bool) -> u8 {
    let pawn_threats = att_p_occ & get_pawn_threat_source(index, black_turn);
    pawn_threats.count_occupied()
}

fn get_knight_threat_count(index: u8, att_k_occupancy: u64) -> u8 {
    let knight_threats = att_k_occupancy & MOVE_DATA.knight_moves[index as usize];
    knight_threats.count_occupied()
}

fn get_rook_threat_count(index: u8, att_r_occupancy: u64, occupancy: u64) -> u8 {
    let occ_without_rooks = occupancy ^ att_r_occupancy;
    let moveboard = MOVE_DATA
        .magic_bitboard_table
        .get_rook_attacks(index as usize, occ_without_rooks);
    (moveboard & att_r_occupancy).count_occupied()
}

fn has_bishop_threat(index: u8, att_b_occ: u64, occupancy: u64) -> bool {
    let moveboard = MOVE_DATA
        .magic_bitboard_table
        .get_bishop_attacks(index as usize, occupancy);
    moveboard & att_b_occ != 0
}

fn has_queen_threat(index: u8, att_q_occ: u64, occupancy: u64) -> bool {
    let moveboard = MOVE_DATA
        .magic_bitboard_table
        .get_bishop_attacks(index as usize, occupancy)
        | MOVE_DATA
            .magic_bitboard_table
            .get_rook_attacks(index as usize, occupancy);
    moveboard & att_q_occ != 0
}

#[cfg(test)]
mod test {
    use crate::shared::board_utils::index_from_coords;

    use super::*;

    #[test]
    pub fn double_pawn_attack() {
        let board = BoardRep::from_fen("k7/8/3p1pb1/4P3/8/2N5/8/K7 w - - 0 1".into());
        let r = board.get_attacked_by(index_from_coords("e5"), true);
        assert_eq!(r.pawns, 2);
        assert_eq!(r.knights, 0);
        assert_eq!(r.bishop, false);
        assert_eq!(r.rooks, 0);
        assert_eq!(r.queen, false);
    }

    #[test]
    pub fn one_knight_defender() {
        let board = BoardRep::from_fen("k7/8/3p1pb1/4P3/8/3N4/8/K7 w - - 0 1".into());
        let r = board.get_attacked_by(index_from_coords("e5"), false);
        assert_eq!(r.pawns, 0);
        assert_eq!(r.knights, 1);
        assert_eq!(r.bishop, false);
        assert_eq!(r.rooks, 0);
        assert_eq!(r.queen, false);
    }

    #[test]
    pub fn one_bishop_attacker() {
        let board = BoardRep::from_fen("k7/8/3p1pb1/4P3/8/3N4/8/K7 w - - 0 1".into());
        let r = board.get_attacked_by(index_from_coords("d3"), true);
        assert_eq!(r.pawns, 0);
        assert_eq!(r.knights, 0);
        assert_eq!(r.bishop, true);
        assert_eq!(r.rooks, 0);
        assert_eq!(r.queen, false);
    }

    #[test]
    pub fn one_queen_defender() {
        let board = BoardRep::from_fen("k7/8/3p1pb1/4P3/8/3N4/8/K4Q2 w - - 0 1".into());
        let r = board.get_attacked_by(index_from_coords("d3"), false);
        assert_eq!(r.pawns, 0);
        assert_eq!(r.knights, 0);
        assert_eq!(r.bishop, false);
        assert_eq!(r.rooks, 0);
        assert_eq!(r.queen, true);
    }

    #[test]
    pub fn stacked_rook_attackers() {
        let board = BoardRep::from_fen("k7/8/3p1pb1/4P3/4N3/8/3R4/K2R1Q2 w - - 0 1".into());
        let r = board.get_attacked_by(index_from_coords("d6"), false);
        assert_eq!(r.pawns, 0);
        assert_eq!(r.knights, 1);
        assert_eq!(r.bishop, false);
        assert_eq!(r.rooks, 2);
        assert_eq!(r.queen, false);
    }
}
