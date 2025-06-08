
pub use sdl3;

pub mod imgui {
    pub use imgui::*;
    pub use imgui_sdl3_support as sdl3_support;

    pub mod renderers {
        pub mod glow {
            pub use imgui_glow_renderer::*;
            pub use imgui_glow_renderer::glow as inner;
        }
    }
}

pub use graph::SDL;

pub mod graph {
    use std::sync::LazyLock;
    use std::cell::RefCell;

    use crate::util::LazyInit;

    pub static SDL: LazyLock<SDLContext> = LazyLock::new(|| {
        let core = RefCell::new(sdl3::init().unwrap());

        let events = RefCell::new(core.borrow().event().unwrap());
        let vid = RefCell::new(core.borrow().video().unwrap());
        let aux = RefCell::new(core.borrow().audio().unwrap());

        let gamepad = LazyInit::new(|| SDL.core.borrow().gamepad().unwrap());
        let joystick = LazyInit::new(|| SDL.core.borrow().joystick().unwrap());
        let sensor = LazyInit::new(|| SDL.core.borrow().sensor().unwrap());
        let haptic = LazyInit::new(|| SDL.core.borrow().haptic().unwrap());

        SDLContext{
            core,
    
            events,
            vid,
            aux,
    
            gamepad,
            joystick,
            sensor,
            haptic,
        }
    });
    
    pub struct SDLContext {
        pub core: RefCell<sdl3::Sdl>,

        pub events: RefCell<sdl3::EventSubsystem>,
        pub vid:    RefCell<sdl3::VideoSubsystem>,
        pub aux:    RefCell<sdl3::AudioSubsystem>,

        // pub cam:        LazyInit<sdl3::CameraSubsystem>,
        pub gamepad:    LazyInit<sdl3::GamepadSubsystem>,
        pub joystick:   LazyInit<sdl3::JoystickSubsystem>,
        pub sensor:     LazyInit<sdl3::SensorSubsystem>,
        pub haptic:     LazyInit<sdl3::HapticSubsystem>,
    }

    impl SDLContext {
        pub fn event_pump(&self) -> Result<sdl3::EventPump, sdl3::Error> {
            self.core.borrow().event_pump()
        }
    }

    unsafe impl Sync for SDLContext {}
    unsafe impl Send for SDLContext {}
}
