use crate::{shared::piece_type::PieceType, MOVE_DATA};

use super::{
    bitboard::Bitboard, board_rep::BoardRep, king_position_analysis::get_pawn_threat_source,
};

#[derive(Clone, Copy, Debug, Default)]
pub struct AttackedBy {
    pub pawns: u8,
    pub knights: u8,
    pub bishop: bool,
    pub rooks: u8,
    pub queen: bool,
    pub king: bool,
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

        if self.king {
            self.king = false;
            return PieceType::King;
        }

        PieceType::None
    }

    pub(crate) fn remove(&mut self, piece_type: PieceType) {
        match piece_type {
            PieceType::Pawn => {
                assert_ne!(self.pawns, 0);
                self.pawns -= 1;
            }
            PieceType::Knight => {
                assert_ne!(self.knights, 0);
                self.knights -= 1;
            }
            PieceType::Rook => {
                assert_ne!(self.rooks, 0);
                self.rooks -= 1;
            }
            PieceType::Bishop => {
                assert_ne!(self.bishop, false);
                self.bishop = false;
            }
            PieceType::Queen => {
                // No assert here as sometimes we pretend a King is a queen during evaluation, and this remove would fail when generating moves for it
                self.queen = false;
            }
            PieceType::King => {
                assert_ne!(self.king, false);
                self.king = false;
            }
            _ => panic!("Unexpected piece type {piece_type:?}"),
        }
    }

    pub(crate) fn any(&self) -> bool {
        self.queen
            || self.bishop
            || self.king
            || self.rooks > 0
            || self.knights > 0
            || self.pawns > 0
    }
}

#[derive(Clone, Copy)]
pub struct AttackAndDefendTable {
    pub white: [Option<AttackedBy>; 64],
    pub black: [Option<AttackedBy>; 64],
}

impl AttackAndDefendTable {
    pub fn new() -> Self {
        Self {
            white: [None; 64],
            black: [None; 64],
        }
    }

    pub fn get_attacked_by(&mut self, index: u8, board: BoardRep, attacker_is_black: bool) -> AttackedBy {
        let existing = if attacker_is_black {
            self.black[index as usize]
        } else {
            self.white[index as usize]
        };

        match existing {
            Some(e) => e,
            None => {
                let e = board.get_attacked_by(index, attacker_is_black);
                if attacker_is_black {
                    self.black[index as usize] = Some(e);
                } else {
                    self.white[index as usize] = Some(e);
                }
                e
            }
        }
    }

    pub fn has_at_least_one_attacker(self, index: u8, attacker_is_black: bool, exclude_king: bool, board: BoardRep) -> bool {
        let existing = if attacker_is_black {
            self.black[index as usize]
        } else {
            self.white[index as usize]
        };

        match existing {
            None => board.has_at_least_one_attacker(index, attacker_is_black, exclude_king),
            Some(e) => e.any()
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
            !attacker_is_black,
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
            attacker_occupancy & self.bishop_bitboard,
            attacker_occupancy & self.rook_bitboard
        );
        let king = has_king_threat(
            index,
            if attacker_is_black {
                self.black_king_position
            } else {
                self.white_king_position
            },
        );
        AttackedBy {
            pawns,
            knights,
            bishop,
            rooks,
            queen,
            king,
        }
    }

    pub fn has_at_least_one_attacker(&self, index: u8, attacker_is_black: bool, exclude_king: bool) -> bool {
        let attacker_occupancy = if attacker_is_black {
            self.black_occupancy
        } else {
            self.white_occupancy
        };

        let mut occupancy = self.occupancy;

        if exclude_king {
             if attacker_is_black {
                occupancy = occupancy.flip(self.white_king_position);
            } else {
                occupancy = occupancy.flip(self.black_king_position);
            }
        }

        (get_pawn_threats(
            index,
            attacker_occupancy & self.pawn_bitboard,
            !attacker_is_black,
        ) > 0)
            || (get_knight_threat_count(index, attacker_occupancy & self.knight_bitboard) > 0)
            || has_bishop_threat(
                index,
                attacker_occupancy & self.bishop_bitboard,
                occupancy,
            )
            || (get_rook_threat_count(
                index,
                attacker_occupancy & self.rook_bitboard,
                occupancy,
            ) > 0)
            || has_queen_threat(
                index,
                attacker_occupancy & self.queen_bitboard,
                occupancy,
                attacker_occupancy & self.bishop_bitboard,
                attacker_occupancy & self.rook_bitboard
            )
            || has_king_threat(
                index,
                if attacker_is_black {
                    self.black_king_position
                } else {
                    self.white_king_position
                },
            )
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

fn has_queen_threat(index: u8, att_q_occ: u64, occupancy: u64, b_occupancy: u64, r_occupancy: u64) -> bool {
    let occupancy = occupancy & !b_occupancy & !r_occupancy;

    let moveboard = MOVE_DATA
        .magic_bitboard_table
        .get_bishop_attacks(index as usize, occupancy)
        | MOVE_DATA
            .magic_bitboard_table
            .get_rook_attacks(index as usize, occupancy);
    moveboard & att_q_occ != 0
}

fn has_king_threat(index: u8, a_king_pos: u8) -> bool {
    let moveboard = MOVE_DATA.king_moves[a_king_pos as usize];
    moveboard.occupied(index)
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
        assert_eq!(r.pawns, 1);
        assert_eq!(r.knights, 1);
        assert_eq!(r.bishop, false);
        assert_eq!(r.rooks, 2);
        assert_eq!(r.queen, false);
    }

    #[test]
    pub fn unthreatened_square() {
        let board = BoardRep::from_fen("1nb1kbnr/pp1rpppp/8/2p5/4PP2/8/PPPqK1PP/R4BNR w k - 0 1".into());
        let r = board.get_attacked_by(index_from_coords("f3"), true);
        assert_eq!(r.pawns, 0);
        assert_eq!(r.knights, 0);
        assert_eq!(r.bishop, false);
        assert_eq!(r.rooks, 0);
        assert_eq!(r.queen, false);
        assert_eq!(r.king, false);
        assert!(!board.has_at_least_one_attacker(index_from_coords("f3"), true, true));
    }

    #[test]
    pub fn queen_behind_bishop_counts_as_attacker() {
        let board = BoardRep::from_fen("1rq2rk1/4bp2/2np2p1/p1p1p3/P1PNP1P1/1PB2P2/1Q4P1/R2R1NK1 b - - 0 1".into());
        let r = board.get_attacked_by(index_from_coords("d4"), false);
        assert_eq!(r.pawns, 0);
        assert_eq!(r.knights, 0);
        assert_eq!(r.bishop, true);
        assert_eq!(r.rooks, 1);
        assert_eq!(r.queen, true);
        assert_eq!(r.king, false);
    }

    #[test]
    pub fn queen_behind_rook_counts_as_attacker() {
        let board = BoardRep::from_fen("1rq2rk1/4bp2/2np2p1/p1p1p3/P1PNP1P1/1PB2P2/3R2P1/R2Q1NK1 b - - 0 1".into());
        let r = board.get_attacked_by(index_from_coords("d4"), false);
        assert_eq!(r.pawns, 0);
        assert_eq!(r.knights, 0);
        assert_eq!(r.bishop, true);
        assert_eq!(r.rooks, 1);
        assert_eq!(r.queen, true);
        assert_eq!(r.king, false);
    }
}
