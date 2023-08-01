use bevy::prelude::*;
use chess::board::{Board, BoardState};
use play_engine::{BoardRes, SQUARE_SIZE};

mod chess;
mod play_engine;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "chimp".into(),
                resolution: (SQUARE_SIZE * 8., SQUARE_SIZE * 8.).into(),
                ..default()
            }),
            ..Default::default()
        }))
        .insert_resource(BoardRes(Board::new(BoardState::from_fen(
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w - - 0 1".into(),
        ))))
        .add_systems(Startup, play_engine::setup_engine)
        .run();
}
