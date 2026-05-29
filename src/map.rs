use bevy::{
    asset::RenderAssetUsages,
    image::{ImageAddressMode, ImageFilterMode, ImageSampler, ImageSamplerDescriptor},
    math::Affine2,
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
};

use crate::collision::{Aabb2, Aabb3};
use crate::combat::{Hitbox, Shootable};
use crate::game_ui::{GameMode, MapKind, SelectedMap, gameplay_unpaused};

pub struct MapPlugin;

impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(MapColliders::default())
            .add_systems(OnEnter(GameMode::Playing), spawn_map)
            .add_systems(OnEnter(GameMode::Menu), cleanup_map)
            .add_systems(
                Update,
                drift_clouds
                    .run_if(in_state(GameMode::Playing))
                    .run_if(gameplay_unpaused),
            );
    }
}

#[derive(Resource, Default)]
pub struct MapColliders {
    pub walls: Vec<Aabb2>,
    pub shot_blockers: Vec<Aabb3>,
}

#[derive(Component)]
struct MapEntity;

#[derive(Clone, Copy)]
enum BlockKind {
    Ground,
    Wall,
    Prop,
    Accent,
    Window,
    Foliage,
    Trunk,
}

#[derive(Clone, Copy)]
struct Block {
    center: Vec3,
    size: Vec3,
    kind: BlockKind,
    collides: bool,
}

impl Block {
    const fn ground(center: Vec3, size: Vec3) -> Self {
        Self {
            center,
            size,
            kind: BlockKind::Ground,
            collides: false,
        }
    }

    const fn solid(center: Vec3, size: Vec3, kind: BlockKind) -> Self {
        Self {
            center,
            size,
            kind,
            collides: true,
        }
    }

    const fn decor(center: Vec3, size: Vec3, kind: BlockKind) -> Self {
        Self {
            center,
            size,
            kind,
            collides: false,
        }
    }

    fn collider(self) -> Aabb2 {
        Aabb2::from_center_size(
            Vec2::new(self.center.x, self.center.z),
            Vec2::new(self.size.x, self.size.z),
        )
    }

    fn shot_blocker(self) -> Aabb3 {
        Aabb3::from_center_size(self.center, self.size)
    }
}

#[derive(Clone, Copy)]
pub struct PlayerSpawn {
    pub position: Vec3,
    pub yaw: f32,
}

pub fn player_spawn(kind: MapKind) -> PlayerSpawn {
    let position = match kind {
        MapKind::Desert => Vec3::new(0.0, 1.65, 31.0),
        MapKind::Forest => Vec3::new(-2.0, 1.65, 31.0),
        MapKind::Night => Vec3::new(0.0, 1.65, 30.0),
    };

    PlayerSpawn {
        position,
        yaw: yaw_toward(position, Vec3::ZERO),
    }
}

pub fn zombie_spawn_points(kind: MapKind) -> [Vec3; 8] {
    match kind {
        MapKind::Desert => [
            Vec3::new(-31.0, 0.9, -25.0),
            Vec3::new(30.0, 0.9, -23.0),
            Vec3::new(-30.0, 0.9, 20.0),
            Vec3::new(29.0, 0.9, 19.0),
            Vec3::new(-10.0, 0.9, -12.0),
            Vec3::new(11.0, 0.9, -10.0),
            Vec3::new(-22.0, 0.9, 0.0),
            Vec3::new(22.0, 0.9, 2.0),
        ],
        MapKind::Forest => [
            Vec3::new(-30.0, 0.9, -25.0),
            Vec3::new(31.0, 0.9, -24.0),
            Vec3::new(-29.0, 0.9, 19.0),
            Vec3::new(28.0, 0.9, 20.0),
            Vec3::new(-9.0, 0.9, -17.0),
            Vec3::new(13.0, 0.9, -14.0),
            Vec3::new(-19.0, 0.9, 3.0),
            Vec3::new(21.0, 0.9, 5.0),
        ],
        MapKind::Night => [
            Vec3::new(-32.0, 0.9, -24.0),
            Vec3::new(32.0, 0.9, -24.0),
            Vec3::new(-31.0, 0.9, 20.0),
            Vec3::new(31.0, 0.9, 20.0),
            Vec3::new(-12.0, 0.9, -10.0),
            Vec3::new(12.0, 0.9, -10.0),
            Vec3::new(-21.0, 0.9, 6.0),
            Vec3::new(21.0, 0.9, 7.0),
        ],
    }
}

fn spawn_map(
    mut commands: Commands,
    selected: Res<SelectedMap>,
    mut clear_color: ResMut<ClearColor>,
    mut ambient: ResMut<GlobalAmbientLight>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
    mut colliders: ResMut<MapColliders>,
    existing: Query<Entity, With<MapEntity>>,
) {
    for entity in &existing {
        commands.entity(entity).despawn();
    }

    colliders.walls.clear();
    colliders.shot_blockers.clear();

    let cube = meshes.add(Cuboid::new(1.0, 1.0, 1.0));
    let palette = MapPalette::new(selected.kind, &mut materials, &mut images);
    let spec = map_spec(selected.kind);

    clear_color.0 = spec.sky_color;
    ambient.color = spec.ambient_color;
    ambient.brightness = spec.ambient_brightness;

    for block in spec.blocks {
        if block.collides {
            colliders.walls.push(block.collider());
            colliders.shot_blockers.push(block.shot_blocker());
        }

        commands.spawn((
            Mesh3d(cube.clone()),
            MeshMaterial3d(palette.material_for(block.kind)),
            Transform {
                translation: block.center,
                scale: block.size,
                ..default()
            },
            MapEntity,
        ));
    }

    spawn_targets(&mut commands, &mut meshes, &mut materials, &spec.targets);
    spawn_clouds(
        &mut commands,
        &mut meshes,
        &mut materials,
        &spec.clouds,
        selected.kind,
    );
    spawn_lighting(&mut commands, selected.kind);
}

fn cleanup_map(
    mut commands: Commands,
    mut colliders: ResMut<MapColliders>,
    entities: Query<Entity, With<MapEntity>>,
) {
    for entity in &entities {
        commands.entity(entity).despawn();
    }

    colliders.walls.clear();
    colliders.shot_blockers.clear();
}

fn spawn_targets(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    targets: &[TargetSpec],
) {
    let mesh = meshes.add(Cuboid::new(1.0, 1.0, 1.0));
    let target_material = materials.add(material(Color::srgb(0.82, 0.08, 0.06)));

    for target in targets {
        commands.spawn((
            Mesh3d(mesh.clone()),
            MeshMaterial3d(target_material.clone()),
            Transform {
                translation: target.center,
                scale: target.size,
                ..default()
            },
            Shootable::new(target.health),
            Hitbox::from_center_size(target.center, target.size),
            MapEntity,
        ));
    }
}

fn spawn_lighting(commands: &mut Commands, kind: MapKind) {
    let (illuminance, position) = match kind {
        MapKind::Desert => (34_000.0, Vec3::new(-8.0, 16.0, 8.0)),
        MapKind::Forest => (21_000.0, Vec3::new(-10.0, 18.0, -6.0)),
        MapKind::Night => (3_200.0, Vec3::new(-6.0, 18.0, 7.0)),
    };

    commands.spawn((
        DirectionalLight {
            illuminance,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_translation(position).looking_at(Vec3::ZERO, Vec3::Y),
        MapEntity,
    ));

    if kind == MapKind::Night {
        for position in [
            Vec3::new(-18.0, 3.2, 8.0),
            Vec3::new(18.0, 3.2, 8.0),
            Vec3::new(-8.0, 3.2, -18.0),
            Vec3::new(8.0, 3.2, -18.0),
        ] {
            commands.spawn((
                PointLight {
                    intensity: 650.0,
                    range: 15.0,
                    shadows_enabled: true,
                    ..default()
                },
                Transform::from_translation(position),
                MapEntity,
            ));
        }
    }
}

#[derive(Component)]
struct Cloud {
    speed: f32,
    wrap_x: f32,
}

#[derive(Clone, Copy)]
struct CloudSpec {
    center: Vec3,
    speed: f32,
}

fn spawn_clouds(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    clouds: &[CloudSpec],
    kind: MapKind,
) {
    let mesh = meshes.add(Cuboid::new(1.0, 1.0, 1.0));
    let color = match kind {
        MapKind::Desert => Color::srgba(0.96, 0.94, 0.88, 0.54),
        MapKind::Forest => Color::srgba(0.90, 0.96, 0.88, 0.46),
        MapKind::Night => Color::srgba(0.13, 0.16, 0.21, 0.45),
    };
    let material = materials.add(StandardMaterial {
        base_color: color,
        alpha_mode: AlphaMode::Blend,
        perceptual_roughness: 1.0,
        unlit: true,
        ..default()
    });
    let puffs = [
        (Vec3::new(0.0, 0.0, 0.0), Vec3::new(7.0, 0.65, 2.7)),
        (Vec3::new(-2.7, 0.18, 0.7), Vec3::new(4.6, 0.85, 2.1)),
        (Vec3::new(2.9, 0.1, -0.4), Vec3::new(5.1, 0.75, 2.2)),
    ];

    for cloud in clouds {
        for (offset, size) in puffs {
            commands.spawn((
                Mesh3d(mesh.clone()),
                MeshMaterial3d(material.clone()),
                Transform {
                    translation: cloud.center + offset,
                    scale: size,
                    ..default()
                },
                Cloud {
                    speed: cloud.speed,
                    wrap_x: 56.0,
                },
                MapEntity,
            ));
        }
    }
}

fn drift_clouds(time: Res<Time>, mut clouds: Query<(&mut Transform, &Cloud)>) {
    for (mut transform, cloud) in &mut clouds {
        transform.translation.x += cloud.speed * time.delta_secs();
        if transform.translation.x > cloud.wrap_x {
            transform.translation.x = -cloud.wrap_x;
        }
    }
}

struct MapSpec {
    blocks: Vec<Block>,
    targets: Vec<TargetSpec>,
    clouds: Vec<CloudSpec>,
    sky_color: Color,
    ambient_color: Color,
    ambient_brightness: f32,
}

fn map_spec(kind: MapKind) -> MapSpec {
    match kind {
        MapKind::Desert => desert_market(),
        MapKind::Forest => forest_clearing(),
        MapKind::Night => night_quarter(),
    }
}

fn desert_market() -> MapSpec {
    let mut blocks = common_bounds(84.0, 72.0, 3.2, BlockKind::Wall);
    blocks.push(Block::ground(
        Vec3::new(0.0, -0.05, 0.0),
        Vec3::new(84.0, 0.1, 72.0),
    ));

    blocks.extend([
        Block::solid(
            Vec3::new(-29.0, 1.65, -12.0),
            Vec3::new(11.0, 3.3, 22.0),
            BlockKind::Wall,
        ),
        Block::solid(
            Vec3::new(29.0, 1.65, -12.0),
            Vec3::new(11.0, 3.3, 22.0),
            BlockKind::Wall,
        ),
        Block::solid(
            Vec3::new(-24.0, 1.55, 20.0),
            Vec3::new(13.0, 3.1, 12.0),
            BlockKind::Wall,
        ),
        Block::solid(
            Vec3::new(24.0, 1.55, 20.0),
            Vec3::new(13.0, 3.1, 12.0),
            BlockKind::Wall,
        ),
        Block::solid(
            Vec3::new(0.0, 1.45, -26.0),
            Vec3::new(18.0, 2.9, 7.0),
            BlockKind::Wall,
        ),
        Block::decor(
            Vec3::new(0.0, 0.01, 7.0),
            Vec3::new(24.0, 0.04, 16.0),
            BlockKind::Accent,
        ),
    ]);

    add_awning(&mut blocks, Vec3::new(-9.0, 1.9, 6.0));
    add_awning(&mut blocks, Vec3::new(9.0, 1.9, 6.0));
    add_windows(&mut blocks, -23.4, -12.0, 20.0, true);
    add_windows(&mut blocks, 23.4, -12.0, 20.0, true);
    add_windows(&mut blocks, -24.0, 14.0, 8.0, false);
    add_windows(&mut blocks, 24.0, 14.0, 8.0, false);

    for position in [
        Vec3::new(-8.0, 0.55, -5.0),
        Vec3::new(7.0, 0.55, -6.0),
        Vec3::new(-17.0, 0.55, 7.0),
        Vec3::new(17.0, 0.55, 7.0),
        Vec3::new(0.0, 0.55, 19.0),
    ] {
        blocks.push(Block::solid(
            position,
            Vec3::new(2.4, 1.1, 2.0),
            BlockKind::Prop,
        ));
    }

    MapSpec {
        blocks,
        targets: targets_for(MapKind::Desert),
        clouds: vec![
            CloudSpec {
                center: Vec3::new(-26.0, 18.0, -18.0),
                speed: 0.42,
            },
            CloudSpec {
                center: Vec3::new(21.0, 20.0, 14.0),
                speed: 0.36,
            },
        ],
        sky_color: Color::srgb(0.61, 0.72, 0.86),
        ambient_color: Color::srgb(0.95, 0.86, 0.68),
        ambient_brightness: 820.0,
    }
}

fn forest_clearing() -> MapSpec {
    let mut blocks = common_bounds(84.0, 72.0, 2.4, BlockKind::Wall);
    blocks.push(Block::ground(
        Vec3::new(0.0, -0.05, 0.0),
        Vec3::new(84.0, 0.1, 72.0),
    ));
    blocks.extend([
        Block::decor(
            Vec3::new(0.0, 0.01, 2.0),
            Vec3::new(15.0, 0.04, 62.0),
            BlockKind::Accent,
        ),
        Block::decor(
            Vec3::new(0.0, 0.02, -6.0),
            Vec3::new(56.0, 0.04, 11.0),
            BlockKind::Accent,
        ),
        Block::solid(
            Vec3::new(-15.0, 1.1, -12.0),
            Vec3::new(12.0, 2.2, 1.0),
            BlockKind::Wall,
        ),
        Block::solid(
            Vec3::new(15.0, 1.1, 12.0),
            Vec3::new(12.0, 2.2, 1.0),
            BlockKind::Wall,
        ),
        Block::solid(
            Vec3::new(-24.0, 1.1, 12.0),
            Vec3::new(1.0, 2.2, 12.0),
            BlockKind::Wall,
        ),
        Block::solid(
            Vec3::new(24.0, 1.1, -14.0),
            Vec3::new(1.0, 2.2, 12.0),
            BlockKind::Wall,
        ),
    ]);

    for position in [
        Vec2::new(-31.0, -22.0),
        Vec2::new(-25.0, 22.0),
        Vec2::new(-13.0, 25.0),
        Vec2::new(14.0, 24.0),
        Vec2::new(30.0, 20.0),
        Vec2::new(31.0, -20.0),
        Vec2::new(17.0, -27.0),
        Vec2::new(-18.0, -26.0),
        Vec2::new(-6.0, 14.0),
        Vec2::new(7.0, -18.0),
    ] {
        add_tree(&mut blocks, position);
    }

    for position in [
        Vec3::new(-8.0, 0.45, -4.0),
        Vec3::new(9.0, 0.45, 4.0),
        Vec3::new(-29.0, 0.5, -2.0),
        Vec3::new(29.0, 0.5, 2.0),
    ] {
        blocks.push(Block::solid(
            position,
            Vec3::new(2.4, 0.9, 1.8),
            BlockKind::Prop,
        ));
    }

    MapSpec {
        blocks,
        targets: targets_for(MapKind::Forest),
        clouds: vec![CloudSpec {
            center: Vec3::new(-18.0, 20.0, 18.0),
            speed: 0.28,
        }],
        sky_color: Color::srgb(0.50, 0.68, 0.77),
        ambient_color: Color::srgb(0.70, 0.94, 0.72),
        ambient_brightness: 640.0,
    }
}

fn night_quarter() -> MapSpec {
    let mut blocks = common_bounds(84.0, 72.0, 3.0, BlockKind::Wall);
    blocks.push(Block::ground(
        Vec3::new(0.0, -0.05, 0.0),
        Vec3::new(84.0, 0.1, 72.0),
    ));
    blocks.extend([
        Block::decor(
            Vec3::new(0.0, 0.01, 1.0),
            Vec3::new(20.0, 0.04, 18.0),
            BlockKind::Accent,
        ),
        Block::solid(
            Vec3::new(-28.0, 1.55, -15.0),
            Vec3::new(12.0, 3.1, 18.0),
            BlockKind::Wall,
        ),
        Block::solid(
            Vec3::new(28.0, 1.55, -15.0),
            Vec3::new(12.0, 3.1, 18.0),
            BlockKind::Wall,
        ),
        Block::solid(
            Vec3::new(-27.0, 1.45, 18.0),
            Vec3::new(13.0, 2.9, 12.0),
            BlockKind::Wall,
        ),
        Block::solid(
            Vec3::new(27.0, 1.45, 18.0),
            Vec3::new(13.0, 2.9, 12.0),
            BlockKind::Wall,
        ),
        Block::solid(
            Vec3::new(0.0, 1.3, -25.0),
            Vec3::new(15.0, 2.6, 6.0),
            BlockKind::Wall,
        ),
        Block::solid(
            Vec3::new(-11.0, 0.65, -2.0),
            Vec3::new(1.0, 1.3, 12.0),
            BlockKind::Wall,
        ),
        Block::solid(
            Vec3::new(11.0, 0.65, 2.0),
            Vec3::new(1.0, 1.3, 12.0),
            BlockKind::Wall,
        ),
    ]);

    for position in [
        Vec3::new(-18.0, 1.4, 8.0),
        Vec3::new(18.0, 1.4, 8.0),
        Vec3::new(-8.0, 1.4, -18.0),
        Vec3::new(8.0, 1.4, -18.0),
    ] {
        blocks.push(Block::decor(
            position,
            Vec3::new(0.35, 2.8, 0.35),
            BlockKind::Trunk,
        ));
        blocks.push(Block::decor(
            position + Vec3::Y * 1.45,
            Vec3::splat(0.9),
            BlockKind::Window,
        ));
    }

    add_windows(&mut blocks, -21.9, -15.0, 11.0, true);
    add_windows(&mut blocks, 21.9, -15.0, 11.0, true);
    add_windows(&mut blocks, -27.0, 12.0, 8.0, false);
    add_windows(&mut blocks, 27.0, 12.0, 8.0, false);

    for position in [
        Vec3::new(-5.0, 0.55, 13.0),
        Vec3::new(6.0, 0.55, 13.0),
        Vec3::new(-18.0, 0.55, -2.0),
        Vec3::new(18.0, 0.55, -2.0),
    ] {
        blocks.push(Block::solid(
            position,
            Vec3::new(2.2, 1.1, 2.0),
            BlockKind::Prop,
        ));
    }

    MapSpec {
        blocks,
        targets: targets_for(MapKind::Night),
        clouds: vec![
            CloudSpec {
                center: Vec3::new(-25.0, 17.0, -12.0),
                speed: 0.22,
            },
            CloudSpec {
                center: Vec3::new(24.0, 19.0, 18.0),
                speed: 0.18,
            },
        ],
        sky_color: Color::srgb(0.05, 0.07, 0.11),
        ambient_color: Color::srgb(0.30, 0.39, 0.58),
        ambient_brightness: 210.0,
    }
}

fn common_bounds(width: f32, depth: f32, height: f32, kind: BlockKind) -> Vec<Block> {
    let half_width = width * 0.5;
    let half_depth = depth * 0.5;
    let y = height * 0.5;

    vec![
        Block::solid(
            Vec3::new(0.0, y, -half_depth),
            Vec3::new(width, height, 1.0),
            kind,
        ),
        Block::solid(
            Vec3::new(0.0, y, half_depth),
            Vec3::new(width, height, 1.0),
            kind,
        ),
        Block::solid(
            Vec3::new(-half_width, y, 0.0),
            Vec3::new(1.0, height, depth),
            kind,
        ),
        Block::solid(
            Vec3::new(half_width, y, 0.0),
            Vec3::new(1.0, height, depth),
            kind,
        ),
    ]
}

fn add_awning(blocks: &mut Vec<Block>, center: Vec3) {
    blocks.push(Block::solid(
        Vec3::new(center.x, 0.48, center.z),
        Vec3::new(3.4, 0.95, 1.7),
        BlockKind::Prop,
    ));
    blocks.push(Block::decor(
        center,
        Vec3::new(4.6, 0.18, 3.2),
        BlockKind::Accent,
    ));
}

fn add_tree(blocks: &mut Vec<Block>, position: Vec2) {
    blocks.push(Block::solid(
        Vec3::new(position.x, 1.15, position.y),
        Vec3::new(0.85, 2.3, 0.85),
        BlockKind::Trunk,
    ));
    blocks.push(Block::decor(
        Vec3::new(position.x, 3.0, position.y),
        Vec3::new(4.3, 2.3, 4.3),
        BlockKind::Foliage,
    ));
}

fn add_windows(blocks: &mut Vec<Block>, x: f32, z: f32, span: f32, vertical_face: bool) {
    let size = if vertical_face {
        Vec3::new(0.12, 0.7, span)
    } else {
        Vec3::new(span, 0.7, 0.12)
    };

    blocks.push(Block::decor(Vec3::new(x, 2.0, z), size, BlockKind::Window));
}

#[derive(Clone, Copy)]
struct TargetSpec {
    center: Vec3,
    size: Vec3,
    health: f32,
}

fn targets_for(kind: MapKind) -> Vec<TargetSpec> {
    let positions = match kind {
        MapKind::Desert => [
            Vec3::new(-14.0, 1.2, 8.0),
            Vec3::new(14.0, 1.2, 8.0),
            Vec3::new(-8.0, 1.2, -18.0),
            Vec3::new(8.0, 1.2, -18.0),
        ],
        MapKind::Forest => [
            Vec3::new(-18.0, 1.2, -5.0),
            Vec3::new(18.0, 1.2, 5.0),
            Vec3::new(-6.0, 1.2, 18.0),
            Vec3::new(7.0, 1.2, -18.0),
        ],
        MapKind::Night => [
            Vec3::new(-14.0, 1.2, 10.0),
            Vec3::new(14.0, 1.2, 10.0),
            Vec3::new(-8.0, 1.2, -20.0),
            Vec3::new(8.0, 1.2, -20.0),
        ],
    };

    positions
        .into_iter()
        .map(|center| TargetSpec {
            center,
            size: Vec3::new(1.0, 1.7, 0.25),
            health: 100.0,
        })
        .collect()
}

struct MapPalette {
    ground: Handle<StandardMaterial>,
    wall: Handle<StandardMaterial>,
    prop: Handle<StandardMaterial>,
    accent: Handle<StandardMaterial>,
    window: Handle<StandardMaterial>,
    foliage: Handle<StandardMaterial>,
    trunk: Handle<StandardMaterial>,
}

impl MapPalette {
    fn new(
        kind: MapKind,
        materials: &mut Assets<StandardMaterial>,
        images: &mut Assets<Image>,
    ) -> Self {
        let colors = palette_colors(kind);
        let ground = images.add(procedural_texture(
            colors.ground.0,
            colors.ground.1,
            colors.ground.2,
            7,
        ));
        let wall = images.add(procedural_texture(
            colors.wall.0,
            colors.wall.1,
            colors.wall.2,
            19,
        ));
        let prop = images.add(procedural_texture(
            colors.prop.0,
            colors.prop.1,
            colors.prop.2,
            31,
        ));
        let accent = images.add(procedural_texture(
            colors.accent.0,
            colors.accent.1,
            colors.accent.2,
            43,
        ));
        let window = images.add(procedural_texture(
            colors.window.0,
            colors.window.1,
            colors.window.2,
            53,
        ));
        let foliage = images.add(procedural_texture(
            colors.foliage.0,
            colors.foliage.1,
            colors.foliage.2,
            67,
        ));
        let trunk = images.add(procedural_texture(
            colors.trunk.0,
            colors.trunk.1,
            colors.trunk.2,
            79,
        ));

        Self {
            ground: materials.add(textured_material(ground, Vec2::new(12.0, 12.0))),
            wall: materials.add(textured_material(wall, Vec2::new(4.0, 3.0))),
            prop: materials.add(textured_material(prop, Vec2::new(2.0, 2.0))),
            accent: materials.add(textured_material(accent, Vec2::new(3.0, 3.0))),
            window: materials.add(textured_material(window, Vec2::new(1.0, 1.0))),
            foliage: materials.add(textured_material(foliage, Vec2::new(2.5, 2.5))),
            trunk: materials.add(textured_material(trunk, Vec2::new(2.0, 2.0))),
        }
    }

    fn material_for(&self, kind: BlockKind) -> Handle<StandardMaterial> {
        match kind {
            BlockKind::Ground => self.ground.clone(),
            BlockKind::Wall => self.wall.clone(),
            BlockKind::Prop => self.prop.clone(),
            BlockKind::Accent => self.accent.clone(),
            BlockKind::Window => self.window.clone(),
            BlockKind::Foliage => self.foliage.clone(),
            BlockKind::Trunk => self.trunk.clone(),
        }
    }
}

struct PaletteColors {
    ground: ([u8; 3], [u8; 3], [u8; 3]),
    wall: ([u8; 3], [u8; 3], [u8; 3]),
    prop: ([u8; 3], [u8; 3], [u8; 3]),
    accent: ([u8; 3], [u8; 3], [u8; 3]),
    window: ([u8; 3], [u8; 3], [u8; 3]),
    foliage: ([u8; 3], [u8; 3], [u8; 3]),
    trunk: ([u8; 3], [u8; 3], [u8; 3]),
}

fn palette_colors(kind: MapKind) -> PaletteColors {
    match kind {
        MapKind::Desert => PaletteColors {
            ground: ([181, 160, 118], [212, 193, 149], [130, 112, 81]),
            wall: ([197, 176, 130], [227, 207, 164], [142, 121, 86]),
            prop: ([116, 91, 63], [151, 121, 83], [72, 52, 35]),
            accent: ([152, 54, 43], [209, 118, 82], [85, 31, 29]),
            window: ([34, 58, 66], [78, 111, 121], [15, 24, 29]),
            foliage: ([88, 106, 55], [130, 149, 76], [46, 61, 34]),
            trunk: ([94, 65, 42], [131, 91, 58], [51, 35, 24]),
        },
        MapKind::Forest => PaletteColors {
            ground: ([75, 119, 69], [115, 163, 94], [43, 75, 43]),
            wall: ([105, 113, 92], [148, 158, 128], [59, 67, 55]),
            prop: ([92, 83, 62], [135, 123, 88], [49, 44, 34]),
            accent: ([143, 128, 80], [190, 170, 111], [83, 75, 50]),
            window: ([76, 103, 87], [129, 157, 132], [38, 58, 49]),
            foliage: ([48, 132, 55], [92, 178, 88], [24, 77, 34]),
            trunk: ([92, 62, 40], [135, 95, 61], [48, 32, 22]),
        },
        MapKind::Night => PaletteColors {
            ground: ([45, 49, 60], [71, 76, 90], [25, 28, 36]),
            wall: ([67, 72, 84], [99, 107, 123], [35, 38, 48]),
            prop: ([75, 57, 46], [111, 84, 67], [39, 30, 26]),
            accent: ([73, 84, 103], [108, 126, 151], [35, 43, 56]),
            window: ([205, 151, 69], [245, 198, 101], [111, 75, 37]),
            foliage: ([33, 71, 58], [54, 103, 82], [17, 41, 35]),
            trunk: ([58, 46, 43], [87, 68, 62], [31, 25, 25]),
        },
    }
}

fn material(base_color: Color) -> StandardMaterial {
    StandardMaterial {
        base_color,
        perceptual_roughness: 0.95,
        metallic: 0.0,
        ..default()
    }
}

fn textured_material(texture: Handle<Image>, uv_scale: Vec2) -> StandardMaterial {
    StandardMaterial {
        base_color: Color::WHITE,
        base_color_texture: Some(texture),
        uv_transform: Affine2::from_scale(uv_scale),
        perceptual_roughness: 0.95,
        metallic: 0.0,
        ..default()
    }
}

fn procedural_texture(base: [u8; 3], light: [u8; 3], dark: [u8; 3], seed: u32) -> Image {
    const SIZE: usize = 32;
    let mut data = Vec::with_capacity(SIZE * SIZE * 4);

    for y in 0..SIZE {
        for x in 0..SIZE {
            let seam = y % 8 == 0 || ((y / 8) % 2 == 0 && x % 16 == 0);
            let hash = noise_hash(x as u32, y as u32, seed);
            let color = if seam {
                blend(base, dark, 0.62)
            } else if hash > 210 {
                blend(base, light, 0.42)
            } else if hash < 55 {
                blend(base, dark, 0.30)
            } else {
                base
            };

            data.extend_from_slice(&[color[0], color[1], color[2], 255]);
        }
    }

    let mut image = Image::new_fill(
        Extent3d {
            width: SIZE as u32,
            height: SIZE as u32,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        &data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD,
    );
    image.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
        address_mode_u: ImageAddressMode::Repeat,
        address_mode_v: ImageAddressMode::Repeat,
        mag_filter: ImageFilterMode::Nearest,
        min_filter: ImageFilterMode::Nearest,
        ..ImageSamplerDescriptor::linear()
    });
    image
}

fn noise_hash(x: u32, y: u32, seed: u32) -> u8 {
    let mut value = x
        .wrapping_mul(374_761_393)
        .wrapping_add(y.wrapping_mul(668_265_263))
        .wrapping_add(seed.wrapping_mul(2_246_822_519));
    value = (value ^ (value >> 13)).wrapping_mul(1_274_126_177);
    ((value ^ (value >> 16)) & 0xff) as u8
}

fn blend(a: [u8; 3], b: [u8; 3], amount: f32) -> [u8; 3] {
    [
        lerp_u8(a[0], b[0], amount),
        lerp_u8(a[1], b[1], amount),
        lerp_u8(a[2], b[2], amount),
    ]
}

fn lerp_u8(a: u8, b: u8, amount: f32) -> u8 {
    (a as f32 + (b as f32 - a as f32) * amount).round() as u8
}

fn yaw_toward(from: Vec3, target: Vec3) -> f32 {
    let direction = Vec2::new(target.x - from.x, target.z - from.z).normalize_or_zero();
    (-direction.x).atan2(-direction.y)
}
