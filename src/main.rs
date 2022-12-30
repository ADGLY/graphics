pub mod context;
pub mod platform;
pub mod window;

use platform::window::OpenGLWindowsWindow;
use window::*;

fn main() {
    let _test = Window::<OpenGLWindowsWindow>::new();
}
