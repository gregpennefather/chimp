pub struct BoardMetrics {
    pub psudolegal_moves: Vec<u16>,
    pub white_threat_bitboard: u64,
    pub black_threat_bitboard: u64,
}

impl BoardMetrics {
    pub fn is_white_check(&self) -> bool {
        false
    }

    pub fn is_black_check(&self) -> bool {
        false
    }
}

#[cfg(test)]
mod test {
    use super::*;

    // #[test]
    // pub fn is_white_check_starting_pos() {
    //     assert!(!BoardState::default().generate_metrics().is_white_check());
    // }

    // #[test]
    // pub fn is_black_check_starting_pos() {
    //     assert!(!BoardState::default().is_black_check());
    // }

    // #[test]
    // pub fn is_black_check_early_b5_bishop() {
    //     assert!(BoardState::from_fen(
    //         &"rnbqkbnr/ppp1pppp/8/1B1p4/4P3/8/PPPP1PPP/RNBQK1NR b KQkq - 0 1".into()
    //     )
    //     .is_black_check())
    // }

    // #[test]
    // pub fn is_white_check_early_c2_knight() {
    //     assert!(BoardState::from_fen(
    //         &"r1bqkbnr/pppppppp/8/8/4P3/3P4/PPnB1PPP/RN1QKBNR w KQkq - 0 1".into()
    //     )
    //     .is_white_check())
    // }
}
