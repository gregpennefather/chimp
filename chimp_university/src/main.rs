use std::mem;

use ch_imp::{
    board::{bitboard::Bitboard, position::Position},
    r#move::{move_magic_bitboards::{generate_legal_bishop_moves, MagicTable, bishop_mask_generation}, move_data::MoveData},
    shared::board_utils::index_from_coords,
};

fn main() {

    //let r = generate_legal_bishop_moves(index_from_coords("e4") as //i64, Bitboard::default());
    //println!("legal_moves: {r}");

    // println!("mask:\n{}", Bitboard::new(bishop_mask_generation(index_from_coords("f3") as i64)));

    // let mt = MagicTable::new();
    // let position = Position::new("8/1k4b1/8/8/8/5B2/4K3/8".into());
    // //let r = mt.get_bishop_attacks(index_from_coords("g7") as usize, //position.occupancy.into());
    // //println!("result {}:\n{}", index_from_coords("g7"), r.to_string());
    // let r2 = mt.get_bishop_attacks(index_from_coords("f3") as usize, position.occupancy.into());
    // println!("result {}:\n{}", index_from_coords("f3"), r2.to_string());

    let move_data = MoveData::new();
    println!("{}", Bitboard::new(move_data.king_moves[index_from_coords("a8") as usize]))
}
