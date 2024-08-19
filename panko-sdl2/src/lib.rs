use panko::backend::*;
use panko::types::*;
use panko::Result;
use sdl2_sys::*;
use std::ffi::c_char;
use std::ffi::c_int;
use std::ffi::CString;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;

static IS_SDL2_INITIALIZED: AtomicBool = AtomicBool::new(false);

pub struct BackendSDL2 {
    window: *mut SDL_Window,
    renderer: *mut SDL_Renderer,
    textures: Vec<Option<*mut SDL_Texture>>,
    fonts: Vec<Option<*mut ttf::TTF_Font>>,
}

impl BackendSDL2 {
    pub fn new(title: &str, config: WindowConfig) -> Result<Self> {
        if IS_SDL2_INITIALIZED.load(Ordering::Relaxed) {
            return Err(String::from("Context SDL2 already initialized."));
        }

        let window_name = CString::new(title).map_err(|e| e.to_string())?;

        unsafe {
            if SDL_Init(SDL_INIT_VIDEO) < 0 {
                return Err(sdl_error());
            }

            let (window_width, window_height) = match config {
                WindowConfig::Bordered(physical_size) | WindowConfig::Borderless(physical_size) => {
                    (physical_size.width, physical_size.height)
                }
                WindowConfig::Fullscreen { .. } => (0, 0),
            };

            let window_flags = match config {
                WindowConfig::Bordered(..) => SDL_WindowFlags::SDL_WINDOW_SHOWN,
                WindowConfig::Borderless(..) => SDL_WindowFlags::SDL_WINDOW_BORDERLESS,
                WindowConfig::Fullscreen { .. } => SDL_WindowFlags::SDL_WINDOW_FULLSCREEN,
            };

            let window = SDL_CreateWindow(
                window_name.as_ptr() as *const c_char,
                SDL_WINDOWPOS_CENTERED_MASK as i32,
                SDL_WINDOWPOS_CENTERED_MASK as i32,
                window_width as c_int,
                window_height as c_int,
                window_flags as u32,
            );

            if window.is_null() {
                return Err(sdl_error());
            }

            let renderer = SDL_CreateRenderer(
                window,
                -1,
                SDL_RendererFlags::SDL_RENDERER_ACCELERATED as u32
                    | SDL_RendererFlags::SDL_RENDERER_PRESENTVSYNC as u32,
            );

            if renderer.is_null() {
                SDL_DestroyWindow(window);
                return Err(sdl_error());
            }

            if SDL_SetHint(
                SDL_HINT_RENDER_SCALE_QUALITY.as_ptr() as *const c_char,
                "1".as_ptr() as *const c_char,
            ) == SDL_bool::SDL_FALSE
            {
                SDL_DestroyRenderer(renderer);
                SDL_DestroyWindow(window);
                return Err(sdl_error());
            }

            IS_SDL2_INITIALIZED.store(true, Ordering::Relaxed);

            Ok(Self {
                window,
                renderer,
                textures: Vec::with_capacity(32),
                fonts: Vec::with_capacity(32),
            })
        }
    }

    fn create_raw_sdl_texture(&mut self, w: u32, h: u32) -> Result<*mut SDL_Texture> {
        unsafe {
            let texture = SDL_CreateTexture(
                self.renderer,
                SDL_PixelFormatEnum::SDL_PIXELFORMAT_ABGR8888 as u32,
                SDL_TextureAccess::SDL_TEXTUREACCESS_TARGET as c_int,
                w as c_int,
                h as c_int,
            );
            if texture.is_null() {
                return Err(sdl_error())?;
            }
            if SDL_SetTextureBlendMode(texture, sdl2_sys::SDL_BlendMode::SDL_BLENDMODE_BLEND) < 0 {
                let error = sdl_error();
                SDL_DestroyTexture(texture);
                return Err(error)?;
            }
            Ok(texture)
        }
    }
}

impl Backend for BackendSDL2 {
    fn window_set_config(&mut self, config: WindowConfig) -> Result {
        let (window_width, window_height) = match config {
            WindowConfig::Bordered(physical_size) | WindowConfig::Borderless(physical_size) => {
                (physical_size.width, physical_size.height)
            }
            WindowConfig::Fullscreen { .. } => (0, 0),
        };

        unsafe {
            match config {
                WindowConfig::Bordered(..) => {
                    SDL_SetWindowSize(self.window, window_width as c_int, window_height as c_int);
                    SDL_SetWindowBordered(self.window, SDL_bool::SDL_TRUE);
                }
                WindowConfig::Borderless(..) => {
                    SDL_SetWindowSize(self.window, window_width as c_int, window_height as c_int);
                    SDL_SetWindowBordered(self.window, SDL_bool::SDL_FALSE);
                }
                WindowConfig::Fullscreen { .. } => {
                    SDL_SetWindowFullscreen(
                        self.window,
                        SDL_WindowFlags::SDL_WINDOW_FULLSCREEN as u32,
                    );
                }
            };
        };

        Ok(())
    }

    fn texture_create(&mut self, w: u32, h: u32) -> Result<TextureData> {
        let texture = self.create_raw_sdl_texture(w, h)?;
        let id = self.textures.len();
        self.textures.push(Some(texture));
        Ok(TextureData {
            id: TextureId(id as u32),
            width: w,
            height: h,
        })
    }

    fn texture_load(&mut self, path: &str) -> Result<TextureData> {
        use std::path::Path;

        if !Path::new(path).exists() {
            return Err(String::from("File does not exist."));
        }

        let c_str = CString::new(path).map_err(|e| e.to_string())?;
        let c_str = c_str.as_ptr();

        let (texture, width, height) = unsafe {
            let texture = sdl2_sys::image::IMG_LoadTexture(self.renderer, c_str);
            if texture.is_null() {
                return Err(sdl_error());
            }

            let mut width: i32 = 0;
            let mut height: i32 = 0;
            if SDL_QueryTexture(
                texture,
                std::ptr::null_mut::<u32>(),
                std::ptr::null_mut::<i32>(),
                &mut width as *mut i32,
                &mut height as *mut i32,
            ) != 0
            {
                let error = sdl_error();
                SDL_DestroyTexture(texture);
                return Err(error);
            }

            (texture, width, height)
        };

        let id = self.textures.len();

        self.textures.push(Some(texture));

        Ok(TextureData {
            id: TextureId(id as u32),
            width: width as u32,
            height: height as u32,
        })
    }

    fn texture_destroy(&mut self, id: TextureId) -> Result {
        let Some(texture) = self.textures.get_mut(id.0 as usize) else {
            return Ok(());
        };
        let Some(texture) = texture.take() else {
            return Ok(());
        };
        unsafe { SDL_DestroyTexture(texture) };
        Ok(())
    }

    fn font_load(&mut self, path: &str, scale: f32) -> Result<FontData> {
        use std::path::Path;

        if !Path::new(path).exists() {
            return Err(String::from("File does not exist."));
        }

        let c_str = CString::new(path).map_err(|e| e.to_string())?;
        let c_str = c_str.as_ptr();

        let point_size = scale as i32;

        let (font, height) = unsafe {
            let font = ttf::TTF_OpenFont(c_str, point_size);
            if font.is_null() {
                return Err(sdl_error());
            }
            let height = ttf::TTF_FontHeight(font) as u32;
            (font, height)
        };

        let id = self.fonts.len();
        self.fonts.push(Some(font));
        Ok(FontData {
            id: FontId(id as u32),
            glyphs_height: height,
        })
    }

    fn font_destroy(&mut self, id: FontId) -> Result {
        let Some(font) = self.fonts.get_mut(id.0 as usize) else {
            return Ok(());
        };
        let Some(font) = font.take() else {
            return Ok(());
        };
        unsafe { ttf::TTF_CloseFont(font) };
        Ok(())
    }

    fn font_glyph_metrics(&mut self, font: FontId, glyph: char) -> Result<GlyphMetrics> {
        let font = self
            .fonts
            .get(font.0 as usize)
            .ok_or(String::from("Font was never registered"))?;
        let font = font.ok_or(String::from("Font was already deleted."))?;

        let mut min_x = 0;
        let mut max_x = 0;
        let mut min_y = 0;
        let mut max_y = 0;
        let mut advance = 0;

        let ret = unsafe {
            ttf::TTF_GlyphMetrics(
                font,
                glyph as u16,
                &mut min_x,
                &mut max_x,
                &mut min_y,
                &mut max_y,
                &mut advance,
            )
        };

        if ret != 0 {
            return Err(String::from("Unable to calculate glyph metrics."));
        }

        Ok(GlyphMetrics {
            min_x,
            max_x,
            min_y,
            max_y,
            advance: advance as u32,
        })
    }

    fn render_set_logical_size(&mut self, w: u32, h: u32) -> Result {
        unsafe {
            if SDL_RenderSetLogicalSize(self.renderer, w as i32, h as i32) != 0 {
                return Err(sdl_error());
            }
        }
        Ok(())
    }

    fn render_set_target(&mut self, target: Option<TextureId>) -> Result {
        match target {
            Some(TextureId(id)) => {
                let index = id as usize;
                let texture = self
                    .textures
                    .get(index)
                    .ok_or(String::from("Texture was never created."))?;
                let texture = texture
                    .clone()
                    .ok_or(String::from("Texture was already deleted."))?;
                unsafe {
                    if SDL_SetRenderTarget(self.renderer, texture) != 0 {
                        return Err(sdl_error());
                    }
                };
            }
            _ => unsafe {
                if SDL_SetRenderTarget(self.renderer, std::ptr::null_mut::<SDL_Texture>()) != 0 {
                    return Err(sdl_error());
                };
            },
        }
        Ok(())
    }

    fn render_set_draw_color(&mut self, color: Color) -> Result {
        unsafe {
            if SDL_SetRenderDrawColor(self.renderer, color.r, color.g, color.b, color.a) != 0 {
                return Err(sdl_error());
            }
        }
        Ok(())
    }

    fn render_clear(&mut self) -> Result {
        unsafe {
            if SDL_RenderClear(self.renderer) != 0 {
                return Err(sdl_error());
            }
        }
        Ok(())
    }

    fn render_present(&mut self) -> Result {
        unsafe { SDL_RenderPresent(self.renderer) };
        Ok(())
    }

    fn render_copy_texture(&mut self, texture: TextureId, options: CopyTextureOptions) -> Result {
        let texture = self
            .textures
            .get(texture.0 as usize)
            .ok_or(String::from("Texture was never created."))?;
        let texture = texture
            .clone()
            .ok_or(String::from("Texture was already deleted."))?;
        let src = options.src.map(rect_to_sdl_rect);
        let src = src
            .as_ref()
            .map_or(std::ptr::null(), |r| r as *const SDL_Rect);
        let dest = options.dest.map(rect_to_sdl_rect);
        let dest = dest
            .as_ref()
            .map_or(std::ptr::null(), |r| r as *const SDL_Rect);
        let center = options.center.map(point_to_sdl_point);
        let center = center
            .as_ref()
            .map_or(std::ptr::null(), |p| p as *const SDL_Point);
        let flip = if options.flip_h {
            SDL_RendererFlip::SDL_FLIP_HORIZONTAL
        } else if options.flip_v {
            SDL_RendererFlip::SDL_FLIP_VERTICAL
        } else {
            SDL_RendererFlip::SDL_FLIP_NONE
        };
        unsafe {
            if SDL_RenderCopyEx(
                self.renderer,
                texture,
                src,
                dest,
                options.angle,
                center,
                flip,
            ) != 0
            {
                return Err(sdl_error());
            }
        }
        Ok(())
    }

    fn render_fill_rect(&mut self, rect: Option<Rect>, color: Color) -> Result {
        let rect = rect.map(rect_to_sdl_rect);
        let rect = rect
            .as_ref()
            .map_or(std::ptr::null(), |r| r as *const SDL_Rect);
        unsafe {
            self.render_set_draw_color(color)?;
            if SDL_RenderFillRect(self.renderer, rect) != 0 {
                return Err(sdl_error());
            }
        }
        Ok(())
    }

    fn render_font_glyph(&mut self, font: FontId, glyph: char, origin: Point) -> Result {
        unsafe {
            let font = self
                .fonts
                .get_mut(font.0 as usize)
                .ok_or(String::from("Font was never created."))?
                .ok_or(String::from("Font was already deleted."))?;

            // TODO: maybe don't allocate again?
            let s = CString::new(glyph.to_string()).map_err(|e| e.to_string())?;
            let glyph_surface = ttf::TTF_RenderUTF8_Blended(
                font,
                s.as_ptr(),
                SDL_Color {
                    r: 0,
                    g: 0,
                    b: 0,
                    a: 255,
                },
            );
            if (glyph_surface as *mut ()).is_null() {
                return Err(sdl_error());
            }

            let surface_ref = &*(glyph_surface as *const _ as *const () as *const SDL_Surface);
            let surface_format =
                &*(surface_ref.format as *const _ as *const () as *const SDL_PixelFormat);
            let dimensions = surface_ref.w.max(surface_ref.h);

            let output_surface = SDL_CreateRGBSurface(
                0,
                dimensions as i32,
                dimensions as i32,
                surface_format.BitsPerPixel as i32,
                surface_format.Rmask,
                surface_format.Gmask,
                surface_format.Bmask,
                surface_format.Amask,
            );

            SDL_FillRect(
                output_surface,
                std::ptr::null(),
                SDL_MapRGBA(surface_ref.format, 0, 0, 0, 0),
            );

            let mut rect = SDL_Rect {
                x: 0,
                y: 0,
                w: surface_ref.w,
                h: surface_ref.h,
            };

            if SDL_UpperBlit(glyph_surface, std::ptr::null(), output_surface, &mut rect) != 0 {
                let err = sdl_error();
                SDL_FreeSurface(glyph_surface);
                SDL_FreeSurface(output_surface);
                return Err(err);
            }

            let glyph_texture = SDL_CreateTextureFromSurface(self.renderer, output_surface);
            if glyph_texture.is_null() {
                let err = sdl_error();
                SDL_FreeSurface(glyph_surface);
                SDL_FreeSurface(output_surface);
                return Err(err);
            }

            let dest_rect = SDL_Rect {
                x: origin.x,
                y: origin.y,
                w: surface_ref.w,
                h: surface_ref.h,
            };

            if SDL_RenderCopy(self.renderer, glyph_texture, std::ptr::null(), &dest_rect) != 0 {
                let err = sdl_error();
                SDL_DestroyTexture(glyph_texture);
                SDL_FreeSurface(glyph_surface);
                SDL_FreeSurface(output_surface);
                return Err(err);
            }
        }

        Ok(())
    }

    fn events_pump(&mut self, events: &mut Vec<Event>) {
        use std::mem::MaybeUninit;

        let mut event: MaybeUninit<SDL_Event> = MaybeUninit::zeroed();

        unsafe {
            while SDL_PollEvent(event.as_mut_ptr()) != 0 {
                let event = event.assume_init();
                if event.type_ == SDL_EventType::SDL_QUIT as u32 {
                    events.push(Event::Close)
                } else if event.type_ == SDL_EventType::SDL_KEYDOWN as u32 {
                } else if event.type_ == SDL_EventType::SDL_KEYUP as u32 {
                }
            }
        }
    }

    fn system_get_millis(&mut self) -> Result<u64> {
        Ok(unsafe { SDL_GetTicks64() })
    }
}

unsafe fn sdl_error() -> String {
    let err = SDL_GetError();
    let str = CString::from_raw(err as *mut i8);
    str.to_string_lossy().to_string()
}

fn rect_to_sdl_rect(rect: Rect) -> SDL_Rect {
    SDL_Rect {
        x: rect.x,
        y: rect.y,
        w: rect.w as i32,
        h: rect.h as i32,
    }
}

fn point_to_sdl_point(point: Point) -> SDL_Point {
    SDL_Point {
        x: point.x,
        y: point.y,
    }
}
