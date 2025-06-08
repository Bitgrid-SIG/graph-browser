
use common::renderer::sdl3::event::Event;
use common::renderer::sdl3::EventPump;
use common::renderer::SDL;

#[must_use = "Iterators are lazy and do nothing unless consumed"]
pub struct GraphEventIterator<'a> {
    window: Option<&'a super::window::GraphWindow>,
    pump: EventPump,
}

impl<'a> GraphEventIterator<'a> {
    pub fn new(window: &'a super::window::GraphWindow) -> Self {
        let pump = SDL.event_pump().unwrap();
        Self{ window: Some(window), pump }
    }
}

impl<'a> Iterator for GraphEventIterator<'a> {
    type Item = Event;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(event) = self.pump.poll_event() {
            Some(event)
        } else {
            if let Some(window) = self.window.take() {
                let mut ui = window.get_ui().unwrap();
                ui.prepare(window, &common::renderer::SDL.event_pump().unwrap());
            }
            None
        }
    }
}
