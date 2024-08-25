use crate::canvas::Canvas;
use crate::types::{FontId, GlyphMetrics};
use crate::{
    BackendRef, BackendWeakRef, Color, CopyTextureOptions, FontData, Point, Rect, Result,
    TextAlign, TextCrossAlign, TextPadding, Texture, TextureId,
};
use alloc::rc::Rc;
use alloc::vec::Vec;
use core::cell::RefCell;
use core::str::Chars;
use hashbrown::HashMap;
use std::ops::Range;

const ATLAS_WIDTH: u32 = 1024;
const ATLAS_HEIGHT: u32 = 1024;

pub struct Font(RefCell<FontInner>);

impl Font {
    pub(crate) fn new(backend: &BackendRef, path: &str, scale: u8) -> Result<Self> {
        Ok(Self(RefCell::new(FontInner::new(backend, path, scale)?)))
    }

    pub(crate) fn draw_text(
        &self,
        canvas: &Canvas,
        text: &str,
        position: Point,
        color: Color,
    ) -> Result {
        self.0.borrow_mut().draw_text(canvas, text, position, color)
    }

    pub(crate) fn draw_text_bounded(
        &self,
        canvas: &Canvas,
        text: &str,
        color: Color,
        rect: Rect,
        align: TextAlign,
        cross_align: TextCrossAlign,
        padding: TextPadding,
    ) -> Result {
        self.0.borrow_mut().draw_text_bounded(
            canvas,
            text,
            color,
            rect,
            align,
            cross_align,
            padding,
        )
    }

    pub(crate) fn atlas(&self, index: usize) -> Option<TextureId> {
        self.0.borrow().atlases.get(index).map(|a| a.texture.id)
    }

    pub(crate) fn register_text(&self, text: &str, canvas: &Canvas) -> Result {
        self.0.borrow_mut().register_glyphs(text, canvas)
    }
}

struct FontInner {
    id: FontId,
    scale: u8,
    glyphs_height: u32,
    backend: BackendWeakRef,
    atlases: Vec<FontAtlas>,
    entries: HashMap<char, FontGlyphEntry>,
}

impl FontInner {
    fn new(backend: &BackendRef, path: &str, scale: u8) -> Result<Self> {
        let FontData { id, glyphs_height } = backend.borrow_mut().font_load(path, scale)?;
        let backend = Rc::downgrade(backend);
        let atlases = vec![FontAtlas::new(
            &backend,
            ATLAS_WIDTH,
            ATLAS_HEIGHT,
            glyphs_height,
        )?];
        Ok(Self {
            id,
            scale,
            glyphs_height,
            backend,
            atlases,
            entries: HashMap::new(),
        })
    }

    fn draw_text(&mut self, canvas: &Canvas, text: &str, position: Point, color: Color) -> Result {
        self.register_glyphs(text, canvas)?;
        self.draw_text_line(position, text, canvas, color)?;
        Ok(())
    }

    /// This is the worst function I've ever wrote. I suck at programming and I'm embarassed.
    fn draw_text_bounded(
        &mut self,
        canvas: &Canvas,
        text: &str,
        color: Color,
        rect: Rect,
        align: TextAlign,
        cross_align: TextCrossAlign,
        padding: TextPadding,
    ) -> Result {
        self.register_glyphs(text, canvas)?;

        let inner_rect = Rect {
            x: rect.x + padding.left as i32,
            y: rect.y + padding.top as i32,
            w: rect.w - padding.left as u32 - padding.right as u32,
            h: rect.h - padding.top as u32 - padding.bottom as u32,
        };

        let mut words = Vec::new();
        let mut cursor = 0;
        let mut is_word_complete = false;
        let mut seeking_word = true;

        for (index, glyph) in text.char_indices() {
            if seeking_word {
                is_word_complete = false;
                if glyph == ' ' {
                    let word = &text[cursor..index];
                    let word_width = word
                        .chars()
                        .map(|g| self.entries.get(&g).unwrap().metrics.advance)
                        .sum::<u32>();
                    words.push((cursor..index, word_width));
                    cursor = index;
                    is_word_complete = true;
                    seeking_word = false;
                }
            } else {
                is_word_complete = false;
                if glyph != ' ' {
                    let word = &text[cursor..index];
                    let word_width = word
                        .chars()
                        .map(|g| self.entries.get(&g).unwrap().metrics.advance)
                        .sum::<u32>();
                    words.push((cursor..index, word_width));
                    cursor = index;
                    is_word_complete = true;
                    seeking_word = true;
                }
            }
        }

        if !is_word_complete {
            let word = &text[cursor..text.len()];
            let word_width = word
                .chars()
                .map(|g| self.entries.get(&g).unwrap().metrics.advance)
                .sum::<u32>();
            words.push((cursor..text.len(), word_width));
        }

        let mut lines = Vec::new();
        let mut line_start = 0;
        let mut line_width = 0;
        let mut max_line_width = 0;

        for (index, (range, word_width)) in words.iter().enumerate() {
            let word = &text[range.clone()];
            let word_width = word
                .chars()
                .map(|g| self.entries.get(&g).unwrap().metrics.advance)
                .sum::<u32>();

            if line_width + word_width as i32 > inner_rect.w as i32 {
                lines.push((&text[line_start..range.start], line_width));
                line_width = word_width as i32;
                line_start = range.start;
            } else {
                line_width += word_width as i32;
            }

            if index == words.len() - 1 {
                lines.push((&text[line_start..range.end], line_width));
            }

            max_line_width = max_line_width.max(line_width);
        }

        let point_y = match cross_align {
            TextCrossAlign::Start => inner_rect.y,
            TextCrossAlign::Center => {
                inner_rect.y + (lines.len() as u32 * self.glyphs_height) as i32 / 2
            }
            TextCrossAlign::End => inner_rect.y + (lines.len() as u32 * self.glyphs_height) as i32,
        };

        for (index, (line, width)) in lines.iter().enumerate() {
            let point_x = match align {
                TextAlign::Left => inner_rect.x,
                TextAlign::Right => inner_rect.x + (inner_rect.w as i32 - width),
                TextAlign::Center => inner_rect.x + (inner_rect.w as i32 - width) / 2,
                TextAlign::Justified => todo!(),
            };
            let point = Point::new(point_x, point_y + index as i32 * self.glyphs_height as i32);
            self.draw_text_line(point, line.trim(), canvas, color)?;
        }

        Ok(())
    }

    fn draw_text_line(
        &mut self,
        position: Point,
        text: &str,
        canvas: &Canvas<'_>,
        color: Color,
    ) -> Result {
        let mut x_cursor = position.x;
        Ok(for glyph in text.chars() {
            let entry = self.entries.get(&glyph).unwrap();
            let atlas = &self.atlases[entry.atlas_index];
            canvas.copy_texture(
                &atlas.texture,
                CopyTextureOptions {
                    src: Some(entry.rect),
                    dest: Some(Rect {
                        x: x_cursor,
                        y: position.y,
                        w: entry.metrics.advance,
                        h: self.glyphs_height,
                    }),
                    color_mod: Some(color),
                    ..Default::default()
                },
            )?;
            x_cursor += entry.metrics.advance as i32;
        })
    }

    fn register_glyphs(&mut self, text: &str, canvas: &Canvas<'_>) -> Result {
        let mut glyphs = text.chars();
        let mut atlas_index = self.atlases.len() - 1;
        let mut atlas = &mut self.atlases[atlas_index];
        loop {
            if register_glyphs(
                self.id,
                atlas_index,
                atlas,
                canvas,
                &mut self.entries,
                &mut glyphs,
            )? {
                break;
            } else {
                self.atlases.push(FontAtlas::new(
                    &self.backend,
                    ATLAS_WIDTH,
                    ATLAS_HEIGHT,
                    self.glyphs_height,
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

/// Returns true if all glyphs were successfully registered inside the `FontAtlas`.
fn register_glyphs(
    font_id: FontId,
    atlas_index: usize,
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

            entries.insert(
                glyph,
                FontGlyphEntry {
                    atlas_index,
                    rect: Rect::new(
                        atlas.x_cursor as i32,
                        atlas.y_cursor as i32,
                        metrics.advance,
                        atlas.glyph_height,
                    ),
                    metrics,
                },
            );

            atlas.x_cursor += metrics.advance;
        }
        finished = true;
        Ok(())
    })?;
    Ok(finished)
}

