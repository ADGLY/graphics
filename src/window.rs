use crate::platform;

pub struct Window {
    window: platform::window::Window,
    created: bool,
}

impl Window {
    pub unsafe fn new() {
        let test_window = platform::window::Window::new();
        println!("End !");
        loop {}
    }

    /*unsafe extern "system" fn wndproc(
        window: platfrHWND,
        message: u32,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> LRESULT {
        match message as u32 {
            WM_PAINT => {
                println!("WM_PAINT");
                LRESULT(0)
            }
            WM_DESTROY => {
                println!("WM_DESTROY");
                PostQuitMessage(0);
                LRESULT(0)
            }
            WM_NCCREATE => {
                println!("NC Create");
                let createstruct: *mut CREATESTRUCTW = lparam.0 as *mut CREATESTRUCTW;
                if createstruct.is_null() {
                    return LRESULT(0);
                }
                let window_data = (*createstruct).lpCreateParams;
                SetWindowLongPtrW(window, GWLP_USERDATA, window_data as isize);
                DefWindowProcW(window, message, wparam, lparam)
            }
            WM_CREATE => {
                println!("Create !");
                let window_data = GetWindowLongPtrW(window, GWLP_USERDATA) as *mut Window;
                (*window_data).created = true;

                LRESULT(0)
            }
            _ => DefWindowProcW(window, message, wparam, lparam),
        }
    }*/
}
