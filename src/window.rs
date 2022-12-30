use crate::platform::{self, WindowImpl};

pub struct Window<T: WindowImpl> {
    window: T,
    created: bool,
}

impl<T: WindowImpl> Window<T> {
    pub fn new() {
        let test_window = T::create();
        test_window.run();
        println!("End !");
    }
}
