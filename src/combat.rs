use bevy::prelude::*;

use crate::audio_fx::{SoundEffects, play_sound};
use crate::collision::Aabb3;
use crate::game_ui::{AmmoHud, GameMode, Score, ScoreValue, gameplay_unpaused};
use crate::map::MapColliders;
use crate::player::PlayerCamera;

pub struct CombatPlugin;

impl Plugin for CombatPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<WeaponInventory>()
            .init_resource::<ViewModelState>()
            .init_resource::<ShotReport>()
            .add_systems(
                OnEnter(GameMode::Playing),
                (reset_weapons, reset_shot_report),
            )
            .add_systems(
                Update,
                (
                    ensure_view_model,
                    switch_weapons,
                    reload_weapon,
                    tick_weapon_timers,
                    fire_weapon,
                    animate_view_model,
                    update_weapon_model_visibility,
                    update_ammo_hud,
                    expire_effects,
                )
                    .chain()
                    .run_if(in_state(GameMode::Playing))
                    .run_if(gameplay_unpaused),
            );
    }
}

const MAX_SHOT_DISTANCE: f32 = 90.0;

#[derive(Component)]
pub struct Shootable {
    health: f32,
    max_health: f32,
}

impl Shootable {
    pub const fn new(health: f32) -> Self {
        Self {
            health,
            max_health: health,
        }
    }

    pub fn damage(&mut self, amount: f32) -> bool {
        self.health = (self.health - amount).max(0.0);
        self.health <= 0.0
    }

    pub fn health_fraction(&self) -> f32 {
        if self.max_health <= f32::EPSILON {
            0.0
        } else {
            (self.health / self.max_health).clamp(0.0, 1.0)
        }
    }
}

#[derive(Component)]
pub struct Hitbox {
    half_extents: Vec3,
}

impl Hitbox {
    pub fn from_center_size(_center: Vec3, size: Vec3) -> Self {
        Self {
            half_extents: size * 0.5,
        }
    }

    fn world_bounds(&self, center: Vec3) -> Aabb3 {
        Aabb3::new(center, self.half_extents)
    }
}

#[derive(Resource, Default, Clone, Copy)]
pub struct ShotReport {
    pub serial: u64,
    pub origin: Vec3,
    pub hit_position: Vec3,
    pub hit_entity: Option<Entity>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum WeaponKind {
    Rifle,
    Pistol,
}

#[derive(Resource)]
struct WeaponInventory {
    active: WeaponKind,
    rifle: WeaponState,
    pistol: WeaponState,
}

impl Default for WeaponInventory {
    fn default() -> Self {
        let rifle = WeaponKind::Rifle.stats();
        let pistol = WeaponKind::Pistol.stats();

        Self {
            active: WeaponKind::Rifle,
            rifle: WeaponState::new(rifle.magazine_size),
            pistol: WeaponState::new(pistol.magazine_size),
        }
    }
}

impl WeaponInventory {
    fn active_state(&self) -> &WeaponState {
        match self.active {
            WeaponKind::Rifle => &self.rifle,
            WeaponKind::Pistol => &self.pistol,
        }
    }

    fn active_state_mut(&mut self) -> &mut WeaponState {
        match self.active {
            WeaponKind::Rifle => &mut self.rifle,
            WeaponKind::Pistol => &mut self.pistol,
        }
    }
}

#[derive(Debug)]
struct WeaponState {
    ammo: u16,
    cooldown_remaining: f32,
    reload_remaining: f32,
}

impl WeaponState {
    const fn new(ammo: u16) -> Self {
        Self {
            ammo,
            cooldown_remaining: 0.0,
            reload_remaining: 0.0,
        }
    }

    fn ready(&self) -> bool {
        self.cooldown_remaining <= 0.0 && self.reload_remaining <= 0.0 && self.ammo > 0
    }
}

#[derive(Clone, Copy)]
struct WeaponStats {
    label: &'static str,
    magazine_size: u16,
    damage: f32,
    cooldown_secs: f32,
    reload_secs: f32,
    automatic: bool,
}

impl WeaponKind {
    const fn stats(self) -> WeaponStats {
        match self {
            WeaponKind::Rifle => WeaponStats {
                label: "Rifle",
                magazine_size: 30,
                damage: 28.0,
                cooldown_secs: 0.095,
                reload_secs: 1.6,
                automatic: true,
            },
            WeaponKind::Pistol => WeaponStats {
                label: "Pistol",
                magazine_size: 7,
                damage: 52.0,
                cooldown_secs: 0.36,
                reload_secs: 1.25,
                automatic: false,
            },
        }
    }
}

#[derive(Resource, Default)]
struct ViewModelState {
    spawned: bool,
    fire_kick: f32,
    bob_phase: f32,
}

#[derive(Component)]
struct WeaponModel {
    kind: WeaponKind,
    part: WeaponPart,
    base_translation: Vec3,
    base_rotation: Quat,
    base_scale: Vec3,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum WeaponPart {
    Body,
    Barrel,
    Magazine,
    Grip,
    Stock,
    Handguard,
    CarryHandle,
    FrontSight,
    Slide,
    Frame,
    TriggerGuard,
    Suppressor,
}

#[derive(Component)]
struct Lifetime {
    timer: Timer,
}

fn ensure_view_model(
    mut commands: Commands,
    mut state: ResMut<ViewModelState>,
    camera: Query<Entity, With<PlayerCamera>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    if state.spawned {
        return;
    }

    let Ok(camera_entity) = camera.single() else {
        return;
    };

    let cube = meshes.add(Cuboid::new(1.0, 1.0, 1.0));
    let rifle_body = materials.add(weapon_material(Color::srgb(0.18, 0.21, 0.19)));
    let rifle_trim = materials.add(weapon_material(Color::srgb(0.10, 0.11, 0.10)));
    let rifle_dark = materials.add(weapon_material(Color::srgb(0.08, 0.09, 0.09)));
    let pistol_body = materials.add(weapon_material(Color::srgb(0.09, 0.10, 0.11)));
    let pistol_slide = materials.add(weapon_material(Color::srgb(0.16, 0.17, 0.18)));
    let crosshair = materials.add(weapon_material(Color::srgb(0.92, 0.98, 0.92)));

    commands.entity(camera_entity).with_children(|parent| {
        // M16-inspired silhouette: long receiver, triangular handguard,
        // carry handle, straight magazine, fixed stock, and long barrel.
        spawn_piece(
            parent,
            &cube,
            &rifle_body,
            WeaponKind::Rifle,
            WeaponPart::Body,
            Vec3::new(0.24, -0.23, -0.48),
            Vec3::new(0.24, 0.15, 0.52),
        );
        spawn_piece(
            parent,
            &cube,
            &rifle_dark,
            WeaponKind::Rifle,
            WeaponPart::Barrel,
            Vec3::new(0.24, -0.18, -1.14),
            Vec3::new(0.045, 0.045, 0.74),
        );
        spawn_piece(
            parent,
            &cube,
            &rifle_trim,
            WeaponKind::Rifle,
            WeaponPart::Handguard,
            Vec3::new(0.24, -0.23, -0.83),
            Vec3::new(0.20, 0.16, 0.42),
        );
        spawn_piece(
            parent,
            &cube,
            &rifle_dark,
            WeaponKind::Rifle,
            WeaponPart::CarryHandle,
            Vec3::new(0.24, -0.10, -0.45),
            Vec3::new(0.16, 0.06, 0.35),
        );
        spawn_piece(
            parent,
            &cube,
            &rifle_dark,
            WeaponKind::Rifle,
            WeaponPart::CarryHandle,
            Vec3::new(0.24, -0.05, -0.45),
            Vec3::new(0.06, 0.08, 0.24),
        );
        spawn_piece(
            parent,
            &cube,
            &rifle_dark,
            WeaponKind::Rifle,
            WeaponPart::FrontSight,
            Vec3::new(0.24, -0.08, -1.28),
            Vec3::new(0.07, 0.14, 0.05),
        );
        spawn_piece(
            parent,
            &cube,
            &rifle_trim,
            WeaponKind::Rifle,
            WeaponPart::Grip,
            Vec3::new(0.36, -0.38, -0.37),
            Vec3::new(0.11, 0.25, 0.13),
        );
        spawn_piece(
            parent,
            &cube,
            &rifle_trim,
            WeaponKind::Rifle,
            WeaponPart::Magazine,
            Vec3::new(0.19, -0.42, -0.50),
            Vec3::new(0.12, 0.31, 0.15),
        );
        spawn_piece(
            parent,
            &cube,
            &rifle_dark,
            WeaponKind::Rifle,
            WeaponPart::Stock,
            Vec3::new(0.36, -0.23, -0.11),
            Vec3::new(0.22, 0.14, 0.32),
        );

        // USP-inspired pistol: boxy slide, compact frame, angled grip,
        // squared trigger guard, and a slim tactical suppressor profile.
        spawn_piece(
            parent,
            &cube,
            &pistol_slide,
            WeaponKind::Pistol,
            WeaponPart::Slide,
            Vec3::new(0.30, -0.30, -0.62),
            Vec3::new(0.16, 0.10, 0.42),
        );
        spawn_piece(
            parent,
            &cube,
            &pistol_body,
            WeaponKind::Pistol,
            WeaponPart::Frame,
            Vec3::new(0.30, -0.38, -0.56),
            Vec3::new(0.14, 0.08, 0.26),
        );
        spawn_piece(
            parent,
            &cube,
            &pistol_body,
            WeaponKind::Pistol,
            WeaponPart::Grip,
            Vec3::new(0.36, -0.51, -0.48),
            Vec3::new(0.10, 0.28, 0.13),
        );
        spawn_piece(
            parent,
            &cube,
            &pistol_body,
            WeaponKind::Pistol,
            WeaponPart::Magazine,
            Vec3::new(0.36, -0.58, -0.48),
            Vec3::new(0.08, 0.16, 0.11),
        );
        spawn_piece(
            parent,
            &cube,
            &pistol_body,
            WeaponKind::Pistol,
            WeaponPart::TriggerGuard,
            Vec3::new(0.29, -0.44, -0.66),
            Vec3::new(0.12, 0.08, 0.08),
        );
        spawn_piece(
            parent,
            &cube,
            &pistol_slide,
            WeaponKind::Pistol,
            WeaponPart::Suppressor,
            Vec3::new(0.30, -0.30, -0.94),
            Vec3::new(0.10, 0.10, 0.38),
        );

        parent.spawn((
            Mesh3d(cube.clone()),
            MeshMaterial3d(crosshair.clone()),
            Transform {
                translation: Vec3::new(0.0, 0.0, -1.2),
                scale: Vec3::new(0.035, 0.004, 0.004),
                ..default()
            },
        ));
        parent.spawn((
            Mesh3d(cube.clone()),
            MeshMaterial3d(crosshair),
            Transform {
                translation: Vec3::new(0.0, 0.0, -1.2),
                scale: Vec3::new(0.004, 0.035, 0.004),
                ..default()
            },
        ));
    });

    state.spawned = true;
}

fn spawn_piece(
    parent: &mut ChildSpawnerCommands,
    mesh: &Handle<Mesh>,
    material: &Handle<StandardMaterial>,
    kind: WeaponKind,
    part: WeaponPart,
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
        WeaponModel {
            kind,
            part,
            base_translation: translation,
            base_rotation: Quat::IDENTITY,
            base_scale: scale,
        },
    ));
}

fn switch_weapons(keys: Res<ButtonInput<KeyCode>>, mut inventory: ResMut<WeaponInventory>) {
    if keys.just_pressed(KeyCode::Digit1) {
        inventory.active = WeaponKind::Rifle;
    }

    if keys.just_pressed(KeyCode::Digit2) {
        inventory.active = WeaponKind::Pistol;
    }
}

fn reload_weapon(keys: Res<ButtonInput<KeyCode>>, mut inventory: ResMut<WeaponInventory>) {
    if !keys.just_pressed(KeyCode::KeyR) {
        return;
    }

    let stats = inventory.active.stats();
    let weapon = inventory.active_state_mut();
    if weapon.ammo < stats.magazine_size {
        weapon.reload_remaining = stats.reload_secs;
    }
}

fn tick_weapon_timers(time: Res<Time>, mut inventory: ResMut<WeaponInventory>) {
    tick_weapon(time.delta_secs(), WeaponKind::Rifle, &mut inventory.rifle);
    tick_weapon(time.delta_secs(), WeaponKind::Pistol, &mut inventory.pistol);
}

fn tick_weapon(delta_secs: f32, kind: WeaponKind, weapon: &mut WeaponState) {
    weapon.cooldown_remaining = (weapon.cooldown_remaining - delta_secs).max(0.0);

    if weapon.reload_remaining > 0.0 {
        weapon.reload_remaining = (weapon.reload_remaining - delta_secs).max(0.0);
        if weapon.reload_remaining <= 0.0 {
            weapon.ammo = kind.stats().magazine_size;
        }
    }
}

fn fire_weapon(
    mut commands: Commands,
    buttons: Res<ButtonInput<MouseButton>>,
    mut inventory: ResMut<WeaponInventory>,
    mut view_model: ResMut<ViewModelState>,
    mut shot_report: ResMut<ShotReport>,
    sounds: Res<SoundEffects>,
    mut score: ResMut<Score>,
    camera: Query<&Transform, With<PlayerCamera>>,
    colliders: Res<MapColliders>,
    targets: Query<(Entity, &Hitbox, &Transform), With<Shootable>>,
    mut health: Query<&mut Shootable>,
    score_values: Query<&ScoreValue>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let stats = inventory.active.stats();
    let wants_fire = if stats.automatic {
        buttons.pressed(MouseButton::Left)
    } else {
        buttons.just_pressed(MouseButton::Left)
    };

    if !wants_fire {
        return;
    }

    let weapon = inventory.active_state_mut();
    if weapon.ammo == 0 && weapon.reload_remaining <= 0.0 {
        weapon.reload_remaining = stats.reload_secs;
        return;
    }

    if !weapon.ready() {
        return;
    }

    let Ok(camera_transform) = camera.single() else {
        return;
    };

    weapon.ammo -= 1;
    weapon.cooldown_remaining = stats.cooldown_secs;
    view_model.fire_kick = 1.0;

    let sound = match inventory.active {
        WeaponKind::Rifle => sounds.rifle_shot.clone(),
        WeaponKind::Pistol => sounds.pistol_shot.clone(),
    };
    let variation = 0.96 + (shot_report.serial % 4) as f32 * 0.035;
    play_sound(&mut commands, sound, 0.38, variation);

    let origin = camera_transform.translation;
    let direction = camera_transform.rotation * Vec3::NEG_Z;
    let wall_distance = colliders
        .shot_blockers
        .iter()
        .filter_map(|blocker| {
            blocker.ray_intersection_distance(origin, direction, MAX_SHOT_DISTANCE)
        })
        .min_by(|left, right| left.total_cmp(right))
        .unwrap_or(MAX_SHOT_DISTANCE);

    let target_hit = targets
        .iter()
        .filter_map(|(entity, hitbox, transform)| {
            hitbox
                .world_bounds(transform.translation)
                .ray_intersection_distance(origin, direction, MAX_SHOT_DISTANCE)
                .map(|distance| (entity, distance))
        })
        .min_by(|(_, left), (_, right)| left.total_cmp(right));

    let mut hit_distance = wall_distance;
    let mut hit_target = None;

    if let Some((entity, target_distance)) = target_hit {
        if target_distance <= wall_distance {
            hit_distance = target_distance;
            hit_target = Some(entity);
        }
    }

    let hit_position = origin + direction * hit_distance;
    shot_report.serial = shot_report.serial.wrapping_add(1);
    shot_report.origin = origin;
    shot_report.hit_position = hit_position;
    shot_report.hit_entity = hit_target;

    spawn_shot_effects(
        &mut commands,
        &mut meshes,
        &mut materials,
        camera_transform,
        hit_position,
    );

    if let Some(entity) = hit_target {
        let Ok(mut shootable) = health.get_mut(entity) else {
            return;
        };

        if score_values.get(entity).is_ok() {
            let pitch = 0.88 + (shot_report.serial % 5) as f32 * 0.045;
            play_sound(&mut commands, sounds.zombie_hit.clone(), 0.24, pitch);
        }

        if shootable.damage(stats.damage) {
            if let Ok(score_value) = score_values.get(entity) {
                score.kills += 1;
                score.points += score_value.points;
            }
            commands.entity(entity).despawn();
        }
    }
}

fn animate_view_model(
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    inventory: Res<WeaponInventory>,
    mut state: ResMut<ViewModelState>,
    mut models: Query<(&WeaponModel, &mut Transform)>,
) {
    state.fire_kick = (state.fire_kick - time.delta_secs() * 9.5).max(0.0);
    let moving = keys.pressed(KeyCode::KeyW)
        || keys.pressed(KeyCode::KeyA)
        || keys.pressed(KeyCode::KeyS)
        || keys.pressed(KeyCode::KeyD);
    let sprinting = moving && keys.pressed(KeyCode::ShiftLeft);
    let bob_speed = if sprinting { 13.5 } else { 8.4 };
    let bob_amount = if sprinting { 0.055 } else { 0.028 };

    if moving {
        state.bob_phase += time.delta_secs() * bob_speed;
    } else {
        state.bob_phase *= (1.0 - time.delta_secs() * 8.0).max(0.0);
    }

    let active = inventory.active;
    let stats = active.stats();
    let weapon = inventory.active_state();
    let reload_progress = if weapon.reload_remaining > 0.0 {
        1.0 - weapon.reload_remaining / stats.reload_secs
    } else {
        0.0
    };
    let reload_wave = (reload_progress * std::f32::consts::PI).sin();
    let mag_drop = if weapon.reload_remaining > 0.0 {
        if reload_progress < 0.45 {
            reload_progress / 0.45
        } else {
            (1.0 - reload_progress) / 0.55
        }
        .clamp(0.0, 1.0)
    } else {
        0.0
    };
    let recoil = state.fire_kick * state.fire_kick;
    let bob_x = state.bob_phase.sin() * bob_amount * 0.55;
    let bob_y = (state.bob_phase * 2.0).sin().abs() * bob_amount;
    let sprint_tilt = if sprinting { 0.20 } else { 0.0 };

    for (model, mut transform) in &mut models {
        let mut offset = Vec3::ZERO;
        let mut rotation = model.base_rotation;

        if model.kind == active {
            offset += Vec3::new(bob_x, -bob_y, if sprinting { 0.04 } else { 0.0 });
            rotation *= Quat::from_rotation_z(-bob_x * 1.35 + sprint_tilt)
                * Quat::from_rotation_x(-bob_y * 0.9);

            offset += Vec3::new(-0.08 * reload_wave, -0.16 * reload_wave, 0.04 * reload_wave);
            rotation *= Quat::from_rotation_z(0.24 * reload_wave)
                * Quat::from_rotation_x(-0.18 * reload_wave);

            offset += Vec3::new(0.0, -0.025 * recoil, 0.09 * recoil);
            rotation *= Quat::from_rotation_x(0.10 * recoil);

            match model.part {
                WeaponPart::Magazine => {
                    offset += Vec3::new(-0.02 * mag_drop, -0.34 * mag_drop, 0.08 * mag_drop);
                    rotation *= Quat::from_rotation_x(0.55 * mag_drop);
                }
                WeaponPart::Barrel => {
                    offset += Vec3::new(0.0, 0.0, 0.06 * recoil);
                }
                WeaponPart::Grip
                | WeaponPart::Stock
                | WeaponPart::Body
                | WeaponPart::Handguard
                | WeaponPart::CarryHandle
                | WeaponPart::FrontSight
                | WeaponPart::Slide
                | WeaponPart::Frame
                | WeaponPart::TriggerGuard
                | WeaponPart::Suppressor => {}
            }
        }

        transform.translation = model.base_translation + offset;
        transform.rotation = rotation;
        transform.scale = model.base_scale;
    }
}

fn spawn_shot_effects(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    camera_transform: &Transform,
    hit_position: Vec3,
) {
    let cube = meshes.add(Cuboid::new(1.0, 1.0, 1.0));
    let tracer_material = materials.add(material(Color::srgb(1.0, 0.9, 0.35)));
    let flash_material = materials.add(material(Color::srgb(1.0, 0.58, 0.16)));
    let impact_material = materials.add(material(Color::srgb(0.95, 0.12, 0.05)));

    let direction = camera_transform.rotation * Vec3::NEG_Z;
    let right = camera_transform.rotation * Vec3::X;
    let down = camera_transform.rotation * Vec3::NEG_Y;
    let muzzle = camera_transform.translation + direction * 0.75 + right * 0.25 + down * 0.22;
    let tracer_midpoint = muzzle.lerp(hit_position, 0.5);
    let tracer_length = muzzle.distance(hit_position).max(0.1);
    let mut tracer_transform = Transform::from_translation(tracer_midpoint);
    tracer_transform.look_at(hit_position, Vec3::Y);
    tracer_transform.scale = Vec3::new(0.025, 0.025, tracer_length);

    commands.spawn((
        Mesh3d(cube.clone()),
        MeshMaterial3d(tracer_material),
        tracer_transform,
        Lifetime {
            timer: Timer::from_seconds(0.045, TimerMode::Once),
        },
    ));

    commands.spawn((
        Mesh3d(cube.clone()),
        MeshMaterial3d(flash_material),
        Transform {
            translation: muzzle,
            scale: Vec3::splat(0.16),
            ..default()
        },
        Lifetime {
            timer: Timer::from_seconds(0.06, TimerMode::Once),
        },
    ));

    commands.spawn((
        Mesh3d(cube),
        MeshMaterial3d(impact_material),
        Transform {
            translation: hit_position - direction * 0.03,
            scale: Vec3::splat(0.12),
            ..default()
        },
        Lifetime {
            timer: Timer::from_seconds(0.12, TimerMode::Once),
        },
    ));
}

fn update_weapon_model_visibility(
    inventory: Res<WeaponInventory>,
    mut models: Query<(&WeaponModel, &mut Visibility)>,
) {
    for (model, mut visibility) in &mut models {
        *visibility = if model.kind == inventory.active {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
}

fn update_ammo_hud(inventory: Res<WeaponInventory>, mut ammo_hud: ResMut<AmmoHud>) {
    let stats = inventory.active.stats();
    let weapon = inventory.active_state();
    ammo_hud.weapon = stats.label;
    ammo_hud.ammo = weapon.ammo;
    ammo_hud.magazine_size = stats.magazine_size;
    ammo_hud.reloading = weapon.reload_remaining > 0.0;
}

fn reset_weapons(mut inventory: ResMut<WeaponInventory>) {
    *inventory = WeaponInventory::default();
}

fn reset_shot_report(mut shot_report: ResMut<ShotReport>) {
    *shot_report = ShotReport::default();
}

fn expire_effects(
    mut commands: Commands,
    time: Res<Time>,
    mut effects: Query<(Entity, &mut Lifetime)>,
) {
    for (entity, mut lifetime) in &mut effects {
        lifetime.timer.tick(time.delta());
        if lifetime.timer.is_finished() {
            commands.entity(entity).despawn();
        }
    }
}

fn material(base_color: Color) -> StandardMaterial {
    StandardMaterial {
        base_color,
        perceptual_roughness: 0.8,
        metallic: 0.0,
        unlit: true,
        ..default()
    }
}

fn weapon_material(base_color: Color) -> StandardMaterial {
    StandardMaterial {
        base_color,
        perceptual_roughness: 0.72,
        metallic: 0.05,
        unlit: true,
        ..default()
    }
}
