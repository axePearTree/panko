use crate::canvas::Canvas;
use crate::types::{FontId, GlyphMetrics};
use crate::{BackendWeakRef, Point, Rect, Result, Texture, TextureId};
use alloc::vec::Vec;
use core::cell::RefCell;
use core::str::Chars;
use hashbrown::HashMap;

const ATLAS_WIDTH: u32 = 1024;
const ATLAS_HEIGHT: u32 = 1024;

pub struct Font(RefCell<FontInner>);

impl Font {
    pub(crate) fn atlas(&self, index: usize) -> Option<TextureId> {
        self.0.borrow().atlases.get(index).map(|a| a.texture.id)
    }

    pub(crate) fn register_text(&self, text: &str, canvas: &Canvas) -> Result {
        self.0.borrow_mut().register_glyphs(text, canvas)
    }
}

struct FontInner {
    id: FontId,
    scale: f32,
    glyph_height: u32,
    backend: BackendWeakRef,
    atlases: Vec<FontAtlas>,
    entries: HashMap<char, FontGlyphEntry>,
}

impl FontInner {
    fn draw_text(&mut self, canvas: &Canvas, text: &str, position: Point) -> Result {
        self.register_glyphs(text, canvas)
    }

    fn register_glyphs(&mut self, text: &str, canvas: &Canvas<'_>) -> Result {
        let mut glyphs = text.chars();
        let mut atlas_index = self.atlases.len() - 1;
        let mut atlas = &mut self.atlases[atlas_index];
        loop {
            if register_glyphs(self.id, atlas, canvas, &mut self.entries, &mut glyphs)? {
                break;
            } else {
                self.atlases.push(FontAtlas::new(
                    &self.backend,
                    ATLAS_WIDTH,
                    ATLAS_HEIGHT,
                    self.glyph_height,
                )?);
                atlas_index += 1;
                atlas = &mut self.atlases[atlas_index];
            }
        }
        Ok(())
    }
}

struct FontGlyphEntry {
    atlas_index: usize,
    rect: Rect,
    metrics: GlyphMetrics,
}

struct FontAtlas {
    texture: Texture,
    glyph_height: u32,
    x_cursor: u32,
    y_cursor: u32,
    full: bool,
}

impl FontAtlas {
    fn new(backend: &BackendWeakRef, width: u32, height: u32, glyph_height: u32) -> Result<Self> {
        let backend = backend.upgrade().unwrap();
        let texture = Texture::new_target(&backend, width, height)?;
        Ok(Self {
            texture,
            glyph_height,
            x_cursor: 0,
            y_cursor: 0,
            full: false,
        })
    }
}

fn register_glyphs(
    font_id: FontId,
    atlas: &mut FontAtlas,
    canvas: &Canvas,
    entries: &mut HashMap<char, FontGlyphEntry>,
    glyphs: &mut Chars,
) -> Result<bool> {
    let mut finished = false;
    let atlas_width = atlas.texture.width();
    let atlas_height = atlas.texture.height();
    canvas.with_target(Some(&mut atlas.texture), |canvas| {
        while let Some(glyph) = glyphs.next() {
            if entries.contains_key(&glyph) {
                continue;
            }
            let metrics = canvas.glyph_metrics(font_id, glyph)?;

            if atlas.x_cursor + metrics.advance > atlas_width {
                // go to next line
                atlas.x_cursor = 0;
                atlas.y_cursor += atlas.glyph_height;
                if atlas.y_cursor + atlas.glyph_height > atlas_height {
                    // atlas is full
                    return Ok(());
                }
            }

            // render the glyph to this target texture...
            canvas.render_glyph(
                font_id,
                glyph,
                Point::new(atlas.x_cursor as i32, atlas.y_cursor as i32),
            )?;
        }
        finished = true;
        Ok(())
    })?;
    Ok(finished)
}
