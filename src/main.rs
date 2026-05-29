mod audio_fx;
mod collision;
mod combat;
mod game_ui;
mod map;
mod player;
mod zombies;

use bevy::prelude::*;
use bevy::window::{CursorGrabMode, CursorOptions, PresentMode, WindowResolution};

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::srgb(0.62, 0.72, 0.86)))
        .insert_resource(GlobalAmbientLight {
            color: Color::srgb(0.95, 0.88, 0.74),
            brightness: 850.0,
            ..default()
        })
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Blox-Z".into(),
                resolution: WindowResolution::new(1280, 720),
                present_mode: PresentMode::AutoVsync,
                ..default()
            }),
            primary_cursor_options: Some(CursorOptions {
                grab_mode: CursorGrabMode::None,
                visible: true,
                ..default()
            }),
            ..default()
        }))
        .add_plugins((
            game_ui::GameUiPlugin,
            audio_fx::AudioFxPlugin,
            map::MapPlugin,
            player::PlayerPlugin,
            combat::CombatPlugin,
            zombies::ZombiePlugin,
        ))
        .run();
}
