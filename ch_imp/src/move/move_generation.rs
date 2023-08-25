use crate::{
    board::position::Position,
    shared::{board_utils::get_file, piece_type, constants::MF_CAPTURE},
};

use super::{Move, move_dict::get_move_bitboard};

pub fn generate_moves(
    position: Position,
    ep_index: u8,
    wkc: bool,
    wqc: bool,
    bkc: bool,
    bqc: bool,
) -> Vec<Move> {
    let mut moves = Vec::new();

    for index in 0..64 {
        if position.occupancy.occupied(index) {
            let is_black = position.black_bitboard.occupied(index);
            moves =
                generate_position_moves(position, index, is_black, ep_index, wkc, wqc, bkc, bqc);
        }
    }

    moves
}

fn generate_position_moves(
    position: Position,
    index: u8,
    is_black: bool,
    ep_index: u8,
    wkc: bool,
    wqc: bool,
    bkc: bool,
    bqc: bool,
) -> Vec<Move> {
    let piece_type = position.get_piece_type_at_index(index);

    match piece_type {
        piece_type::PieceType::None => panic!("Unknown piece"),
        piece_type::PieceType::Pawn => generate_pawn_moves(position, index, is_black, ep_index),
        piece_type::PieceType::Knight => generate_knight_moves(position, index, is_black),
        piece_type::PieceType::Bishop => generate_bishop_moves(position, index, is_black),
        piece_type::PieceType::Rook => generate_rook_moves(position, index, is_black),
        piece_type::PieceType::Queen => generate_queen_moves(position, index, is_black),
        piece_type::PieceType::King => {
            generate_king_moves(position, index, is_black, wkc, wqc, bkc, bqc)
        }
    }
}

fn generate_king_moves(
    position: Position,
    index: u8,
    is_black: bool,
    wkc: bool,
    wqc: bool,
    bkc: bool,
    bqc: bool,
) -> Vec<Move> {
    todo!()
}

fn generate_queen_moves(position: Position, index: u8, is_black: bool) -> Vec<Move> {
    todo!()
}

fn generate_rook_moves(position: Position, index: u8, is_black: bool) -> Vec<Move> {
    todo!()
}

fn generate_bishop_moves(position: Position, index: u8, is_black: bool) -> Vec<Move> {
    todo!()
}

fn generate_knight_moves(position: Position, index: u8, is_black: bool) -> Vec<Move> {
    let mut results: Vec<_> = Vec::new();
    let file = get_file(index);
    let opponent_bitboard = if is_black {
        position.white_bitboard
    } else {
        position.black_bitboard
    };
    // U2R1 = +16-1 = 15
    if index <= 48 && file != 7 {
        let tar = index + 15;
        if !position.occupancy.occupied(tar) {
            results.push(Move::new(index, tar, 0b0, piece_type::PieceType::Knight));
        } else if opponent_bitboard.occupied(tar) {
            results.push(Move::new(index, tar, MF_CAPTURE, piece_type::PieceType::Knight));
        }
    }
    // U1R2 = +8-2 = 6
    if index <= 55 && file < 6 {
        let tar = index + 6;
        if !position.occupancy.occupied(tar) {
            results.push(Move::new(index, tar, 0b0, piece_type::PieceType::Knight));
        } else if opponent_bitboard.occupied(tar) {
            results.push(Move::new(index, tar, MF_CAPTURE, piece_type::PieceType::Knight));
        }
    }
    // D1R2 = -8-2 = -10
    if index >= 10 && file < 6 {
        let tar = index - 10;
        if !position.occupancy.occupied(tar) {
            results.push(Move::new(index, tar, 0b0, piece_type::PieceType::Knight));
        } else if opponent_bitboard.occupied(tar) {
            results.push(Move::new(index, tar, MF_CAPTURE, piece_type::PieceType::Knight));
        }
    }
    // D2R1 = -16-1 = -17
    if index >= 17 && file != 7 {
        let tar = index - 17;
        if !position.occupancy.occupied(tar) {
            results.push(Move::new(index, tar, 0b0, piece_type::PieceType::Knight));
        } else if opponent_bitboard.occupied(tar) {
            results.push(Move::new(index, tar, MF_CAPTURE, piece_type::PieceType::Knight));
        }
    }
    // D2L1 = -16+1 = -15
    if index >= 15 && file != 0 {
        let tar = index - 15;
        if !position.occupancy.occupied(tar) {
            results.push(Move::new(index, tar, 0b0, piece_type::PieceType::Knight));
        } else if opponent_bitboard.occupied(tar) {
            results.push(Move::new(index, tar, MF_CAPTURE, piece_type::PieceType::Knight));
        }
    }
    // D1L2 = -8+2 = -6
    if index >= 6 && file > 1 {
        let tar = index - 6;
        if !position.occupancy.occupied(tar) {
            results.push(Move::new(index, tar, 0b0, piece_type::PieceType::Knight));
        } else if opponent_bitboard.occupied(tar) {
            results.push(Move::new(index, tar, MF_CAPTURE, piece_type::PieceType::Knight));
        }
    }
    // U1L2 = 8+2 = 10
    if index <= 53 && file > 1 {
        let tar = index + 10;
        if !position.occupancy.occupied(tar) {
            results.push(Move::new(index, tar, 0b0, piece_type::PieceType::Knight));
        } else if opponent_bitboard.occupied(tar) {
            results.push(Move::new(index, tar, MF_CAPTURE, piece_type::PieceType::Knight));
        }
    }
    // U2L1 = 16+1 = 17
    if index <= 46 && file != 0 {
        let tar = index + 17;
        if !position.occupancy.occupied(tar) {
            results.push(Move::new(index, tar, 0b0, piece_type::PieceType::Knight));
        } else if opponent_bitboard.occupied(tar) {
            results.push(Move::new(index, tar, MF_CAPTURE, piece_type::PieceType::Knight));
        }
    }
    results
}

fn generate_pawn_moves(position: Position, index: u8, is_black: bool, ep_index: u8) -> Vec<Move> {
    todo!()
}

#[cfg(test)]
mod test {
    use crate::{board::position::Position, shared::{board_utils::index_from_coords, constants::MF_CAPTURE, piece_type::PieceType}, r#move::{Move, move_generation::generate_knight_moves}};


    #[test]
    pub fn generate_knight_moves_e4() {
        let position = Position::new("k7/8/8/8/4N3/8/8/7K".into());
        let moves = generate_knight_moves(position, index_from_coords("e4"), false);
        assert_eq!(moves.len(), 8);
    }

    #[test]
    pub fn generate_knight_moves_g7_capture_on_f5() {
        let position = Position::new("k7/6N1/8/5p2/8/8/8/7K".into());
        let moves = generate_knight_moves(position, index_from_coords("g7"), false);

        assert_eq!(moves.len(), 4);
        let capture_move = Move::new(index_from_coords("g7"), index_from_coords("f5"), MF_CAPTURE, PieceType::Knight);
        assert!(moves.contains(&capture_move))
    }

}
