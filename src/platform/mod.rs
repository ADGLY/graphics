pub trait WindowImpl {
    type WindowImplType: WindowImpl;
    fn create() -> Self::WindowImplType;
    fn run(&self);
}

#[cfg(target_os = "windows")]
#[path = "windows/window.rs"]
pub mod window;
