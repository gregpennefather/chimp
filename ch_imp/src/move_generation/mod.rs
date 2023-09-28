use log::info;

use crate::{
    board::{
        attack_and_defend_lookups::{AttackAndDefendTable, AttackedBy},
        bitboard::Bitboard,
        board_rep::BoardRep,
        king_position_analysis::{KingPositionAnalysis, ThreatRaycastCollision, ThreatType},
        see::piece_safety,
    },
    r#move::Move,
    shared::piece_type::{self},
};

mod king;
mod knight;
pub mod legal_move;
pub(crate) mod pawn;
pub(crate) mod sliding;
mod tests;

fn generate_moves(
    king_analysis: &KingPositionAnalysis,
    opponent_king_analysis: &KingPositionAnalysis,
    board: BoardRep,
) -> Vec<Move> {
    let mut friendly_occupancy = if board.black_turn {
        board.black_occupancy
    } else {
        board.white_occupancy
    };
    let opponent_occupancy = if !board.black_turn {
        board.black_occupancy
    } else {
        board.white_occupancy
    };

    let (king_pos, king_side_castling, queen_side_castling) = if board.black_turn {
        (
            board.black_king_position,
            board.black_king_side_castling,
            board.black_queen_side_castling,
        )
    } else {
        (
            board.white_king_position,
            board.white_king_side_castling,
            board.white_queen_side_castling,
        )
    };

    let mut ad_table = AttackAndDefendTable::new();

    let mut moves = king::generate_king_moves(
        king_pos,
        opponent_occupancy,
        board.occupancy,
        &king_analysis,
        board.black_turn,
        king_side_castling,
        queen_side_castling,
        board,
        &mut ad_table,
    );

    // In the event of double king check we can only avoid check by moving the king
    if king_analysis.double_check {
        return moves;
    }

    // Check if any of our pieces moving is a reveal attack
    let reveal_attacks: Vec<ThreatRaycastCollision> = opponent_king_analysis
        .pins
        .clone()
        .into_iter()
        .filter(|p| p.reveal_attack)
        .collect();

    while friendly_occupancy != 0 {
        let piece_position = friendly_occupancy.trailing_zeros() as u8;
        moves.extend(generate_index_moves(
            board,
            &mut ad_table,
            piece_position,
            king_analysis,
            &reveal_attacks,
        ));
        friendly_occupancy ^= 1 << piece_position;
    }

    moves.sort();
    moves
}

pub fn generate_moves_for_board(board: BoardRep) -> Vec<Move> {
    let (king_analysis, opponent_king_analysis) = if board.black_turn {
        (
            board.get_black_king_analysis(),
            board.get_white_king_analysis(),
        )
    } else {
        (
            board.get_white_king_analysis(),
            board.get_black_king_analysis(),
        )
    };

    generate_moves(&king_analysis, &opponent_king_analysis, board)
}

fn generate_index_moves(
    board: BoardRep,
    ad_table: &mut AttackAndDefendTable,
    index: u8,
    king_analysis: &KingPositionAnalysis,
    reveal_attacks: &Vec<ThreatRaycastCollision>,
) -> Vec<Move> {
    let piece_type = board.get_piece_type_at_index(index);
    let opponent_occupancy = if board.black_turn {
        board.white_occupancy
    } else {
        board.black_occupancy
    };
    let pin = Option::<&ThreatRaycastCollision>::copied(
        king_analysis.pins.iter().find(|p| p.at == index),
    );
    // If we're pinned but the king is also threatened we can't help
    if pin != None && king_analysis.threat_source != None {
        return vec![];
    }

    // If we're a reveal attack we can ignore opponent response in a SEE
    let reveal_attack =
        Option::<&ThreatRaycastCollision>::copied(reveal_attacks.iter().find(|p| p.at == index));

    match piece_type {
        piece_type::PieceType::Pawn => pawn::generate_pawn_moves(
            board,
            ad_table,
            index,
            opponent_occupancy,
            king_analysis.threat_source,
            pin,
            reveal_attack,
        ),
        piece_type::PieceType::Knight => match pin {
            Some(_) => vec![],
            None => knight::generate_knight_moves(
                index,
                opponent_occupancy,
                board.occupancy,
                king_analysis.threat_source,
                board,
                ad_table,
                reveal_attack,
            ),
        },
        piece_type::PieceType::Bishop => {
            match pin {
                Some(pin) => {
                    if pin.threat_type != ThreatType::DiagonalSlide {
                        return vec![];
                    } else {
                    }
                }
                None => {}
            }
            sliding::bishop::generate_bishop_moves(
                index,
                board,
                ad_table,
                opponent_occupancy,
                board.occupancy,
                king_analysis.threat_source,
                pin,
                reveal_attack,
            )
        }
        piece_type::PieceType::Rook => {
            match pin {
                Some(pin) => {
                    if pin.threat_type != ThreatType::OrthogonalSlide {
                        return vec![];
                    } else {
                    }
                }
                None => {}
            }
            sliding::rook::generate_rook_moves(
                index,
                board,
                ad_table,
                opponent_occupancy,
                board.occupancy,
                king_analysis.threat_source,
                pin,
                reveal_attack,
            )
        }
        piece_type::PieceType::Queen => sliding::queen::generate_queen_moves(
            index,
            board,
            ad_table,
            opponent_occupancy,
            board.occupancy,
            king_analysis.threat_source,
            pin,
            reveal_attack,
        ),
        piece_type::PieceType::King => Vec::new(),
        _ => panic!(
            "Unexpected piece {piece_type:?} at position {index} : {}",
            board.to_fen()
        ),
    }
}

fn moveboard_to_moves(
    from_index: u8,
    piece_type: piece_type::PieceType,
    moveboard: u64,
    opponent_occupancy: u64,
    occupancy: u64,
    board: BoardRep,
    ad_table: &mut AttackAndDefendTable,
    reveal_attack: Option<ThreatRaycastCollision>,
) -> Vec<Move> {
    let mut generated_moves = Vec::new();
    let mut m_b = moveboard;
    while m_b != 0 {
        let lsb = m_b.trailing_zeros() as u8;
        let friendly = ad_table.get_attacked_by(lsb, board, board.black_turn);
        let opponent = match reveal_attack {
            None => ad_table.get_attacked_by(lsb, board, !board.black_turn),
            Some(ra) => {
                if !ra.threat_ray_mask.occupied(lsb) {
                    AttackedBy::default()
                } else {
                    ad_table.get_attacked_by(lsb, board, !board.black_turn)
                }
            }
        };

        if opponent_occupancy.occupied(lsb) {
            let attacked_piece_type = board.get_piece_type_at_index(lsb);

            generated_moves.push(Move::capture_move(
                from_index,
                lsb,
                piece_type,
                attacked_piece_type,
                board.black_turn,
                friendly,
                opponent,
            ));
        } else if !occupancy.occupied(lsb) {
            generated_moves.push(Move::new(
                from_index,
                lsb,
                0b0,
                piece_type,
                board.black_turn,
                piece_safety(piece_type, true, opponent, friendly),
            ));
        };
        m_b ^= 1 << lsb;
    }

    generated_moves
}
