//! Helpers for 2D pixel games on Bevy.
//!
//! Start with [`app::AppExt::setup_for_default_2d_pixel_game`]. It opens the
//! window, installs the pixel-perfect camera, LDtk, Aseprite animation,
//! movement, and UI plugins, and caps the frame rate at 60.
//!
//! [`prelude`] re-exports the pieces most games import. Collision, movement,
//! meshes, and LDtk grid queries live in their own modules.

pub mod sets;
pub mod app;
pub mod aseprite;
pub mod assets;
pub mod collision;
pub mod colors;
pub mod components;
mod debug_draw;
mod hitbox;
pub mod inputs;
pub mod ldtk;
pub mod lifecycle;
pub mod mesh;
pub mod movement;
pub mod parallax_bg;
pub mod pixel_perfect_cam;
pub mod prelude;
mod resources;
pub mod ui;
mod util;
