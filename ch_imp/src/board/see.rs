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

        if (attacking_piece_value > defending_piece_value) {
            println!("{attacking_piece:?} taking {attacked_piece:?} not performed as becomes en prise by {lvd:?}");
            return None;
        }

        let continued_exchange_see = see(lvd, defended_by, attacking_piece, attacked_by);

        match continued_exchange_see {
            Some(v) => capture_value = capture_value - v,
            None => {}
        }
    }

    if capture_value < 0 {
        println!(
            "{attacking_piece:?} taking {attacked_piece:?} resulting in see of {capture_value} is a bad exchange so not performing"
        );
        return None;
    }

    println!("{attacking_piece:?} taking {attacked_piece:?} resulting in see of {capture_value}");
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
        };

        let opponent = AttackedBy {
            pawns: 0,
            knights: 0,
            rooks: 0,
            bishop: false,
            queen: false,
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
        };

        let opponent = AttackedBy {
            pawns: 1,
            knights: 0,
            rooks: 0,
            bishop: false,
            queen: false,
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
        };

        let opponent = AttackedBy {
            pawns: 1,
            knights: 0,
            rooks: 0,
            bishop: false,
            queen: false,
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
        };

        let opponent = AttackedBy {
            pawns: 1,
            knights: 0,
            rooks: 0,
            bishop: false,
            queen: false,
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
        };

        let opponent = AttackedBy {
            pawns: 1,
            knights: 0,
            rooks: 1,
            bishop: true,
            queen: false,
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
        };

        let opponent = AttackedBy {
            pawns: 0,
            knights: 0,
            rooks: 1,
            bishop: true,
            queen: false,
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
        };

        let opponent = AttackedBy {
            pawns: 0,
            knights: 0,
            rooks: 1,
            bishop: false,
            queen: false,
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
        };

        let opponent = AttackedBy {
            pawns: 0,
            knights: 0,
            rooks: 2,
            bishop: false,
            queen: false,
        };

        assert_eq!(
            see_from_capture(PieceType::Knight, friendly, PieceType::Pawn, opponent),
            -2
        )
    }
}