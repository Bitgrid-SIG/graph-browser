use parking_lot::RwLock;

use common::renderer::SDL;
use common::renderer::sdl3::EventPump;
use common::renderer::sdl3::event::Event;

use std::sync::Arc;

use super::window::GraphWindow;

/// Iterator over SDL events for a [`GraphWindow`], handling GUI integration per event.
///
/// The iterator is lazy: it only polls events when consumed. On each event:
/// - If a GUI is attached to the window, the event is given to the GUI first so it
///   can update automatically.
/// - The event is returned.
///
/// When the iterator is exhausted of events:
/// - Performs start-of-frame UI preparation via `ui.prepare(window)` if a GUI exists.
/// - Subsequent calls to `next()` will not repeat preparation (window reference is taken).
///     - Taking the window reference is done to provide a limit on the use of a single
///       [`GraphEventIterator`]. Reuse is discouraged and disallowed.
#[must_use = "Iterators are lazy and do nothing unless consumed"]
pub struct GraphEventIterator<'a> {
    window: Option<&'a GraphWindow>,
    pump: Arc<RwLock<EventPump>>,
}

impl<'a> GraphEventIterator<'a> {
    /// Create a new event iterator for the given [`GraphWindow`].
    ///
    /// Grabs the global SDL event pump and holds a reference to the window for UI handling.
    /// <br />
    /// The window reference is dropped upon the first instance of the event pump returning
    /// [None], causing a panic if next() is called afterwards.
    pub fn new(window: &'a GraphWindow) -> Self {
        let pump = SDL.event_pump();
        Self {
            window: Some(window),
            pump,
        }
    }
}

impl Iterator for GraphEventIterator<'_> {
    type Item = Event;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(event) = self.pump.write().poll_event() {
            if let Some(mut ui) = self.window.unwrap().get_ui() {
                ui.handle_event(&event);
            }
            Some(event)
        } else {
            if let Some(window) = self.window.take() {
                if let Some(mut ui) = window.get_ui() {
                    ui.prepare(window);
                }
            }
            None
        }
    }
}
