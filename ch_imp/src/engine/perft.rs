use std::time::Instant;

use log::info;

use crate::{
    match_state::game_state::GameState,
    r#move::{move_generation::generate_moves_for_board, Move},
};

pub fn perft(name: String, fen: String, counts: Vec<usize>) {
    info!("--- {name} ---");
    println!("--- {name} ---");
    let origin_game_state = GameState::new(fen);
    let mut top_level_states = Vec::new();

    let start: Instant = Instant::now();
    for m in generate_moves_for_board(origin_game_state.position.board) {
        if m.is_empty() {
            break;
        }
        if m.is_black() != origin_game_state.position.board.black_turn {
            continue;
        }

        match origin_game_state.make(m) {
            Some(new_state) => top_level_states.push((m, 0, vec![new_state])),
            None => continue,
        }
    }
    let duration = start.elapsed();
    info!("0: {}/{} - {duration:?}", top_level_states.len(), counts[0]);
    println!("0: {}/{} - {duration:?}", top_level_states.len(), counts[0]);
    if top_level_states.len() != counts[0] {
        print_move_counts(&top_level_states);
        return;
    }

    for depth in 1..counts.len() {
        let start: Instant = Instant::now();
        let mut depth_count = 0;

        for top_level_state in top_level_states.iter_mut() {
            let mut new_edge_states = Vec::new();
            for game_state in &top_level_state.2 {
                for m in generate_moves_for_board(game_state.position.board) {
                    if m.is_empty() {
                        break;
                    }
                    if m.is_black() != game_state.position.board.black_turn {
                        continue;
                    }
                    match game_state.make(m) {
                        Some(new_state) => new_edge_states.push(new_state),
                        None => continue,
                    }
                }
            }
            depth_count += new_edge_states.len();
            top_level_state.1 = new_edge_states.len();
            top_level_state.2 = new_edge_states;
        }

        let duration = start.elapsed();
        info!("{depth}: {}/{} - {duration:?}", depth_count, counts[depth]);
        println!("{depth}: {}/{} - {duration:?}", depth_count, counts[depth]);
        if depth_count != counts[depth] {
            print_move_counts(&top_level_states);
            return;
        }
    }
}

fn print_move_counts(top_level_states: &Vec<(Move, usize, Vec<GameState>)>) {
    for top_level_state in top_level_states {
        info!("{}: {}", top_level_state.0.uci(), top_level_state.1);
        println!("{}: {}", top_level_state.0.uci(), top_level_state.1);
    }
}
