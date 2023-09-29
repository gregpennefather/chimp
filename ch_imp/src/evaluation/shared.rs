use crate::{
    board::{
        self, attack_and_defend_lookups::AttackAndDefendTable, bitboard::Bitboard,
        board_rep::BoardRep, king_position_analysis::get_pawn_threat_source, see::piece_safety,
    },
    evaluation::pawn_structure::calculate_attack_frontspan,
    move_generation::pawn::get_pawn_threat_positions,
    shared::{
        board_utils::{get_coords_from_index, index_from_coords},
        piece_type::PieceType,
    },
    MOVE_DATA,
};

use super::{eval_precomputed_data::PieceValues, pawn_structure::{calculate_frontspan, get_open_files}};

pub(super) const BOARD_FILES: [u64; 8] = [
    9259542123273814144,
    4629771061636907072,
    2314885530818453536,
    1157442765409226768,
    578721382704613384,
    289360691352306692,
    144680345676153346,
    72340172838076673,
];

pub(super) const BOARD_RANKS: [u64; 8] = [
    255,
    65280,
    16711680,
    4278190080,
    1095216660480,
    280375465082880,
    71776119061217280,
    18374686479671623680,
];

pub(super) const ABC_FLANK: u64 = BOARD_FILES[0] | BOARD_FILES[1] | BOARD_FILES [2];
pub(super) const FGH_FLANK: u64 = BOARD_FILES[5] | BOARD_FILES[6] | BOARD_FILES [7];

pub(super) const CENTER_FILES: u64 =
    BOARD_FILES[2] | BOARD_FILES[3] | BOARD_FILES[4] | BOARD_FILES[5];

pub(super) const BLACK_RANKS: u64 =
    BOARD_RANKS[7] | BOARD_RANKS[6] | BOARD_RANKS[5] | BOARD_RANKS[4];
pub(super) const WHITE_RANKS: u64 =
    BOARD_RANKS[0] | BOARD_RANKS[1] | BOARD_RANKS[2] | BOARD_RANKS[3];

const SPACE_WHITE_CENTER: u64 = 1010580480;
const SPACE_BLACK_CENTER: u64 = 16954726998343680;

pub(super) fn count_knight_outposts(
    is_black: bool,
    mut knight_occupancy: u64,
    friendly_pawn_occupancy: u64,
    opponent_pawn_occupancy: u64,
) -> i16 {
    knight_occupancy = if is_black {
        knight_occupancy & WHITE_RANKS
    } else {
        knight_occupancy & BLACK_RANKS
    };

    let mut defended_knights = 0;
    while knight_occupancy != 0 {
        let lsb = knight_occupancy.trailing_zeros() as u8;
        let pawn_defenders = get_pawn_threat_source(lsb, !is_black);
        if pawn_defenders & friendly_pawn_occupancy != 0 {
            defended_knights = defended_knights.flip(lsb);
        }
        knight_occupancy = knight_occupancy.flip(lsb)
    }

    if defended_knights == 0 {
        return 0;
    }

    // Get the opponent pawns front_attack_spans - if we're white that means that we need to flip the opponent_pawn_occupancy
    let opponent_p_attack = if is_black {
        calculate_attack_frontspan(opponent_pawn_occupancy)
    } else {
        calculate_attack_frontspan(opponent_pawn_occupancy.flip_orientation()).flip_orientation()
    };

    let outpost_knights = defended_knights & !opponent_p_attack;

    outpost_knights.count_ones() as i16
}

pub(super) fn get_fork_wins(
    is_black: bool,
    board: BoardRep,
    material_values: PieceValues,
    white_king_check: bool,
    black_king_check: bool,
    ad_table: &mut AttackAndDefendTable,
) -> i16 {
    let mut r = 0;
    let mut occupancy = if is_black {
        board.black_occupancy
    } else {
        board.white_occupancy
    } & (board.pawn_bitboard | board.knight_bitboard);

    while occupancy != 0 {
        let lsb = occupancy.trailing_zeros() as u8;
        r += match detect_fork(lsb, board, white_king_check, black_king_check, ad_table) {
            None => 0,
            Some(lvp) => material_values[lvp as usize] / 4 * 3,
        };
        occupancy = occupancy.flip(lsb);
    }

    r
}

// Adapted from Stockfish
pub(super) fn calculate_controlled_space_score(
    is_black: bool,
    board: BoardRep,
    ad_table: &mut AttackAndDefendTable,
) -> i16 {
    let piece_count =  1 + if is_black {
        board.black_occupancy.count_ones()
    } else {
        board.white_occupancy.count_ones()
    } as i16;

    let friendly_pawns = board.pawn_bitboard & if is_black { board.black_occupancy } else { board.white_occupancy};
    let open_files = get_open_files(friendly_pawns).count_ones() as i16;

    let weight = piece_count - (2*open_files);
    let controlled_area = calculate_controlled_space_area_for_player(is_black, board, ad_table);
    controlled_area * weight * weight / 16
}

// https://www.chessprogramming.org/Space
pub fn calculate_controlled_space_area_for_player(
    is_black: bool,
    board: BoardRep,
    ad_table: &mut AttackAndDefendTable,
) -> i16 {
    let mut count = 0;
    let mut space = 0;

    let mut test_space = if is_black {
        SPACE_BLACK_CENTER
    } else {
        SPACE_WHITE_CENTER
    };

    let friendly_pawns = board.pawn_bitboard
        & if is_black {
            board.black_occupancy
        } else {
            board.white_occupancy
        };
    let pawn_check_offset: i8 = if is_black { -8 } else { 8 };

    // Remove squares occupied by pawns
    test_space = test_space & !friendly_pawns;

    while test_space != 0 {
        let lsb = test_space.trailing_zeros() as u8;

        let attacks = ad_table.get_attacked_by(lsb as u8, board, !is_black);

        if attacks.pawns == 0 {
            space = space.flip(lsb);
            count += 1;
        }
        let lsb_i8 = lsb as i8;

        let behind_pawn_mask = 0
            .flip((lsb_i8 + pawn_check_offset) as u8)
            .flip((lsb_i8 + (2 * pawn_check_offset)) as u8)
            .flip((lsb_i8 + (3 * pawn_check_offset)) as u8);
        if behind_pawn_mask & friendly_pawns != 0 && !attacks.any() {
            count += 1;
        }
        test_space = test_space.flip(lsb);
    }

    count
}

pub(super) fn detect_fork(
    index: u8,
    board: BoardRep,
    white_king_check: bool,
    black_king_check: bool,
    ad_table: &mut AttackAndDefendTable,
) -> Option<PieceType> {
    let is_black = board.black_occupancy.occupied(index);
    let piece_type = board.get_piece_type_at_index(index);

    // Early exit scenario:
    // - Not our turn, opponent not in check, piece unsafe : We're open to a counter exchange
    let opponent_in_check = if is_black {
        white_king_check
    } else {
        black_king_check
    };
    if board.black_turn != is_black
        && !opponent_in_check
        && piece_safety(
            piece_type,
            false,
            ad_table.get_attacked_by(index, board, !is_black),
            ad_table.get_attacked_by(index, board, is_black),
        ) < 0
    {
        return None;
    }

    let opponent_occupancy = if is_black {
        board.white_occupancy
    } else {
        board.black_occupancy
    };

    let forked_pieces = match piece_type {
        PieceType::Pawn => get_pawn_threat_positions(index, is_black) & opponent_occupancy,
        PieceType::Knight => MOVE_DATA.knight_moves[index as usize] & opponent_occupancy,
        _ => 0,
    };

    if forked_pieces.count_ones() > 1 {
        get_least_valueable_piece_in_mask(forked_pieces, board)
    } else {
        None
    }
}

pub(super) fn get_least_valueable_piece_in_mask(mask: u64, board: BoardRep) -> Option<PieceType> {
    if board.pawn_bitboard & mask != 0 {
        Some(PieceType::Pawn)
    } else if board.knight_bitboard & mask != 0 {
        Some(PieceType::Knight)
    } else if board.bishop_bitboard & mask != 0 {
        Some(PieceType::Bishop)
    } else if board.rook_bitboard & mask != 0 {
        Some(PieceType::Rook)
    } else if board.queen_bitboard & mask != 0 {
        Some(PieceType::Queen)
    } else if mask.occupied(board.white_king_position) || mask.occupied(board.black_king_position) {
        Some(PieceType::King)
    } else {
        None
    }
}

pub(super) fn get_pawn_controlled_squares(is_black: bool, board: BoardRep) -> u64 {
    let mut control = 0;

    let mut pawn_occupancy = board.pawn_bitboard & if is_black { board.black_occupancy } else { board.white_occupancy };

    while pawn_occupancy != 0 {
        let lsb = pawn_occupancy.trailing_zeros() as u8;
        control |= get_pawn_threat_positions(lsb, is_black);
        pawn_occupancy = pawn_occupancy.flip(lsb)
    }

    control
}

#[cfg(test)]
mod test {
    use crate::{
        board::{attack_and_defend_lookups::AttackAndDefendTable, board_rep::BoardRep},
        shared::{board_utils::index_from_coords, piece_type::PieceType},
    };

    use super::{calculate_controlled_space_area_for_player, count_knight_outposts, detect_fork};

    #[test]
    fn white_knight_with_no_pawn_support() {
        let board =
            BoardRep::from_fen("r4rk1/ppp2ppp/3p2n1/3N4/8/8/PPP2PPP/2KRR3 w - - 0 1".into());

        let r = count_knight_outposts(
            false,
            board.knight_bitboard & board.white_occupancy,
            board.white_occupancy & board.pawn_bitboard,
            board.black_occupancy & board.pawn_bitboard,
        );

        assert_eq!(r, 0);
    }

    #[test]
    fn white_knight_with_pawn_support_attackable_by_enemy_pawn() {
        let board =
            BoardRep::from_fen("r4rk1/ppp2ppp/3p2n1/3N4/4P3/8/PPP2PPP/2KRR3 b - - 1 1".into());

        let r = count_knight_outposts(
            false,
            board.knight_bitboard & board.white_occupancy,
            board.white_occupancy & board.pawn_bitboard,
            board.black_occupancy & board.pawn_bitboard,
        );

        assert_eq!(r, 0);
    }

    #[test]
    fn white_knight_with_pawn_support_no_longer_attackable_by_enemy_pawns() {
        let board =
            BoardRep::from_fen("r4rk1/pp3ppp/3p2n1/2pN4/2P5/8/PPP2PPP/2KRR3 b - - 1 1".into());

        let r = count_knight_outposts(
            false,
            board.knight_bitboard & board.white_occupancy,
            board.white_occupancy & board.pawn_bitboard,
            board.black_occupancy & board.pawn_bitboard,
        );

        assert_eq!(r, 1);
    }

    #[test]
    fn black_knight_with_pawn_support_no_longer_attackable_by_enemy_pawns() {
        let board =
            BoardRep::from_fen("r4rk1/pp4pp/3p4/2pNp3/5n2/8/PPP2P1P/2KRR3 b - - 1 1".into());

        let r = count_knight_outposts(
            true,
            board.knight_bitboard & board.black_occupancy,
            board.black_occupancy & board.pawn_bitboard,
            board.white_occupancy & board.pawn_bitboard,
        );

        assert_eq!(r, 1);
    }

    #[test]
    fn detect_fork_pawn_undefended_on_opponent_turn() {
        let board = BoardRep::from_fen("k7/8/8/2r1b3/3P4/8/8/K7 b - - 0 1".into());

        let r = detect_fork(
            index_from_coords("d4"),
            board,
            false,
            false,
            &mut AttackAndDefendTable::new(),
        );

        assert_eq!(r, None);
    }

    #[test]
    fn detect_fork_pawn_undefended_on_friendly_turn() {
        let board = BoardRep::from_fen("k7/8/8/2r1b3/3P4/8/8/K7 w - - 0 1".into());

        let r = detect_fork(
            index_from_coords("d4"),
            board,
            false,
            false,
            &mut AttackAndDefendTable::new(),
        );

        assert_ne!(r, None);
    }

    #[test]
    fn detect_fork_pawn_undefended_on_opponent_turn_but_opponent_in_check() {
        let board = BoardRep::from_fen("k7/8/8/2r1b3/3P4/8/R7/1K6 b - - 0 1".into());

        let r = detect_fork(
            index_from_coords("d4"),
            board,
            false,
            true,
            &mut AttackAndDefendTable::new(),
        );

        assert_ne!(r, None);
    }

    #[test]
    fn detect_fork_pawn_forking_bishop_and_rook() {
        let board = BoardRep::from_fen("k7/8/8/2r1b3/3P4/4P3/8/K7 w - - 0 1".into());

        let r = detect_fork(
            index_from_coords("d4"),
            board,
            false,
            false,
            &mut AttackAndDefendTable::new(),
        );

        assert_eq!(r, Some(PieceType::Bishop))
    }

    #[test]
    fn detect_knight_forking_bishop_and_queen() {
        let board =
            BoardRep::from_fen("rnb2knr/ppN2ppp/4q2b/8/8/8/PPPPPPPP/RNBQKB1R w KQkq - 0 1".into());

        let r = detect_fork(
            index_from_coords("c7"),
            board,
            false,
            false,
            &mut AttackAndDefendTable::new(),
        );

        assert_eq!(r, Some(PieceType::Rook))
    }

    #[test]
    fn detect_knight_forking_bishop_and_king() {
        let board =
            BoardRep::from_fen("rnb1k1nr/ppN2ppp/7b/7q/8/8/PPPPPPPP/RNBQKB1R b KQkq - 0 1".into());

        let r = detect_fork(
            index_from_coords("c7"),
            board,
            false,
            true,
            &mut AttackAndDefendTable::new(),
        );

        assert_eq!(r, Some(PieceType::Rook))
    }

    #[test]
    fn detect_fork_case_0() {
        let board = BoardRep::from_fen(
            "1rq2rk1/1b3p1p/p2b1np1/3pP3/Q2P4/P1N5/3BB1PP/1R3RK1 b - - 0 3".into(),
        );

        let r = detect_fork(
            index_from_coords("e5"),
            board,
            false,
            false,
            &mut AttackAndDefendTable::new(),
        );

        assert_eq!(r, Some(PieceType::Knight))
    }

    #[test]
    fn detect_fork_case_1() {
        let board = BoardRep::from_fen(
            "1rq2rk1/1b3p1p/p2b1np1/3pP3/Q2P4/P1N5/3BB1PP/1R3RK1 b - - 0 3".into(),
        );

        let r = detect_fork(
            index_from_coords("c3"),
            board,
            false,
            false,
            &mut AttackAndDefendTable::new(),
        );

        assert_eq!(r, None)
    }

    #[test]
    fn calculate_controlled_space_case_0() {
        let board = BoardRep::from_fen(
            "rn1qk2r/pp5p/1b1bP1p1/1Pp2pn1/1QP2P2/P7/7P/RNB1K2R w KQkq - 1 9
        "
            .into(),
        );

        let white: i16 = calculate_controlled_space_area_for_player(
            false,
            board,
            &mut AttackAndDefendTable::new(),
        );
        let black: i16 = calculate_controlled_space_area_for_player(
            true,
            board,
            &mut AttackAndDefendTable::new(),
        );

        assert_eq!(white, 12);
        assert_eq!(black, 7);
    }
}
