use log::trace;
use rand::seq::index;

use crate::{
    board::{
        self,
        attack_and_defend_lookups::{AttackAndDefendTable, AttackedBy},
        bitboard::Bitboard,
        board_rep::BoardRep,
        king_position_analysis::{KingPositionAnalysis, ThreatRaycastCollision},
        see::{piece_safety, square_control},
    },
    move_generation::sliding::queen::generate_queen_moves,
    shared::{
        board_utils::{
            chebyshev_distance, get_coords_from_index, get_file, get_rank, index_from_coords,
        },
        piece_type::PieceType,
    },
    MOVE_DATA,
};

use super::{
    eval_precomputed_data::{PieceValueBoard, PieceValues},
    get_piece_safety,
    shared::{calculate_controlled_space_score, count_knight_outposts, get_fork_wins},
    subcategories::{mobility::get_mobility, rook::on_open_file::count_rooks_on_open_file},
    utils::*,
    PieceSafetyInfo,
};

const MATERIAL_VALUES: PieceValues = [
    110,  // Pawn
    400,  // Knight
    450,  // Bishop
    700,  // Rook
    1250, // Queen
    0,    // King
];

const WHITE_PAWN_SQUARE_SCORE: PieceValueBoard = [
    0, 0, 0, 0, 0, 0, 0, 0, -35, -1, -20, -23, -15, 24, 38, -22, -26, -4, -4, -10, 3, 3, 33, -12,
    -27, -2, -5, 12, 17, 6, 10, -25, -14, 13, 6, 21, 23, 12, 17, -23, -6, 7, 26, 31, 65, 56, 25,
    -20, 98, 134, 61, 95, 68, 126, 34, -11, 0, 0, 0, 0, 0, 0, 0, 0,
];
const BLACK_PAWN_SQUARE_SCORE: PieceValueBoard = [
    0, 0, 0, 0, 0, 0, 0, 0, 98, 134, 61, 95, 68, 126, 34, -11, -6, 7, 26, 31, 65, 56, 25, -20, -14,
    13, 6, 21, 23, 12, 17, -23, -27, -2, -5, 12, 17, 6, 10, -25, -26, -4, -4, -10, 3, 3, 33, -12,
    -35, -1, -20, -23, -15, 24, 38, -22, 0, 0, 0, 0, 0, 0, 0, 0,
];

const WHITE_KNIGHT_SQUARE_SCORE: PieceValueBoard = [
    -105, -21, -58, -33, -17, -28, -19, -23, -29, -53, -12, -3, -1, 18, -14, -19, -23, -9, 12, 10,
    19, 17, 25, -16, -13, 4, 16, 13, 28, 19, 21, -8, -9, 17, 19, 53, 37, 69, 18, 22, -47, 60, 37,
    65, 84, 129, 73, 44, -73, -41, 72, 36, 23, 62, 7, -17, -167, -89, -34, -49, 61, -97, -15, -107,
];
const BLACK_KNIGHT_SQUARE_SCORE: PieceValueBoard = [
    -167, -89, -34, -49, 61, -97, -15, -107, -73, -41, 72, 36, 23, 62, 7, -17, -47, 60, 37, 65, 84,
    129, 73, 44, -9, 17, 19, 53, 37, 69, 18, 22, -13, 4, 16, 13, 28, 19, 21, -8, -23, -9, 12, 10,
    19, 17, 25, -16, -29, -53, -12, -3, -1, 18, -14, -19, -105, -21, -58, -33, -17, -28, -19, -23,
];

const WHITE_BISHOP_SQUARE_SCORE: PieceValueBoard = [
    -33, -3, -14, -21, -13, -12, -39, -21, 4, 15, 16, 0, 7, 21, 33, 1, 0, 15, 15, 15, 14, 27, 18,
    10, -6, 13, 13, 26, 34, 12, 10, 4, -4, 5, 19, 50, 37, 37, 7, -2, -16, 37, 43, 40, 35, 50, 37,
    -2, -26, 16, -18, -13, 30, 59, 18, -47, -29, 4, -82, -37, -25, -42, 7, -8,
];
const BLACK_BISHOP_SQUARE_SCORE: PieceValueBoard = [
    -29, 4, -82, -37, -25, -42, 7, -8, -26, 16, -18, -13, 30, 59, 18, -47, -16, 37, 43, 40, 35, 50,
    37, -2, -4, 5, 19, 50, 37, 37, 7, -2, -6, 13, 13, 26, 34, 12, 10, 4, 0, 15, 15, 15, 14, 27, 18,
    10, 4, 15, 16, 0, 7, 21, 33, 1, -33, -3, -14, -21, -13, -12, -39, -21,
];

const WHITE_ROOK_SQUARE_SCORE: PieceValueBoard = [
    -19, -13, 1, 17, 16, 7, -37, -26, -44, -16, -20, -9, -1, 11, -6, -71, -45, -25, -16, -17, 3, 0,
    -5, -33, -36, -26, -12, -1, 9, -7, 6, -23, -24, -11, 7, 26, 24, 35, -8, -20, -5, 19, 26, 36,
    17, 45, 61, 16, 27, 32, 58, 62, 80, 67, 26, 44, 32, 42, 32, 51, 63, 9, 31, 43,
];
const BLACK_ROOK_SQUARE_SCORE: PieceValueBoard = [
    32, 42, 32, 51, 63, 9, 31, 43, 27, 32, 58, 62, 80, 67, 26, 44, -5, 19, 26, 36, 17, 45, 61, 16,
    -24, -11, 7, 26, 24, 35, -8, -20, -36, -26, -12, -1, 9, -7, 6, -23, -45, -25, -16, -17, 3, 0,
    -5, -33, -44, -16, -20, -9, -1, 11, -6, -71, -19, -13, 1, 17, 16, 7, -37, -26,
];

const WHITE_QUEEN_SQUARE_SCORE: PieceValueBoard = [
    -1, -18, -9, 10, -15, -25, -31, -50, -35, -8, 11, 2, 8, 15, -3, 1, -14, 2, -11, -2, -5, 2, 14,
    5, -9, -26, -9, -10, -2, -4, 3, -3, -27, -27, -16, -16, -1, 17, -2, 1, -13, -17, 7, 8, 29, 56,
    47, 57, -24, -39, -5, 1, -16, 57, 28, 54, -28, 0, 29, 12, 59, 44, 43, 45,
];
const BLACK_QUEEN_SQUARE_SCORE: PieceValueBoard = [
    -28, 0, 29, 12, 59, 44, 43, 45, -24, -39, -5, 1, -16, 57, 28, 54, -13, -17, 7, 8, 29, 56, 47,
    57, -27, -27, -16, -16, -1, 17, -2, 1, -9, -26, -9, -10, -2, -4, 3, -3, -14, 2, -11, -2, -5, 2,
    14, 5, -35, -8, 11, 2, 8, 15, -3, 1, -1, -18, -9, 10, -15, -25, -31, -50,
];

const WHITE_KING_SQUARE_SCORE: PieceValueBoard = [
    -15, 36, 12, -54, 8, -28, 24, 14, 1, 7, -8, -64, -43, -16, 9, 8, -14, -14, -22, -46, -44, -30,
    -15, -27, -49, -1, -27, -39, -46, -44, -33, -51, -17, -20, -12, -27, -30, -25, -14, -36, -9,
    24, 2, -16, -20, 6, 22, -22, 29, -1, -20, -7, -8, -4, -38, -29, -65, 23, 16, -15, -56, -34, 2,
    13,
];
const BLACK_KING_SQUARE_SCORE: PieceValueBoard = [
    -65, 23, 16, -15, -56, -34, 2, 13, 29, -1, -20, -7, -8, -4, -38, -29, -9, 24, 2, -16, -20, 6,
    22, -22, -17, -20, -12, -27, -30, -25, -14, -36, -49, -1, -27, -39, -46, -44, -33, -51, -14,
    -14, -22, -46, -44, -30, -15, -27, 1, 7, -8, -64, -43, -16, 9, 8, -15, 36, 12, -54, 8, -28, 24,
    14,
];

const CENTER_CONTROL_REWARD: PieceValueBoard = [
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 2, 2, 1, 0, 0, 0, 0, 2, 4, 4, 2, 0, 0,
    0, 0, 2, 4, 4, 2, 0, 0, 0, 0, 1, 2, 2, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
];
const CENTER_SCALE_FACTOR: i16 = 15;
const CENTER_FIRST: usize = 18; // F3
const CENTER_LAST: usize = 64 - 18; // C6

const SAFETY_TABLE: [i16; 100] = [
    0, 0, 1, 2, 3, 5, 7, 9, 12, 15, 18, 22, 26, 30, 35, 39, 44, 50, 56, 62, 68, 75, 82, 85, 89, 97,
    105, 113, 122, 131, 140, 150, 169, 180, 191, 202, 213, 225, 237, 248, 260, 272, 283, 295, 307,
    319, 330, 342, 354, 366, 377, 389, 401, 412, 424, 436, 448, 459, 471, 483, 494, 500, 500, 500,
    500, 500, 500, 500, 500, 500, 500, 500, 500, 500, 500, 500, 500, 500, 500, 500, 500, 500, 500,
    500, 500, 500, 500, 500, 500, 500, 500, 500, 500, 500, 500, 500, 500, 500, 500, 500,
];

const UNDER_DEVELOPED_PENALTY_POSITIONS: [(PieceType, u8); 4] = [
    (PieceType::Knight, 1),
    (PieceType::Bishop, 2),
    (PieceType::Bishop, 5),
    (PieceType::Knight, 6),
];
const UNDER_DEVELOPED_PENALTY_FACTOR: i16 = 25;

const DOUBLE_BISHOP_REWARD: i16 = 100;
const KNIGHT_OUTPOST_REWARD: i16 = 75;
const ROOK_ON_OPEN_FILE_REWARD: i16 = 100;
const TEMPO_REWARD: i16 = 50;

const CAN_NOT_CASTLE_PENALTY: i16 = 25;

const PAWN_DIFFERENCE_SCORE: [i16; 8] = [0, 12, 26, 42, 60, 80, 102, 126];

pub fn calculate(
    board: BoardRep,
    white_pinned: &Vec<ThreatRaycastCollision>,
    black_pinned: &Vec<ThreatRaycastCollision>,
    pawn_structure_eval: i16,
    open_files: u64,
    piece_safety_results: &Vec<PieceSafetyInfo>,
    ad_table: &mut AttackAndDefendTable,
    white_in_check: bool,
    black_in_check: bool,
) -> i16 {
    let mut eval = pawn_structure_eval as i16;

    // Material
    eval += material_score(board);

    // Piece Positioning
    eval += piece_positioning_score(board, white_in_check, black_in_check, open_files, ad_table);

    // Board control
    eval += get_center_control_score(ad_table, board);

    // King Safety
    eval += king_safety(board, ad_table);

    // Turn order advantages
    eval += turn_order_advantage(board, white_pinned, black_pinned);

    // Piece Safety
    eval += get_piece_safety_penalty(piece_safety_results, MATERIAL_VALUES, board.black_turn);

    // Space Control
    eval += space_control(board, ad_table);

    // Mobility
    eval += mobility_score(board);

    // Tempo
    eval += if board.black_turn {
        -TEMPO_REWARD
    } else {
        TEMPO_REWARD
    };

    eval
}

fn material_score(board: BoardRep) -> i16 {
    let mut score = 0;
    score += piece_aggregate_score(board, board.white_occupancy, MATERIAL_VALUES);
    score -= piece_aggregate_score(board, board.black_occupancy, MATERIAL_VALUES);

    // Double Bishop reward
    score += if (board.white_occupancy & board.bishop_bitboard).count_ones() == 2 {
        DOUBLE_BISHOP_REWARD
    } else {
        0
    };
    score -= if (board.black_occupancy & board.bishop_bitboard).count_ones() == 2 {
        DOUBLE_BISHOP_REWARD
    } else {
        0
    };

    // Pawn advantage
    let pawn_difference = (board.pawn_bitboard & board.white_occupancy).count_ones() as i16
        - (board.pawn_bitboard & board.black_occupancy).count_ones() as i16;
    let difference_score =
        i16::signum(pawn_difference) * PAWN_DIFFERENCE_SCORE[i16::abs(pawn_difference) as usize];
    score += difference_score;

    score
}

fn piece_positioning_score(
    board: BoardRep,
    white_in_check: bool,
    black_in_check: bool,
    open_files: u64,
    ad_table: &mut AttackAndDefendTable,
) -> i16 {
    let mut eval = 0;
    // Square score
    eval += piece_square_score(
        board.white_occupancy & board.pawn_bitboard,
        WHITE_PAWN_SQUARE_SCORE,
    );
    eval -= piece_square_score(
        board.black_occupancy & board.pawn_bitboard,
        BLACK_PAWN_SQUARE_SCORE,
    );

    eval += piece_square_score(
        board.white_occupancy & board.knight_bitboard,
        WHITE_KNIGHT_SQUARE_SCORE,
    );
    eval -= piece_square_score(
        board.black_occupancy & board.knight_bitboard,
        BLACK_KNIGHT_SQUARE_SCORE,
    );

    eval += piece_square_score(
        board.white_occupancy & board.bishop_bitboard,
        WHITE_BISHOP_SQUARE_SCORE,
    );
    eval -= piece_square_score(
        board.black_occupancy & board.bishop_bitboard,
        BLACK_BISHOP_SQUARE_SCORE,
    );

    eval += piece_square_score(
        board.white_occupancy & board.rook_bitboard,
        WHITE_ROOK_SQUARE_SCORE,
    );
    eval -= piece_square_score(
        board.black_occupancy & board.rook_bitboard,
        BLACK_ROOK_SQUARE_SCORE,
    );

    eval += piece_square_score(
        board.white_occupancy & board.queen_bitboard,
        WHITE_QUEEN_SQUARE_SCORE,
    );
    eval -= piece_square_score(
        board.black_occupancy & board.queen_bitboard,
        BLACK_QUEEN_SQUARE_SCORE,
    );
    eval += WHITE_KING_SQUARE_SCORE[board.white_king_position as usize];
    eval -= BLACK_KING_SQUARE_SCORE[board.black_king_position as usize];

    // Development
    eval += under_developed_penalty(board, board.white_occupancy);
    eval -= under_developed_penalty(board, board.black_occupancy.flip_orientation());

    // Knight Outpost
    eval += count_knight_outposts(
        false,
        board.white_occupancy & board.knight_bitboard,
        board.white_occupancy & board.pawn_bitboard,
        board.black_occupancy & board.pawn_bitboard,
    ) * KNIGHT_OUTPOST_REWARD;
    eval -= count_knight_outposts(
        false,
        board.white_occupancy & board.knight_bitboard,
        board.white_occupancy & board.pawn_bitboard,
        board.black_occupancy & board.pawn_bitboard,
    ) * KNIGHT_OUTPOST_REWARD;

    // Rook on open file
    eval += count_rooks_on_open_file(board.rook_bitboard & board.white_occupancy, open_files)
        * ROOK_ON_OPEN_FILE_REWARD;
    eval -= count_rooks_on_open_file(board.rook_bitboard & board.black_occupancy, open_files)
        * ROOK_ON_OPEN_FILE_REWARD;

    // How much can we win from a fork
    eval += get_fork_wins(
        false,
        board,
        MATERIAL_VALUES,
        white_in_check,
        black_in_check,
        ad_table,
    );
    eval -= get_fork_wins(
        true,
        board,
        MATERIAL_VALUES,
        white_in_check,
        black_in_check,
        ad_table,
    );

    eval
}

fn king_safety(board: BoardRep, ad_table: &mut AttackAndDefendTable) -> i16 {
    let mut score = 0;
    score -= king_openness(board.white_king_position, board, ad_table);
    score += king_openness(board.black_king_position, board, ad_table);

    score -= king_neighbourhood_treat_level(board.white_king_position, false, board);
    score += king_neighbourhood_treat_level(board.black_king_position, true, board);

    if !board.white_king_side_castling {
        score -= CAN_NOT_CASTLE_PENALTY;
    }
    if !board.white_queen_side_castling {
        score -= CAN_NOT_CASTLE_PENALTY;
    }
    if !board.black_king_side_castling {
        score += CAN_NOT_CASTLE_PENALTY;
    }
    if !board.black_queen_side_castling {
        score += CAN_NOT_CASTLE_PENALTY;
    }
    score
}

fn under_developed_penalty(board: BoardRep, orientated_side_occupancy: u64) -> i16 {
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

fn turn_order_advantage(
    board: BoardRep,
    white_pinned: &[ThreatRaycastCollision],
    black_pinned: &[ThreatRaycastCollision],
) -> i16 {
    let mut score = 0;
    for white_pin in white_pinned {
        if white_pin.reveal_attack == false {
            let piece: PieceType = board.get_piece_type_at_index(white_pin.at);
            score -= MATERIAL_VALUES[piece as usize - 1] / 2
        } else {
            score -= 25; // Todo improve this - currently a flat penalty for opponent having a possible reveal attack
        }
    }
    for black_pin in black_pinned {
        if black_pin.reveal_attack == false {
            let piece = board.get_piece_type_at_index(black_pin.at);
            score += MATERIAL_VALUES[piece as usize - 1] / 2
        } else {
            score += 25; // Todo improve this - currently a flat penalty for opponent having a possible reveal attack
        }
    }
    score
}

fn space_control(board: BoardRep, ad_table: &mut AttackAndDefendTable) -> i16 {
    let w = calculate_controlled_space_score(false, board, ad_table);
    let b = calculate_controlled_space_score(true, board, ad_table);
    w - b
}

fn mobility_score(board: BoardRep) -> i16 {
    let w = get_mobility(false, board) as i16 - 50;
    let b = get_mobility(true, board) as i16 - 50;
    (w - b) * 2
}

// King openness is a penalty for each square the king could reach if they were a queen
fn king_openness(king_pos: u8, board: BoardRep, ad_table: &mut AttackAndDefendTable) -> i16 {
    let possible_queen_moves = generate_queen_moves(
        king_pos,
        board,
        ad_table,
        0,
        board.occupancy,
        None,
        None,
        None,
    );
    possible_queen_moves.len() as i16
}

fn king_neighbourhood_treat_level(king_pos: u8, is_black: bool, board: BoardRep) -> i16 {
    let mut neigbourhood = get_neighbourhood_mask(king_pos, is_black);
    let mut score: usize = 0;

    while neigbourhood != 0 {
        let i = neigbourhood.trailing_zeros();
        let attacked_by = board.get_attacked_by(i as u8, !is_black);
        score += get_score(attacked_by);

        neigbourhood ^= 1 << i;
    }
    // println!("{} king safety: {score}", if is_black { "black"} else {"white"});
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

fn get_center_control_score(ad_table: &mut AttackAndDefendTable, board: BoardRep) -> i16 {
    let mut r = 0;
    for i in CENTER_FIRST..CENTER_LAST {
        if CENTER_CONTROL_REWARD[i] != 0 {
            let control = get_square_control(i as u8, ad_table, board);
            r += control * CENTER_CONTROL_REWARD[i];
            // println!(
            //     "{}:{control}*{}",
            //     get_coords_from_index(i as u8),
            //     CENTER_CONTROL_REWARD[i]
            // )
        }
    }
    // println!("center control: {r}");
    r * CENTER_SCALE_FACTOR
}

fn get_square_control(index: u8, ad_table: &mut AttackAndDefendTable, board: BoardRep) -> i16 {
    let white = ad_table.get_attacked_by(index, board, false);
    let black = ad_table.get_attacked_by(index, board, true);

    // If white control the square through occupancy, confirm its safe
    if board.white_occupancy.occupied(index) {
        let piece_type = board.get_piece_type_at_index(index);
        let piece_safety = piece_safety(piece_type, false, black, white);
        if piece_safety == 0 {
            return 1;
        } else {
            return -1;
        }
    }

    // If black control the square through occupancy, confirm its safe
    if board.black_occupancy.occupied(index) {
        let piece_type = board.get_piece_type_at_index(index);
        let piece_safety = -1 * piece_safety(piece_type, false, white, black);
        if piece_safety == 0 {
            return -1;
        } else {
            return 1;
        }
    }

    // Else see who controls the square with the least valuable piece
    square_control(white, black) as i16
}

#[cfg(test)]
mod test {
    use crate::{
        board::{attack_and_defend_lookups::AttackAndDefendTable, board_rep::BoardRep},
        evaluation::opening::get_square_control,
        shared::board_utils::index_from_coords,
    };

    #[test]
    fn get_square_control_center_square_controlled_by_pawn_due_to_threat() {
        let board =
            BoardRep::from_fen("rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR w KQkq - 0 1".into());

        let control = get_square_control(
            index_from_coords("d5"),
            &mut AttackAndDefendTable::new(),
            board,
        );

        assert_eq!(control, 1)
    }

    #[test]
    fn get_square_control_center_square_controlled_by_white_because_threatening_unoccupied_with_lower_value_piece(
    ) {
        let board = BoardRep::from_fen(
            "rnbqkb1r/pppppppp/5n2/8/4P3/8/PPPP1PPP/RNBQKBNR w KQkq - 0 1".into(),
        );

        let control = get_square_control(
            index_from_coords("d5"),
            &mut AttackAndDefendTable::new(),
            board,
        );

        assert_eq!(control, 1)
    }

    #[test]
    fn get_square_control_center_square_controlled_by_neither_because_threatening_unoccupied_with_same_value_pieces(
    ) {
        let board = BoardRep::from_fen(
            "rnbqkbnr/pp1ppppp/2p5/8/4P3/8/PPPP1PPP/RNBQKBNR w KQkq - 0 1".into(),
        );

        let control = get_square_control(
            index_from_coords("d5"),
            &mut AttackAndDefendTable::new(),
            board,
        );

        assert_eq!(control, 0)
    }

    #[test]
    fn get_square_control_center_square_controlled_by_black_due_to_multiple_threats() {
        let board = BoardRep::from_fen(
            "rnbqkb1r/pp1ppppp/2p2n2/8/4P3/8/PPPP1PPP/RNBQKBNR w KQkq - 0 1".into(),
        );

        let control = get_square_control(
            index_from_coords("d5"),
            &mut AttackAndDefendTable::new(),
            board,
        );

        assert_eq!(control, -1)
    }

    #[test]
    fn get_square_control_center_square_controlled_by_white_due_to_occupied_plus_winning_see() {
        let board = BoardRep::from_fen(
            "r1bqkbnr/pppppppp/2n5/8/3P4/8/PPP1PPPP/RNBQKBNR w KQkq - 0 1".into(),
        );

        let control = get_square_control(
            index_from_coords("d4"),
            &mut AttackAndDefendTable::new(),
            board,
        );

        assert_eq!(control, 1)
    }

    #[test]
    fn get_square_control_center_square_controlled_by_black_because_capturing_white_piece_is_winning(
    ) {
        let board = BoardRep::from_fen(
            "r1bqkbnr/pppp1ppp/2n5/4p3/3P4/8/PPP1PPPP/RNBQKBNR w KQkq - 0 1".into(),
        );

        let control = get_square_control(
            index_from_coords("d4"),
            &mut AttackAndDefendTable::new(),
            board,
        );

        assert_eq!(control, -1)
    }
}
