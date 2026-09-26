use crate::prelude::*;

use super::{ParallaxLayerNode, ParallaxScroller};

pub(crate) fn update_parallax_scroll(
    camera: Single<&Transform, With<PixelPerfectCamera>>,
    config: Res<PixelPerfectCameraConfig>,
    time: Res<Time>,
    mut layers: Query<
        (&ParallaxLayerNode, &mut ParallaxScroller, &mut Transform),
        Without<PixelPerfectCamera>,
    >,
) {
    let camera_xy = camera.get_2d_pos();
    let view = Vec2::new(config.game_width, config.game_height);
    let dt = time.delta_secs();

    for (layer, mut scroller, mut transform) in &mut layers {
        let speed = scroller.speed;
        scroller.offset += speed * dt;
        let offset = camera_xy * layer.scroll + scroller.offset;
        let origin = covering_origin(camera_xy, offset, layer.tile_size, view);
        transform.translation = origin.extend(layer.z);
    }
}

/// Phase-aligned tile-(0,0) center, shifted so the grid covers the viewport.
fn covering_origin(camera: Vec2, offset: Vec2, tile_size: Vec2, view: Vec2) -> Vec2 {
    let tile = Vec2::new(tile_size.x.max(1.0), tile_size.y.max(1.0));
    let wrapped = Vec2::new(offset.x.rem_euclid(tile.x), offset.y.rem_euclid(tile.y));
    let mut origin = camera - wrapped;
    let view_min = camera - view * 0.5;

    let left = origin.x - tile.x * 0.5;
    if left > view_min.x {
        origin.x -= ((left - view_min.x) / tile.x).ceil() * tile.x;
    }

    let bottom = origin.y - tile.y * 0.5;
    if bottom > view_min.y {
        origin.y -= ((bottom - view_min.y) / tile.y).ceil() * tile.y;
    }

    origin
}
