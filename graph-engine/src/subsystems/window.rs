use common::renderer::SDL;

use std::cell::{RefCell, RefMut};

use super::ui::{GraphUi, GraphUiBuilder, UiFrameGuard};
use crate::sdl3::video::{Window, WindowBuilder};

/// Possible rendering backends for a window.
#[allow(dead_code)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum RenderBackend {
    None,
    OpenGL,
    Vulkan,
    Metal,
    Cpu,
}

/// A window with optional [`GraphUi`] state attached.
pub struct GraphWindow {
    /// Underlying SDL window.
    inner: Window,
    /// Optional [`GraphUi`] instance for this window, with interior mutability.
    gui: RefCell<Option<GraphUi>>,
}

/// Builder for `GraphWindow`, allowing configuration of SDL window flags and render backend.
/// An idiomatic builder for [`GraphWindow`].
///
/// Allows optional configuration of:
/// - flags (see [`set_window_flags()`](WindowBuilder::set_window_flags))
/// - position (x + y or centered)
/// - fullscreen status
/// - pixel-density
/// - rendering backend
/// - hidden status
/// - borderless status
/// - resizeable status
/// - minimized/maximized status
/// - focused status
pub struct GraphWindowBuilder(WindowBuilder, RenderBackend);

impl GraphWindowBuilder {
    fn new(title: &str, width: u32, height: u32) -> Self {
        Self(
            WindowBuilder::new(&SDL.video().borrow(), title, width, height),
            RenderBackend::None,
        )
    }

    /// Finalize building the window and return a `GraphWindow`.
    ///
    /// Errors if SDL fails to build the window, or if no rendering backend was selected.
    pub fn build(self) -> Result<GraphWindow, crate::sdl3::video::WindowBuildError> {
        matches!(self.1, RenderBackend::None)
            .then(|| panic!("No render backend was selected before building the graph window"));

        let inner = self.0.build()?;

        // TODO: Why is this not working?

        // match self.1 {
        //     RenderBackend::OpenGL => {
        //         let gl_context = inner.gl_create_context().unwrap();
        //         inner.gl_make_current(&gl_context).unwrap();
        //         inner.subsystem().gl_set_swap_interval(1).unwrap();
        //         println!("Initializing OpenGL");
        //     },
        //     RenderBackend::None => {}, // already checked
        //     _ => panic!("Backend '{:?}' is not supported", self.1)
        // }

        Ok(GraphWindow {
            inner,
            gui: RefCell::new(None),
        })
    }

    /// Sets the underlying window flags. <br />
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
    /// Begin building a [`GraphWindow`] with the given title and size.
    pub fn builder(title: &str, width: u32, height: u32) -> GraphWindowBuilder {
        GraphWindowBuilder::new(title, width, height)
    }

    /// Create a new GUI for this window.
    ///
    /// Returns a builder for [`GraphUi`].
    /// Calling [`GraphUiBuilder::build()`] automatically sets it as this window's gui.
    pub fn new_ui(&mut self) -> GraphUiBuilder {
        GraphUi::builder(self)
    }

    /// Set the GUI instance for this window.
    ///
    /// Called internally after building via `new_ui().build(...)`.
    pub(crate) fn set_ui(&mut self, ui: GraphUi) {
        self.gui.replace(Some(ui));
    }

    /// Get a mutable reference to the GUI if it exists.
    ///
    /// Returns `Some(RefMut<GraphUi>)` if a GUI was set, otherwise `None`.
    ///
    /// See also: [`RefMut`].
    pub fn get_ui(&self) -> Option<RefMut<GraphUi>> {
        RefMut::filter_map(self.gui.borrow_mut(), |o| o.as_mut()).ok()
    }

    /// Internal helper to get a mutable reference to the GUI.
    fn ui_mut(&mut self) -> Option<&mut GraphUi> {
        self.gui.get_mut().as_mut()
    }

    /// Poll SDL events, returning an iterator over unprocessed [`crate::sdl3::event::Event`]s.
    ///
    /// The iterator does some automatic state-handling, including:
    /// - Sending each event to the current [`imgui`](crate::imgui) ui, if the window has a [`GraphUi`].
    ///     - See also: [`GraphUi::handle_event()`]
    /// - When the iterator is exhausted, before returning [None]
    pub fn poll_events(&self) -> super::event::GraphEventIterator {
        super::event::GraphEventIterator::new(self)
    }

    /// Begin a new UI frame, returning a guard for frame lifetime.
    ///
    /// Panics if no GUI has been set.
    pub fn ui_frame_begin(&mut self) -> UiFrameGuard<'_> {
        let ui = self
            .ui_mut()
            .expect("Tried to begin a ui frame on a window with no ui");
        UiFrameGuard::new(ui)
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
