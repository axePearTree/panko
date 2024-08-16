use crate::types::CopyTextureOptions;
use crate::texture::Texture;
use crate::BackendRef;
use crate::Color;
use crate::Result;
use alloc::rc::Rc;

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
        self.backend.borrow_mut().render_copy_texture(texture.id, options)
    }
}

impl<'a> Drop for Canvas<'a> {
    fn drop(&mut self) {
        if self.target.is_none() {
            let _ = self.backend.borrow_mut().render_present();
        }
    }
}
