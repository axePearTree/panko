use crate::font::Font;
use crate::texture::Texture;
use crate::types::CopyTextureOptions;
use crate::BackendRef;
use crate::Color;
use crate::FontId;
use crate::GlyphMetrics;
use crate::Point;
use crate::Result;
use alloc::rc::Rc;
use alloc::string::String;

pub struct Canvas<'a> {
    backend: BackendRef,
    target: Option<&'a mut Texture>,
}

impl<'a> Canvas<'a> {
    pub(crate) fn new(backend: &BackendRef, target: Option<&'a mut Texture>) -> Result<Self> {
        let backend = Rc::clone(backend);
        backend
            .borrow_mut()
            .render_set_target(target.as_ref().map(|t| t.id))?;
        Ok(Self { target, backend })
    }

    pub fn clear(&self, color: Color) -> Result {
        self.backend.borrow_mut().render_fill_rect(None, color)
    }

    pub fn with_target(
        &self,
        target: Option<&mut Texture>,
        cb: impl FnOnce(&Canvas) -> Result,
    ) -> Result {
        let canvas = Canvas::new(&self.backend, target)?;
        cb(&canvas)?;
        self.backend
            .borrow_mut()
            .render_set_target(self.target.as_ref().map(|t| t.id))?;
        Ok(())
    }

    pub fn copy_texture(&self, texture: &Texture, options: CopyTextureOptions) -> Result {
        self.backend
            .borrow_mut()
            .render_copy_texture(texture.id, options)
    }

    pub fn copy_font_atlas(
        &self,
        font: &Font,
        index: usize,
        options: CopyTextureOptions,
    ) -> Result {
        let atlas_id = font.atlas(index).ok_or(String::from("Atlas not found."))?;
        self.backend
            .borrow_mut()
            .render_copy_texture(atlas_id, options)
    }

    pub fn register_text(&self, font: &Font, text: &str) -> Result {
        font.register_text(text, self)
    }

    pub(crate) fn render_glyph(&self, font_id: FontId, glyph: char, position: Point) -> Result {
        self.backend
            .borrow_mut()
            .render_font_glyph(font_id, glyph, position)
    }

    pub(crate) fn glyph_metrics(&self, font_id: FontId, glyph: char) -> Result<GlyphMetrics> {
        self.backend.borrow_mut().font_glyph_metrics(font_id, glyph)
    }
}

impl<'a> Drop for Canvas<'a> {
    fn drop(&mut self) {
        if self.target.is_none() {
            let _ = self.backend.borrow_mut().render_present();
        }
    }
}
