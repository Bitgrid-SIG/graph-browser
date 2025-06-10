//! # Graph's Common Renderer Objects
//!
//! Because this crate is how each dependent crate accesses the [`sdl3`] and
//! [`imgui`] modules (and more), it is idiomatic for dependent crates within
//! the graph workspace to include the following in their `lib.rs`/`main.rs`:
//!
//! ```rust
//! use common::renderer::imgui;
//! use common::renderer::sdl3;
//! ```
//!
//! Accessing those modules elsewhere in the crate is then done using
//! `crate::imgui::<>` and `crate::sdl3::<>`.

pub use sdl3;

use parking_lot::RwLock;

use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::sync::{Arc, LazyLock};

use crate::util::DropNotify;

/// A wrapper around the [`imgui`] crate. This is how all dependent crates
/// access the imgui crate.
pub mod imgui {
    pub use imgui::*;
    pub use imgui_sdl3_support as sdl3_support;

    /// An alias for renderer backends
    pub mod renderers {
        /// A [glow](https://github.com/grovesNL/glow) backend for [`imgui`]
        /// using [`imgui_glow_renderer`].
        pub mod glow {
            pub use imgui_glow_renderer::glow as inner;
            pub use imgui_glow_renderer::*;
        }
    }
}

/// Global SDL context for the workspace, lazily initialized on first access.
///
/// This static provides a shared [`SdlContext`] that initializes and holds all
/// currently-available SDL3 subsystems and related rendering resources exactly
/// once. Every dependent crate can access SDL and its subsystems via this object,
/// avoiding the need to hold and pass references back and forth through the
/// codebase.
///
/// Usage:
/// - Access via `common::renderer::SDL`, e.g. `use common::renderer::SDL;` or
///   call methods on `SDL` to fetch specific subsystems (e.g. event pump, video,
///   audio).
/// - Initialization happens on first access; no additional setup is needed in
///   downstream crates beyond referencing `common::SDL`. Auto-initialization
///   happens for the sdl3 core and specific subsystems. All other subsystems are
///   lazy-loaded on access. The following subsystems are auto-initialized
///   along-side SDL itself:
///     - [`sdl3::EventSubsystem`]
///         - [`sdl3::EventPump`]
///     - [`sdl3::VideoSubsystem`]
///     - [`sdl3::AudioSubsystem`]
///
/// Thread-safety:
/// - Backed by [`std::sync::LazyLock`]<[`SdlContext`]>, so both initialization
///   and access of the object itself are atomic and safe across threads.
///   Individual subsystem access use interior locking, and are not guaranteed
///   or otherwise implied to be thread-safe, unless otherwise specified.
/// - None of the fields are thread-safe except for the [`sdl3::EventPump`]
///   so this is primarily accessed from the main thread, and should not be
///   accessed from other threads (event pump notwithstanding). For more
///   information, see [`crate::renderer::Scf`], [`crate::renderer::ScfAsync`],
///   and [`crate::renderer::LazyScf`].
///
/// Cleanup:
/// - Because statics are not dropped automatically upon program exit, if clean
///   shutdown logic is required (including dropping subsystems in a defined order),
///   use the [`SdlContext::close()`] function to drop each of the SDL subsystems.
///
/// Example:
/// ```rust
/// // In any crate depending on `common`:
/// use common::renderer::SDL;
/// // ...
///
/// let mut event_pump = SDL.event_pump().write();
///
/// // process events, render, etc.
///
/// drop(event_pump); // make sure to release the write-lock
/// ```
///
/// Note: Avoid re-initializing SDL elsewhere; always use this shared static.
pub static SDL: LazyLock<SdlContext> = LazyLock::new(|| {
    let core_inner = Scf::new(sdl3::init().unwrap());

    let events_inner = Scf::new(core_inner.get().borrow().event().unwrap().into());
    let vid_inner = Scf::new(core_inner.get().borrow().video().unwrap().into());
    let aux_inner = Scf::new(core_inner.get().borrow().audio().unwrap().into());
    let event_pump_inner = ScfAsync::new(core_inner.get().borrow().event_pump().unwrap());

    let gamepad_inner = LazyScf::new(|| SDL.core().borrow().gamepad().unwrap().into());
    let joystick_inner = LazyScf::new(|| SDL.core().borrow().joystick().unwrap().into());
    let sensor_inner = LazyScf::new(|| SDL.core().borrow().sensor().unwrap().into());
    let haptic_inner = LazyScf::new(|| SDL.core().borrow().haptic().unwrap().into());

    SdlContext {
        core_inner,

        events_inner,
        vid_inner,
        aux_inner,
        event_pump_inner,

        gamepad_inner,
        joystick_inner,
        sensor_inner,
        haptic_inner,
    }
});

/// ## Single-Threaded "SDL Context Field".
///
/// Internally holds an [Option]<[Rc]<[RefCell]\<T>>>, allowing shared or
/// exclusive access (via `Rc<RefCell<T>>`) when open, or `None` when closed.
///
/// Typical usage:
/// - [`Scf::empty()`] to start with no value.
/// - [`Scf::new()`] to populate immediately.
/// - [`Scf::get()`] clones and returns the `Rc<RefCell<T>>`, panicking if closed.
/// - [`Scf::close()`] removes the inner value, dropping it when last `Rc` is dropped.
struct Scf<T>(RefCell<Option<Rc<RefCell<T>>>>);

/// ## Multi-Threaded "SDL Context Field".
///
/// Internally holds an [Option]<[Arc]<[parking_lot::RwLock]\<T>>>, allowing
/// shared or exclusive access across threads when open, or `None` when closed.
///
/// Typical usage:
/// - [`ScfAsync::empty()`] to start with no value.
/// - [`ScfAsync::new()`] to populate immediately.
/// - [`ScfAsync::get()`] clones and returns the `Arc<RwLock<T>>`, panicking if closed.
/// - [`ScfAsync::close()`] removes the inner value, dropping when last `Arc` is dropped.
struct ScfAsync<T>(RwLock<Option<Arc<RwLock<T>>>>);

/// ## Single-Threaded Lazy-Initialized "SDL Context Field".
///
/// Contains an [`Scf<T>`] for the stored value and a
/// [Cell]<Option<Box<dyn FnOnce() -> T>>> holding the initialization closure.
/// The first `get()` call runs the closure to produce the value, thereafter
/// stored in the inner `Scf`.
///
/// Typical usage:
/// - `LazyScf::new(f)` where `f` produces `T`. Initially `is_open() == false`.
/// - On first `get()`, the closure is taken and invoked; the result is stored.
/// - Subsequent `get()` returns the existing value.
/// - `close()` drops the stored value; after closing, `get()` will panic.
struct LazyScf<T>(Scf<T>, Cell<Option<Box<dyn FnOnce() -> T>>>);

/// A shared object for holding all of SDL's subsystems in one place.
pub struct SdlContext {
    core_inner: Scf<sdl3::Sdl>,

    events_inner: Scf<DropNotify<sdl3::EventSubsystem>>,
    vid_inner: Scf<DropNotify<sdl3::VideoSubsystem>>,
    aux_inner: Scf<DropNotify<sdl3::AudioSubsystem>>,
    event_pump_inner: ScfAsync<sdl3::EventPump>,

    // pub cam:        LazyScf<DropNotify<sdl3::CameraSubsystem>>,
    gamepad_inner: LazyScf<DropNotify<sdl3::GamepadSubsystem>>,
    joystick_inner: LazyScf<DropNotify<sdl3::JoystickSubsystem>>,
    sensor_inner: LazyScf<DropNotify<sdl3::SensorSubsystem>>,
    haptic_inner: LazyScf<DropNotify<sdl3::HapticSubsystem>>,
}

impl<T> Scf<T> {
    /// Create an empty field (no inner value).
    fn empty() -> Self {
        Self(RefCell::new(None))
    }

    /// Create and populate the field immediately with `value`.
    fn new(value: T) -> Self {
        Self(RefCell::new(Some(Rc::new(RefCell::new(value)))))
    }

    /// Returns `true` if the field currently holds a value
    /// (ie. has not been closed).
    fn is_open(&self) -> bool {
        self.0.borrow().is_some()
    }

    /// Get a shared handle to the inner value. <br />
    /// ***NOT THREAD-SAFE***
    ///
    /// Panics if the field is empty/closed.
    fn get(&self) -> Rc<RefCell<T>> {
        self.0
            .borrow()
            .as_ref()
            .cloned()
            .expect("Tried to access Scf with no value inside")
    }

    /// Close the field, removing the inner value.
    ///
    /// Panics if the field is empty/closed.
    fn close(&self) {
        self.0
            .borrow_mut()
            .take()
            .expect("Tried to close Scf with no value inside");
    }
}

impl<T> ScfAsync<T> {
    /// Create an empty field (no inner value).
    #[allow(dead_code)]
    fn empty() -> Self {
        Self(RwLock::new(None))
    }

    /// Create and populate the field immediately with `value`.
    fn new(value: T) -> Self {
        Self(RwLock::new(Some(Arc::new(RwLock::new(value)))))
    }

    /// Returns `true` if the field currently holds a value
    /// (ie. has not been closed).
    fn is_open(&self) -> bool {
        self.0.read().is_some()
    }

    /// Get a shared handle to the inner value.
    ///
    /// Panics if the field is empty/closed.
    fn get(&self) -> Arc<RwLock<T>> {
        self.0
            .read()
            .as_ref()
            .cloned()
            .expect("Tried to access ScfAsync value after it was closed")
    }

    /// Close the field, removing the inner value.
    ///
    /// Panics if the field is empty/closed.
    fn close(&self) {
        self.0
            .write()
            .take()
            .expect("Tried to close ScfAsync with no value inside");
    }
}

impl<T> LazyScf<T> {
    /// Create an empty field (no inner value). Uses `f` to create `T` when
    /// accessed via [`LazyScf::get()`] for the first time.
    fn new(f: impl FnOnce() -> T + 'static) -> Self {
        Self(Scf::empty(), Cell::new(Some(Box::new(f))))
    }

    /// Returns `true` if the field currently holds a value
    /// (ie. has not been closed).
    fn is_open(&self) -> bool {
        self.0.0.borrow().is_some()
    }

    /// Get a shared handle to the inner value.
    ///
    /// Panics if the field is empty/closed.
    fn get(&self) -> Rc<RefCell<T>> {
        if self.0.0.borrow().is_none() {
            let f = self
                .1
                .take()
                .expect("Tried to access LazyScf value after it was closed");
            let fcell = Rc::new(RefCell::new(f()));
            *self.0.0.borrow_mut() = Some(fcell);
        }
        self.0.0.borrow().as_ref().cloned().unwrap()
    }

    /// Close the field, removing the inner value.
    ///
    /// Panics if the field is empty/closed.
    fn close(&self) {
        let mut fcell = self.1.take();
        fcell.is_some().then(|| fcell.take().unwrap());
        self.0
            .0
            .borrow_mut()
            .take()
            .expect("Tried to close LazyScf with no value inside");
    }
}

impl SdlContext {
    /// Get a shared handle to SDL3. <br />
    /// ***NOT THREAD-SAFE***
    ///
    /// See [`SDL`] for more information.
    pub fn core(&self) -> Rc<RefCell<sdl3::Sdl>> {
        self.core_inner.get()
    }

    /// Get a shared handle to SDL3's events subsystem. <br />
    /// ***NOT THREAD-SAFE***
    ///
    /// See [`SDL`] for more information.
    pub fn events(&self) -> Rc<RefCell<DropNotify<sdl3::EventSubsystem>>> {
        self.events_inner.get()
    }

    /// Get a shared handle to SDL3's video subsystem. <br />
    /// ***NOT THREAD-SAFE***
    ///
    /// See [`SDL`] for more information.
    pub fn video(&self) -> Rc<RefCell<DropNotify<sdl3::VideoSubsystem>>> {
        self.vid_inner.get()
    }

    /// Get a shared handle to SDL3's audio subsystem. <br />
    /// ***NOT THREAD-SAFE***
    ///
    /// See [`SDL`] for more information.
    pub fn audio(&self) -> Rc<RefCell<DropNotify<sdl3::AudioSubsystem>>> {
        self.aux_inner.get()
    }

    /// Get a shared handle to SDL3's event pump.
    ///
    /// See [`SDL`] for more information.
    pub fn event_pump(&self) -> Arc<RwLock<sdl3::EventPump>> {
        self.event_pump_inner.get()
    }

    /// Get a shared handle to SDL3's gamepad subsystem. <br />
    /// ***NOT THREAD-SAFE***
    ///
    /// See [`SDL`] for more information.
    pub fn gamepad(&self) -> Rc<RefCell<DropNotify<sdl3::GamepadSubsystem>>> {
        self.gamepad_inner.get()
    }

    /// Get a shared handle to SDL3's joystick subsystem. <br />
    /// ***NOT THREAD-SAFE***
    ///
    /// See [`SDL`] for more information.
    pub fn joystick(&self) -> Rc<RefCell<DropNotify<sdl3::JoystickSubsystem>>> {
        self.joystick_inner.get()
    }

    /// Get a shared handle to SDL3's sensor subsystem. <br />
    /// ***NOT THREAD-SAFE***
    ///
    /// See [`SDL`] for more information.
    pub fn sensor(&self) -> Rc<RefCell<DropNotify<sdl3::SensorSubsystem>>> {
        self.sensor_inner.get()
    }

    /// Get a shared handle to SDL3's haptic subsystem. <br />
    /// ***NOT THREAD-SAFE***
    ///
    /// See [`SDL`] for more information.
    pub fn haptic(&self) -> Rc<RefCell<DropNotify<sdl3::HapticSubsystem>>> {
        self.haptic_inner.get()
    }

    /// Drop each of SDL3's subsystems, SDL3's event pump, and SDL3. Graceful de-init. <br />
    /// ***NOT THREAD-SAFE***
    ///
    /// See [`SDL`] for information.
    pub fn close(&self) {
        self.haptic_inner
            .is_open()
            .then(|| self.haptic_inner.close());
        self.sensor_inner
            .is_open()
            .then(|| self.sensor_inner.close());
        self.joystick_inner
            .is_open()
            .then(|| self.joystick_inner.close());
        self.gamepad_inner
            .is_open()
            .then(|| self.gamepad_inner.close());

        self.event_pump_inner
            .is_open()
            .then(|| self.event_pump_inner.close());
        self.aux_inner.is_open().then(|| self.aux_inner.close());
        self.vid_inner.is_open().then(|| self.vid_inner.close());
        self.events_inner
            .is_open()
            .then(|| self.events_inner.close());

        self.core_inner.is_open().then(|| self.core_inner.close());
    }
}

unsafe impl Sync for SdlContext {}
unsafe impl Send for SdlContext {}
