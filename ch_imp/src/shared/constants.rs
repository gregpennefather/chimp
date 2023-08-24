// https://www.chessprogramming.org/Encoding_Moves
pub const MF_DOUBLE_PAWN_PUSH: u16 = 0b0001;
pub const MF_KING_CASTLING: u16 = 0b0010;
pub const MF_QUEEN_CASTLING: u16 = 0b0011;
pub const MF_CAPTURE: u16 = 0b0100;
pub const MF_EP_CAPTURE: u16 = 0b0101;
pub const MF_PROMOTION: u16 = 0b1000;
pub const MF_KNIGHT_PROMOTION: u16 = 0b1000;
pub const MF_KNIGHT_CAPTURE_PROMOTION: u16 = 0b1100;
pub const MF_BISHOP_PROMOTION: u16 = 0b1001;
pub const MF_BISHOP_CAPTURE_PROMOTION: u16 = 0b1101;
pub const MF_ROOK_PROMOTION: u16 = 0b1010;
pub const MF_ROOK_CAPTURE_PROMOTION: u16 = 0b1110;
pub const MF_QUEEN_PROMOTION: u16 = 0b1011;
pub const MF_QUEEN_CAPTURE_PROMOTION: u16 = 0b1111;