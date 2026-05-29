use bevy::input::mouse::AccumulatedMouseMotion;
use bevy::prelude::*;
use bevy::window::{CursorGrabMode, CursorOptions, PrimaryWindow};

use crate::audio_fx::{SoundEffects, play_sound};
use crate::collision::move_circle_through_aabbs;
use crate::game_ui::GameMode;
use crate::map::MapColliders;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<FootstepClock>()
            .add_systems(Startup, spawn_player)
            .add_systems(
                Update,
                (
                    toggle_cursor_capture,
                    update_look,
                    move_player,
                    play_footsteps,
                )
                    .chain()
                    .run_if(in_state(GameMode::Playing)),
            );
    }
}

const PLAYER_EYE_HEIGHT: f32 = 1.65;
const PLAYER_RADIUS: f32 = 0.35;
const WALK_SPEED: f32 = 5.2;
const SPRINT_SPEED: f32 = 8.0;
const MOUSE_SENSITIVITY: f32 = 0.0024;

#[derive(Component)]
pub struct PlayerCamera;

#[derive(Component)]
struct PlayerController {
    yaw: f32,
    pitch: f32,
}

#[derive(Resource, Default)]
struct FootstepClock {
    remaining: f32,
}

fn spawn_player(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, PLAYER_EYE_HEIGHT, 30.0)
            .looking_at(Vec3::new(0.0, PLAYER_EYE_HEIGHT, 0.0), Vec3::Y),
        PlayerCamera,
        PlayerController {
            yaw: 0.0,
            pitch: 0.0,
        },
    ));
}

fn toggle_cursor_capture(
    keys: Res<ButtonInput<KeyCode>>,
    buttons: Res<ButtonInput<MouseButton>>,
    mut cursor_options: Query<&mut CursorOptions, With<PrimaryWindow>>,
) {
    let Ok(mut cursor_options) = cursor_options.single_mut() else {
        return;
    };

    if keys.just_pressed(KeyCode::Escape) {
        cursor_options.visible = true;
        cursor_options.grab_mode = CursorGrabMode::None;
    }

    if buttons.just_pressed(MouseButton::Left) {
        cursor_options.visible = false;
        cursor_options.grab_mode = CursorGrabMode::Locked;
    }
}

fn update_look(
    mouse_motion: Res<AccumulatedMouseMotion>,
    cursor_options: Query<&CursorOptions, With<PrimaryWindow>>,
    mut player: Query<(&mut Transform, &mut PlayerController)>,
) {
    let Ok(cursor_options) = cursor_options.single() else {
        return;
    };

    if cursor_options.visible {
        return;
    }

    let Ok((mut transform, mut controller)) = player.single_mut() else {
        return;
    };

    controller.yaw -= mouse_motion.delta.x * MOUSE_SENSITIVITY;
    controller.pitch =
        (controller.pitch - mouse_motion.delta.y * MOUSE_SENSITIVITY).clamp(-1.45, 1.45);

    transform.rotation =
        Quat::from_rotation_y(controller.yaw) * Quat::from_rotation_x(controller.pitch);
}

fn move_player(
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    colliders: Res<MapColliders>,
    mut player: Query<(&mut Transform, &PlayerController)>,
) {
    let Ok((mut transform, controller)) = player.single_mut() else {
        return;
    };

    let mut input = Vec3::ZERO;
    let yaw_rotation = Quat::from_rotation_y(controller.yaw);
    let forward = yaw_rotation * Vec3::NEG_Z;
    let right = yaw_rotation * Vec3::X;

    if keys.pressed(KeyCode::KeyW) {
        input += forward;
    }
    if keys.pressed(KeyCode::KeyS) {
        input -= forward;
    }
    if keys.pressed(KeyCode::KeyD) {
        input += right;
    }
    if keys.pressed(KeyCode::KeyA) {
        input -= right;
    }

    if input.length_squared() <= f32::EPSILON {
        let position = Vec2::new(transform.translation.x, transform.translation.z);
        transform.translation.y = PLAYER_EYE_HEIGHT + colliders.floor_height_at(position);
        return;
    }

    let speed = if keys.pressed(KeyCode::ShiftLeft) {
        SPRINT_SPEED
    } else {
        WALK_SPEED
    };

    let movement = input.normalize() * speed * time.delta_secs();
    let position = Vec2::new(transform.translation.x, transform.translation.z);
    let movement_xz = Vec2::new(movement.x, movement.z);
    let resolved =
        move_circle_through_aabbs(position, movement_xz, PLAYER_RADIUS, &colliders.walls);

    transform.translation.x = resolved.x;
    transform.translation.y = PLAYER_EYE_HEIGHT + colliders.floor_height_at(resolved);
    transform.translation.z = resolved.y;
}

fn play_footsteps(
    mut commands: Commands,
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    cursor_options: Query<&CursorOptions, With<PrimaryWindow>>,
    sounds: Res<SoundEffects>,
    mut clock: ResMut<FootstepClock>,
) {
    let Ok(cursor_options) = cursor_options.single() else {
        return;
    };

    if cursor_options.visible {
        clock.remaining = 0.0;
        return;
    }

    let moving = keys.pressed(KeyCode::KeyW)
        || keys.pressed(KeyCode::KeyA)
        || keys.pressed(KeyCode::KeyS)
        || keys.pressed(KeyCode::KeyD);

    if !moving {
        clock.remaining = 0.0;
        return;
    }

    clock.remaining -= time.delta_secs();
    if clock.remaining > 0.0 {
        return;
    }

    let sprinting = keys.pressed(KeyCode::ShiftLeft);
    clock.remaining = if sprinting { 0.22 } else { 0.36 };
    play_sound(
        &mut commands,
        sounds.footstep.clone(),
        if sprinting { 0.22 } else { 0.16 },
        if sprinting { 1.18 } else { 0.92 },
    );
}
