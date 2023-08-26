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
    shared::board_utils::index_from_coords,
};

fn main() {
    // let magic_table = MagicTable::new();
    // //println!("{}", Bitboard::new(magic_table.get_bishop_attacks(4, 18446462598732906495)));
    // //generate_blocker_patterns(rook_mask_generation(0));

    // let move_data = MoveData::new();
    // let game_state = GameState::new("rnbqkbnr/ppp1pppp/8/3p4/8/P7/1PPPPPPP/RNBQKBNR w KQkq - 0 2".into// ());
    // let moves = game_state.generate_moves(&move_data);
    // println!("{moves:?}");
    // for m in &moves {
    //     println!("{}: 1",m.uci())
    // }
    // println!("{}", moves.len());
    // println!("{}", Bitboard::new(magic_table.get_bishop_attacks(index_from_coords("f4") as usize, game_state.position.occupancy.into())));
    // println!("{}",index_from_coords("f4"));

    perft(
      "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1".into(),
      vec![20, 400, 8902, 197281],
    )

    //perft(
    //    "rnbqkbnr/pppppppp/8/8/8/P7/1PPPPPPP/RNBQKBNR b KQkq - 0 1".into(),
    //    vec![20, 380, 8457],
    //  )

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
