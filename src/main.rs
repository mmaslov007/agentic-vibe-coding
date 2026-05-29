mod collision;
mod map;
mod player;

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
                title: "Bevy FPS Dust Blockout".into(),
                resolution: WindowResolution::new(1280, 720),
                present_mode: PresentMode::AutoVsync,
                ..default()
            }),
            primary_cursor_options: Some(CursorOptions {
                grab_mode: CursorGrabMode::Locked,
                visible: false,
                ..default()
            }),
            ..default()
        }))
        .add_plugins((map::MapPlugin, player::PlayerPlugin))
        .run();
}
