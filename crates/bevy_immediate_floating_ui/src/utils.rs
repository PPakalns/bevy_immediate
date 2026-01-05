use bevy_math::bounding::Aabb2d;

/// Calculates if [`Aabb2d`] is fully inside another [`Aabb2d`]
pub fn fully_inside(inside: &Aabb2d, outside: &Aabb2d) -> bool {
    outside.min.x <= inside.min.x
        && inside.max.x <= outside.max.x
        && outside.min.y <= inside.min.y
        && inside.max.y <= outside.max.y
}

/// Calculates overlap of two [`Aabb2d`]. Check [`bevy_math::bounding::BoundingVolume::visible_area`]
/// to see if it even overlaps.
pub fn aabb_overlap(a: &Aabb2d, b: &Aabb2d) -> Aabb2d {
    Aabb2d {
        min: a.min.max(b.min),
        max: a.max.min(b.max),
    }
}
