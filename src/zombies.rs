use bevy::prelude::*;

use crate::audio_fx::{SoundEffects, play_sound};
use crate::collision::move_circle_through_aabbs;
use crate::combat::{Hitbox, Shootable, ShotReport};
use crate::game_ui::{GameMode, ScoreValue, SelectedMap, gameplay_unpaused};
use crate::map::{MapColliders, zombie_spawn_points};
use crate::player::PlayerCamera;

pub struct ZombiePlugin;

impl Plugin for ZombiePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameMode::Playing), spawn_zombies)
            .add_systems(OnEnter(GameMode::Menu), cleanup_zombies)
            .add_systems(
                Update,
                (
                    wake_zombies,
                    move_zombies,
                    animate_zombies,
                    update_zombie_health_bars,
                )
                    .chain()
                    .run_if(in_state(GameMode::Playing))
                    .run_if(gameplay_unpaused),
            );
    }
}

const ZOMBIE_RADIUS: f32 = 0.42;
const ZOMBIE_CENTER_Y: f32 = 0.9;
const WANDER_SPEED: f32 = 0.85;
const CHASE_SPEED: f32 = 2.55;
const PROXIMITY_AGGRO_RADIUS: f32 = 11.0;
const SHOT_HEARING_RADIUS: f32 = 24.0;
const NEAR_IMPACT_RADIUS: f32 = 7.5;

#[derive(Component)]
pub struct Zombie;

#[derive(Component)]
struct ZombieBrain {
    state: ZombieState,
    wander_direction: Vec2,
    wander_remaining: f32,
    last_shot_serial: u64,
    step_phase: f32,
    groan_cooldown: f32,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum ZombieState {
    Wander,
    Chase,
}

#[derive(Component)]
struct ZombiePart {
    base_translation: Vec3,
}

#[derive(Component)]
struct ZombieHealthFill {
    width: f32,
}

fn spawn_zombies(
    mut commands: Commands,
    selected_map: Res<SelectedMap>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let cube = meshes.add(Cuboid::new(1.0, 1.0, 1.0));
    let body = materials.add(zombie_material(Color::srgb(0.25, 0.45, 0.22)));
    let shirt = materials.add(zombie_material(Color::srgb(0.20, 0.26, 0.34)));
    let dark = materials.add(zombie_material(Color::srgb(0.08, 0.09, 0.08)));
    let health_back = materials.add(zombie_material(Color::srgb(0.28, 0.04, 0.03)));
    let health_fill = materials.add(zombie_material(Color::srgb(0.18, 0.86, 0.22)));

    for (index, position) in zombie_spawn_points(selected_map.kind)
        .into_iter()
        .enumerate()
    {
        let direction = initial_direction(index);
        commands
            .spawn((
                Transform::from_translation(position),
                Visibility::Visible,
                Zombie,
                ZombieBrain {
                    state: ZombieState::Wander,
                    wander_direction: direction,
                    wander_remaining: 1.4 + index as f32 * 0.27,
                    last_shot_serial: 0,
                    step_phase: index as f32,
                    groan_cooldown: 2.4 + index as f32 * 0.43,
                },
                Shootable::new(120.0),
                Hitbox::from_center_size(position, Vec3::new(0.85, 1.85, 0.75)),
                ScoreValue::new(100),
            ))
            .with_children(|parent| {
                spawn_part(
                    parent,
                    &cube,
                    &shirt,
                    Vec3::new(0.0, -0.12, 0.0),
                    Vec3::new(0.58, 1.18, 0.38),
                );
                spawn_part(
                    parent,
                    &cube,
                    &body,
                    Vec3::new(0.0, 0.67, 0.0),
                    Vec3::new(0.42, 0.38, 0.42),
                );
                spawn_part(
                    parent,
                    &cube,
                    &body,
                    Vec3::new(-0.42, 0.04, 0.05),
                    Vec3::new(0.16, 0.92, 0.16),
                );
                spawn_part(
                    parent,
                    &cube,
                    &body,
                    Vec3::new(0.42, 0.04, 0.05),
                    Vec3::new(0.16, 0.92, 0.16),
                );
                spawn_part(
                    parent,
                    &cube,
                    &dark,
                    Vec3::new(-0.16, -0.86, 0.0),
                    Vec3::new(0.18, 0.62, 0.16),
                );
                spawn_part(
                    parent,
                    &cube,
                    &dark,
                    Vec3::new(0.16, -0.86, 0.0),
                    Vec3::new(0.18, 0.62, 0.16),
                );
                parent.spawn((
                    Mesh3d(cube.clone()),
                    MeshMaterial3d(health_back.clone()),
                    Transform {
                        translation: Vec3::new(0.0, 1.18, -0.46),
                        scale: Vec3::new(0.82, 0.08, 0.045),
                        ..default()
                    },
                ));
                parent.spawn((
                    Mesh3d(cube.clone()),
                    MeshMaterial3d(health_fill.clone()),
                    Transform {
                        translation: Vec3::new(0.0, 1.185, -0.51),
                        scale: Vec3::new(0.76, 0.055, 0.045),
                        ..default()
                    },
                    ZombieHealthFill { width: 0.76 },
                ));
            });
    }
}

fn spawn_part(
    parent: &mut ChildSpawnerCommands,
    mesh: &Handle<Mesh>,
    material: &Handle<StandardMaterial>,
    translation: Vec3,
    scale: Vec3,
) {
    parent.spawn((
        Mesh3d(mesh.clone()),
        MeshMaterial3d(material.clone()),
        Transform {
            translation,
            scale,
            ..default()
        },
        ZombiePart {
            base_translation: translation,
        },
    ));
}

fn wake_zombies(
    mut commands: Commands,
    player: Query<&Transform, With<PlayerCamera>>,
    shot_report: Res<ShotReport>,
    sounds: Res<SoundEffects>,
    mut zombies: Query<(Entity, &Transform, &mut ZombieBrain), With<Zombie>>,
) {
    let Ok(player_transform) = player.single() else {
        return;
    };
    let player_position = xz(player_transform.translation);

    for (entity, transform, mut brain) in &mut zombies {
        let zombie_position = xz(transform.translation);
        if zombie_position.distance(player_position) <= PROXIMITY_AGGRO_RADIUS {
            alert_zombie(&mut commands, &sounds, &mut brain);
        }

        if shot_report.serial != 0 && shot_report.serial != brain.last_shot_serial {
            brain.last_shot_serial = shot_report.serial;

            let heard_shot =
                zombie_position.distance(xz(shot_report.origin)) <= SHOT_HEARING_RADIUS;
            let heard_impact =
                zombie_position.distance(xz(shot_report.hit_position)) <= NEAR_IMPACT_RADIUS;
            let direct_hit = shot_report.hit_entity == Some(entity);

            if heard_shot || heard_impact || direct_hit {
                alert_zombie(&mut commands, &sounds, &mut brain);
            }
        }
    }
}

fn move_zombies(
    mut commands: Commands,
    time: Res<Time>,
    sounds: Res<SoundEffects>,
    player: Query<&Transform, (With<PlayerCamera>, Without<Zombie>)>,
    colliders: Res<MapColliders>,
    mut zombies: Query<(&mut Transform, &mut ZombieBrain), (With<Zombie>, Without<PlayerCamera>)>,
) {
    let Ok(player_transform) = player.single() else {
        return;
    };
    let player_position = xz(player_transform.translation);
    let delta_secs = time.delta_secs();

    for (mut transform, mut brain) in &mut zombies {
        let current = xz(transform.translation);
        let direction = match brain.state {
            ZombieState::Chase => (player_position - current).normalize_or_zero(),
            ZombieState::Wander => {
                brain.wander_remaining -= delta_secs;
                if brain.wander_remaining <= 0.0 {
                    brain.wander_direction = rotate_direction(brain.wander_direction);
                    brain.wander_remaining = 1.5 + (brain.step_phase.sin() + 1.0) * 0.9;
                }
                brain.wander_direction
            }
        };

        if direction.length_squared() <= f32::EPSILON {
            continue;
        }

        let speed = if brain.state == ZombieState::Chase {
            CHASE_SPEED
        } else {
            WANDER_SPEED
        };
        let movement = direction.normalize() * speed * delta_secs;
        let resolved =
            move_circle_through_aabbs(current, movement, ZOMBIE_RADIUS, &colliders.walls);

        if resolved.distance_squared(current) < movement.length_squared() * 0.15
            && brain.state == ZombieState::Wander
        {
            brain.wander_direction = rotate_direction(brain.wander_direction);
            brain.wander_remaining = 1.0;
        }

        transform.translation.x = resolved.x;
        transform.translation.y = ZOMBIE_CENTER_Y;
        transform.translation.z = resolved.y;

        let facing = if resolved.distance_squared(current) > 0.0001 {
            resolved - current
        } else {
            direction
        };
        let look_target = Vec3::new(
            transform.translation.x + facing.x,
            ZOMBIE_CENTER_Y,
            transform.translation.z + facing.y,
        );
        transform.look_at(look_target, Vec3::Y);

        brain.step_phase += delta_secs * speed * 4.0;

        brain.groan_cooldown -= delta_secs;
        if brain.groan_cooldown <= 0.0 {
            match brain.state {
                ZombieState::Chase => {
                    let pitch = 0.78 + (brain.step_phase.sin() + 1.0) * 0.08;
                    play_sound(&mut commands, sounds.zombie_groan.clone(), 0.22, pitch);
                    brain.groan_cooldown = 1.8 + (brain.step_phase.cos() + 1.0) * 0.65;
                }
                ZombieState::Wander => {
                    let pitch = 0.68 + (brain.step_phase.cos() + 1.0) * 0.06;
                    play_sound(&mut commands, sounds.zombie_idle.clone(), 0.13, pitch);
                    brain.groan_cooldown = 4.2 + (brain.step_phase.sin() + 1.0) * 1.1;
                }
            }
        }
    }
}

fn alert_zombie(commands: &mut Commands, sounds: &SoundEffects, brain: &mut ZombieBrain) {
    if brain.state == ZombieState::Wander {
        let pitch = 0.88 + (brain.step_phase.sin() + 1.0) * 0.12;
        play_sound(commands, sounds.zombie_alert.clone(), 0.28, pitch);
        brain.groan_cooldown = 0.6;
    }

    brain.state = ZombieState::Chase;
}

fn animate_zombies(
    parents: Query<(&ZombieBrain, &Children), With<Zombie>>,
    mut parts: Query<(&ZombiePart, &mut Transform)>,
) {
    for (brain, children) in &parents {
        let pace = if brain.state == ZombieState::Chase {
            0.11
        } else {
            0.05
        };
        let bob = brain.step_phase.sin() * pace;

        for child in children.iter() {
            let Ok((part, mut transform)) = parts.get_mut(child) else {
                continue;
            };

            transform.translation = part.base_translation + Vec3::new(0.0, bob.abs() * 0.45, 0.0);
        }
    }
}

fn update_zombie_health_bars(
    zombies: Query<(&Shootable, &Children), With<Zombie>>,
    mut bars: Query<(&ZombieHealthFill, &mut Transform)>,
) {
    for (shootable, children) in &zombies {
        let fraction = shootable.health_fraction();
        for child in children.iter() {
            let Ok((bar, mut transform)) = bars.get_mut(child) else {
                continue;
            };

            let width = (bar.width * fraction).max(0.02);
            transform.scale.x = width;
            transform.translation.x = -(bar.width - width) * 0.5;
        }
    }
}

fn cleanup_zombies(mut commands: Commands, zombies: Query<Entity, With<Zombie>>) {
    for entity in &zombies {
        commands.entity(entity).despawn();
    }
}

fn initial_direction(index: usize) -> Vec2 {
    let angle = index as f32 * 1.37 + 0.35;
    Vec2::new(angle.cos(), angle.sin()).normalize()
}

fn rotate_direction(direction: Vec2) -> Vec2 {
    let angle: f32 = 1.93;
    Vec2::new(
        direction.x * angle.cos() - direction.y * angle.sin(),
        direction.x * angle.sin() + direction.y * angle.cos(),
    )
    .normalize()
}

fn xz(position: Vec3) -> Vec2 {
    Vec2::new(position.x, position.z)
}

fn zombie_material(base_color: Color) -> StandardMaterial {
    StandardMaterial {
        base_color,
        perceptual_roughness: 0.92,
        metallic: 0.0,
        ..default()
    }
}
