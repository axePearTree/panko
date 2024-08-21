
#[macro_use]
extern crate alloc;

pub mod backend;
pub mod canvas;
pub mod font;
pub mod input;
pub mod texture;
pub mod types;

use alloc::rc::{Rc, Weak};
use alloc::string::String;
use alloc::vec::Vec;
use backend::*;
use canvas::Canvas;
use font::Font;
use core::cell::RefCell;
use input::InputState;
use texture::*;
use types::*;

pub type Result<T = ()> = core::result::Result<T, String>;
pub(crate) type BackendRef = Rc<RefCell<dyn Backend>>;
pub(crate) type BackendWeakRef = Weak<RefCell<dyn Backend>>;

pub trait Application {
    fn update(&mut self, context: &mut Context, delta_ms: u64) -> Result;
    fn fixed_update(&mut self, context: &mut Context, fixed_ms: u64) -> Result;
    fn draw(&mut self, canvas: &mut Canvas, alpha_secs: f32) -> Result;
}

pub struct Context {
    pub(crate) backend: BackendRef,
    input: InputState,
    events: Vec<Event>,
}

impl Context {
    pub fn new(context: impl Backend + 'static) -> Self {
        Self {
            backend: Rc::new(RefCell::new(context)),
            events: Vec::with_capacity(16),
            input: InputState::default(),
        }
    }

    pub fn set_window_config(&self, config: WindowConfig) -> Result {
        self.backend.borrow_mut().window_set_config(config)
    }

    pub fn load_texture(&self, path: &str) -> Result<Texture> {
        Texture::new_static(&self.backend, path)
    }

    pub fn create_target(&self, w: u32, h: u32) -> Result<Texture> {
        Texture::new_target(&self.backend, w, h)
    }

    pub fn load_font(&self, path: &str, scale: u8) -> Result<Font> {
        Font::new(&self.backend, path, scale)
    }

    fn millis(&self) -> Result<u64> {
        self.backend.borrow_mut().system_get_millis()
    }

    fn refresh_events(&mut self) {
        self.events.clear();
        self.backend.borrow_mut().events_pump(&mut self.events);
    }

    fn canvas(&self) -> Result<Canvas> {
        self.backend.borrow_mut().render_clear()?;
        Canvas::new(&self.backend, None)
    }
}

pub fn run_event_loop<T: Application>(
    backend: impl Backend + 'static,
    load: impl FnOnce(&Context) -> Result<T>,
) -> Result {
    const FIXED_TIMESTEP_MILLIS: u64 = 16;

    let mut context = Context::new(backend);

    let mut app = load(&mut context)?;

    let mut millis_now = context.millis()?;
    let mut acc_millis = 0;

    'game_loop: loop {
        let millis_before = millis_now;
        millis_now = context.millis()?;

        let delta_millis = millis_now - millis_before;
        acc_millis += delta_millis;

        context.input.keyboard.clear_memory();
        context.refresh_events();
        for event in context.events.iter() {
            #[allow(unreachable_patterns)]
            match event {
                Event::KeyDown(key) => context.input.keyboard.on_key_down(*key),
                Event::KeyUp(key) => context.input.keyboard.on_key_up(*key),
                Event::Close => break 'game_loop,
                _ => {}
            }
        }

        app.update(&mut context, delta_millis)?;

        if acc_millis >= FIXED_TIMESTEP_MILLIS {
            acc_millis -= FIXED_TIMESTEP_MILLIS;
            app.fixed_update(&mut context, FIXED_TIMESTEP_MILLIS)?;
        }

        let alpha = acc_millis as f32 / FIXED_TIMESTEP_MILLIS as f32;

        app.draw(&mut context.canvas()?, alpha)?;
    }

    Ok(())
}
