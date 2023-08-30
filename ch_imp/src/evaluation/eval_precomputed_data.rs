pub(super) type PieceValues = [i32; 6];

// --- Values ---
// Reward
pub(super) static PAWN_MATERIAL: i32 = 100;
pub(super) static KNIGHT_MATERIAL: i32 = 300;
pub(super) static BISHOP_MATERIAL: i32 = 300;
pub(super) static ROOK_MATERIAL: i32 = 500;
pub(super) static QUEEN_MATERIAL: i32 = 900;
pub(super) static KING_MATERIAL: i32 = 0;

pub(super) static MATERIAL_VALUES: PieceValues = [
    PAWN_MATERIAL,
    KNIGHT_MATERIAL,
    BISHOP_MATERIAL,
    ROOK_MATERIAL,
    QUEEN_MATERIAL,
    KING_MATERIAL,
];
pub(super) static THREATENED_PIECE_VALUE: PieceValues = [
    PAWN_MATERIAL / 4,
    KNIGHT_MATERIAL / 4,
    BISHOP_MATERIAL / 4,
    ROOK_MATERIAL / 4,
    QUEEN_MATERIAL / 4,
    100,
];
pub(super) static HANGING_PIECE_VALUE: PieceValues = [
    PAWN_MATERIAL / 2,
    KNIGHT_MATERIAL / 2,
    BISHOP_MATERIAL / 2,
    ROOK_MATERIAL / 2,
    QUEEN_MATERIAL / 2,
    0,
];
pub(super) static PHASE_MATERIAL_VALUES: PieceValues = [
    0,
    1,
    1,
    2,
    4,
    0,
];

// AGGREGATES
pub(super) static AGGREGATE_MOBILITY_AREA_REWARD: i32 = 1;
pub(super) static AGGREGATE_THREAT_AREA_REWARD: i32 = 2;

// Holding
pub(super) static PAWN_HOLDING_CENTER_REWARD: i32 = 12;
// Threatening
pub(super) static THREATENING_CENTER_REWARD: i32 = AGGREGATE_THREAT_AREA_REWARD * 5;

// Misc
pub(super) static CASTLING_REWARD: i32 = 75;

// Punish
pub(super) static KNIGHT_ON_EDGE: i32 = -10;

// --- Positions ---
// General
pub(super) static EDGE_SQUARES: u64 = 18411139144890810879;

// Early
pub(super) static CENTER_FOUR: u64 = 103481868288;
