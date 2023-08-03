// MSB: 1 = white 0 = black
// 0: None
// 1: Pawn
// 10: Knight
// 11: Bishop
// 100: Rook
// 101: Queen
// 110: King

// Example: Black King = {0}{110}
// White Pawn = {1}{001}

// Bit bits per square
// 8x8 squares => 8x8x4 = 256;

use self::{
    position::*,
    r#move::Move,
    utils::{check_board_position, is_white_piece},
};
use super::constants::*;
use crate::chess::board::piece::*;

mod r#move;
mod piece;
mod position;
mod utils;

// Bitboard, Pieces, Game flags, Half Move Count, Full Move Count
// Flags: lsb->msb WhiteTurn:WhiteQueenCastling:WhiteKingCastling:BlackQueenCastling:BlackKingCastling:EPRank:EPRank:EPRank
pub struct BoardState(pub u64, pub u128, pub u8, pub usize, pub usize);

impl BoardState {
    pub fn get_piece(&self, piece_index: usize) -> u8 {
        let pieces = self.1;
        let sub = pieces >> (4 * piece_index) & (COLOURED_PIECE_MASK as u128);

        sub as u8
    }

    pub fn from_fen(fen: String) -> BoardState {
        let mut positions: u64 = 0;
        let mut pieces: u128 = 0;
        let mut flags: u8 = 0;

        let mut file: i64 = 7;
        let mut rank: u64 = 0;
        let mut piece_index: u16 = 0;

        let mut i: usize = 0;
        // Pieces
        while i < fen.len() {
            let char: char = fen.chars().nth(i).unwrap();
            i += 1;

            if char.is_ascii_digit() {
                let digit = char as i32 - 0x30;
                rank += digit as u64;
                continue;
            }

            if char == '/' {
                rank = 0;
                file -= 1;
                continue;
            }

            if char == ' ' {
                break;
            }

            let piece_position: u64 = 1 << ((file * 8) as u64 + rank);

            positions = positions + piece_position;
            rank = rank + 1;

            let piece: u8 = match char {
                'P' => PAWN_INDEX,
                'p' => PAWN_INDEX | BLACK_MASK,
                'B' => BISHOP_INDEX,
                'b' => BISHOP_INDEX | BLACK_MASK,
                'N' => KNIGHT_INDEX,
                'n' => KNIGHT_INDEX | BLACK_MASK,
                'R' => ROOK_INDEX,
                'r' => ROOK_INDEX | BLACK_MASK,
                'Q' => QUEEN_INDEX,
                'q' => QUEEN_INDEX | BLACK_MASK,
                'K' => KING_INDEX,
                'k' => KING_INDEX | BLACK_MASK,
                _ => 0,
            };

            let piece_u128: u128 = (piece as u128) << (4 * piece_index);
            pieces = pieces | piece_u128;
            piece_index += 1;
        }

        // Turn
        let white_turn = if fen.chars().nth(i).unwrap() == 'w' { 1 } else { 0 };
        flags += white_turn;
        i += 2;

        // Castling
        let mut can_castle: u8 = 0;
        while let c =  fen.chars().nth(i).unwrap() {
            i += 1;
            match c {
                'K' => can_castle += 1,
                'Q' => can_castle += 2,
                'k' => can_castle += 4,
                'q' => can_castle += 8,
                ' ' => { i -= 1; break; }
                _ => break
            }
        }
        flags += can_castle << 1;
        i += 1;

        // En Passant
        let ep_char = fen.chars().nth(i).unwrap().to_ascii_uppercase();
        println!("ep_char {ep_char}");
        if ep_char != '-' {
            let rank = RANKS.find(ep_char).unwrap() as u8;
            println!("rank found {rank} aka {rank:b} with current flags {flags:b}");
            flags += rank << 5;
            i+=1;
        }
        i += 2;
        println!("Flags: {flags:b}");

        // Half moves
        let remaining_fen = &fen[i..];
        let next_space = remaining_fen.find(' ').unwrap();
        let half_moves_str = &remaining_fen[0..next_space];
        println!("half_moves_str {half_moves_str}");
        let half_moves = half_moves_str.parse::<usize>().unwrap();


        // Full moves
        let full_remaining_fen = &remaining_fen[next_space+1..];
        let next_space = match full_remaining_fen.find(' ') {
            Some(pos) => pos,
            _ => full_remaining_fen.len()
        };
        let full_moves_str = &full_remaining_fen[0..next_space];
        println!("full_moves_str {full_moves_str}");
        let full_moves = full_moves_str.parse::<usize>().unwrap();


        BoardState(positions, pieces, flags, half_moves, full_moves)
    }
}

pub struct Board {
    state: BoardState,
    white_bitboard: u64,
    black_bitboard: u64,
    pub pieces: [Piece; 32],
}

impl Board {
    pub fn new(state: BoardState) -> Board {
        let mut pieces: [Piece; 32] = [Piece::default(); 32];
        let mut white_bitboard: u64 = 0;
        let mut black_bitboard: u64 = 0;
        let mut piece_index: usize = 0;
        for y in 0..8 {
            for rank in 0..8 {
                let file = 7 - y;
                if check_board_position(state.0, rank, file) {
                    let code = state.get_piece(piece_index);
                    let piece = Piece {
                        pos: Position { file, rank },
                        code,
                    };
                    if is_white_piece(code) {
                        white_bitboard += 1 << ((file * 8) as u64 + rank as u64);
                    } else {
                        black_bitboard += 1 << ((file * 8) as u64 + rank as u64);
                    }
                    pieces[piece_index] = piece;
                    piece_index += 1;
                }
            }
        }

        print!(
            "num pieces {}, whitebb {}, blackbb {}",
            pieces.len(),
            white_bitboard,
            black_bitboard
        );

        Self {
            state,
            white_bitboard,
            black_bitboard,
            pieces,
        }
    }

    pub fn get_moves(&self, white_move: bool) -> Vec<Move> {
        let mut moves = Vec::new();
        let friendly_bitboard = if white_move {
            self.white_bitboard
        } else {
            self.black_bitboard
        };
        let opponent_bitboard = if white_move {
            self.black_bitboard
        } else {
            self.white_bitboard
        };
        for piece in self.pieces {
            if is_white_piece(piece.code) == white_move {
                let new_moves = match piece.code & PIECE_MASK {
                    PAWN_INDEX => get_pawn_moves(
                        piece.code,
                        piece.pos,
                        Move::default(),
                        true,
                        friendly_bitboard,
                        opponent_bitboard,
                    ),
                    KNIGHT_INDEX => get_knight_moves(
                        piece.code,
                        piece.pos,
                        friendly_bitboard,
                        opponent_bitboard,
                    ),
                    BISHOP_INDEX => get_bishop_moves(
                        piece.code,
                        piece.pos,
                        friendly_bitboard,
                        opponent_bitboard,
                    ),
                    ROOK_INDEX => {
                        get_rook_moves(piece.code, piece.pos, friendly_bitboard, opponent_bitboard)
                    }
                    QUEEN_INDEX => {
                        get_queen_moves(piece.code, piece.pos, friendly_bitboard, opponent_bitboard)
                    }
                    KING_INDEX => {
                        get_king_moves(piece.code, piece.pos, friendly_bitboard, opponent_bitboard)
                    }
                    _ => panic!("Unknown {piece:?}!"),
                };
                println!("{} new moves for {piece:?}", new_moves.len());
                moves.extend(new_moves);
            }
        }
        moves
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_fen_white_move() {
        let state =
            BoardState::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1".into());
        let flag = state.2 & 1;
        assert_eq!(flag, 1);
    }

    #[test]
    fn from_fen_black_move() {
        let state =
            BoardState::from_fen("rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1".into());
        let flag = state.2 & 1;
        assert_eq!(flag, 0);
    }

    #[test]
    fn from_white_king_both_castling_available() {
        let state =
            BoardState::from_fen("rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1".into());
        let white_kingside = (state.2 >> 1) & 1;
        assert_eq!(white_kingside, 1);
        let white_queenside = (state.2 >> 2) & 1;
        assert_eq!(white_queenside, 1);
    }

    #[test]
    fn from_black_king_both_castling_available() {
        let state =
            BoardState::from_fen("rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1".into());
        let black_kingside = (state.2 >> 3) & 1;
        assert_eq!(black_kingside, 1);
        let black_queenside = (state.2 >> 4) & 1;
        assert_eq!(black_queenside, 1);
    }

    #[test]
    fn from_white_can_queen_castle_black_can_king() {
        let state =
            BoardState::from_fen("rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b Qk e3 0 1".into());
        assert_eq!(state.2 >> 1 & 0b1111 , 0b0110); // 0110
    }

    #[test]
    fn from_no_one_can_castle() {
        let state =
            BoardState::from_fen("rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b - - 0 1".into());
        assert_eq!(state.2, 0);
    }

    #[test]
    fn from_fen_no_en_passant() {
        let state =
            BoardState::from_fen("rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b - - 0 1".into());
        let ep_rank = state.2 >> 5 & 5;
        assert_eq!(ep_rank, 0);
    }

    #[test]
    fn from_fen_e3_en_passant() {
        let state =
            BoardState::from_fen("rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b - e3 0 1".into());
        let ep_rank = state.2 >> 5 & 0b111;
        assert_eq!(ep_rank, 4); // 4 = e rank
    }

    #[test]
    fn from_fen_h3_en_passant() {
        let state =
            BoardState::from_fen("rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b - h3 0 1".into());
        let ep_rank = state.2 >> 5 & 0b111;
        assert_eq!(ep_rank, 7); // 7 = h rank
    }

    #[test]
    fn from_fen_half_moves() {
        let state =
            BoardState::from_fen("rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b - h3 23 1".into());
        assert_eq!(state.3, 23);
    }

    #[test]
    fn from_fen_no_half_moves() {
        let state =
            BoardState::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1".into());
        assert_eq!(state.3, 0);
    }

    #[test]
    fn from_fen_initial_full_moves() {
        let state =
            BoardState::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1".into());
        assert_eq!(state.4, 1);
    }

    #[test]
    fn from_fen_fifty_full_moves() {
        let state =
            BoardState::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 50".into());
        assert_eq!(state.4, 50);
    }


}
