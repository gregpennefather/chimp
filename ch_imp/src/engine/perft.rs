use std::time::Instant;

use crate::{match_state::game_state::GameState, r#move::Move, POSITION_TRANSPOSITION_TABLE};

pub fn perft(name: String, fen: String, counts: Vec<usize>) {
    println!("--- {name} ---");
    let origin_game_state = GameState::new(fen);
    let mut top_level_states = Vec::new();

    let start: Instant = Instant::now();
    for &m in &origin_game_state.moves {
        if m.is_black() != origin_game_state.position.black_turn {
            continue;
        }

        let new_state = origin_game_state.make(m);
        if new_state.legal() {
            top_level_states.push((m, 0, vec![new_state]));
        }
    }
    let duration = start.elapsed();
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
                for &m in &game_state.moves {
                    if m.is_black() != game_state.position.black_turn {
                        continue;
                    }
                    let new_state = game_state.make(m);
                    if new_state.legal() {
                        new_edge_states.push(new_state);
                    }
                }
            }
            depth_count += new_edge_states.len();
            top_level_state.1 = new_edge_states.len();
            top_level_state.2 = new_edge_states;
        }

        let duration = start.elapsed();
        println!("{depth}: {}/{} - {duration:?}", depth_count, counts[depth]);
        if depth_count != counts[depth] {
            print_move_counts(&top_level_states);
            return;
        }
    }
    println!(
        "Table size: {}",
        POSITION_TRANSPOSITION_TABLE.read().unwrap().len()
    )
}

fn print_move_counts(top_level_states: &Vec<(Move, usize, Vec<GameState>)>) {
    for top_level_state in top_level_states {
        println!("{}: {}", top_level_state.0.uci(), top_level_state.1);
    }
}
