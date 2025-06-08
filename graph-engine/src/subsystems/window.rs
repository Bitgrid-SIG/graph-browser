
use common::renderer::SDL;

use std::cell::{RefCell, RefMut};

use super::ui::{UiFrameGuard, GraphUiBuilder, GraphUi};
use crate::sdl3::video::Window;

#[allow(dead_code)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum RenderBackend {
    None,
    OpenGL,
    Vulkan,
    Metal,
    CPU,
}

pub struct GraphWindow {
    inner: Window,
    gui: RefCell<Option<GraphUi>>,
}

pub struct GraphWindowBuilder(crate::sdl3::video::WindowBuilder, RenderBackend);

impl GraphWindowBuilder {
    fn new(title: &str, width: u32, height: u32) -> Self {
        let vid = SDL.vid.borrow();
        Self(crate::sdl3::video::WindowBuilder::new(&vid, title, width, height), RenderBackend::None)
    }

    pub fn build(self) -> Result<GraphWindow, crate::sdl3::video::WindowBuildError> {
        let inner = self.0.build()?;

        // match self.1 {
        //     RenderBackend::None => {
        //         panic!("No render backend was set!");
        //     },
        //     RenderBackend::OpenGL => {
        //         let gl_context = inner.gl_create_context().unwrap();
        //         inner.gl_make_current(&gl_context).unwrap();
        //         inner.subsystem().gl_set_swap_interval(1).unwrap();
        //         println!("Initializing OpenGL");
        //     }
        //     _ => panic!("Backend '{:?}' is not supported", self.1)
        // }

        Ok(GraphWindow{
            inner,
            gui: RefCell::new(None),
        })
    }

    /// Sets the underlying window flags.
    /// This will effectively undo any previous build operations, excluding window size and position.
    pub fn set_window_flags(mut self, flags: u32) -> GraphWindowBuilder {
        self.0.set_window_flags(flags);
        self
    }

    /// Sets the window position.
    pub fn position(mut self, x: i32, y: i32) -> GraphWindowBuilder {
        self.0.position(x, y);
        self
    }

    /// Centers the window.
    pub fn position_centered(mut self) -> GraphWindowBuilder {
        self.0.position_centered();
        self
    }

    /// Sets the window to fullscreen.
    pub fn fullscreen(mut self) -> GraphWindowBuilder {
        self.0.fullscreen();
        self
    }

    /// Window uses high pixel density back buffer if possible.
    pub fn high_pixel_density(mut self) -> GraphWindowBuilder {
        self.0.high_pixel_density();
        self
    }

    /// Sets the window to be usable with an OpenGL context
    pub fn opengl(mut self) -> GraphWindowBuilder {
        self.0.opengl();
        self.1 = RenderBackend::OpenGL;
        self
    }

    /// Sets the window to be usable with a Vulkan instance
    pub fn vulkan(mut self) -> GraphWindowBuilder {
        self.0.vulkan();
        self.1 = RenderBackend::Vulkan;
        self
    }

    /// Hides the window.
    pub fn hidden(mut self) -> GraphWindowBuilder {
        self.0.hidden();
        self
    }

    /// Removes the window decoration.
    pub fn borderless(mut self) -> GraphWindowBuilder {
        self.0.borderless();
        self
    }

    /// Sets the window to be resizable.
    pub fn resizable(mut self) -> GraphWindowBuilder {
        self.0.resizable();
        self
    }

    /// Minimizes the window.
    pub fn minimized(mut self) -> GraphWindowBuilder {
        self.0.minimized();
        self
    }

    /// Maximizes the window.
    pub fn maximized(mut self) -> GraphWindowBuilder {
        self.0.maximized();
        self
    }

    /// Sets the window to have grabbed input focus.
    pub fn input_grabbed(mut self) -> GraphWindowBuilder {
        self.0.input_grabbed();
        self
    }

    /// Create a SDL_MetalView when constructing the window.
    /// This is required when using the raw_window_handle feature on macOS.
    /// Has no effect no other platforms.
    pub fn metal_view(mut self) -> GraphWindowBuilder {
        self.0.metal_view();
        self
    }
}

impl GraphWindow {
    pub fn new(title: &str, width: u32, height: u32) -> GraphWindowBuilder {
        GraphWindowBuilder::new(title, width, height)
    }

    pub fn new_ui(&mut self) -> GraphUiBuilder {
        GraphUi::new(self)
    }

    pub(crate) fn set_ui(&mut self, ui: GraphUi) {
        self.gui.replace(Some(ui));
    }

    pub fn get_ui(&self) -> Option<RefMut<GraphUi>> {
        RefMut::filter_map(self.gui.borrow_mut(), |o| o.as_mut()).ok()
    }

    fn ui_mut(&mut self) -> Option<&mut GraphUi> {
        self.gui.get_mut().as_mut()
    }

    pub fn poll_events(&self) -> super::event::GraphEventIterator {
        super::event::GraphEventIterator::new(self)
    }

    pub fn ui_frame_begin<'a>(&'a mut self) -> UiFrameGuard<'a> {
        UiFrameGuard::new(self.ui_mut().unwrap())
    }
}

impl std::ops::Deref for GraphWindow {
    type Target = Window;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl std::ops::DerefMut for GraphWindow {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}
