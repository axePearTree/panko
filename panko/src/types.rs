pub type ResourceId = u32;

#[derive(Copy, Clone, Debug)]
pub struct TextureId(pub ResourceId);

#[derive(Copy, Clone, Debug)]
pub struct TextureData {
    pub id: TextureId,
    pub width: u32,
    pub height: u32,
}

#[derive(Copy, Clone, Debug)]
pub struct FontId(pub ResourceId);

#[derive(Copy, Clone, Debug)]
pub struct FontData {
    pub id: FontId,
    pub glyphs_height: u32,
}

#[derive(Copy, Clone, Debug)]
pub struct GlyphMetrics {
}

#[derive(Copy, Clone, Debug, Default)]
pub struct CopyTextureOptions {
    pub src: Option<Rect>,
    pub dest: Option<Rect>,
    pub center: Option<Point>,
    pub angle: f64,
    pub flip_h: bool,
    pub flip_v: bool,
}

#[derive(Copy, Clone, Debug)]
pub enum WindowConfig {
    Borderless(Dimensions),
    Bordered(Dimensions),
    Fullscreen,
}

#[derive(Copy, Clone, Debug)]
pub struct Dimensions {
    pub width: u32,
    pub height: u32,
}

#[derive(Copy, Clone, Debug)]
pub enum Event {
    KeyDown(Key),
    KeyUp(Key),
    Close,
}

#[rustfmt::skip]
#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Key {
    W, A, S, D,

    Count
}

#[derive(Copy, Clone, Debug, Default)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

impl Point {
    pub const fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
}

#[derive(Copy, Clone, Debug, Default)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub w: u32,
    pub h: u32,
}

impl Rect {
    pub const fn new(x: i32, y: i32, w: u32, h: u32) -> Self {
        Self { x, y, w, h }
    }
}

#[derive(Copy, Clone, Debug, Default)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    pub const BLACK: Self = Self::new(0, 0, 0, 0);
    pub const WHITE: Self = Self::new(255, 255, 255, 255);
    pub const GREEN: Self = Self::new(0, 255, 0, 255);

    pub const fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }
}
