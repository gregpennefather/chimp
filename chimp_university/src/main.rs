use std::mem;

use ch_imp::{
    board::{bitboard::Bitboard, position::Position},
    engine::perft::perft,
    match_state::game_state::{self, GameState},
    r#move::{
        move_data::MoveData,
        move_magic_bitboards::{
            find_bishop_magics, find_rook_magics, generate_blocker_patterns, rook_mask_generation,
            MagicTable, BISHOP_LEFT_SHIFT_BITS, ROOK_LEFT_SHIFT_BITS,
        },
    },
    search::zorb_set::ZorbSet,
    shared::{board_utils::index_from_coords, piece_type::PieceType},
};

fn main() {
    // let magic_table = MagicTable::new();
    // //println!("{}", Bitboard::new(magic_table.get_bishop_attacks(4, 18446462598732906495)));
    // //generate_blocker_patterns(rook_mask_generation(0));

    //let move_data = MoveData::new();
    //let game_state = GameState::new("rnbqkbnr/pppppppp/8/8/P7/8/1PPPPPPP/RNBQKBNR b KQkq - 0 1".into());
    //let moves = move_data.generate_position_moves(game_state.position, 54, true, 23, true, true, true, true);
    //println!("{moves:?}");
    //for m in &moves {
    //    println!("{}: 1",m.uci())
    //}
    // println!("{}", moves.len());
    // println!("{}", Bitboard::new(magic_table.get_bishop_attacks(index_from_coords("f4") as usize, game_state.position.occupancy.into())));
    // println!("{}",index_from_coords("f4"));

    perft(
        "Perft".into(),
        "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1".into(),
        vec![20, 400, 8902, 197281],
    );

    perft(
        "Kiwipete Perft".into(),
        "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1".into(),
        vec![48, 2039, 97862, 4085603],
    );


    //let position =
    //    Position::from_fen("rnbqkbnr/pppp1ppp/4p3/8/8/BP6/P1PPPPPP/RN1QKBNR b KQkq - 1 2".into());
    //for mi in 0..64 {
    //    let m = position.black_moves[mi];
    //    if m.from() == m.to() && m.from() == 0 {
    //        break;
    //    }
    //    if (m.piece_type() == PieceType::King) {
    //        println!("pre-move");
    //    }
    //    let applied = position.make(m);
    //    if (m.piece_type() == PieceType::King) {
    //        println!("kingpos:\n{}", applied.king_bitboard);
    //        println!("wthreat:\n{}", Bitboard::new(applied.white_threatboard));
    //    }
    //    println!("{m:?}: {}", applied.legal());
    //}

    // perft(
    //     "rnbqkbnr/ppp1pppp/3p4/8/4P3/8/PPPP1PPP/RNBQKBNR w KQkq - 0 2".into(),
    //     vec![30, 781, 24086],
    //   )

    // let zorb_set = ZorbSet::new();
    // println!("{zorb_set:?}");

    // let mut magics = [0; 64];
    // for i in 0..64usize {
    //     magics[i] = find_rook_magics(i as i64, ROOK_LEFT_SHIFT_BITS[i]);
    // }
    // println!("{magics:?}");

    // let mut magics = [0; 64];
    // for i in 0..64usize {
    //     magics[i] = find_bishop_magics(i as i64, BISHOP_LEFT_SHIFT_BITS[i]);
    // }
    // println!("{magics:?}");
}
