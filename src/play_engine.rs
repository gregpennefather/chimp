use bevy::prelude::*;
use rand::Rng;
use crate::Board;

pub const SQUARE_SIZE: f32 = 64.;

#[derive(Resource)]
pub struct BoardRes(pub Board);

pub fn setup_engine(board: Res<BoardRes>, mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2dBundle::default());

    let c_w = Color::rgb(1., 1., 1.);
    let c_b = Color::rgb(0.471, 0.318, 0.664);
    let x_offset = -(SQUARE_SIZE * 4.) + (SQUARE_SIZE / 2.);
    let y_offset = -(SQUARE_SIZE * 4.) + (SQUARE_SIZE / 2.);

    for x in 0..8 {
        for y in 0..8 {
            let colour = if (x + y) % 2 == 0 { c_b } else { c_w };
            commands.spawn(SpriteBundle {
                sprite: Sprite {
                    color: colour,
                    custom_size: Some(Vec2::new(SQUARE_SIZE, SQUARE_SIZE)),
                    ..default()
                },
                transform: Transform::from_translation(Vec3::new(
                    x_offset + SQUARE_SIZE * x as f32,
                    y_offset + SQUARE_SIZE * y as f32,
                    0.,
                )),
                ..default()
            });
        }
    }

    let moves = board.0.get_moves(true);

    println!("White moves: {}", moves.len());

    let rand: usize = rand::thread_rng().gen_range(0..moves.len());

    let new_state = board.0.apply_move(&moves[rand]);
    let bb = Board::new(new_state);

    for piece in bb.pieces {
        if !piece.empty() {
            commands.spawn(SpriteBundle {
                texture: asset_server.load(get_sprite_file_path(piece.code)),
                sprite: Sprite {
                    custom_size: Some(Vec2::new(SQUARE_SIZE, SQUARE_SIZE)),
                    ..default()
                },
                transform: Transform::from_translation(position_to_translation(
                    piece.pos.file,
                    piece.pos.rank,
                    x_offset,
                    y_offset,
                )),
                ..default()
            });
        }
    }
}

fn position_to_translation(file: i8, rank: i8, x_offset: f32, y_offset: f32) -> Vec3 {
    Vec3::new(
        x_offset + SQUARE_SIZE * (rank as f32),
        y_offset + SQUARE_SIZE * (file as f32),
        0.,
    )
}

fn get_sprite_file_path(code: u8) -> String {
    let c = if (code >> 3) > 0 { "b" } else { "w" };
    let p = match code & crate::chess::constants::PIECE_MASK {
        crate::chess::constants::PAWN_INDEX => "_pawn_png_shadow_128px.png".to_string(),
        crate::chess::constants::KNIGHT_INDEX => "_knight_png_shadow_128px.png".to_string(),
        crate::chess::constants::BISHOP_INDEX => "_bishop_png_shadow_128px.png".to_string(),
        crate::chess::constants::ROOK_INDEX => "_rook_png_shadow_128px.png".to_string(),
        crate::chess::constants::QUEEN_INDEX => "_queen_png_shadow_128px.png".to_string(),
        crate::chess::constants::KING_INDEX => "_king_png_shadow_128px.png".to_string(),
        _ => panic!("Unknown piece code {}", code),
    };
    format!("{c}{p}")
}
