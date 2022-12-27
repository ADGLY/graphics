pub mod context;
pub mod platform;
pub mod window;

use window::*;

fn main() {
    let _test = unsafe { Window::new() };
}
