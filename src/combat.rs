use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use crate::collision::Aabb3;
use crate::map::MapColliders;
use crate::player::PlayerCamera;

pub struct CombatPlugin;

impl Plugin for CombatPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<WeaponInventory>()
            .init_resource::<ViewModelState>()
            .add_systems(
                Update,
                (
                    ensure_view_model,
                    switch_weapons,
                    reload_weapon,
                    tick_weapon_timers,
                    fire_weapon,
                    update_weapon_model_visibility,
                    update_window_title,
                    expire_effects,
                )
                    .chain(),
            );
    }
}

const MAX_SHOT_DISTANCE: f32 = 90.0;

#[derive(Component)]
pub struct Shootable {
    health: f32,
}

impl Shootable {
    pub const fn new(health: f32) -> Self {
        Self { health }
    }
}

#[derive(Component)]
pub struct Hitbox {
    bounds: Aabb3,
}

impl Hitbox {
    pub fn from_center_size(center: Vec3, size: Vec3) -> Self {
        Self {
            bounds: Aabb3::from_center_size(center, size),
        }
    }
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
}

#[derive(Component)]
struct WeaponModel {
    kind: WeaponKind,
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
    let rifle_body = materials.add(material(Color::srgb(0.16, 0.18, 0.17)));
    let rifle_trim = materials.add(material(Color::srgb(0.45, 0.38, 0.24)));
    let pistol_body = materials.add(material(Color::srgb(0.07, 0.08, 0.09)));
    let crosshair = materials.add(material(Color::srgb(0.92, 0.98, 0.92)));

    commands.entity(camera_entity).with_children(|parent| {
        spawn_piece(
            parent,
            &cube,
            &rifle_body,
            WeaponKind::Rifle,
            Vec3::new(0.32, -0.30, -0.72),
            Vec3::new(0.16, 0.12, 0.54),
        );
        spawn_piece(
            parent,
            &cube,
            &rifle_trim,
            WeaponKind::Rifle,
            Vec3::new(0.32, -0.24, -1.08),
            Vec3::new(0.05, 0.05, 0.48),
        );
        spawn_piece(
            parent,
            &cube,
            &rifle_trim,
            WeaponKind::Rifle,
            Vec3::new(0.44, -0.36, -0.50),
            Vec3::new(0.11, 0.20, 0.13),
        );

        spawn_piece(
            parent,
            &cube,
            &pistol_body,
            WeaponKind::Pistol,
            Vec3::new(0.30, -0.33, -0.62),
            Vec3::new(0.14, 0.11, 0.34),
        );
        spawn_piece(
            parent,
            &cube,
            &pistol_body,
            WeaponKind::Pistol,
            Vec3::new(0.36, -0.46, -0.52),
            Vec3::new(0.09, 0.25, 0.12),
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
        WeaponModel { kind },
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
    camera: Query<&Transform, With<PlayerCamera>>,
    colliders: Res<MapColliders>,
    targets: Query<(Entity, &Hitbox), With<Shootable>>,
    mut health: Query<&mut Shootable>,
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
    if !weapon.ready() {
        return;
    }

    let Ok(camera_transform) = camera.single() else {
        return;
    };

    weapon.ammo -= 1;
    weapon.cooldown_remaining = stats.cooldown_secs;

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
        .filter_map(|(entity, hitbox)| {
            hitbox
                .bounds
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

        shootable.health -= stats.damage;
        if shootable.health <= 0.0 {
            commands.entity(entity).despawn();
        }
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

fn update_window_title(
    inventory: Res<WeaponInventory>,
    mut windows: Query<&mut Window, With<PrimaryWindow>>,
) {
    let Ok(mut window) = windows.single_mut() else {
        return;
    };

    let stats = inventory.active.stats();
    let weapon = inventory.active_state();
    let status = if weapon.reload_remaining > 0.0 {
        "Reloading"
    } else {
        "Ready"
    };

    window.title = format!(
        "Bevy FPS Dust Blockout - {} {}/{} - {}",
        stats.label, weapon.ammo, stats.magazine_size, status
    );
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
        ..default()
    }
}
