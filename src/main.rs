pub mod context;
pub mod platform;
pub mod window;

use window::*;

fn main() {
    let test = unsafe { Window::new() };
}
