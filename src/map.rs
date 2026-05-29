use bevy::prelude::*;

use crate::collision::Aabb2;

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

    spawn_lighting(&mut commands);
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
        Block::floor(Vec3::new(0.0, -0.05, 0.0), Vec3::new(64.0, 0.1, 54.0)),
        // Outer readable arena bounds.
        Block::wall(Vec3::new(0.0, 1.5, -27.0), Vec3::new(64.0, 3.0, 1.0)),
        Block::wall(Vec3::new(0.0, 1.5, 27.0), Vec3::new(64.0, 3.0, 1.0)),
        Block::wall(Vec3::new(-32.0, 1.5, 0.0), Vec3::new(1.0, 3.0, 54.0)),
        Block::wall(Vec3::new(32.0, 1.5, 0.0), Vec3::new(1.0, 3.0, 54.0)),
        // T spawn, long lane, and A-side elbow.
        Block::wall(Vec3::new(-21.0, 1.3, 15.0), Vec3::new(16.0, 2.6, 1.0)),
        Block::wall(Vec3::new(-13.5, 1.3, 20.5), Vec3::new(1.0, 2.6, 10.0)),
        Block::wall(Vec3::new(4.0, 1.3, 20.5), Vec3::new(24.0, 2.6, 1.0)),
        Block::wall(Vec3::new(15.5, 1.3, 15.0), Vec3::new(1.0, 2.6, 10.0)),
        Block::wall(Vec3::new(24.0, 1.3, 10.0), Vec3::new(15.0, 2.6, 1.0)),
        // Mid spine and doors.
        Block::wall(Vec3::new(-5.0, 1.35, 1.5), Vec3::new(22.0, 2.7, 1.0)),
        Block::wall(Vec3::new(11.0, 1.35, -2.5), Vec3::new(1.0, 2.7, 9.0)),
        Block::door(Vec3::new(2.5, 1.2, -2.8), Vec3::new(0.45, 2.4, 3.2)),
        Block::door(Vec3::new(5.0, 1.2, -2.8), Vec3::new(0.45, 2.4, 3.2)),
        Block::wall(Vec3::new(-18.0, 1.35, -8.0), Vec3::new(1.0, 2.7, 16.0)),
        Block::wall(Vec3::new(-10.0, 1.35, -15.5), Vec3::new(16.0, 2.7, 1.0)),
        // B tunnels and platform.
        Block::wall(Vec3::new(-23.0, 1.35, -20.0), Vec3::new(14.0, 2.7, 1.0)),
        Block::wall(Vec3::new(-16.5, 1.35, -21.5), Vec3::new(1.0, 2.7, 10.0)),
        Block::wall(Vec3::new(-28.0, 1.35, -10.0), Vec3::new(1.0, 2.7, 16.0)),
        Block::wall(Vec3::new(-24.0, 1.35, -2.0), Vec3::new(8.0, 2.7, 1.0)),
        // CT-side channels and A ramp suggestion.
        Block::wall(Vec3::new(19.0, 1.35, -13.0), Vec3::new(1.0, 2.7, 16.0)),
        Block::wall(Vec3::new(25.0, 1.35, -6.0), Vec3::new(12.0, 2.7, 1.0)),
        Block::wall(Vec3::new(24.0, 1.35, 3.5), Vec3::new(16.0, 2.7, 1.0)),
        Block::wall(Vec3::new(9.0, 1.35, 8.0), Vec3::new(1.0, 2.7, 10.0)),
        Block::wall(Vec3::new(3.0, 1.35, 13.0), Vec3::new(12.0, 2.7, 1.0)),
        // Bombsite-ish floor color pads.
        Block::site(Vec3::new(22.5, 0.01, 17.0), Vec3::new(11.0, 0.04, 8.0)),
        Block::site(Vec3::new(-23.0, 0.01, -15.0), Vec3::new(10.0, 0.04, 8.0)),
        // Basic cover props.
        Block::prop(Vec3::new(23.5, 0.65, 17.5), Vec3::new(2.5, 1.3, 2.5)),
        Block::prop(Vec3::new(18.8, 0.55, 14.5), Vec3::new(2.2, 1.1, 1.8)),
        Block::prop(Vec3::new(-24.5, 0.65, -15.0), Vec3::new(2.6, 1.3, 2.2)),
        Block::prop(Vec3::new(-20.0, 0.55, -11.8), Vec3::new(2.0, 1.1, 2.0)),
        Block::prop(Vec3::new(-2.0, 0.45, 5.5), Vec3::new(2.0, 0.9, 2.0)),
    ];

    add_arch_posts(&mut blocks, Vec3::new(-8.0, 1.4, 13.2));
    add_arch_posts(&mut blocks, Vec3::new(15.0, 1.4, -6.0));

    blocks
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
