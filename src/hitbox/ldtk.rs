use super::*;

/// Usage:
///
///```ignore
/// #[with(ldtk_hitbox)]
/// hitbox: Hitbox
/// ```
pub fn ldtk_hitbox(instance: &EntityInstance) -> Hitbox {
    let size = uvec2(instance.width as u32, instance.height as u32);
    Hitbox::new(size.x, size.y, Anchor(instance.pivot))
}
