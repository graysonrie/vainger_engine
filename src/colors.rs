use crate::prelude::*;

/// UFO 50's 32-color palette, left to right from the 1x strip.
///
/// Channels are the sRGB bytes from that image. Black and white stay in linear
/// RGB so they match [`Color::BLACK`] and [`Color::WHITE`].
pub const UFO_WHITE: Color = Color::linear_rgb(1., 1., 1.);
pub const UFO_DUSTY_ROSE: Color = Color::srgb_u8(164, 128, 128);
pub const UFO_PEACH: Color = Color::srgb_u8(254, 184, 84);
pub const UFO_YELLOW: Color = Color::srgb_u8(232, 234, 74);
pub const UFO_MINT: Color = Color::srgb_u8(88, 245, 177);
pub const UFO_TEAL: Color = Color::srgb_u8(100, 164, 164);
pub const UFO_LILAC: Color = Color::srgb_u8(204, 104, 228);
pub const UFO_SALMON: Color = Color::srgb_u8(254, 98, 110);

pub const UFO_LIGHT_GRAY: Color = Color::srgb_u8(200, 200, 200);
pub const UFO_RED: Color = Color::srgb_u8(224, 60, 50);
pub const UFO_ORANGE: Color = Color::srgb_u8(254, 112, 0);
pub const UFO_GREEN: Color = Color::srgb_u8(99, 179, 29);
pub const UFO_LIME: Color = Color::srgb_u8(164, 240, 34);
pub const UFO_CYAN: Color = Color::srgb_u8(39, 186, 219);
pub const UFO_MAGENTA: Color = Color::srgb_u8(254, 72, 222);
pub const UFO_CRIMSON: Color = Color::srgb_u8(209, 15, 76);

pub const UFO_GRAY: Color = Color::srgb_u8(112, 112, 112);
pub const UFO_DARK_RED: Color = Color::srgb_u8(153, 21, 21);
pub const UFO_RUST: Color = Color::srgb_u8(202, 74, 0);
pub const UFO_OLIVE: Color = Color::srgb_u8(163, 163, 36);
pub const UFO_DARK_GREEN: Color = Color::srgb_u8(0, 132, 86);
pub const UFO_BLUE: Color = Color::srgb_u8(0, 106, 180);
pub const UFO_PURPLE: Color = Color::srgb_u8(150, 0, 220);
pub const UFO_WINE: Color = Color::srgb_u8(134, 22, 80);

pub const UFO_BLACK: Color = Color::linear_rgb(0., 0., 0.);
pub const UFO_MAROON: Color = Color::srgb_u8(76, 0, 0);
pub const UFO_PLUM: Color = Color::srgb_u8(120, 52, 80);
pub const UFO_BROWN: Color = Color::srgb_u8(138, 96, 66);
pub const UFO_FOREST: Color = Color::srgb_u8(0, 61, 16);
pub const UFO_NAVY: Color = Color::srgb_u8(32, 32, 64);
pub const UFO_EGGPLANT: Color = Color::srgb_u8(52, 0, 88);
pub const UFO_INDIGO: Color = Color::srgb_u8(68, 48, 186);
