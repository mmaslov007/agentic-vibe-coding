use bevy::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Aabb2 {
    pub center: Vec2,
    pub half_extents: Vec2,
}

impl Aabb2 {
    pub const fn new(center: Vec2, half_extents: Vec2) -> Self {
        Self {
            center,
            half_extents,
        }
    }

    pub fn from_center_size(center: Vec2, size: Vec2) -> Self {
        Self::new(center, size * 0.5)
    }

    pub fn intersects(self, other: Self) -> bool {
        let delta = (self.center - other.center).abs();
        delta.x < self.half_extents.x + other.half_extents.x
            && delta.y < self.half_extents.y + other.half_extents.y
    }
}

pub fn move_circle_through_aabbs(
    start: Vec2,
    movement: Vec2,
    radius: f32,
    colliders: &[Aabb2],
) -> Vec2 {
    let mut next = start;

    next.x += movement.x;
    if hits_any(next, radius, colliders) {
        next.x = start.x;
    }

    next.y += movement.y;
    if hits_any(next, radius, colliders) {
        next.y = start.y;
    }

    next
}

fn hits_any(center: Vec2, radius: f32, colliders: &[Aabb2]) -> bool {
    let player = Aabb2::new(center, Vec2::splat(radius));
    colliders
        .iter()
        .any(|collider| player.intersects(*collider))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn aabbs_intersect_when_overlapping() {
        let a = Aabb2::from_center_size(Vec2::ZERO, Vec2::new(2.0, 2.0));
        let b = Aabb2::from_center_size(Vec2::new(0.75, 0.0), Vec2::new(2.0, 2.0));

        assert!(a.intersects(b));
    }

    #[test]
    fn movement_slides_along_blocked_axis() {
        let wall = Aabb2::from_center_size(Vec2::new(2.0, 0.0), Vec2::new(1.0, 4.0));
        let moved = move_circle_through_aabbs(Vec2::ZERO, Vec2::new(1.75, 0.5), 0.35, &[wall]);

        assert_eq!(moved, Vec2::new(0.0, 0.5));
    }
}
