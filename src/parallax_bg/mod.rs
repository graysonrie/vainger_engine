//! Parallax backgrounds from a single Aseprite file.
//!
//! Aseprite **document layers are flattened** by the loader. Author each
//! parallax piece as a **tag** (animated full-canvas frames) or a **slice**
//! (named rect on the sheet).
//!
//! The game crate supplies the `.aseprite` path and a [`LoadingState`]; this
//! plugin loads that sheet and spawns the configured stacks.

mod fade;
mod scroll;

use bevy::asset::UntypedHandle;
use bevy::platform::collections::HashMap;
use bevy::state::state::FreelyMutableState;

use crate::aseprite::AseInfo;
use crate::prelude::*;

pub use fade::CrossfadeBackground;

/// Where a layer's pixels come from inside the aseprite file.
#[derive(Clone, Copy, Debug)]
pub enum LayerSource {
    Tag(&'static str),
    Slice(&'static str),
}

/// One repeating parallax layer inside a [`BackgroundDef`].
#[derive(Clone, Copy, Debug)]
pub struct ParallaxLayer {
    pub source: LayerSource,
    pub z: f32,
    pub scroll: Vec2,
    pub auto_scroll: Vec2,
}

impl ParallaxLayer {
    pub fn tag(name: &'static str) -> Self {
        Self {
            source: LayerSource::Tag(name),
            z: -10.0,
            scroll: Vec2::ZERO,
            auto_scroll: Vec2::ZERO,
        }
    }

    pub fn slice(name: &'static str) -> Self {
        Self {
            source: LayerSource::Slice(name),
            z: -10.0,
            scroll: Vec2::ZERO,
            auto_scroll: Vec2::ZERO,
        }
    }

    pub fn z(mut self, z: f32) -> Self {
        self.z = z;
        self
    }

    /// `self.z(-12.)`
    pub fn z_far(self) -> Self {
        self.z(-12.)
    }

    /// `self.z(-8.)`
    pub fn z_mid(self) -> Self {
        self.z(-8.)
    }

    /// `self.z(-6.)`
    pub fn z_near(self) -> Self {
        self.z(-6.)
    }

    /// Camera-relative scroll factor. `0` locks to the camera, `1` locks to the world.
    pub fn scroll(mut self, x: f32, y: f32) -> Self {
        self.scroll = Vec2::new(x, y);
        self
    }

    /// Extra motion in pixels per second, wrapped with the layer.
    pub fn auto_scroll(mut self, x: f32, y: f32) -> Self {
        self.auto_scroll = Vec2::new(x, y);
        self
    }
}

/// A named stack of parallax layers that can be shown or crossfaded.
#[derive(Clone, Debug)]
pub struct BackgroundDef {
    pub id: &'static str,
    pub layers: Vec<ParallaxLayer>,
}

impl BackgroundDef {
    /// Define the name (id) of the background. Use `.layer` next to add layers to it
    pub fn new(id: &'static str) -> Self {
        Self {
            id,
            layers: Vec::new(),
        }
    }

    pub fn layer(mut self, layer: ParallaxLayer) -> Self {
        self.layers.push(layer);
        self
    }
}

/// Loaded aseprite sheet used by every parallax stack.
#[derive(Resource, Clone)]
pub struct ParallaxAseprite {
    pub handle: Handle<Aseprite>,
}

#[derive(Resource, Clone, Copy)]
struct ParallaxAsepritePath(&'static str);

impl AssetCollection for ParallaxAseprite {
    fn create(world: &mut World) -> Self {
        let path = world.resource::<ParallaxAsepritePath>().0;
        let handle = world.resource::<AssetServer>().load(path);
        Self { handle }
    }

    fn load(world: &mut World) -> Vec<UntypedHandle> {
        let path = world.resource::<ParallaxAsepritePath>().0;
        let handle: Handle<Aseprite> = world.resource::<AssetServer>().load(path);
        vec![handle.untyped()]
    }
}

#[derive(Resource, Clone)]
pub(crate) struct BackgroundCatalog {
    defs: HashMap<&'static str, BackgroundDef>,
    initial: &'static str,
}

impl BackgroundCatalog {
    fn new(defs: &[BackgroundDef], initial: Option<&'static str>) -> Self {
        let mut map = HashMap::new();
        for def in defs {
            map.insert(def.id, def.clone());
        }
        let initial = initial
            .or_else(|| defs.first().map(|def| def.id))
            .unwrap_or("");
        Self { defs: map, initial }
    }

    pub(crate) fn get(&self, id: &str) -> Option<&BackgroundDef> {
        self.defs.get(id)
    }
}

/// Root of a spawned background stack.
#[derive(Component)]
pub(crate) struct ParallaxBackground {
    pub id: &'static str,
}

/// Parent of the repeating tile copies for one authored layer.
#[derive(Component)]
pub(crate) struct ParallaxLayerNode {
    pub scroll: Vec2,
    pub tile_size: Vec2,
    pub z: f32,
}

/// Accumulated auto-scroll offset (pixels).
#[derive(Component)]
pub(crate) struct ParallaxScroller {
    pub offset: Vec2,
    pub speed: Vec2,
}

/// A single wrap-copy sprite belonging to `root`.
#[derive(Component)]
pub(crate) struct ParallaxTile {
    pub root: Entity,
}

#[derive(Resource, Default)]
pub(crate) struct ActiveBackgrounds {
    pub current: Option<Entity>,
    pub fade: Option<BackgroundFade>,
}

pub(crate) struct BackgroundFade {
    pub from: Entity,
    pub to: Entity,
    pub elapsed: f32,
    pub duration: f32,
}

/// 2D parallax backgrounds from a single Aseprite file.
///
/// Pass the game's loading state and the path of the sheet. `load_state` must
/// already have a [`LoadingState`] registered (see the game crate's `main`).
///
/// ```ignore
/// app.add_plugins(
///     ParallaxBackgroundPlugin::new(
///         AppState::InitialLoad,
///         "art/backgrounds/bgs.aseprite",
///         [BackgroundDef::new("surface").layer(ParallaxLayer::tag("sky"))],
///     )
///     .with_initial("surface"),
/// );
/// ```
pub struct ParallaxBackgroundPlugin<S: FreelyMutableState> {
    load_state: S,
    aseprite_path: &'static str,
    defs: Vec<BackgroundDef>,
    initial: Option<&'static str>,
}

impl<S: FreelyMutableState> ParallaxBackgroundPlugin<S> {
    pub fn new(
        load_state: S,
        aseprite_path: &'static str,
        defs: impl IntoIterator<Item = BackgroundDef>,
    ) -> Self {
        Self {
            load_state,
            aseprite_path,
            defs: defs.into_iter().collect(),
            initial: None,
        }
    }

    pub fn with_initial(mut self, id: &'static str) -> Self {
        self.initial = Some(id);
        self
    }
}

impl<S: FreelyMutableState> Plugin for ParallaxBackgroundPlugin<S> {
    fn build(&self, app: &mut App) {
        app.insert_resource(ParallaxAsepritePath(self.aseprite_path))
            .insert_resource(BackgroundCatalog::new(&self.defs, self.initial))
            .init_resource::<ActiveBackgrounds>()
            .add_message::<CrossfadeBackground>()
            .configure_loading_state(
                LoadingStateConfig::new(self.load_state.clone())
                    .load_collection::<ParallaxAseprite>(),
            )
            .add_systems(
                Update,
                (
                    spawn_initial_background.run_if(needs_initial_background),
                    fade::handle_crossfade_messages,
                )
                    .chain()
                    .run_if(resource_exists::<ParallaxAseprite>),
            )
            .add_systems(
                Last,
                (scroll::update_parallax_scroll, fade::apply_background_fade)
                    .chain()
                    .run_if(
                        resource_exists::<ParallaxAseprite>
                            .and_then(any_with_component::<PixelPerfectCamera>),
                    ),
            );
    }
}

fn needs_initial_background(active: Res<ActiveBackgrounds>) -> bool {
    active.current.is_none() && active.fade.is_none()
}

#[derive(SystemParam)]
pub(crate) struct BackgroundSpawner<'w, 's> {
    commands: Commands<'w, 's>,
    aseprite: Res<'w, ParallaxAseprite>,
    ase_info: AseInfo<'w>,
    camera_config: Res<'w, PixelPerfectCameraConfig>,
    catalog: Res<'w, BackgroundCatalog>,
}

impl BackgroundSpawner<'_, '_> {
    pub(crate) fn spawn(&mut self, id: &str, alpha: f32) -> Option<Entity> {
        let Some(def) = self.catalog.get(id).cloned() else {
            warn!("unknown background `{id}`");
            return None;
        };

        let ase_handle = self.aseprite.handle.clone();
        if self.ase_info.get(&ase_handle).is_none() {
            warn!("parallax aseprite was not loaded; skipping background `{id}`");
            return None;
        }

        let view = Vec2::new(
            self.camera_config.game_width,
            self.camera_config.game_height,
        );

        let root = self
            .commands
            .spawn((
                ParallaxBackground { id: def.id },
                Transform::default(),
                Visibility::default(),
            ))
            .id();

        for layer in &def.layers {
            let Some(tile_size) = tile_size_for_layer(&self.ase_info, &ase_handle, layer.source)
            else {
                match layer.source {
                    LayerSource::Tag(name) => {
                        warn!("background `{id}`: missing aseprite tag `{name}`");
                    }
                    LayerSource::Slice(name) => {
                        warn!("background `{id}`: missing aseprite slice `{name}`");
                    }
                }
                continue;
            };

            if tile_size.x <= 0.0 || tile_size.y <= 0.0 {
                warn!("background `{id}`: layer tile size is zero, skipping");
                continue;
            }

            let tiles_x = tile_count(view.x, tile_size.x);
            let tiles_y = tile_count(view.y, tile_size.y);

            self.commands.entity(root).with_children(|parent| {
                parent
                    .spawn((
                        ParallaxLayerNode {
                            scroll: layer.scroll,
                            tile_size,
                            z: layer.z,
                        },
                        ParallaxScroller {
                            offset: Vec2::ZERO,
                            speed: layer.auto_scroll,
                        },
                        Transform::from_z(layer.z),
                        Visibility::default(),
                    ))
                    .with_children(|layer_parent| {
                        for iy in 0..tiles_y {
                            for ix in 0..tiles_x {
                                let mut tile = layer_parent.spawn((
                                    ParallaxTile { root },
                                    Transform::from_xyz(
                                        ix as f32 * tile_size.x,
                                        iy as f32 * tile_size.y,
                                        0.0,
                                    ),
                                    Sprite {
                                        color: Color::srgba(1.0, 1.0, 1.0, alpha),
                                        ..default()
                                    },
                                ));
                                match layer.source {
                                    LayerSource::Tag(name) => {
                                        tile.insert(AseAnimation {
                                            aseprite: ase_handle.clone(),
                                            animation: Animation::tag(name),
                                        });
                                    }
                                    LayerSource::Slice(name) => {
                                        tile.insert(AseSlice {
                                            name: name.to_string(),
                                            aseprite: ase_handle.clone(),
                                        });
                                    }
                                }
                            }
                        }
                    });
            });
        }

        Some(root)
    }

    pub(crate) fn despawn(&mut self, entity: Entity) {
        self.commands.entity(entity).despawn();
    }
}

fn spawn_initial_background(mut spawner: BackgroundSpawner, mut active: ResMut<ActiveBackgrounds>) {
    let id = spawner.catalog.initial;
    if id.is_empty() {
        warn!("ParallaxBackgroundPlugin has no catalog entries");
        return;
    }
    active.current = spawner.spawn(id, 1.0);
}

fn tile_size_for_layer(
    ase_info: &AseInfo,
    handle: &Handle<Aseprite>,
    source: LayerSource,
) -> Option<Vec2> {
    match source {
        LayerSource::Slice(name) => ase_info.size_of_slice(handle, name),
        LayerSource::Tag(name) => ase_info.size_of_tag(handle, name),
    }
}

fn tile_count(view: f32, tile: f32) -> u32 {
    let tile = tile.max(1.0);
    (view / tile).ceil() as u32 + 2
}
