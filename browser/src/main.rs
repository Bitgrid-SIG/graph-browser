
use graph_engine::subsystems::window::GraphWindow;

use common::renderer::sdl3::video::GLProfile;
use common::renderer::sdl3::event::Event;
use common::renderer::SDL;

fn main() {
    let vid = SDL.vid.borrow();
    
    let gl_attr = vid.gl_attr();
    gl_attr.set_context_version(3, 3);
    gl_attr.set_context_profile(GLProfile::Core);

    let mut window = GraphWindow::new("Graph Browser", 480, 270)
        .position_centered()
        .resizable()
        .build()
        .unwrap();

    window.new_ui()
        .platform("imgui_impl_sdl3")
        .renderer("imgui_impl_opengl3")
        .build();

    'main: loop {
        for event in window.poll_events() {
            if let Event::Quit{..} = event {
                break 'main;
            }
        }

        let mut ui_frame = window.ui_frame_begin();
        {
            let gui = ui_frame.get();

            gui.show_demo_window(&mut true);
        }
        ui_frame.end();
    }
}
