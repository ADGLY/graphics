use crate::platform;

pub struct Window {
    window: platform::window::Window,
    created: bool,
}

impl Window {
    pub fn new() {
        let test_window = platform::window::Window::new();
        test_window.run();
        println!("End !");
    }
}
