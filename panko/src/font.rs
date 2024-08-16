use core::str::Chars;

use crate::canvas::Canvas;
use crate::types::{FontId, GlyphMetrics};
use crate::{BackendWeakRef, Point, Rect, Result, Texture};
use alloc::vec::Vec;
use hashbrown::HashMap;

const ATLAS_WIDTH: u32 = 1024;
const ATLAS_HEIGHT: u32 = 1024;

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
            if register_glyphs(atlas, canvas, &mut self.entries, &mut glyphs)? {
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
    x_cursor: i32,
    y_cursor: i32,
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
    atlas: &mut FontAtlas,
    canvas: &Canvas,
    entries: &mut HashMap<char, FontGlyphEntry>,
    glyphs: &mut Chars,
) -> Result<bool> {
    let mut finished = false;
    canvas.with_target(Some(&mut atlas.texture), |canvas| {
        while let Some(glyph) = glyphs.next() {
            if entries.contains_key(&glyph) {
                continue;
            }
            // render the glyph to this target texture...
        }
        finished = true;
        Ok(())
    })?;
    Ok(finished)
}
