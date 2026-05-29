use bevy::prelude::*;

use crate::collision::{Aabb2, Aabb3};
use crate::combat::{Hitbox, Shootable};

pub struct MapPlugin;

impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(MapColliders::default())
            .add_systems(Startup, spawn_map);
    }
}

#[derive(Resource, Default)]
pub struct MapColliders {
    pub walls: Vec<Aabb2>,
    pub shot_blockers: Vec<Aabb3>,
}

#[derive(Clone, Copy)]
enum BlockKind {
    Floor,
    Wall,
    Prop,
    Site,
    Door,
}

#[derive(Clone, Copy)]
struct Block {
    center: Vec3,
    size: Vec3,
    kind: BlockKind,
    collides: bool,
}

impl Block {
    const fn floor(center: Vec3, size: Vec3) -> Self {
        Self {
            center,
            size,
            kind: BlockKind::Floor,
            collides: false,
        }
    }

    const fn wall(center: Vec3, size: Vec3) -> Self {
        Self {
            center,
            size,
            kind: BlockKind::Wall,
            collides: true,
        }
    }

    const fn prop(center: Vec3, size: Vec3) -> Self {
        Self {
            center,
            size,
            kind: BlockKind::Prop,
            collides: true,
        }
    }

    const fn site(center: Vec3, size: Vec3) -> Self {
        Self {
            center,
            size,
            kind: BlockKind::Site,
            collides: false,
        }
    }

    const fn door(center: Vec3, size: Vec3) -> Self {
        Self {
            center,
            size,
            kind: BlockKind::Door,
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

fn spawn_map(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut colliders: ResMut<MapColliders>,
) {
    let palette = MapPalette::new(&mut materials);
    let cube = meshes.add(Cuboid::new(1.0, 1.0, 1.0));

    for block in dust_blockout() {
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
        ));
    }

    spawn_targets(&mut commands, &mut meshes, &mut materials);
    spawn_lighting(&mut commands);
}

fn spawn_targets(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) {
    let mesh = meshes.add(Cuboid::new(1.0, 1.0, 1.0));
    let target_material = materials.add(material(Color::srgb(0.82, 0.08, 0.06)));

    for target in shooting_targets() {
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
        ));
    }
}

fn spawn_lighting(commands: &mut Commands) {
    commands.spawn((
        DirectionalLight {
            illuminance: 32_000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(-8.0, 16.0, 8.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}

struct MapPalette {
    floor: Handle<StandardMaterial>,
    wall: Handle<StandardMaterial>,
    prop: Handle<StandardMaterial>,
    site: Handle<StandardMaterial>,
    door: Handle<StandardMaterial>,
}

impl MapPalette {
    fn new(materials: &mut Assets<StandardMaterial>) -> Self {
        Self {
            floor: materials.add(material(Color::srgb(0.72, 0.64, 0.48))),
            wall: materials.add(material(Color::srgb(0.78, 0.69, 0.52))),
            prop: materials.add(material(Color::srgb(0.49, 0.41, 0.31))),
            site: materials.add(material(Color::srgb(0.63, 0.53, 0.36))),
            door: materials.add(material(Color::srgb(0.34, 0.24, 0.15))),
        }
    }

    fn material_for(&self, kind: BlockKind) -> Handle<StandardMaterial> {
        match kind {
            BlockKind::Floor => self.floor.clone(),
            BlockKind::Wall => self.wall.clone(),
            BlockKind::Prop => self.prop.clone(),
            BlockKind::Site => self.site.clone(),
            BlockKind::Door => self.door.clone(),
        }
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

fn dust_blockout() -> Vec<Block> {
    let mut blocks = vec![
        Block::floor(Vec3::new(0.0, -0.05, 0.0), Vec3::new(90.0, 0.1, 78.0)),
        // Outer arena bounds. The routes inside are a Dust2-style blockout:
        // T spawn south, Long A west, A site northwest, Mid/Cat center,
        // CT spawn north-center, B tunnels east, and B site northeast.
        Block::wall(Vec3::new(0.0, 1.5, -39.0), Vec3::new(90.0, 3.0, 1.0)),
        Block::wall(Vec3::new(0.0, 1.5, 39.0), Vec3::new(90.0, 3.0, 1.0)),
        Block::wall(Vec3::new(-45.0, 1.5, 0.0), Vec3::new(1.0, 3.0, 78.0)),
        Block::wall(Vec3::new(45.0, 1.5, 0.0), Vec3::new(1.0, 3.0, 78.0)),
        // T spawn, outside long, and long doors.
        Block::wall(Vec3::new(-20.0, 1.35, 26.0), Vec3::new(1.0, 2.7, 24.0)),
        Block::wall(Vec3::new(14.0, 1.35, 27.0), Vec3::new(1.0, 2.7, 20.0)),
        Block::wall(Vec3::new(-31.5, 1.35, 3.0), Vec3::new(1.0, 2.7, 35.0)),
        Block::wall(Vec3::new(-37.5, 1.35, 19.8), Vec3::new(3.8, 2.7, 0.8)),
        Block::wall(Vec3::new(-31.2, 1.35, 19.8), Vec3::new(2.6, 2.7, 0.8)),
        Block::door(Vec3::new(-34.8, 1.25, 19.4), Vec3::new(0.45, 2.5, 2.7)),
        Block::door(Vec3::new(-33.2, 1.25, 19.4), Vec3::new(0.45, 2.5, 2.7)),
        Block::prop(Vec3::new(-35.0, 0.45, 29.0), Vec3::new(2.8, 0.9, 2.0)),
        // A Long, pit, car, cross, and A site.
        Block::wall(Vec3::new(-31.5, 1.35, -13.0), Vec3::new(1.0, 2.7, 13.0)),
        Block::wall(Vec3::new(-24.0, 1.35, -18.5), Vec3::new(14.0, 2.7, 1.0)),
        Block::wall(Vec3::new(-14.0, 1.35, -27.0), Vec3::new(1.0, 2.7, 17.0)),
        Block::wall(Vec3::new(-24.0, 1.35, -34.2), Vec3::new(23.0, 2.7, 1.0)),
        Block::wall(Vec3::new(-38.5, 0.8, -25.5), Vec3::new(5.0, 1.6, 5.0)),
        Block::prop(Vec3::new(-22.0, 0.7, -20.8), Vec3::new(3.0, 1.4, 1.8)),
        Block::prop(Vec3::new(-28.0, 0.55, -30.0), Vec3::new(2.4, 1.1, 2.2)),
        Block::prop(Vec3::new(-18.5, 0.55, -31.0), Vec3::new(2.2, 1.1, 2.2)),
        Block::site(Vec3::new(-24.5, 0.01, -28.5), Vec3::new(14.0, 0.04, 10.0)),
        // Mid, Xbox, mid doors, CT spawn, and CT-to-B/A rotations.
        Block::wall(Vec3::new(-7.5, 1.35, 18.0), Vec3::new(1.0, 2.7, 22.0)),
        Block::wall(Vec3::new(8.0, 1.35, 18.5), Vec3::new(1.0, 2.7, 19.0)),
        Block::wall(Vec3::new(-1.0, 1.35, 7.0), Vec3::new(12.0, 2.7, 1.0)),
        Block::prop(Vec3::new(-2.4, 0.65, 2.0), Vec3::new(3.0, 1.3, 2.2)),
        Block::door(Vec3::new(4.4, 1.25, -7.5), Vec3::new(0.45, 2.5, 3.4)),
        Block::door(Vec3::new(6.3, 1.25, -7.5), Vec3::new(0.45, 2.5, 3.4)),
        Block::wall(Vec3::new(10.0, 1.35, -9.0), Vec3::new(1.0, 2.7, 12.0)),
        Block::wall(Vec3::new(17.0, 1.35, -16.0), Vec3::new(14.0, 2.7, 1.0)),
        Block::wall(Vec3::new(2.5, 1.35, -21.5), Vec3::new(15.0, 2.7, 1.0)),
        Block::wall(Vec3::new(18.5, 1.35, -26.0), Vec3::new(1.0, 2.7, 13.0)),
        // Catwalk, short, and A short stairs.
        Block::wall(Vec3::new(-18.0, 1.35, -7.8), Vec3::new(20.0, 2.7, 1.0)),
        Block::wall(Vec3::new(-18.0, 1.35, -13.6), Vec3::new(20.0, 2.7, 1.0)),
        Block::wall(Vec3::new(-27.5, 1.35, -16.0), Vec3::new(1.0, 2.7, 6.0)),
        Block::decor(
            Vec3::new(-22.0, 0.18, -10.8),
            Vec3::new(9.0, 0.25, 4.3),
            BlockKind::Floor,
        ),
        Block::prop(Vec3::new(-24.0, 0.35, -14.8), Vec3::new(3.0, 0.7, 1.2)),
        // Outside tunnels, lower tunnels, upper tunnels, and B entrance.
        Block::wall(Vec3::new(21.0, 1.35, 18.0), Vec3::new(1.0, 2.7, 24.0)),
        Block::wall(Vec3::new(37.0, 1.35, 6.0), Vec3::new(1.0, 2.7, 42.0)),
        Block::wall(Vec3::new(23.5, 1.35, 19.0), Vec3::new(5.0, 2.7, 1.0)),
        Block::wall(Vec3::new(35.5, 1.35, 19.0), Vec3::new(3.0, 2.7, 1.0)),
        Block::wall(Vec3::new(23.5, 1.35, -12.0), Vec3::new(5.0, 2.7, 1.0)),
        Block::wall(Vec3::new(35.5, 1.35, -12.0), Vec3::new(3.0, 2.7, 1.0)),
        Block::wall(Vec3::new(14.5, 1.35, 5.2), Vec3::new(13.0, 2.7, 1.0)),
        Block::wall(Vec3::new(14.5, 1.35, 0.0), Vec3::new(13.0, 2.7, 1.0)),
        Block::wall(Vec3::new(21.0, 1.35, -5.0), Vec3::new(1.0, 2.7, 11.0)),
        Block::prop(Vec3::new(29.0, 0.55, 28.0), Vec3::new(2.4, 1.1, 2.2)),
        // B site, B doors, platform, car, and boxes.
        Block::wall(Vec3::new(24.0, 1.35, -18.5), Vec3::new(11.0, 2.7, 1.0)),
        Block::wall(Vec3::new(23.5, 1.35, -30.5), Vec3::new(1.0, 2.7, 16.0)),
        Block::wall(Vec3::new(34.5, 1.35, -31.5), Vec3::new(20.0, 2.7, 1.0)),
        Block::door(Vec3::new(20.7, 1.25, -18.5), Vec3::new(0.45, 2.5, 3.1)),
        Block::door(Vec3::new(22.4, 1.25, -18.5), Vec3::new(0.45, 2.5, 3.1)),
        Block::site(Vec3::new(32.5, 0.01, -25.5), Vec3::new(13.0, 0.04, 10.0)),
        Block::prop(Vec3::new(31.5, 0.75, -25.2), Vec3::new(3.0, 1.5, 2.8)),
        Block::prop(Vec3::new(36.5, 0.55, -23.0), Vec3::new(2.3, 1.1, 2.2)),
        Block::prop(Vec3::new(27.0, 0.55, -28.8), Vec3::new(2.4, 1.1, 2.0)),
        Block::prop(Vec3::new(39.0, 0.45, -28.0), Vec3::new(1.8, 0.9, 2.6)),
    ];

    add_arch_posts(&mut blocks, Vec3::new(-34.0, 1.4, 19.3));
    add_arch_posts(&mut blocks, Vec3::new(5.4, 1.4, -7.5));
    add_arch_posts(&mut blocks, Vec3::new(21.6, 1.4, -18.5));

    blocks
}

#[derive(Clone, Copy)]
struct TargetSpec {
    center: Vec3,
    size: Vec3,
    health: f32,
}

fn shooting_targets() -> [TargetSpec; 6] {
    [
        TargetSpec {
            center: Vec3::new(-36.0, 1.2, -8.0),
            size: Vec3::new(1.0, 1.7, 0.25),
            health: 100.0,
        },
        TargetSpec {
            center: Vec3::new(-21.5, 1.2, -29.0),
            size: Vec3::new(1.0, 1.7, 0.25),
            health: 100.0,
        },
        TargetSpec {
            center: Vec3::new(-13.0, 1.2, -10.8),
            size: Vec3::new(1.0, 1.7, 0.25),
            health: 100.0,
        },
        TargetSpec {
            center: Vec3::new(6.0, 1.2, -13.0),
            size: Vec3::new(1.0, 1.7, 0.25),
            health: 100.0,
        },
        TargetSpec {
            center: Vec3::new(28.0, 1.2, -22.5),
            size: Vec3::new(1.0, 1.7, 0.25),
            health: 100.0,
        },
        TargetSpec {
            center: Vec3::new(31.0, 1.2, 6.0),
            size: Vec3::new(1.0, 1.7, 0.25),
            health: 100.0,
        },
    ]
}

fn add_arch_posts(blocks: &mut Vec<Block>, center: Vec3) {
    blocks.push(Block::wall(
        Vec3::new(center.x - 1.8, center.y, center.z),
        Vec3::new(0.6, 2.8, 1.0),
    ));
    blocks.push(Block::wall(
        Vec3::new(center.x + 1.8, center.y, center.z),
        Vec3::new(0.6, 2.8, 1.0),
    ));
    blocks.push(Block::decor(
        Vec3::new(center.x, center.y + 1.2, center.z),
        Vec3::new(4.2, 0.5, 1.0),
        BlockKind::Wall,
    ));
}
