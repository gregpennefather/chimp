use crate::{
    board::{
        bitboard::Bitboard,
        board_rep::BoardRep,
        king_position_analysis::{KingPositionAnalysis, ThreatRaycastCollision},
        position::Position,
    },
    r#move::Move,
    shared::{
        board_utils::get_file,
        constants::{MF_CAPTURE, MF_EP_CAPTURE},
        piece_type::PieceType,
    },
};

use super::{
    king::is_legal_king_move,
    knight::is_legal_knight_move,
    pawn::{ep_leads_to_orthogonal_check, legal_move::is_legal_pawn_move},
    sliding::{bishop::is_legal_bishop_move, queen::is_legal_queen_move, rook::is_legal_rook_move},
};

impl Position {
    pub fn is_legal_move(&self, m: Move) -> bool {
        // Move for correct colour
        if self.board.black_turn != m.is_black() {
            return false;
        }

        let friendly_occupancy = if self.board.black_turn {
            self.board.black_occupancy
        } else {
            self.board.white_occupancy
        };

        // If we dont have a piece of the correct type on the from square for the correct colour this cant be a legal move
        if !friendly_occupancy.occupied(m.from())
            || self.board.get_piece_type_at_index(m.from()) != m.piece_type()
        {
            return false;
        }

        // Not a capture but square occupied
        if !m.is_capture() && self.board.occupancy.occupied(m.to()) {
            return false
        }

        let opponent_occupancy = self.board.get_opponent_occupancy();
        // If its a capture (not EP capture) but theres no opponent in the to square
        if m.flags() == MF_CAPTURE && !opponent_occupancy.occupied(m.to()) {
            return false;
        }

        // If EP Capture and not a legal ep move
        if m.flags() == MF_EP_CAPTURE && m.to() != self.board.ep_index {
            return false;
        }

        // Is double check only king moves are legal
        if self.double_check && m.piece_type() != PieceType::King {
            return false;
        }

        let king_analysis = if self.board.black_turn {
            self.board.get_black_king_analysis()
        } else {
            self.board.get_white_king_analysis()
        };

        // If in check see if this move removes check
        if self.current_in_check() && !move_removes_check(m, &king_analysis) {
            return false;
        }

        // If not in check, ensure this move doesn't result in check by moving a pinned piece
        if king_analysis.pins.len() > 0 && !is_legal_pinned_piece_move(m, &king_analysis) {
            return false;
        }

        // Move is not EP Capture resulting in check
        if m.flags() == MF_EP_CAPTURE {
            let captured_pawn_position = (m.from() as i8
                + if get_file(m.to()) > get_file(m.from()) {
                    -1
                } else {
                    1
                }) as u8;
            if ep_leads_to_orthogonal_check(
                self.board,
                m.from(),
                captured_pawn_position,
                opponent_occupancy,
            ) {
                return false;
            }
        }

        // Move is not king move into check

        return self.is_legal_piece_move(m, &king_analysis);
    }

    fn is_legal_piece_move(&self, m: Move, king_analysis: &KingPositionAnalysis) -> bool {
        match m.piece_type() {
            PieceType::Pawn => is_legal_pawn_move(m, self.board),
            PieceType::Knight => is_legal_knight_move(m, self.board),
            PieceType::Bishop => is_legal_bishop_move(m, self.board),
            PieceType::Rook => is_legal_rook_move(m, self.board),
            PieceType::Queen => is_legal_queen_move(m, self.board),
            PieceType::King => is_legal_king_move(m, self.board, king_analysis),
            _ => panic!("Move piece type unknown! {m:?}"),
        }
    }
}

fn move_removes_check(m: Move, king_analysis: &KingPositionAnalysis) -> bool {
    let pin = Option::<&ThreatRaycastCollision>::copied(
        king_analysis.pins.iter().find(|p| p.at == m.from()),
    );
    match pin {
        // Can't capture threat if pinned
        Some(_) => false,
        // Must capture threat or move into threat ray to remove check
        None => match king_analysis.threat_source {
            Some(threat) => {
                if m.to() == threat.from {
                    return true;
                }
                if m.piece_type() != PieceType::King && (1 << m.to()) & threat.threat_ray_mask != 0 {
                    return true;
                }
                false
            }
            None => panic!("Unexpected lack on threat {king_analysis:?}"),
        }
    }
}

fn is_legal_pinned_piece_move(m: Move, king_analysis: &KingPositionAnalysis) -> bool {
    let pin = Option::<&ThreatRaycastCollision>::copied(
        king_analysis.pins.iter().find(|p| p.at == m.from()),
    );
    match pin {
        None => true,
        Some(p) => {
            (m.is_capture() && m.to() == p.from) || // Is a capture of pin source
                p.threat_ray_mask & (1 << m.to()) != 0 // Is a move inside of the pin ray
        }
    }
}

#[cfg(test)]
mod test {
    use crate::{
        board::position::Position,
        move_generation::legal_move::{is_legal_pinned_piece_move, move_removes_check},
        r#move::Move,
        shared::{
            board_utils::index_from_coords,
            constants::{MF_CAPTURE, MF_EP_CAPTURE},
            piece_type::PieceType,
        },
    };

    #[test]
    fn is_not_king_move_in_double_check() {
        let position = Position::from_fen("k7/8/8/8/8/1b2n3/3P4/3K4 w - - 0 1".into());
        let m = Move::new(
            index_from_coords("d2"),
            index_from_coords("e3"),
            MF_CAPTURE,
            PieceType::Pawn,
            false,
            2,
        );
        assert!(!position.is_legal_move(m));
    }

    #[test]
    fn is_not_white_move_during_white_turn() {
        let position = Position::from_fen("k7/8/8/8/b7/5n2/2PP4/3K4 w - - 0 1".into());
        let m = Move::new(
            index_from_coords("a8"),
            index_from_coords("a7"),
            0b0,
            PieceType::King,
            true,
            0,
        );
        assert!(!position.is_legal_move(m));
    }

    #[test]
    fn in_check_moving_piece_but_does_not_block_threat() {
        let position = Position::from_fen("k7/8/8/8/b7/5n2/1N1P4/3K4 w - - 0 1".into());
        let m = Move::new(
            index_from_coords("b2"),
            index_from_coords("d3"),
            0b0,
            PieceType::Knight,
            false,
            0,
        );
        assert!(!move_removes_check(
            m,
            &position.board.get_white_king_analysis()
        ));
    }

    #[test]
    fn in_check_moving_piece_captures_threat() {
        let position = Position::from_fen("k7/8/8/8/3r4/8/2N5/3K4 w - - 0 1".into());
        let m = Move::new(
            index_from_coords("c2"),
            index_from_coords("d4"),
            MF_CAPTURE,
            PieceType::Knight,
            false,
            0,
        );
        assert!(move_removes_check(
            m,
            &position.board.get_white_king_analysis()
        ));
    }

    #[test]
    fn in_check_piece_moving_to_block_threat_is_pinned() {
        let position = Position::from_fen("k2r4/8/8/8/b7/8/2N5/3K4 w - - 0 1".into());
        let m = Move::new(
            index_from_coords("c2"),
            index_from_coords("d4"),
            0b0,
            PieceType::Knight,
            false,
            0,
        );
        assert!(!move_removes_check(
            m,
            &position.board.get_white_king_analysis()
        ));
    }

    #[test]
    fn in_check_piece_capturing_threat_is_pinned() {
        let position = Position::from_fen("k7/8/8/8/b2r4/8/2N5/3K4 w - - 0 1".into());
        let m = Move::new(
            index_from_coords("c2"),
            index_from_coords("d4"),
            MF_CAPTURE,
            PieceType::Knight,
            false,
            0,
        );
        assert!(!move_removes_check(
            m,
            &position.board.get_white_king_analysis()
        ));
    }

    #[test]
    fn not_in_check_but_piece_is_pinned_and_moving_off_threat_ray() {
        let position = Position::from_fen("k7/8/2b5/8/8/8/6B1/5K2 b - - 0 1".into());
        let m = Move::new(
            index_from_coords("c6"),
            index_from_coords("b5"),
            0b0,
            PieceType::Bishop,
            true,
            0,
        );
        assert!(!position.is_legal_move(m));
    }

    #[test]
    fn not_in_check_but_ep_capture_would_result_in_check() {
        let position = Position::from_fen("8/8/8/8/R4pPk/8/8/K7 b - g3 0 1".into());
        let m = Move::new(
            index_from_coords("f4"),
            index_from_coords("g3"),
            MF_EP_CAPTURE,
            PieceType::Pawn,
            true,
            0,
        );
        assert!(!position.is_legal_move(m));
    }

    #[test]
    fn is_legal_pinned_piece_move_stays_inside_threat_ray() {
        let position = Position::from_fen("k7/8/2b5/8/8/8/6B1/5K2 b - - 0 1".into());
        let m = Move::new(
            index_from_coords("c6"),
            index_from_coords("d5"),
            0b0,
            PieceType::Bishop,
            true,
            0,
        );
        assert!(is_legal_pinned_piece_move(
            m,
            &position.board.get_black_king_analysis()
        ));
    }

    #[test]
    fn is_legal_pinned_piece_move_captures_pinning_piece() {
        let position = Position::from_fen("k7/8/2b5/8/8/8/6B1/5K2 b - - 0 1".into());
        let m = Move::new(
            index_from_coords("c6"),
            index_from_coords("g2"),
            MF_CAPTURE,
            PieceType::Bishop,
            true,
            0,
        );
        assert!(is_legal_pinned_piece_move(
            m,
            &position.board.get_black_king_analysis()
        ));
    }

    #[test]
    fn is_legal_scenario_0() {
        let position = Position::from_fen(
            "rnbqkbnr/ppp1p1p1/8/3p1p1p/2B1P2P/8/PPPP1PP1/RNBQK1NR w KQkq d6 0 4".into(),
        );
        let m = Move::new(
            index_from_coords("c4"),
            index_from_coords("d5"),
            MF_CAPTURE,
            PieceType::Pawn,
            false,
            0,
        );
        assert!(!position.is_legal_move(m));
    }

    #[test]
    fn is_legal_scenario_1() {
        let position = Position::from_fen(
            "rnbqkbnr/ppppppp1/8/7p/7P/8/PPPPPPP1/RNBQKBNR w KQkq h6 0 ".into(),
        );
        let m = Move::new(index_from_coords("f1"), index_from_coords("e2"), 0b0, PieceType::Bishop, false, 0);
        assert!(!position.is_legal_move(m));
    }

    #[test]
    fn is_legal_scenario_2() {
        let position = Position::from_fen(
            "r1bq1bnr/ppp1kppp/2np4/1B2Q3/4PP2/8/PPPP2PP/RNB1K1NR b KQ - 0 5".into(),
        );
        let m = Move::new(index_from_coords("e7"), index_from_coords("e6"), 0b0, PieceType::King, true, 0);
        assert!(!position.is_legal_move(m));
    }

    #[test]
    fn is_legal_scenario_3() {
        let position = Position::from_fen(
            "rn3b1r/ppp2kpp/3p4/8/P3P1nP/4q3/4K3/4R1NR w - - 6 21".into(),
        );
        let m = Move::new(index_from_coords("e1"), index_from_coords("e3"), MF_CAPTURE, PieceType::Rook, false, 0);
        assert!(!position.is_legal_move(m));
    }

    #[test]
    fn is_legal_scenario_4() {
        let position = Position::from_fen(
            "rn3b1r/ppp2kpp/3p1n2/8/P3P1bP/5P2/1BP5/R2Kq1NR w - - 1 15".into(),
        );
        let m = Move::new(index_from_coords("f3"), index_from_coords("g4"), MF_CAPTURE, PieceType::Pawn, false, 0);
        assert!(!position.is_legal_move(m));
    }
}
