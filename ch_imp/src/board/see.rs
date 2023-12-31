use crate::shared::piece_type::PIECE_TYPE_EXCHANGE_VALUE;
use crate::{shared::piece_type::PieceType, MOVE_DATA};

use super::bitboard::Bitboard;
use super::{
    attack_and_defend_lookups::AttackedBy, board_rep::BoardRep,
    king_position_analysis::get_pawn_threat_source,
};

pub fn see_from_capture(
    attacking_piece: PieceType,
    mut attacked_by: AttackedBy,
    attacked_piece: PieceType,
    defended_by: AttackedBy,
) -> i8 {
    attacked_by.remove(attacking_piece);
    let exchange = see(attacking_piece, attacked_by, attacked_piece, defended_by);
    match exchange {
        Some(v) => v,
        None => {
            PIECE_TYPE_EXCHANGE_VALUE[attacked_piece as usize]
                - PIECE_TYPE_EXCHANGE_VALUE[attacking_piece as usize]
        }
    }
}

// Calculates if a piece is safe from an opponent winning capture - should not be used to evaluate square control
pub fn piece_safety(piece_type: PieceType, is_move: bool, mut attacked_by: AttackedBy, mut defended_by: AttackedBy) -> i8 {
    // If this is a move we need to remove the moving piece from the defenders list if its not a pawn - as it cannot defend itself
    if is_move && piece_type != PieceType::Pawn {
        defended_by.remove(piece_type);
    }
    let attacking_piece = attacked_by.pop_least_valuable();
    match attacking_piece {
        PieceType::None | PieceType::King => 0,
        _ => match see(attacking_piece, attacked_by, piece_type, defended_by) {
            Some(i) => -i,
            None => 0
        }
    }
}

// See who controls an unoccupied square with the lowest value piece. Returns 1 if friendly controls, 0 if equal or no control, -1 if opponent controls
pub fn square_control(friendly: AttackedBy, opponent: AttackedBy) -> i8 {
    if !friendly.any() && !opponent.any() {
        return 0
    }
    if friendly.any() && !opponent.any() {
        return 1
    }
    if !friendly.any() && opponent.any() {
        return -1
    }

    let pawn_diff = friendly.pawns as i8 - opponent.pawns as i8;
    match pawn_diff {
        -2 | -1 => return -1,
        1 | 2 => return 1,
        _ => {}
    }
    let knight_and_bishop_dif = friendly.knights as i8 - opponent.knights as i8 + if friendly.bishop { 1} else { 0} - if opponent.bishop { 1} else { 0 };
    match knight_and_bishop_dif {
        -3| -2 | -1 => return -1,
        1 | 2 | 3 => return 1,
        _ => {}
    }

    let rook_dif = friendly.rooks as i8 - opponent.rooks as i8;
    match rook_dif {
        -2 | -1 => return -1,
        1 | 2  => return 1,
        _ => {}
    }

    match (friendly.queen, opponent.queen) {
        (true, false) => return 1,
        (false, true)  => return -1,
        _ => {}
    }

    return 0
}

pub fn see(
    attacking_piece: PieceType,
    attacked_by: AttackedBy,
    attacked_piece: PieceType,
    mut defended_by: AttackedBy,
) -> Option<i8> {
    let mut capture_value = PIECE_TYPE_EXCHANGE_VALUE[attacked_piece as usize];

    let lvd = defended_by.pop_least_valuable();
    if lvd != PieceType::None {
        let attacking_piece_value = PIECE_TYPE_EXCHANGE_VALUE[attacking_piece as usize];
        let defending_piece_value = PIECE_TYPE_EXCHANGE_VALUE[lvd as usize];

        if attacking_piece_value > defending_piece_value {
            return None;
        }

        let continued_exchange_see = see(lvd, defended_by, attacking_piece, attacked_by);

        match continued_exchange_see {
            Some(v) => capture_value = capture_value - v,
            None => {}
        }
    }

    if capture_value < 0 {
        return None;
    }

    Some(capture_value)
}

#[cfg(test)]
mod test {
    use crate::shared::board_utils::index_from_coords;

    use super::*;

    // #[test]
    // fn see_no_captures() {
    //     let board = BoardRep::from_fen("rn4kr/1p4p1/1p6/4p2p/7K/6p1/5q2/8 w - - 0 40".into());

    //     assert_eq!(board.see(index_from_coords("b3")), 0)
    // }

    #[test]
    fn see_from_capture_pawn_capture_with_no_en_prise() {
        // k7/8/8/4p3/3P4/8/8/K7 w - - 0 1
        // Attacking e5
        let friendly = AttackedBy {
            pawns: 1,
            knights: 0,
            rooks: 0,
            bishop: false,
            queen: false,
            king: false,
        };

        let opponent = AttackedBy {
            pawns: 0,
            knights: 0,
            rooks: 0,
            bishop: false,
            queen: false,
            king: false,
        };

        assert_eq!(
            see_from_capture(PieceType::Pawn, friendly, PieceType::Pawn, opponent),
            1
        )
    }

    #[test]
    fn see_from_capture_pawn_equal_exchange() {
        // k7/8/3p4/4p3/3P4/8/8/K7 w - - 0 1
        // Attacking e5
        let friendly = AttackedBy {
            pawns: 1,
            knights: 0,
            rooks: 0,
            bishop: false,
            queen: false,
            king: false,
        };

        let opponent = AttackedBy {
            pawns: 1,
            knights: 0,
            rooks: 0,
            bishop: false,
            queen: false,
            king: false,
        };

        assert_eq!(
            see_from_capture(PieceType::Pawn, friendly, PieceType::Pawn, opponent),
            0
        )
    }

    #[test]
    fn see_from_capture_pawn_winning_exchange() {
        // k7/8/3p4/4p3/3P4/8/4Q3/K7 w - - 0 1
        // Attacking e5
        let friendly = AttackedBy {
            pawns: 1,
            knights: 0,
            rooks: 0,
            bishop: false,
            queen: true,
            king: false,
        };

        let opponent = AttackedBy {
            pawns: 1,
            knights: 0,
            rooks: 0,
            bishop: false,
            queen: false,
            king: false,
        };

        assert_eq!(
            see_from_capture(PieceType::Pawn, friendly, PieceType::Pawn, opponent),
            1
        )
    }

    #[test]
    fn see_from_capture_knight_for_pawn_exchange() {
        // k7/8/3p4/4p3/8/3N4/8/K7 w - - 0 1
        // Attacking e5
        let friendly = AttackedBy {
            pawns: 0,
            knights: 1,
            rooks: 0,
            bishop: false,
            queen: false,
            king: false,
        };

        let opponent = AttackedBy {
            pawns: 1,
            knights: 0,
            rooks: 0,
            bishop: false,
            queen: false,
            king: false,
        };

        assert_eq!(
            see_from_capture(PieceType::Knight, friendly, PieceType::Pawn, opponent),
            -2
        )
    }

    #[test]
    fn see_from_capture_scenario_0() {
        // k3r3/6b1/3p4/4p3/3P2N1/8/4Q3/K7 w - - 0 1
        // attacking e5
        // after pawn exchange should stop as full exchange is losing
        let friendly = AttackedBy {
            pawns: 1,
            knights: 1,
            rooks: 0,
            bishop: false,
            queen: true,
            king: false,
        };

        let opponent = AttackedBy {
            pawns: 1,
            knights: 0,
            rooks: 1,
            bishop: true,
            queen: false,
            king: false,
        };

        assert_eq!(
            see_from_capture(PieceType::Pawn, friendly, PieceType::Pawn, opponent),
            0
        )
    }

    #[test]
    fn see_from_capture_scenario_1() {
        // k3r3/6b1/8/4p3/6N1/8/4Q3/K7 w - - 0 1
        // attacking e5
        // knight capturing pawn but becoming en prise is a losing exchange
        let friendly = AttackedBy {
            pawns: 0,
            knights: 1,
            rooks: 0,
            bishop: false,
            queen: true,
            king: false,
        };

        let opponent = AttackedBy {
            pawns: 0,
            knights: 0,
            rooks: 1,
            bishop: true,
            queen: false,
            king: false,
        };

        assert_eq!(
            see_from_capture(PieceType::Knight, friendly, PieceType::Pawn, opponent),
            -2
        )
    }

    #[test]
    fn see_from_capture_scenario_2() {
        // k3r3/8/8/4p3/6N1/8/4Q3/K7 w - - 0 1
        // attacking e5
        // knight capturing pawn is defended by queen and thus is winning
        let friendly = AttackedBy {
            pawns: 0,
            knights: 1,
            rooks: 0,
            bishop: false,
            queen: true,
            king: false,
        };

        let opponent = AttackedBy {
            pawns: 0,
            knights: 0,
            rooks: 1,
            bishop: false,
            queen: false,
            king: false,
        };

        assert_eq!(
            see_from_capture(PieceType::Knight, friendly, PieceType::Pawn, opponent),
            1
        )
    }

    #[test]
    fn see_from_capture_scenario_3() {
        // k3r3/4r3/8/4p3/8/5N2/4Q3/K7 w - - 0 1
        // attacking e5
        // knight capturing pawn is defended by queen but queen taking leaves it en prise
        let friendly = AttackedBy {
            pawns: 0,
            knights: 1,
            rooks: 0,
            bishop: false,
            queen: true,
            king: false,
        };

        let opponent = AttackedBy {
            pawns: 0,
            knights: 0,
            rooks: 2,
            bishop: false,
            queen: false,
            king: false,
        };

        assert_eq!(
            see_from_capture(PieceType::Knight, friendly, PieceType::Pawn, opponent),
            -2
        )
    }

    #[test]
    fn see_from_capture_scenario_4() {
        // 1r1q1rk1/1b1n1p1p/p2b1np1/3pN3/3P1P2/P1N5/3BB1PP/1R1Q1RK1 b - - 0 1
        // attacking e5
        // pawns defending knight attacked by knight and bishop should be equal terminating after pawn count take
        let friendly = AttackedBy {
            pawns: 0,
            knights: 1,
            rooks: 0,
            bishop: true,
            queen: true,
            king: false,
        };

        let opponent = AttackedBy {
            pawns: 2,
            knights: 0,
            rooks: 0,
            bishop: false,
            queen: false,
            king: false,
        };

        assert_eq!(
            see_from_capture(PieceType::Knight, friendly, PieceType::Bishop, opponent),
            0
        )
    }


    #[test]
    fn piece_safety_bishop_safe_move() {
        // 1r1n1rk1/3qp2p/P2p2p1/1p6/5pP1/1p3P1P/5PB1/R1QR2K1 w - - 0 1
        // retreating to h1
        let board = BoardRep::from_fen("1r1n1rk1/3qp2p/P2p2p1/1p6/5pP1/1p3P1P/5PB1/R1QR2K1 w - - 0 1".into());

        let friendly = board.get_attacked_by(0, false);
        let opponent = board.get_attacked_by(0, true);

        assert_eq!(
            piece_safety(PieceType::Bishop, true, opponent, friendly),
            0
        )
    }

    #[test]
    fn piece_safety_queen_unsafe_move() {
        // 1r1n1rk1/3qp2p/P2p2p1/1p6/5pP1/1p3P1P/5PB1/R1QR2K1 w - - 0 1
        // queen moving to unsafe c2 where a pawn can capture
        let board = BoardRep::from_fen("1r1n1rk1/3qp2p/P2p2p1/1p6/5pP1/1p3P1P/5PB1/R1QR2K1 w - - 0 1".into());

        let friendly = board.get_attacked_by(index_from_coords("c2"), false);
        let opponent = board.get_attacked_by(index_from_coords("c2"), true);

        assert_eq!(
            piece_safety(PieceType::Queen, true, opponent, friendly),
            -PIECE_TYPE_EXCHANGE_VALUE[5]
        )
    }

    #[test]

    fn piece_safety_scenario_0() {
        // 1r1q1rk1/1b1n1p1p/p4np1/3pN3/Q2P1P2/b1N5/3BB1PP/1R3RK1 b - - 1
        // pawn threatened by queen and bishop but defended by bishop would be a negative exchange of a single pawn
        let board = BoardRep::from_fen("1r1q1rk1/1b1n1p1p/p4np1/3pN3/Q2P1P2/b1N5/3BB1PP/1R3RK1 b - - 1 2 ".into());

        let friendly: AttackedBy = board.get_attacked_by(index_from_coords("a6"), true);
        let opponent = board.get_attacked_by(index_from_coords("a6"), false);

        assert_eq!(
            piece_safety(PieceType::Pawn, false, opponent, friendly),
            -PIECE_TYPE_EXCHANGE_VALUE[1]
        )
    }
}
