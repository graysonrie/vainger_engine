use crate::prelude::*;
mod ufo_fonts;

/// Resolved default bitmap font, built after [`EngineArtHandles`] finishes loading.
#[derive(Resource, Clone)]
pub struct EngineFonts {
    pub default: BitmapFont,
}

/// An 8×8 atlas font. Glyph index `n` is codepoint `32 + n`.
#[derive(Clone)]
pub struct BitmapFont {
    pub image: Handle<Image>,
    pub layout: Handle<TextureAtlasLayout>,
    pub glyph_size: UVec2,
    first_code: u32,
    last_code: u32,
}

impl BitmapFont {
    pub fn glyph_index(&self, c: char) -> Option<usize> {
        let code = c as u32;
        if (self.first_code..=self.last_code).contains(&code) {
            Some((code - self.first_code) as usize)
        } else {
            None
        }
    }
}

/// This impl is required so that `bevy_asset_loader` can init the EngineFonts
/// resource after the handles get loaded  
impl FromWorld for EngineFonts {
    fn from_world(world: &mut World) -> Self {
        let default = ufo_fonts::default_font(world);

        Self { default }
    }
}

/// Pixel-perfect UI text using the engine bitmap font.
///
/// Spawn on the default UI camera (the low-res game canvas). Mutate `text`,
/// `color`, `scale`, or `max_width` and the glyph children are rebuilt
/// automatically.
///
/// ```ignore
/// commands.spawn((
///     BitmapText::new("SCORE 0000")
///         .with_color(Color::WHITE)
///         .with_max_width(144),
///     Node {
///         position_type: PositionType::Absolute,
///         top: Val::Px(4.0),
///         left: Val::Px(4.0),
///         ..default()
///     },
/// ));
/// ```
///
/// Icon glyphs live in slots `128..=252`. Insert them with `code as char`
/// (GameMaker `chr(n)`).
#[derive(Component, Clone, Debug)]
#[require(Node)]
pub struct BitmapText {
    pub text: String,
    pub color: Color,
    /// Integer multiple of the 8px glyph size. Values below 1 are treated as 1.
    pub scale: u32,
    /// Optional wrap width in canvas pixels. `None` wraps only on `\n`.
    /// Floored to a whole number of glyphs; values narrower than one glyph are ignored.
    pub max_width: Option<u32>,
}

impl Default for BitmapText {
    fn default() -> Self {
        Self {
            text: String::new(),
            color: Color::WHITE,
            scale: 1,
            max_width: None,
        }
    }
}

impl BitmapText {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            ..default()
        }
    }

    #[must_use]
    pub fn with_color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    #[must_use]
    pub fn with_scale(mut self, scale: u32) -> Self {
        self.scale = scale.max(1);
        self
    }

    /// Enables word-wrapping, setting the max_width (in pixels) that the
    /// text box can go
    #[must_use]
    pub fn with_max_width(mut self, max_width: u32) -> Self {
        self.max_width = Some(max_width);
        self
    }
}

/// Column of glyph rows owned by a [`BitmapText`] entity.
#[derive(Component)]
struct BitmapTextGlyphs;

/// Handles rebuilding nodes with `BitmapText`
pub struct BitmapTextPlugin;

impl Plugin for BitmapTextPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            rebuild_bitmap_text.run_if(resource_exists::<EngineFonts>),
        );
    }
}

/// The way it works is:
/// Whenever `BitmapText` is inserted into an entity, an EngineFont is changed, or a 
/// `BitmapText` is edited, then the glyphs are rebuilt
fn rebuild_bitmap_text(
    mut commands: Commands,
    fonts: Res<EngineFonts>,
    texts: Query<(Entity, Ref<BitmapText>, Option<&Children>)>,
    glyph_roots: Query<(), With<BitmapTextGlyphs>>,
) {
    let font_added = fonts.is_added();
    for (entity, text, children) in &texts {
        if font_added || text.is_changed() {
            rebuild_glyphs(&mut commands, entity, &text, children, &fonts, &glyph_roots);
        }
    }
}

fn rebuild_glyphs(
    commands: &mut Commands,
    entity: Entity,
    text: &BitmapText,
    children: Option<&Children>,
    fonts: &EngineFonts,
    glyph_roots: &Query<(), With<BitmapTextGlyphs>>,
) {
    if let Some(children) = children {
        for &child in children {
            if glyph_roots.contains(child) {
                commands.entity(child).despawn();
            }
        }
    }

    let font = &fonts.default;
    let scale = text.scale.max(1);
    let glyph_px = font.glyph_size.x * scale;
    let glyph_px_f = glyph_px as f32;
    let max_cols = text.max_width.and_then(|width| {
        let cols = width / glyph_px;
        (cols > 0).then_some(cols as usize)
    });
    let image = font.image.clone();
    let layout = font.layout.clone();
    let color = text.color;
    let lines = wrapped_lines(&text.text, max_cols);

    commands.entity(entity).with_children(|parent| {
        parent
            .spawn((
                Node {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Start,
                    max_width: text
                        .max_width
                        .map(|width| Val::Px(width as f32))
                        .unwrap_or(Val::Auto),
                    ..default()
                },
                BitmapTextGlyphs,
            ))
            .with_children(|column| {
                for line in &lines {
                    column
                        .spawn(Node {
                            flex_direction: FlexDirection::Row,
                            min_height: Val::Px(glyph_px_f),
                            ..default()
                        })
                        .with_children(|row| {
                            for ch in line.chars() {
                                let (index, repeats) = match ch {
                                    '\t' => (font.glyph_index(' '), 2),
                                    ch => {
                                        (font.glyph_index(ch).or_else(|| font.glyph_index('?')), 1)
                                    }
                                };
                                let Some(index) = index else {
                                    continue;
                                };
                                for _ in 0..repeats {
                                    row.spawn((
                                        ImageNode::from_atlas_image(
                                            image.clone(),
                                            TextureAtlas {
                                                layout: layout.clone(),
                                                index,
                                            },
                                        )
                                        .with_color(color)
                                        .with_mode(NodeImageMode::Stretch),
                                        Node {
                                            width: Val::Px(glyph_px_f),
                                            height: Val::Px(glyph_px_f),
                                            flex_shrink: 0.0,
                                            ..default()
                                        },
                                    ));
                                }
                            }
                        });
                }
            });
    });
}

fn wrapped_lines(text: &str, max_cols: Option<usize>) -> Vec<String> {
    let text = text.replace('\r', "");
    match max_cols {
        None | Some(0) => text.split('\n').map(str::to_string).collect(),
        Some(max_cols) => text
            .split('\n')
            .flat_map(|paragraph| wrap_paragraph(paragraph, max_cols))
            .collect(),
    }
}

fn wrap_paragraph(paragraph: &str, max_cols: usize) -> Vec<String> {
    let paragraph = paragraph.replace('\t', "  ");
    let words: Vec<&str> = paragraph.split_whitespace().collect();
    if words.is_empty() {
        return vec![String::new()];
    }

    let mut lines = Vec::new();
    let mut current = String::new();
    for word in words {
        if current.is_empty() {
            push_word(&mut lines, &mut current, word, max_cols);
            continue;
        }
        if current.chars().count() + 1 + word.chars().count() <= max_cols {
            current.push(' ');
            current.push_str(word);
        } else {
            lines.push(std::mem::take(&mut current));
            push_word(&mut lines, &mut current, word, max_cols);
        }
    }
    if !current.is_empty() {
        lines.push(current);
    }
    lines
}

fn push_word(lines: &mut Vec<String>, current: &mut String, word: &str, max_cols: usize) {
    let mut rest = word;
    loop {
        let remaining = max_cols.saturating_sub(current.chars().count());
        if remaining == 0 {
            lines.push(std::mem::take(current));
            continue;
        }
        if rest.chars().count() <= remaining {
            current.push_str(rest);
            break;
        }
        let (head, tail) = split_at_chars(rest, remaining);
        current.push_str(head);
        lines.push(std::mem::take(current));
        rest = tail;
    }
}

fn split_at_chars(s: &str, n: usize) -> (&str, &str) {
    match s.char_indices().nth(n) {
        Some((i, _)) => s.split_at(i),
        None => (s, ""),
    }
}

#[cfg(test)]
mod tests {
    use super::wrapped_lines;

    #[test]
    fn no_max_width_keeps_hard_breaks() {
        assert_eq!(
            wrapped_lines("one\ntwo", None),
            vec!["one".to_string(), "two".to_string()]
        );
    }

    #[test]
    fn wraps_on_spaces() {
        assert_eq!(
            wrapped_lines("the quick brown fox", Some(10)),
            vec!["the quick".to_string(), "brown fox".to_string()]
        );
    }

    #[test]
    fn hard_breaks_then_wraps() {
        assert_eq!(
            wrapped_lines("hello world\nfoo bar baz", Some(8)),
            vec![
                "hello".to_string(),
                "world".to_string(),
                "foo bar".to_string(),
                "baz".to_string()
            ]
        );
    }

    #[test]
    fn preserves_blank_lines() {
        assert_eq!(
            wrapped_lines("a\n\nb", Some(8)),
            vec!["a".to_string(), String::new(), "b".to_string()]
        );
    }

    #[test]
    fn breaks_long_words() {
        assert_eq!(
            wrapped_lines("abcdefghij", Some(4)),
            vec!["abcd".to_string(), "efgh".to_string(), "ij".to_string()]
        );
    }
}
