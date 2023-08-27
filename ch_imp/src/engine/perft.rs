use std::time::Instant;

use crate::{match_state::game_state::GameState, r#move::{move_data::MoveData, Move}};

pub fn perft(fen: String, counts: Vec<usize>) {
    println!("Perft:");
    let move_data = MoveData::new();
    let origin_game_state = GameState::new(fen);
    let mut top_level_states = Vec::new();

    let moves = origin_game_state.generate_moves(&move_data);
    let start: Instant = Instant::now();
    for m in moves {
        top_level_states.push((m, 0, vec![origin_game_state.make(m)]));
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
            let mut count = 0;
            let mut new_edge_states = Vec::new();
            for game_state in &top_level_state.2 {
                let moves = game_state.generate_moves(&move_data);
                if top_level_state.0.uci().eq("a2a4") && depth == 2 {
                    println!("depth {depth}: {} => {}", game_state.to_fen(), moves.len());
                }
                count += moves.len();
                for m in moves {
                    new_edge_states.push(game_state.make(m));
                }
            }
            depth_count += count;
            top_level_state.1 = count;
            top_level_state.2 = new_edge_states;
        }

        let duration = start.elapsed();
        println!("{depth}: {}/{} - {duration:?}", depth_count, counts[depth]);
        if depth_count != counts[depth] {
            print_move_counts(&top_level_states);
            return;
        }
    }
}

fn print_move_counts(top_level_states: &Vec<(Move, usize, Vec<GameState>)>) {
    for top_level_state in top_level_states {
        println!("{}: {}", top_level_state.0.uci(), top_level_state.1);
    }
}
