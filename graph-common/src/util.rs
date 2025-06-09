use imgui::{ClipboardBackend, DummyClipboardContext, SharedFontAtlas};

use std::path::PathBuf;

/// Debugging Wrapper for printing an object when it's dropped in a dev build.
pub struct DropNotify<T: std::fmt::Debug>(T);


/// An idiomatic builder object for [`imgui::Context`].
pub struct ImguiBuilder<C: ClipboardBackend = DummyClipboardContext> {
    fonts: Option<SharedFontAtlas>,
    clipboard: Option<C>,

    ini_file: Option<PathBuf>,
    log_file: Option<PathBuf>,

    platform_name: Option<String>,
    renderer_name: Option<String>,
}

impl<C: ClipboardBackend> ImguiBuilder<C> {
    pub fn new() -> Self {
        Self {
            fonts: None,
            clipboard: None,

            ini_file: None,
            log_file: None,

            platform_name: None,
            renderer_name: None,
        }
    }

    pub fn build(self) -> imgui::Context {
        let mut ctx = self.fonts.map_or_else(
            imgui::Context::create,
            imgui::Context::create_with_shared_font_atlas,
        );

        if self.clipboard.is_some() {
            ctx.set_clipboard_backend(self.clipboard.unwrap());
        }
        ctx.set_ini_filename(self.ini_file);
        ctx.set_log_filename(self.log_file);
        ctx.set_platform_name(self.platform_name);
        ctx.set_renderer_name(self.renderer_name);

        ctx
    }

    pub fn font_atlas(mut self, atlas: SharedFontAtlas) -> Self {
        self.fonts = Some(atlas);
        self
    }

    pub fn clipboard_backend(mut self, backend: C) -> Self {
        self.clipboard = Some(backend);
        self
    }

    pub fn ini(mut self, path: impl Into<PathBuf>) -> Self {
        self.ini_file = Some(path.into());
        self
    }

    pub fn log(mut self, path: impl Into<PathBuf>) -> Self {
        self.log_file = Some(path.into());
        self
    }

    pub fn platform(mut self, path: impl Into<String>) -> Self {
        self.platform_name = Some(path.into());
        self
    }

    pub fn renderer(mut self, path: impl Into<String>) -> Self {
        self.renderer_name = Some(path.into());
        self
    }
}

impl<T: std::fmt::Debug> From<T> for DropNotify<T> {
    fn from(value: T) -> Self {
        Self(value)
    }
}

impl<T: std::fmt::Debug> std::ops::Deref for DropNotify<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T: std::fmt::Debug> std::ops::DerefMut for DropNotify<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[cfg(debug_assertions)]
impl<T: std::fmt::Debug> std::ops::Drop for DropNotify<T> {
    fn drop(&mut self) {
        println!("{:?}", self.0);
    }
}

impl<C: ClipboardBackend> std::default::Default for ImguiBuilder<C> {
    fn default() -> Self {
        Self::new()
    }
}
