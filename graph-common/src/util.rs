
use imgui::{DummyClipboardContext, SharedFontAtlas, ClipboardBackend};

use std::cell::{RefCell, RefMut, Cell, Ref};
use std::path::PathBuf;

pub struct LazyInit<T>(RefCell<Option<T>>, Cell<Option<Box<dyn FnOnce() -> T>>>);

pub struct ImguiBuilder<C: ClipboardBackend = DummyClipboardContext> {
    fonts: Option<SharedFontAtlas>,
    clipboard: Option<C>,

    ini_file: Option<PathBuf>,
    log_file: Option<PathBuf>,

    platform_name: Option<String>,
    renderer_name: Option<String>,
}

impl<T> LazyInit<T> {
    pub fn new(f: impl FnOnce() -> T + 'static) -> Self {
        Self(RefCell::new(None), Cell::new(Some(Box::new(f))))
    }

    pub fn exists(&self) -> bool {
        self.0.borrow().is_some()
    }

    fn init(&self) {
        if !self.exists() {
            let f: Box<dyn FnOnce() -> T> = self.1.take().unwrap();
            self.0.replace(Some(f()));
        }
    }

    pub fn borrow(&self) -> Option<Ref<T>> {
        self.init();
        Ref::filter_map(self.0.borrow(), |o| o.as_ref()).ok()
    }

    pub fn borrow_mut(&mut self) -> Option<RefMut<T>> {
        self.init();
        RefMut::filter_map(self.0.borrow_mut(), |o| o.as_mut()).ok()
    }

    pub fn get_mut(&mut self) -> &mut T {
        self.init();
        self.0.get_mut().as_mut().unwrap()
    }

    pub fn r#use(&self) -> &T {
        self.init();
        // if contents cannot be safely borrowed, panic.
        drop(self.0.try_borrow().unwrap());
        // guaranteed to be safe because if we can get the pointer
        // in the first place, then it must exist. So immediately
        // dereferencing the pointer is always safe.
        let inner = unsafe { &*self.0.as_ptr() };
        inner.as_ref().unwrap()
    }

    pub fn use_mut(&self) -> &mut T {
        self.init();
        // if contents cannot be safely mutably borrowed, panic.
        drop(self.0.try_borrow_mut().unwrap());
        // guaranteed to be safe because if we can get the pointer
        // in the first place, then it must exist. So immediately
        // dereferencing the pointer is always safe.
        let inner = unsafe { &mut *self.0.as_ptr() };
        inner.as_mut().unwrap()
    }

    pub fn take(&mut self) -> Option<T> {
        self.0.get_mut().take()
    }

    pub fn swap(&mut self, mut new: T) -> Option<T> {
        let result = &mut *self.borrow_mut().unwrap();
        std::mem::swap(result, &mut new);
        Some(new)
    }

}

impl<C: ClipboardBackend> ImguiBuilder<C> {
    pub fn new() -> Self {
        Self{
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
            || imgui::Context::create(),
            |atlas| imgui::Context::create_with_shared_font_atlas(atlas)
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

impl<T> std::ops::Deref for LazyInit<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.init();
        self.r#use()
    }
}

impl<T> std::ops::DerefMut for LazyInit<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.init();
        self.get_mut()
    }
}
