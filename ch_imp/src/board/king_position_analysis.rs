use crate::{
    shared::board_utils::{get_file, get_rank},
    MOVE_DATA,
};

use super::bitboard::Bitboard;

const DIAGONAL_DELTAS: [(i8, i8); 4] = [(1, 1), (-1, 1), (-1, -1), (1, -1)];
const ORTHOGONAL_DELTAS: [(i8, i8); 4] = [(1, 0), (0, 1), (0, -1), (-1, 0)];

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum ThreatType {
    None,
    DiagonalSlide,
    OrthogonalSlide,
    Knight,
    Pawn,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct ThreatRaycastCollision {
    pub from: u8,
    pub at: u8,
    pub reveal_attack: bool,
    pub threat_type: ThreatType,
    pub threat_ray_mask: u64,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct ThreatSource {
    pub from: u8,
    pub threat_type: ThreatType,
    pub threat_ray_mask: u64,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct KingPositionAnalysis {
    pub check: bool,
    pub double_check: bool, // In double check only a king move can escape check so we can generate less moves
    pub threat_source: Option<ThreatSource>,
    pub pins: Vec<ThreatRaycastCollision>,
}

pub fn analyze_king_position(
    king_pos: u8,
    black_king: bool,
    occupancy: u64,
    friendly_occupancy: u64,
    opponent_occupancy: u64,
    pawn_bitboard: u64,
    knight_bitboard: u64,
    bishop_bitboard: u64,
    rook_bitboard: u64,
    queen_bitboard: u64,
    shallow: bool,
) -> KingPositionAnalysis {
    let mut check = false;
    let mut double_check = false;
    let mut threat_source = None;

    let mut pins = Vec::new();

    let knights: u64 = knight_bitboard & opponent_occupancy;
    if knights.count_ones() > 0 {
        let (c, dc, ts) = in_check_from_knights(king_pos, knights, check);

        double_check |= dc;
        check |= c;
        if ts != None {
            threat_source = ts;
        }

        if double_check || (check && shallow) {
            return KingPositionAnalysis {
                check: true,
                double_check: true,
                threat_source,
                pins,
            };
        }
    }

    let diagonal_threats = (bishop_bitboard | queen_bitboard) & opponent_occupancy;
    if diagonal_threats.count_ones() > 0 {
        let (c, dc, ts, p) = in_sliding_check(
            king_pos,
            true,
            diagonal_threats,
            occupancy,
            friendly_occupancy,
            check,
        );

        double_check |= dc;
        check |= c;
        if ts != None {
            threat_source = ts;
        }
        pins.extend(p);

        if double_check || (check && shallow) {
            return KingPositionAnalysis {
                check: true,
                double_check: true,
                threat_source: threat_source,
                pins: pins,
            };
        }
    }

    let orthogonal_threats = (rook_bitboard | queen_bitboard) & opponent_occupancy;

    if orthogonal_threats.count_ones() > 0 {
        let (c, dc, ts, p) = in_sliding_check(
            king_pos,
            false,
            orthogonal_threats,
            occupancy,
            friendly_occupancy,
            check,
        );

        double_check |= dc;
        check |= c;
        if ts != None {
            threat_source = ts;
        }
        pins.extend(p);

        if double_check || (check && shallow) {
            return KingPositionAnalysis {
                check: true,
                double_check: true,
                threat_source,
                pins,
            };
        }
    }

    let pawn_threats: u64 = pawn_bitboard & opponent_occupancy;
    if pawn_threats.count_ones() > 0 {
        let (c, dc, ts) = is_pawn_check(king_pos, black_king, pawn_threats, check);

        double_check |= dc;
        check |= c;
        if ts != None {
            threat_source = ts;
        }

        if double_check || (check && shallow) {
            return KingPositionAnalysis {
                check: true,
                double_check: true,
                threat_source,
                pins,
            };
        }
    }

    KingPositionAnalysis {
        check,
        double_check,
        threat_source,
        pins,
    }
}

fn is_pawn_check(
    king_pos: u8,
    black_king: bool,
    opponent_pawns: u64,
    mut check: bool,
) -> (bool, bool, Option<ThreatSource>) {
    if (black_king && king_pos < 15) || (!black_king && king_pos > 48) {
        return (check, false, None);
    }

    let king_threat_sources = get_pawn_threat_source(king_pos, black_king);

    let actual_threat_positions = opponent_pawns & king_threat_sources;

    let threat_count = actual_threat_positions.count_ones();
    if threat_count == 0 {
        return (check, false, None);
    }
    if threat_count > 1 {
        return (true, true, None);
    }

    let double_check = check;
    check = true;
    let lsb = actual_threat_positions.trailing_zeros() as u8;
    let threat = ThreatSource {
        from: lsb,
        threat_type: ThreatType::Pawn,
        threat_ray_mask: 0,
    };

    (check, double_check, Some(threat))
}

pub(crate) fn get_pawn_threat_source(pos: u8, piece_is_black: bool) -> u64 {
    let rank = get_rank(pos);
    if (rank == 0 && piece_is_black) || (rank == 7 && !piece_is_black) {
        return 0
    }

    let king_file = get_file(pos);

    let mut r = 0;
    if king_file != 7 {
        r |= 1 << (pos as i32 + if piece_is_black { -9 } else { 7 });
    }

    if king_file != 0 {
        r |= 1 << (pos as i32 + if piece_is_black { -7 } else { 9 });
    }
    r
}

fn in_check_from_knights(
    king_pos: u8,
    mut knights: u64,
    mut check: bool,
) -> (bool, bool, Option<ThreatSource>) {
    let mut double_check = false;
    // We only need to keep track of up to one pin
    let mut threat = None;
    while knights != 0 && !double_check {
        let knight_pos = knights.trailing_zeros() as usize;
        let knight_threat_board = MOVE_DATA.knight_moves[knight_pos];
        if knight_threat_board.occupied(king_pos) {
            double_check = check;
            check = true;
            threat = Some(ThreatSource {
                from: knight_pos as u8,
                threat_type: ThreatType::Knight,
                threat_ray_mask: 0,
            })
        }
        knights ^= 1 << knight_pos;
    }

    (check, double_check, threat)
}

fn in_sliding_check(
    king_pos: u8,
    diagonal: bool,
    threats: u64,
    occupancy: u64,
    friendlies: u64,
    mut check: bool,
) -> (
    bool,
    bool,
    Option<ThreatSource>,
    Vec<ThreatRaycastCollision>,
) {
    let mut double_check = false;
    let mut threat = None;
    let mut pins = Vec::new();

    let deltas = if diagonal {
        if threats & MOVE_DATA.diagonal_threat_boards[king_pos as usize] == 0 {
            return (check, false, None, Vec::new());
        }
        DIAGONAL_DELTAS
    } else {
        if threats & MOVE_DATA.orthogonal_threat_board[king_pos as usize] == 0 {
            return (check, false, None, Vec::new());
        }
        ORTHOGONAL_DELTAS
    };

    for dir in deltas {
        let r = walk_slide(king_pos, dir.0, dir.1, threats, occupancy, friendlies);

        match r.0 {
            Some(found_threat) => {
                double_check = check;
                check = true;
                if double_check {
                    return (true, true, None, Vec::new());
                } else {
                    threat = Some(found_threat);
                }
            }
            None => {}
        }

        match r.1 {
            Some(found_pin) => pins.push(found_pin),
            None => {}
        }
    }

    (check, double_check, threat, pins)
}

fn walk_slide(
    pos: u8,
    file_delta: i8,
    rank_delta: i8,
    threats: u64,
    occupancy: u64,
    friendlies: u64,
) -> (Option<ThreatSource>, Option<ThreatRaycastCollision>) {
    let start_rank = get_rank(pos);
    let start_file = get_file(pos);
    let mut check_pos = pos as i8 + (8 * rank_delta) - file_delta;
    let mut potential_pin = u8::MAX;
    let threat_type = if file_delta == 0 || rank_delta == 0 {
        ThreatType::OrthogonalSlide
    } else {
        ThreatType::DiagonalSlide
    };
    let mut slide_path = 0;
    while valid_slide(check_pos, file_delta, rank_delta, start_file, start_rank) {
        let collision = occupancy.occupied(check_pos as u8);
        if collision {
            let threat_collision = threats.occupied(check_pos as u8);
            if threat_collision {
                if potential_pin == u8::MAX {
                    return (
                        Some(ThreatSource {
                            from: check_pos as u8,
                            threat_type,
                            threat_ray_mask: slide_path,
                        }),
                        None,
                    );
                } else {
                    let pin = ThreatRaycastCollision {
                        from: check_pos as u8,
                        at: potential_pin,
                        reveal_attack: !friendlies.occupied(potential_pin),
                        threat_type,
                        threat_ray_mask: slide_path,
                    };
                    return (None, Some(pin));
                }
            }
            if potential_pin != u8::MAX {
                return (None, None);
            }
            potential_pin = check_pos as u8;
        }

        slide_path |= 1 << check_pos;
        check_pos += (8 * rank_delta) - file_delta;
    }

    return (None, None);
}

fn valid_slide(pos: i8, file_delta: i8, rank_delta: i8, start_file: u8, start_rank: u8) -> bool {
    if pos < 0 || pos > 63 {
        return false;
    }
    let rank = get_rank(pos as u8);
    if rank_delta == 0 && rank != start_rank {
        return false;
    }
    if rank_delta == -1 && rank > start_rank {
        return false;
    } else if rank_delta == 1 && rank < start_rank {
        return false;
    }

    let file: u8 = get_file(pos as u8);
    if file_delta == 0 && file != start_file {
        return false;
    }
    if file_delta == -1 && file > start_file {
        return false;
    } else if file_delta == 1 && file < start_file {
        return false;
    }
    return true;
}

#[cfg(test)]
mod test {
    use crate::{board::board_rep::BoardRep, shared::board_utils::index_from_coords};

    use super::*;

    #[test]
    fn in_check_from_knights_one_knight() {
        let knights = (1 << 9) | (1 << 55);
        let king_pos = 3;
        let result = in_check_from_knights(king_pos, knights, false);
        assert_eq!(result.0, true, "Check should be true");
        assert_eq!(result.1, false, "Double check should be false");
        assert_eq!(
            result.2,
            Some(ThreatSource {
                from: 9,
                threat_type: ThreatType::Knight,
                threat_ray_mask: 0
            })
        );
    }

    #[test]
    fn in_check_from_knights_one_with_previous_check() {
        let knights = (1 << 9) | (1 << 55);
        let king_pos = 3;
        let result = in_check_from_knights(king_pos, knights, true);
        assert_eq!(result.0, true, "Check should be true");
        assert_eq!(result.1, true, "Double check should be true");
    }

    #[test]
    fn in_diagonal_check_one_source() {
        let diagonal_sliders = (1 << index_from_coords("f3")) | (1 << index_from_coords("d5"));
        let king_pos = index_from_coords("f7");
        let result = in_sliding_check(king_pos, true, diagonal_sliders, diagonal_sliders, 0, false);
        assert_eq!(result.0, true, "Check should be true");
        assert_eq!(result.1, false, "Double check should be false");
        let expected_mask = 1 << index_from_coords("e6");
        assert_eq!(
            result.2,
            Some(ThreatSource {
                from: index_from_coords("d5"),
                threat_type: ThreatType::DiagonalSlide,
                threat_ray_mask: expected_mask
            })
        );
    }

    #[test]
    fn in_diagonal_check_a4_to_e8() {
        let diagonal_sliders = 1 << index_from_coords("a4");
        let king_pos = index_from_coords("e8");
        let result = in_sliding_check(king_pos, true, 2147483684, diagonal_sliders, 0, false);
        assert_eq!(result.0, true, "Check should be true");
        assert_eq!(result.1, false, "Double check should be false");
        let expected_mask = (1 << index_from_coords("b5"))
            | (1 << index_from_coords("c6"))
            | (1 << index_from_coords("d7"));
        assert_eq!(
            result.2,
            Some(ThreatSource {
                from: index_from_coords("a4"),
                threat_type: ThreatType::DiagonalSlide,
                threat_ray_mask: expected_mask
            })
        );
    }

    #[test]
    fn in_diagonal_check_one_source_with_previous_check() {
        let diagonal_sliders = (1 << index_from_coords("f3")) | (1 << index_from_coords("d5"));
        let king_pos = index_from_coords("f7");
        let result = in_sliding_check(king_pos, true, diagonal_sliders, diagonal_sliders, 0, true);
        assert_eq!(result.0, true, "Check should be true");
        assert_eq!(result.1, true, "Double check should be true");
    }

    #[test]
    fn in_diagonal_check_many_sources_with_previous_check() {
        let diagonal_sliders = (1 << index_from_coords("f3"))
            | (1 << index_from_coords("d5"))
            | (1 << index_from_coords("h5"))
            | (1 << index_from_coords("g8"));
        let king_pos = index_from_coords("f7");
        let result = in_sliding_check(king_pos, true, diagonal_sliders, diagonal_sliders, 0, false);
        assert_eq!(result.0, true, "Check should be true");
        assert_eq!(result.1, true, "Double check should be true");
    }

    #[test]
    fn in_diagonal_check_multiple_pieces_pinned() {
        let king_pos = index_from_coords("b6");
        let diagonal_sliders = (1 << index_from_coords("d8")) | (1 << index_from_coords("g1"));
        let nonthreatening_opponents = 1 << index_from_coords("c7");
        let friendlies = 1 << index_from_coords("c5");

        let result = in_sliding_check(
            king_pos,
            true,
            diagonal_sliders,
            nonthreatening_opponents | diagonal_sliders | friendlies,
            friendlies,
            true,
        );
        assert_eq!(result.0, true, "Check should be true");
        assert_eq!(result.1, false, "Double check should be false");
        assert_eq!(result.2, None, "Threat should be none");
        assert_eq!(
            result.3.len(),
            2,
            "There should be 2 diagonally pinned pieces"
        );
        println!("{:?}", result.3);
        println!("c5: {}", index_from_coords("c5"));
        assert_eq!(result.3.len(), 2);
        assert!(result.3.contains(&ThreatRaycastCollision {
            from: index_from_coords("g1"),
            at: index_from_coords("c5"),
            reveal_attack: false,
            threat_type: ThreatType::DiagonalSlide,
            threat_ray_mask: (1 << index_from_coords("f2"))
                | (1 << index_from_coords("e3"))
                | (1 << index_from_coords("d4"))
                | (1 << index_from_coords("c5"))
        }));
        assert!(result.3.contains(&ThreatRaycastCollision {
            from: index_from_coords("d8"),
            at: index_from_coords("c7"),
            reveal_attack: true,
            threat_type: ThreatType::DiagonalSlide,
            threat_ray_mask: (1 << index_from_coords("c7"))
        }));
    }

    #[test]
    fn in_diagonal_check_one_threat_and_one_pin() {
        let king_pos = index_from_coords("e1");
        let diagonal_sliders = (1 << index_from_coords("h4"))
            | (1 << index_from_coords("a5"))
            | (1 << index_from_coords("a7"));
        let friendlies = (1 << index_from_coords("f2"));

        let result = in_sliding_check(
            king_pos,
            true,
            diagonal_sliders,
            diagonal_sliders | friendlies,
            friendlies,
            false,
        );
        assert_eq!(result.0, true, "Check should be true");
        assert_eq!(result.1, false, "Double check should be false");
        let expected_mask = (1 << index_from_coords("b4"))
            | (1 << index_from_coords("c3"))
            | (1 << index_from_coords("d2"));
        assert_eq!(
            result.2,
            Some(ThreatSource {
                from: index_from_coords("a5"),
                threat_type: ThreatType::DiagonalSlide,
                threat_ray_mask: expected_mask
            }),
            "Threat should a threat from a5"
        );
        assert_eq!(
            result.3.len(),
            1,
            "There should be 1 diagonally pinned pieces"
        );
        assert!(result.3.contains(&ThreatRaycastCollision {
            at: index_from_coords("f2"),
            from: index_from_coords("h4"),
            reveal_attack: false,
            threat_type: ThreatType::DiagonalSlide,
            threat_ray_mask: (1 << index_from_coords("g3")) | (1 << index_from_coords("f2"))
        }));
    }

    #[test]
    fn in_orthogonal_check_vertical_file() {
        let king_pos = index_from_coords("e8");
        let orthogonal_sliders = 1 << index_from_coords("e3");
        let friendlies = 0;

        let result = in_sliding_check(
            king_pos,
            false,
            orthogonal_sliders,
            orthogonal_sliders | friendlies,
            friendlies,
            false,
        );
        assert_eq!(result.0, true, "Check should be true");
        assert_eq!(result.1, false, "Double check should be false");
        let expected_mask = (1 << index_from_coords("e4"))
            | (1 << index_from_coords("e5"))
            | (1 << index_from_coords("e6"))
            | (1 << index_from_coords("e7"));
        assert_eq!(
            result.2,
            Some(ThreatSource {
                from: index_from_coords("e3"),
                threat_type: ThreatType::OrthogonalSlide,
                threat_ray_mask: expected_mask
            }),
            "Threat should a threat from e3"
        );
        assert_eq!(result.3.len(), 0, "There should be no pins");
    }

    #[test]
    fn in_orthogonal_check_vertical_file_two_blockers_no_pin() {
        let king_pos = index_from_coords("e8");
        let orthogonal_sliders = 1 << index_from_coords("e1");
        let friendlies = 1 << index_from_coords("e7");
        let occupancy = (1 << index_from_coords("e2")) | friendlies | orthogonal_sliders;

        let result = in_sliding_check(
            king_pos,
            false,
            orthogonal_sliders,
            occupancy,
            friendlies,
            false,
        );
        assert_eq!(result.0, false, "Check should be true");
        assert_eq!(result.1, false, "Double check should be false");
        assert_eq!(result.2, None, "There should no threats");
        assert_eq!(result.3.len(), 0, "There should be no pins");
    }

    #[test]
    fn in_orthogonal_check_scenario_0() {
        let king_pos = 24;
        let orthogonal_sliders = 34359738368;
        let friendlies = 0;
        let occupancy = 9025719170763264;

        let result = in_sliding_check(
            king_pos,
            false,
            orthogonal_sliders,
            occupancy,
            friendlies,
            true,
        );

        assert_eq!(result.1, false, "Double check should be false");
        assert_eq!(result.2, None, "There should no threats");
        assert_eq!(result.3.len(), 0, "There should be no pins");
    }

    #[test]
    fn in_diagonal_check_complex_board_0() {
        let king_pos = index_from_coords("f8");
        let occupancy = 8643851417268799366;
        let threats = 1103807643648;
        let friendlies = 8643850244179623936;

        let (check, double_check, threat_source, new_pins) =
            in_sliding_check(king_pos, true, threats, occupancy, friendlies, false);

        assert_eq!(check, true);
        assert_eq!(double_check, false);
        let expected_mask = 1 << index_from_coords("g7");
        assert_eq!(
            threat_source,
            Some(ThreatSource {
                from: index_from_coords("h6"),
                threat_type: ThreatType::DiagonalSlide,
                threat_ray_mask: expected_mask
            })
        );
        assert_eq!(new_pins.len(), 0);
    }

    #[test]
    fn walk_slide_reveal_attack() {
        let position = index_from_coords("c3");
        let threats = 1 << index_from_coords("f6");
        let friendlies = 0;
        let occupancy = (1 << index_from_coords("e5")) | threats | friendlies;

        let result = walk_slide(position, 1, 1, threats, occupancy, friendlies);

        assert_eq!(result.0, None, "There are no threats");
        assert_eq!(
            result.1,
            Some(ThreatRaycastCollision {
                from: index_from_coords("f6"),
                at: index_from_coords("e5"),
                reveal_attack: true,
                threat_type: ThreatType::DiagonalSlide,
                threat_ray_mask: (1 << index_from_coords("e5")) | (1 << index_from_coords("d4"))
            })
        )
    }

    #[test]
    fn walk_slide_horizontal_pin() {
        let position = index_from_coords("h8");
        let threats = 1 << index_from_coords("c8");
        let friendlies = 1 << index_from_coords("e8");
        let occupancy = threats | friendlies;

        let result = walk_slide(position, -1, 0, threats, occupancy, friendlies);

        assert_eq!(result.0, None, "There are no threats");
        assert_eq!(
            result.1,
            Some(ThreatRaycastCollision {
                from: index_from_coords("c8"),
                at: index_from_coords("e8"),
                reveal_attack: false,
                threat_type: ThreatType::OrthogonalSlide,
                threat_ray_mask: (1 << index_from_coords("d8"))
                    | (1 << index_from_coords("e8"))
                    | (1 << index_from_coords("f8"))
                    | (1 << index_from_coords("g8"))
            })
        )
    }

    #[test]
    fn walk_slide_horizontal_threat() {
        let position = index_from_coords("e5");
        let threats = 1 << index_from_coords("e3");
        let friendlies = (1 << index_from_coords("e2")) | 1 << index_from_coords("e6");
        let occupancy = threats | friendlies;

        let result = walk_slide(position, 0, -1, threats, occupancy, friendlies);

        let expected_mask = 1 << index_from_coords("e4");
        assert_eq!(
            result.0,
            Some(ThreatSource {
                from: index_from_coords("e3"),
                threat_type: ThreatType::OrthogonalSlide,
                threat_ray_mask: expected_mask
            })
        );
        assert_eq!(result.1, None, "There are no pins")
    }

    #[test]
    fn is_pawn_check_single_threat() {
        let king_pos = index_from_coords("e3");
        let threats = 1 << index_from_coords("d4");
        let result = is_pawn_check(king_pos, false, threats, false);

        assert!(result.0, "Pawn check should be true");
        assert!(!result.1, "Should not be double check");
        assert_eq!(
            result.2,
            Some(ThreatSource {
                from: index_from_coords("d4"),
                threat_type: ThreatType::Pawn,
                threat_ray_mask: 0
            })
        );
    }

    #[test]
    fn analyze_active_king_position_scenario_0() {
        // Double knight attack
        let board = BoardRep::from_fen("6k1/8/3n4/8/2K5/4n3/8/8 w - - 0 1".into());

        let result = analyze_king_position(
            board.white_king_position,
            board.black_turn,
            board.occupancy,
            board.white_occupancy,
            board.black_occupancy,
            board.pawn_bitboard,
            board.knight_bitboard,
            board.bishop_bitboard,
            board.rook_bitboard,
            board.queen_bitboard,
            false,
        );

        assert_eq!(result.check, true);
        assert_eq!(result.double_check, true);
    }

    #[test]
    fn analyze_active_king_position_scenario_1() {
        // Single knight attack, includes threat
        let board = BoardRep::from_fen("6k1/8/8/8/8/8/6K1/4n3 w - - 0 1".into());

        let result = analyze_king_position(
            board.white_king_position,
            board.black_turn,
            board.occupancy,
            board.white_occupancy,
            board.black_occupancy,
            board.pawn_bitboard,
            board.knight_bitboard,
            board.bishop_bitboard,
            board.rook_bitboard,
            board.queen_bitboard,
            false,
        );

        assert_eq!(result.check, true);
        assert_eq!(result.double_check, false);
        assert_eq!(
            result.threat_source,
            Some(ThreatSource {
                from: 3,
                threat_type: ThreatType::Knight,
                threat_ray_mask: 0
            })
        );
    }

    #[test]
    fn analyze_active_king_position_scenario_2() {
        // Single check from h3 bishop
        let board = BoardRep::from_fen(
            "1rbq1knr/pppp1p1p/2n4B/3N3Q/2P4N/P2B4/1P3PPP/R4RK1 b - - 6 16".into(),
        );

        let result = analyze_king_position(
            board.black_king_position,
            board.black_turn,
            board.occupancy,
            board.black_occupancy,
            board.white_occupancy,
            board.pawn_bitboard,
            board.knight_bitboard,
            board.bishop_bitboard,
            board.rook_bitboard,
            board.queen_bitboard,
            false,
        );

        assert_eq!(result.check, true);
        assert_eq!(result.double_check, false);
        assert_eq!(result.pins.len(), 0);
        let expected_mask = 1 << index_from_coords("g7");
        assert_eq!(
            result.threat_source,
            Some(ThreatSource {
                from: index_from_coords("h6"),
                threat_type: ThreatType::DiagonalSlide,
                threat_ray_mask: expected_mask
            })
        );
    }

    #[test]
    fn analyze_active_king_position_scenario_3() {
        // Single check from e1 & pins on f3 & h2
        let board = BoardRep::from_fen(
            "r3kb2/pp3ppp/2n2n1r/1Bpp4/4b3/2N1PP2/PPPP3P/R1BQq2K w q - 0 11".into(),
        );
        let result = analyze_king_position(
            board.white_king_position,
            board.black_turn,
            board.occupancy,
            board.white_occupancy,
            board.black_occupancy,
            board.pawn_bitboard,
            board.knight_bitboard,
            board.bishop_bitboard,
            board.rook_bitboard,
            board.queen_bitboard,
            false,
        );

        assert_eq!(result.check, true);
        assert_eq!(result.double_check, false);
        let expected_mask = (1 << index_from_coords("f1")) | (1 << index_from_coords("g1"));
        assert_eq!(
            result.threat_source,
            Some(ThreatSource {
                from: index_from_coords("e1"),
                threat_type: ThreatType::OrthogonalSlide,
                threat_ray_mask: expected_mask
            })
        );
        assert_eq!(result.pins.len(), 2);
        assert!(result.pins.contains(&ThreatRaycastCollision {
            from: index_from_coords("h6"),
            at: index_from_coords("h2"),
            reveal_attack: false,
            threat_type: ThreatType::OrthogonalSlide,
            threat_ray_mask: (1 << index_from_coords("h2"))
                | (1 << index_from_coords("h3"))
                | (1 << index_from_coords("h4"))
                | (1 << index_from_coords("h5"))
        }));
        assert!(result.pins.contains(&ThreatRaycastCollision {
            from: index_from_coords("e4"),
            at: index_from_coords("f3"),
            reveal_attack: false,
            threat_type: ThreatType::DiagonalSlide,
            threat_ray_mask: (1 << index_from_coords("f3")) | (1 << index_from_coords("g2"))
        }));
    }

    #[test]
    fn analyze_active_king_position_scenario_4() {
        // Double check queen and knight
        let board = BoardRep::from_fen(
            "rnbqk1nr/pppp1pNp/2Pb4/8/1B6/4Q3/PP1PPPPP/RN2KB1R b KQkq - 0 1".into(),
        );
        let result = analyze_king_position(
            board.black_king_position,
            board.black_turn,
            board.occupancy,
            board.black_occupancy,
            board.white_occupancy,
            board.pawn_bitboard,
            board.knight_bitboard,
            board.bishop_bitboard,
            board.rook_bitboard,
            board.queen_bitboard,
            false,
        );

        assert_eq!(result.check, true);
        assert_eq!(result.double_check, true);
        assert_eq!(result.pins.len(), 0);
    }

    #[test]
    fn analyze_active_king_position_scenario_5() {
        // Pawn threat
        let board = BoardRep::from_fen("k7/8/8/8/8/3p4/2K5/8 w - - 0 1".into());
        let result = analyze_king_position(
            board.white_king_position,
            board.black_turn,
            board.occupancy,
            board.white_occupancy,
            board.black_occupancy,
            board.pawn_bitboard,
            board.knight_bitboard,
            board.bishop_bitboard,
            board.rook_bitboard,
            board.queen_bitboard,
            false,
        );

        assert_eq!(result.check, true);
        assert_eq!(result.double_check, false);
        assert_eq!(result.pins.len(), 0);
        assert_eq!(
            result.threat_source,
            Some(ThreatSource {
                from: index_from_coords("d3"),
                threat_type: ThreatType::Pawn,
                threat_ray_mask: 0
            })
        )
    }

    #[test]
    fn analyze_active_king_position_scenario_6() {
        // Queen threat on d4
        let board = BoardRep::from_fen(
            "r3kb2/pp3ppp/2n2n1r/1Bpp4/3qb3/2N2P2/PPPPP1PP/R1B3K1 w q - 0 11".into(),
        );
        let result = analyze_king_position(
            board.white_king_position,
            false,
            board.occupancy,
            board.white_occupancy,
            board.black_occupancy,
            board.pawn_bitboard,
            board.knight_bitboard,
            board.bishop_bitboard,
            board.rook_bitboard,
            board.queen_bitboard,
            false,
        );

        assert_eq!(result.check, true);
        assert_eq!(result.double_check, false);
        assert_eq!(result.pins.len(), 0);
        let expected_mask: u64 = (1 << index_from_coords("e3")) | (1 << index_from_coords("f2"));
        assert_eq!(
            result.threat_source,
            Some(ThreatSource {
                from: index_from_coords("d4"),
                threat_type: ThreatType::DiagonalSlide,
                threat_ray_mask: expected_mask
            })
        )
    }

    #[test]
    fn analyze_king_position_shallow_scenario_1() {
        let board = BoardRep::from_fen(
            "rnbqkb1r/ppp1pppp/3p3n/8/Q7/2P5/PP1PPPPP/RNB1KBNR w KQkq - 0 1".into(),
        );

        let result = analyze_king_position(
            board.black_king_position,
            true,
            board.occupancy,
            board.white_occupancy,
            board.black_occupancy,
            board.pawn_bitboard,
            board.knight_bitboard,
            board.bishop_bitboard,
            board.rook_bitboard,
            board.queen_bitboard,
            true,
        );

        assert!(result.check);
    }

    #[test]
    fn analyze_king_position_shallow_scenario_2() {
        let board = BoardRep::from_fen("8/2p5/1K1p4/1P5r/1R3p1k/8/4P1P1/8 w - - 0 1".into());

        let result = analyze_king_position(
            board.black_king_position,
            true,
            board.occupancy,
            board.white_occupancy,
            board.black_occupancy,
            board.pawn_bitboard,
            board.knight_bitboard,
            board.bishop_bitboard,
            board.rook_bitboard,
            board.queen_bitboard,
            true,
        );

        assert!(result.check);
    }
}
