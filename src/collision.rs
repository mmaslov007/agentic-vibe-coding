use bevy::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Aabb2 {
    pub center: Vec2,
    pub half_extents: Vec2,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Aabb3 {
    pub center: Vec3,
    pub half_extents: Vec3,
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

impl Aabb3 {
    pub const fn new(center: Vec3, half_extents: Vec3) -> Self {
        Self {
            center,
            half_extents,
        }
    }

    pub fn from_center_size(center: Vec3, size: Vec3) -> Self {
        Self::new(center, size * 0.5)
    }

    pub fn ray_intersection_distance(
        self,
        origin: Vec3,
        direction: Vec3,
        max_distance: f32,
    ) -> Option<f32> {
        let min = self.center - self.half_extents;
        let max = self.center + self.half_extents;
        let mut t_min = 0.0;
        let mut t_max = max_distance;

        update_slab(origin.x, direction.x, min.x, max.x, &mut t_min, &mut t_max)?;
        update_slab(origin.y, direction.y, min.y, max.y, &mut t_min, &mut t_max)?;
        update_slab(origin.z, direction.z, min.z, max.z, &mut t_min, &mut t_max)?;

        Some(t_min)
    }
}

fn update_slab(
    origin: f32,
    direction: f32,
    min: f32,
    max: f32,
    t_min: &mut f32,
    t_max: &mut f32,
) -> Option<()> {
    if direction.abs() <= f32::EPSILON {
        return (origin >= min && origin <= max).then_some(());
    }

    let inverse = 1.0 / direction;
    let mut near = (min - origin) * inverse;
    let mut far = (max - origin) * inverse;

    if near > far {
        std::mem::swap(&mut near, &mut far);
    }

    *t_min = t_min.max(near);
    *t_max = t_max.min(far);

    (t_min <= t_max).then_some(())
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

    #[test]
    fn ray_reports_nearest_aabb_face() {
        let wall = Aabb3::from_center_size(Vec3::new(0.0, 1.0, -5.0), Vec3::new(2.0, 2.0, 2.0));
        let distance = wall.ray_intersection_distance(Vec3::new(0.0, 1.0, 0.0), Vec3::NEG_Z, 100.0);

        assert_eq!(distance, Some(4.0));
    }

    #[test]
    fn ray_misses_parallel_aabb() {
        let wall = Aabb3::from_center_size(Vec3::new(4.0, 1.0, -5.0), Vec3::new(2.0, 2.0, 2.0));
        let distance = wall.ray_intersection_distance(Vec3::new(0.0, 1.0, 0.0), Vec3::NEG_Z, 100.0);

        assert_eq!(distance, None);
    }
}
