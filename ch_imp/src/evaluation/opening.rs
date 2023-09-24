use rand::seq::index;

use crate::{
    board::{
        self, attack_and_defend_lookups::AttackedBy, bitboard::Bitboard, board_rep::BoardRep,
        king_position_analysis::ThreatRaycastCollision,
    },
    move_generation::sliding::queen::generate_queen_moves,
    shared::{
        board_utils::{chebyshev_distance, get_coords_from_index, get_file, get_rank},
        piece_type::PieceType,
    },
    MOVE_DATA,
};

use super::{
    eval_precomputed_data::{PieceValueBoard, PieceValues},
    utils::*,
    PieceSafetyInfo,
};

const MATERIAL_VALUES: PieceValues = [
    100, // Pawn
    300, // Knight
    300, // Bishop
    500, // Rook
    900, // Queen
    0,   // King
];

const WHITE_PAWN_SQUARE_SCORE: PieceValueBoard = [
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, -1, -1, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 2, 2, 2, 3, 3, 2, 2,
    2, 2, 2, 2, 3, 3, 2, 2, 2, 4, 4, 4, 4, 4, 4, 4, 4, 5, 5, 5, 5, 5, 5, 5, 5, 0, 0, 0, 0, 0, 0, 0,
    0,
];
const BLACK_PAWN_SQUARE_SCORE: PieceValueBoard = [
    0, 0, 0, 0, 0, 0, 0, 0, 5, 5, 5, 5, 5, 5, 5, 5, 4, 4, 4, 4, 4, 4, 4, 4, 2, 2, 2, 3, 3, 2, 2, 2,
    2, 2, 2, 3, 3, 2, 2, 2, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, -1, -1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0,
];
const PAWN_SQUARE_FACTOR: i32 = 6;

const KNIGHT_SQUARE_SCORE: PieceValueBoard = [
    -1, -1, -1, -1, -1, -1, -1, -1, -1, 0, 0, 0, 0, 0, 0, -1, -1, 0, 1, 0, 0, 1, 0, -1, -1, 0, 0,
    0, 0, 0, 0, -1, -1, 0, 0, 0, 0, 0, 0, -1, -1, 0, 1, 0, 0, 1, 0, -1, -1, 0, 0, 0, 0, 0, 0, -1,
    -1, -1, -1, -1, -1, -1, -1, -1,
];
const KNIGHT_SQUARE_FACTOR: i32 = 3;

const BISHOP_SQUARE_SCORE: PieceValueBoard = [
    -1, -1, -1, -1, -1, -1, -1, -1, -1, 3, 0, 0, 0, 0, 3, -1, -1, 0, 2, 0, 0, 2, 0, -1, -1, 0, 2,
    0, 0, 2, 0, -1, -1, 0, 2, 0, 0, 2, 0, -1, -1, 0, 2, 0, 0, 2, 0, -1, -1, 3, 0, 0, 0, 0, 3, -1,
    -1, -1, -1, -1, -1, -1, -1, -1,
];
const BISHOP_SQUARE_FACTOR: i32 = 3;

const BOARD_CONTROL_SQUARE_REWARD: PieceValueBoard = [
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 2, 3, 3, 3, 3, 2, 1, 1, 2, 3, 6, 6, 3, 2, 1,
    1, 2, 3, 6, 6, 3, 2, 1, 1, 2, 3, 3, 3, 3, 2, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
];

const SAFETY_TABLE: [i32; 100] = [
    0, 0, 1, 2, 3, 5, 7, 9, 12, 15, 18, 22, 26, 30, 35, 39, 44, 50, 56, 62, 68, 75, 82, 85, 89, 97,
    105, 113, 122, 131, 140, 150, 169, 180, 191, 202, 213, 225, 237, 248, 260, 272, 283, 295, 307,
    319, 330, 342, 354, 366, 377, 389, 401, 412, 424, 436, 448, 459, 471, 483, 494, 500, 500, 500,
    500, 500, 500, 500, 500, 500, 500, 500, 500, 500, 500, 500, 500, 500, 500, 500, 500, 500, 500,
    500, 500, 500, 500, 500, 500, 500, 500, 500, 500, 500, 500, 500, 500, 500, 500, 500,
];

const BOARD_CONTROL_SQUARES_PER_POINT: i32 = 6;

const UNDER_DEVELOPED_PENALTY_POSITIONS: [(PieceType, u8); 4] = [
    (PieceType::Knight, 1),
    (PieceType::Bishop, 2),
    (PieceType::Bishop, 5),
    (PieceType::Knight, 6),
];
const UNDER_DEVELOPED_PENALTY_FACTOR: i32 = 10;

const DOUBLE_BISHOP_REWARD: i32 = MATERIAL_VALUES[0] / 2;

const CAN_NOT_CASTLE_PENALTY: i32 = 5;

pub fn calculate(
    board: BoardRep,
    white_pinned: &Vec<ThreatRaycastCollision>,
    black_pinned: &Vec<ThreatRaycastCollision>,
    white_threatboard: u64,
    black_threatboard: u64,
    pawn_structure_eval: i16,
    piece_safety_results: &Vec<PieceSafetyInfo>,
) -> i32 {
    let mut eval = pawn_structure_eval as i32;
    eval += piece_aggregate_score(board, board.white_occupancy, MATERIAL_VALUES);
    eval -= piece_aggregate_score(board, board.black_occupancy, MATERIAL_VALUES);

    eval += piece_square_score(
        board.white_occupancy & board.pawn_bitboard,
        WHITE_PAWN_SQUARE_SCORE,
    ) * PAWN_SQUARE_FACTOR;
    eval -= piece_square_score(
        board.black_occupancy & board.pawn_bitboard,
        BLACK_PAWN_SQUARE_SCORE,
    ) * PAWN_SQUARE_FACTOR;

    eval += piece_square_score(
        board.white_occupancy & board.knight_bitboard,
        KNIGHT_SQUARE_SCORE,
    ) * KNIGHT_SQUARE_FACTOR;
    eval -= piece_square_score(
        board.black_occupancy & board.knight_bitboard,
        KNIGHT_SQUARE_SCORE,
    ) * KNIGHT_SQUARE_FACTOR;

    eval += piece_square_score(
        board.white_occupancy & board.bishop_bitboard,
        BISHOP_SQUARE_SCORE,
    ) * BISHOP_SQUARE_FACTOR;
    eval -= piece_square_score(
        board.black_occupancy & board.bishop_bitboard,
        BISHOP_SQUARE_SCORE,
    ) * BISHOP_SQUARE_FACTOR;

    eval += piece_centralization_score(
        board.white_occupancy & board.pawn_bitboard & board.knight_bitboard,
    );
    eval -= piece_centralization_score(
        board.black_occupancy & board.pawn_bitboard & board.knight_bitboard,
    );

    // Double Bishop reward
    eval += if (board.white_occupancy & board.bishop_bitboard).count_ones() == 2 {
        DOUBLE_BISHOP_REWARD
    } else {
        0
    };
    eval -= if (board.black_occupancy & board.bishop_bitboard).count_ones() == 2 {
        DOUBLE_BISHOP_REWARD
    } else {
        0
    };

    eval += under_developed_penalty(board, board.white_occupancy);
    eval -= under_developed_penalty(board, board.black_occupancy.flip_orientation());

    eval += piece_square_score(
        white_threatboard | board.white_occupancy,
        BOARD_CONTROL_SQUARE_REWARD,
    ) / BOARD_CONTROL_SQUARES_PER_POINT;
    eval -= piece_square_score(
        black_threatboard | board.black_occupancy,
        BOARD_CONTROL_SQUARE_REWARD,
    ) / BOARD_CONTROL_SQUARES_PER_POINT;

    // eval += king_tropism(board.white_king_position, board.black_occupancy, board);
    // eval -= king_tropism(board.black_king_position, board.white_occupancy, board);

    eval -= king_openness(board.white_king_position, board);
    eval += king_openness(board.black_king_position, board);

    eval -= king_neighbourhood_treat_level(board.white_king_position, false, board);
    eval += king_neighbourhood_treat_level(board.black_king_position, true, board);

    if !board.white_king_side_castling {
        eval -= CAN_NOT_CASTLE_PENALTY;
    }
    if !board.white_queen_side_castling {
        eval -= CAN_NOT_CASTLE_PENALTY;
    }
    if !board.black_king_side_castling {
        eval += CAN_NOT_CASTLE_PENALTY;
    }
    if !board.black_queen_side_castling {
        eval += CAN_NOT_CASTLE_PENALTY;
    }

    for white_pin in white_pinned {
        if white_pin.reveal_attack == false {
            let piece: PieceType = board.get_piece_type_at_index(white_pin.at);
            eval -= MATERIAL_VALUES[piece as usize - 1] / 2
        } else {
            eval -= 25; // Todo improve this - currently a flat penalty for opponent having a possible reveal attack
        }
    }
    for black_pin in black_pinned {
        if black_pin.reveal_attack == false {
            let piece = board.get_piece_type_at_index(black_pin.at);
            eval += MATERIAL_VALUES[piece as usize - 1] / 2
        } else {
            eval += 25; // Todo improve this - currently a flat penalty for opponent having a possible reveal attack
        }
    }

    eval += sum_piece_safety_penalties(piece_safety_results, MATERIAL_VALUES);

    eval
}

fn piece_centralization_score(side_occupancy: u64) -> i32 {
    let mut occ = side_occupancy;
    let mut score = 0;
    while occ != 0 {
        let pos = occ.trailing_zeros();
        score += 3 - distance_to_center(pos as u8);
        occ ^= 1 << pos;
    }
    score
}

fn under_developed_penalty(board: BoardRep, orientated_side_occupancy: u64) -> i32 {
    let mut score = 0;

    for penalty in UNDER_DEVELOPED_PENALTY_POSITIONS {
        if orientated_side_occupancy.occupied(penalty.1)
            && board.get_piece_type_at_index(penalty.1) == penalty.0
        {
            score += 1;
        }
    }

    score * UNDER_DEVELOPED_PENALTY_FACTOR
}

// // https://www.chessprogramming.org/King_Safety
// // The higher the safer the king is
// fn king_tropism(king_pos: u8, opponent_occupancy: u64, board: BoardRep) -> i32 {
//     let mut occ = opponent_occupancy;
//     let mut score = 0;
//     while occ != 0 {
//         let pos = occ.trailing_zeros() as u8;
//         let sc = match board.get_piece_type_at_index(pos) {
//             PieceType::Knight => chebyshev_distance(pos as i8, king_pos as i8),
//             PieceType::Bishop | PieceType::Rook => chebyshev_distance(pos as i8, king_pos as i8),
//             PieceType::Queen => chebyshev_distance(pos as i8, king_pos as i8) * 2,
//             PieceType::Pawn | PieceType::King => 0, // Should never happen
//             PieceType::None => panic!("Unknown piece type"),
//         } as i32;
//         score += sc;
//         occ ^= 1 << pos;
//     }
//     score
// }

// King openness is a penalty for each square the king could reach if they were a queen
fn king_openness(king_pos: u8, board: BoardRep) -> i32 {
    let possible_queen_moves =
        generate_queen_moves(king_pos, board, 0, board.occupancy, None, None);
    possible_queen_moves.len() as i32
}

fn king_neighbourhood_treat_level(king_pos: u8, is_black: bool, board: BoardRep) -> i32 {
    let mut neigbourhood = get_neighbourhood_mask(king_pos, is_black);
    let mut score: usize = 0;

    while neigbourhood != 0 {
        let i = neigbourhood.trailing_zeros();
        let attacked_by = board.get_attacked_by(i as u8, !is_black);
        score += get_score(attacked_by);

        neigbourhood ^= 1 << i;
    }
    score = usize::min(100, score);
    SAFETY_TABLE[score]
}

fn get_score(attacked_by: AttackedBy) -> usize {
    ((2 * (attacked_by.pawns + attacked_by.knights + if attacked_by.bishop { 1 } else { 0 }))
        + (3 * attacked_by.rooks)
        + if attacked_by.queen { 5 } else { 0 }) as usize
}

fn get_neighbourhood_mask(king_pos: u8, is_black: bool) -> u64 {
    let neigbourhood = MOVE_DATA.king_moves[king_pos as usize];

    let rank = get_rank(king_pos);

    if (is_black && rank <= 4) || (!is_black && rank >= 5) {
        return neigbourhood;
    }

    let opponent_dir_offset: i8 = if is_black { -17 } else { 15 };
    let offset = king_pos as i8 + opponent_dir_offset;

    let file = get_file(king_pos);
    let mask = if file == 0 {
        if is_black {
            0b11
        } else {
            0b11
        }
    } else if file == 7 {
        if is_black {
            0b11
        } else {
            0b11
        }
    } else {
        0b111
    };
    neigbourhood | mask << offset
}
