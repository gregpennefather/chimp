use bevy::prelude::*;
use chess::board::{Board, BoardState};
use play_engine::{BoardRes, BoardStateChange, SQUARE_SIZE};

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
            "4k3/4p3/8/8/8/8/3KP3/8 w - - 0 1".into(), // "4k3/4p3/8/8/8/8/3KP3/8 w - - 0 1"
        ))))
        .add_systems(Startup, play_engine::setup_engine)
        .add_event::<BoardStateChange>()
        .add_systems(
            Update,
            play_engine::draw_board,
        ).add_systems(Update, play_engine::keyboard_input)
        .run();
}
