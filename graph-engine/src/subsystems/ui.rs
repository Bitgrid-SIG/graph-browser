
use std::path::PathBuf;

use crate::imgui::sdl3_support::SdlPlatform;
use crate::imgui::{
    Context as ImguiContext,
    SharedFontAtlas as FontAtlas,

    DummyClipboardContext,
    ClipboardBackend,
};

use crate::glow::inner::{HasContext, Context};
use crate::glow::AutoRenderer;

use crate::sdl3::video::Window;
use crate::sdl3::event::Event;
use crate::sdl3::EventPump;

use super::window::GraphWindow;

pub struct GraphUi {
    imgui:      ImguiContext,
    platform:   SdlPlatform,
    pub(super) renderer: AutoRenderer,
}

pub struct GraphUiBuilder<'a, C: ClipboardBackend = DummyClipboardContext>(common::util::ImguiBuilder<C>, &'a mut super::window::GraphWindow);

pub struct UiFrameGuard<'a> {
    pub(crate) gui: &'a mut GraphUi,
}

pub struct UiDropGuard<'a> {
    pub(crate) ui: &'a mut crate::imgui::Ui,
}

fn glow_context(window: &Window) -> Context {
    unsafe {
        Context::from_loader_function(
            |s| {
                window.subsystem().gl_get_proc_address(s)
                    .expect(&format!("Expected function '{s}' but did not")) as _
            }
        )
    }
}

impl GraphUi {
    pub fn new(window: &mut super::window::GraphWindow) -> GraphUiBuilder {
        GraphUiBuilder(common::util::ImguiBuilder::new(), window)
    }

    pub fn handle_event(&mut self, event: &Event) {
        self.platform.handle_event(&mut self.imgui, event);
    }

    pub fn prepare(&mut self, window: &GraphWindow, event_pump: &EventPump) {
        self.platform.prepare_frame(&mut common::renderer::SDL.core.borrow_mut(), &mut self.imgui, window, event_pump);
        unsafe { self.renderer.gl_context().clear(crate::glow::inner::COLOR_BUFFER_BIT) };
    }

    pub fn context(&mut self) -> &mut ImguiContext {
        &mut self.imgui
    }

    pub fn frame_render(&mut self) {
        let draw_data = self.imgui.render();
        self.renderer.render(draw_data).unwrap();
    }
}

impl<'a, C: ClipboardBackend> GraphUiBuilder<'a, C> {
    pub fn build(self) {
        let mut imgui = self.0.build();

        let platform = SdlPlatform::new(&mut imgui);

        let gl = glow_context(self.1);
        let renderer = AutoRenderer::new(gl, &mut imgui).unwrap();

        let result = GraphUi{
            imgui,
            platform,
            renderer,
        };

        self.1.set_ui(result);
    }

    pub fn font_atlas(mut self, atlas: FontAtlas) -> Self {
        self.0 = self.0.font_atlas(atlas);
        self
    }

    pub fn clipboard_backend(mut self, backend: C) -> Self {
        self.0 = self.0.clipboard_backend(backend);
        self
    }

    pub fn ini(mut self, path: impl Into<PathBuf>) -> Self {
        self.0 = self.0.ini(path);
        self
    }

    pub fn log(mut self, path: impl Into<PathBuf>) -> Self {
        self.0 = self.0.log(path);
        self
    }

    pub fn platform(mut self, path: impl Into<String>) -> Self {
        self.0 = self.0.platform(path);
        self
    }

    pub fn renderer(mut self, path: impl Into<String>) -> Self {
        self.0 = self.0.renderer(path);
        self
    }
}

impl<'a> UiFrameGuard<'a> {
    pub(crate) fn new(gui: &'a mut GraphUi) -> Self {
        Self{ gui }
    }

    pub fn get(&mut self) -> UiDropGuard {
        UiDropGuard::new(self)
    }

    pub fn end(self) {
        self.gui.frame_render();
    }
}

impl<'a> UiDropGuard<'a> {
    pub(crate) fn new(guard: &'a mut UiFrameGuard) -> Self {
        Self{ ui: guard.gui.context().new_frame() }
    }
}

impl<'a> std::ops::Deref for GraphUiBuilder<'a> {
    type Target = common::util::ImguiBuilder;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a> std::ops::DerefMut for GraphUiBuilder<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<'a> std::ops::Deref for UiDropGuard<'a> {
    type Target = crate::imgui::Ui;

    fn deref(&self) -> &Self::Target {
        &self.ui
    }
}

impl<'a> std::ops::DerefMut for UiDropGuard<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.ui
    }
}
