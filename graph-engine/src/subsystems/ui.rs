use std::path::PathBuf;

use crate::imgui::sdl3_support::SdlPlatform;
use crate::imgui::{
    ClipboardBackend, Context as ImguiContext, DummyClipboardContext, SharedFontAtlas as FontAtlas,
};

use crate::imgui::renderers::glow::AutoRenderer;
use crate::imgui::renderers::glow::inner::{Context, HasContext};

use crate::sdl3::event::Event;
use crate::sdl3::video::Window;

use super::window::GraphWindow;

use common::renderer::SDL;

pub struct GraphUi {
    imgui: ImguiContext,
    platform: SdlPlatform,
    pub(super) renderer: AutoRenderer,
}

pub struct GraphUiBuilder<'a, C: ClipboardBackend = DummyClipboardContext>(
    common::util::ImguiBuilder<C>,
    &'a mut super::window::GraphWindow,
);

pub struct UiFrameGuard<'a> {
    pub(crate) gui: &'a mut GraphUi,
}

pub struct UiDropGuard<'a> {
    pub(crate) ui: &'a mut crate::imgui::Ui,
}

fn glow_context(window: &Window) -> Context {
    unsafe {
        Context::from_loader_function(|s| {
            window
                .subsystem()
                .gl_get_proc_address(s)
                .unwrap_or_else(|| panic!("Expected function '{s}' but did not")) as _
        })
    }
}

impl GraphUi {
    pub(crate) fn builder(window: &mut super::window::GraphWindow) -> GraphUiBuilder {
        GraphUiBuilder(common::util::ImguiBuilder::new(), window)
    }

    pub(crate) fn handle_event(&mut self, event: &Event) {
        self.platform.handle_event(&mut self.imgui, event);
    }

    pub(crate) fn prepare(&mut self, window: &GraphWindow) {
        self.platform.prepare_frame(
            &mut SDL.core().borrow_mut(),
            &mut self.imgui,
            window,
            &SDL.event_pump().read(),
        );
        unsafe {
            self.renderer
                .gl_context()
                .clear(crate::imgui::renderers::glow::inner::COLOR_BUFFER_BIT)
        };
    }

    pub(crate) fn context(&mut self) -> &mut ImguiContext {
        &mut self.imgui
    }

    pub(crate) fn frame_render(&mut self) {
        let draw_data = self.imgui.render();
        self.renderer.render(draw_data).unwrap();
    }
}

impl<C: ClipboardBackend> GraphUiBuilder<'_, C> {
    pub fn build(self) {
        let mut imgui = self.0.build();

        let platform = SdlPlatform::new(&mut imgui);

        let gl = glow_context(self.1);
        let renderer = AutoRenderer::new(gl, &mut imgui).unwrap();

        let result = GraphUi {
            imgui,
            platform,
            renderer,
        };

        self.1.set_ui(result);
    }

    /// Add a shared font atlas to the imfui-rs context when building.
    pub fn font_atlas(mut self, atlas: FontAtlas) -> Self {
        self.0 = self.0.font_atlas(atlas);
        self
    }

    /// Sets the clipboard backend used for clipboard operations.
    pub fn clipboard_backend(mut self, backend: C) -> Self {
        self.0 = self.0.clipboard_backend(backend);
        self
    }

    /// Sets the path to the imgui ini file.
    ///
    /// imgui ini files are disabled by default.
    pub fn ini(mut self, path: impl Into<PathBuf>) -> Self {
        self.0 = self.0.ini(path);
        self
    }

    /// Sets the path to the imgui log file.
    ///
    /// imgui log files are disabled by default.
    pub fn log(mut self, path: impl Into<PathBuf>) -> Self {
        self.0 = self.0.log(path);
        self
    }

    /// Sets the backend platform name.
    pub fn platform(mut self, path: impl Into<String>) -> Self {
        self.0 = self.0.platform(path);
        self
    }

    /// Sets the backend renderer name.
    pub fn renderer(mut self, path: impl Into<String>) -> Self {
        self.0 = self.0.renderer(path);
        self
    }
}

impl<'a> UiFrameGuard<'a> {
    pub(crate) fn new(gui: &'a mut GraphUi) -> Self {
        Self { gui }
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
        Self {
            ui: guard.gui.context().new_frame(),
        }
    }
}

impl std::ops::Deref for UiDropGuard<'_> {
    type Target = crate::imgui::Ui;

    fn deref(&self) -> &Self::Target {
        self.ui
    }
}

impl std::ops::DerefMut for UiDropGuard<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.ui
    }
}
