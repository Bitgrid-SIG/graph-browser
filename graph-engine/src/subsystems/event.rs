use parking_lot::RwLock;

use std::sync::Arc;

use common::renderer::SDL;
use common::renderer::sdl3::EventPump;
use common::renderer::sdl3::event::Event;

#[must_use = "Iterators are lazy and do nothing unless consumed"]
pub struct GraphEventIterator<'a> {
    window: Option<&'a super::window::GraphWindow>,
    pump: Arc<RwLock<EventPump>>,
}

impl<'a> GraphEventIterator<'a> {
    pub fn new(window: &'a super::window::GraphWindow) -> Self {
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
        let mut pump = self.pump.write();
        if let Some(event) = pump.poll_event() {
            if let Some(mut ui) = self.window.unwrap().get_ui() {
                ui.handle_event(&event);
            }
            Some(event)
        } else {
            if let Some(window) = self.window.take() {
                let mut ui = window.get_ui().unwrap();
                drop(pump);
                ui.prepare(window);
            }
            None
        }
    }
}
