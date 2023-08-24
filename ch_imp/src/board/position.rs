use super::{bitboard::Bitboard, piece::PieceType};

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Position {
    pub bitboard: Bitboard,
    pub white_bitboard: Bitboard,
    pub black_bitboard: Bitboard,
    pub pawn_bitboard: Bitboard,
    pub knight_bitboard: Bitboard,
    pub bishop_bitboard: Bitboard,
    pub rook_bitboard: Bitboard,
    pub queen_bitboard: Bitboard,
    pub king_bitboard: Bitboard,
    pub check: bool,
}

impl Position {
    pub fn new(fen: String) -> Self {
        let mut bitboard = Bitboard::default();
        let mut white_bitboard = Bitboard::default();
        let mut black_bitboard = Bitboard::default();
        let mut pawn_bitboard = Bitboard::default();
        let mut knight_bitboard = Bitboard::default();
        let mut bishop_bitboard = Bitboard::default();
        let mut rook_bitboard = Bitboard::default();
        let mut queen_bitboard = Bitboard::default();
        let mut king_bitboard = Bitboard::default();
        let mut check = false;

        let mut position_index = 63;
        let mut rank_lenght = 0; // The number of spaces we've worked through in the current rank
        for i in 0..fen.len() {
            // Get char
            let char = fen.chars().nth(i).unwrap();

            // Check if is a shift digit
            if char.is_ascii_digit() {
                let digit = (char as i16 - 0x30) as i8;
                position_index -= digit;
                rank_lenght += digit;
                continue;
            }

            if char == '/' {
                position_index -= 8 - rank_lenght;
                rank_lenght = 0;
                continue;
            }

            let piece_type = match char {
                'P' | 'p' => PieceType::Pawn,
                'B' | 'b' => PieceType::Bishop,
                'N' | 'n' => PieceType::Knight,
                'R' | 'r' => PieceType::Rook,
                'Q' | 'q' => PieceType::Queen,
                'K' | 'k' => PieceType::King,
                _ => panic!("Unknown piece type {}", char),
            };

            let piece_is_black = char.is_ascii_lowercase();
            let mut colour_board_to_change = if piece_is_black {
                black_bitboard
            } else {
                white_bitboard
            };

            match piece_type {
                PieceType::Pawn => {
                    (pawn_bitboard, colour_board_to_change, bitboard) = flip_piece(
                        position_index,
                        pawn_bitboard,
                        colour_board_to_change,
                        bitboard,
                    )
                }
                PieceType::Knight => {
                    (knight_bitboard, colour_board_to_change, bitboard) = flip_piece(
                        position_index,
                        knight_bitboard,
                        colour_board_to_change,
                        bitboard,
                    )
                }
                PieceType::Bishop => {
                    (bishop_bitboard, colour_board_to_change, bitboard) = flip_piece(
                        position_index,
                        bishop_bitboard,
                        colour_board_to_change,
                        bitboard,
                    )
                }
                PieceType::Rook => {
                    (rook_bitboard, colour_board_to_change, bitboard) = flip_piece(
                        position_index,
                        rook_bitboard,
                        colour_board_to_change,
                        bitboard,
                    )
                }
                PieceType::Queen => {
                    (queen_bitboard, colour_board_to_change, bitboard) = flip_piece(
                        position_index,
                        queen_bitboard,
                        colour_board_to_change,
                        bitboard,
                    )
                }
                PieceType::King => {
                    (king_bitboard, colour_board_to_change, bitboard) = flip_piece(
                        position_index,
                        king_bitboard,
                        colour_board_to_change,
                        bitboard,
                    )
                }
            }

            if piece_is_black {
                black_bitboard = colour_board_to_change;
            } else {
                white_bitboard = colour_board_to_change;
            }

            position_index -= 1;
            rank_lenght += 1;
        }

        Self {
            bitboard,
            white_bitboard,
            black_bitboard,
            pawn_bitboard,
            knight_bitboard,
            bishop_bitboard,
            rook_bitboard,
            queen_bitboard,
            king_bitboard,
            check,
        }
    }
}

impl Default for Position {
    fn default() -> Self {
        Self {
            bitboard: Bitboard::new(18446462598732906495),
            white_bitboard: Bitboard::new(65535),
            black_bitboard: Bitboard::new(18446462598732840960),
            pawn_bitboard: Bitboard::new(71776119061282560),
            knight_bitboard: Bitboard::new(4755801206503243842),
            bishop_bitboard: Bitboard::new(2594073385365405732),
            rook_bitboard: Bitboard::new(9295429630892703873),
            queen_bitboard: Bitboard::new(1152921504606846992),
            king_bitboard: Bitboard::new(576460752303423496),
            check: false,
        }
    }
}

fn flip_piece(
    index: i8,
    piece_bitboard: Bitboard,
    colour_bitboard: Bitboard,
    all_bitboard: Bitboard,
) -> (Bitboard, Bitboard, Bitboard) {
    (
        piece_bitboard.flip(index as u8),
        colour_bitboard.flip(index as u8),
        all_bitboard.flip(index as u8),
    )
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    pub fn new_start_pos() {
        let result = Position::new("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR".into());
        assert_eq!(result.bitboard, Bitboard::new(18446462598732906495));
        assert_eq!(result.black_bitboard, Bitboard::new(18446462598732840960));
        assert_eq!(result.white_bitboard, Bitboard::new(65535));
        assert_eq!(result.pawn_bitboard, Bitboard::new(71776119061282560));
        assert_eq!(result.bishop_bitboard, Bitboard::new(2594073385365405732));
        assert_eq!(result.knight_bitboard, Bitboard::new(4755801206503243842));
        assert_eq!(result.rook_bitboard, Bitboard::new(9295429630892703873));
        assert_eq!(result.queen_bitboard, Bitboard::new(1152921504606846992));
        assert_eq!(result.king_bitboard, Bitboard::new(576460752303423496));
    }
}
