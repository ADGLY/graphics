use std::ffi::CString;
use windows::{
    core::*,
    Win32::{
        Foundation::*,
        Graphics::{
            Gdi::{GetDC, ReleaseDC, HDC},
            OpenGL::{
                wglCreateContext, wglDeleteContext, wglGetProcAddress, wglMakeCurrent,
                ChoosePixelFormat, DescribePixelFormat, SetPixelFormat, GL_TRUE, HGLRC,
                PFD_DOUBLEBUFFER, PFD_DRAW_TO_WINDOW, PFD_MAIN_PLANE, PFD_PIXEL_TYPE,
                PFD_SUPPORT_OPENGL, PFD_TYPE_RGBA, PIXELFORMATDESCRIPTOR,
            },
        },
        System::LibraryLoader::*,
        UI::WindowsAndMessaging::*,
    },
};

use crate::context::context_impl;

#[derive(Debug)]
struct WindowData {
    destroyed: bool,
}

#[cfg(feature = "opengl")]
#[derive(Debug)]
pub struct Window {
    window_handle: HWND,
    dc: HDC,
    context: HGLRC,
    window_data: Box<WindowData>,
}

impl Window {
    fn new_opengl_window() -> Window {
        fn choose_pixel_format(dc: HDC, pfd: &mut PIXELFORMATDESCRIPTOR) -> i32 {
            *pfd = PIXELFORMATDESCRIPTOR {
                nSize: std::mem::size_of::<PIXELFORMATDESCRIPTOR>() as u16,
                nVersion: 1,
                dwFlags: PFD_DRAW_TO_WINDOW | PFD_SUPPORT_OPENGL | PFD_DOUBLEBUFFER,
                iPixelType: PFD_TYPE_RGBA,
                cColorBits: 32,
                cDepthBits: 24,
                cStencilBits: 8,
                iLayerType: PFD_MAIN_PLANE,
                ..Default::default()
            };
            unsafe { ChoosePixelFormat(dc, pfd) }
        }

        let _temporary_window = Window::create_window_and_context(
            w!("Temporary window class"),
            w!("Temporary window"),
            choose_pixel_format,
            Some(Window::temp_wndproc),
        );

        context_impl::load_context();

        fn choose_pixel_format_ogl46(dc: HDC, pfd: &mut PIXELFORMATDESCRIPTOR) -> i32 {
            let wgl_choose_pixel_format_str = CString::new("wglChoosePixelFormatARB").unwrap();
            let proc = unsafe {
                wglGetProcAddress(PCSTR(wgl_choose_pixel_format_str.as_ptr() as *const u8))
            }
            .unwrap();
            let wgl_choose_pixel_format_arb = unsafe {
                std::mem::transmute::<
                    unsafe extern "system" fn() -> isize,
                    fn(HDC, *const i32, *const f32, u32, *mut i32, *mut u32) -> BOOL,
                >(proc)
            };

            let mut pixel_fmt: i32 = 0;
            let mut num_pixel_fmt: u32 = 0;
            static PIXEL_ATTRIBS: [i32; 21] = [
                context_impl::WGL_SUPPORT_OPENGL_ARB,
                GL_TRUE as i32,
                context_impl::WGL_ACCELERATION_ARB,
                context_impl::WGL_FULL_ACCELERATION_ARB,
                context_impl::WGL_DRAW_TO_WINDOW_ARB,
                GL_TRUE as i32,
                context_impl::WGL_DOUBLE_BUFFER_ARB,
                GL_TRUE as i32,
                context_impl::WGL_PIXEL_TYPE_ARB,
                context_impl::WGL_TYPE_RGBA_ARB,
                context_impl::WGL_COLOR_BITS_ARB,
                24,
                context_impl::WGL_DEPTH_BITS_ARB,
                24,
                context_impl::WGL_STENCIL_BITS_ARB,
                8,
                context_impl::WGL_SAMPLE_BUFFERS_ARB,
                GL_TRUE as i32,
                context_impl::WGL_SAMPLES_ARB,
                4,
                0,
            ];

            debug_assert!(wgl_choose_pixel_format_arb(
                dc,
                PIXEL_ATTRIBS.as_ptr(),
                std::ptr::null(),
                1,
                &mut pixel_fmt,
                &mut num_pixel_fmt,
            )
            .as_bool());
            debug_assert!(
                unsafe {
                    DescribePixelFormat(
                        dc,
                        PFD_PIXEL_TYPE(pixel_fmt as i8),
                        std::mem::size_of::<PIXELFORMATDESCRIPTOR>() as u32,
                        pfd,
                    )
                } != 0
            );
            pixel_fmt
        }

        let window = Window::create_window_and_context(
            w!("Renderer window class"),
            w!("Rendered window"),
            choose_pixel_format_ogl46,
            Some(Window::wndproc),
        );

        unsafe { ShowWindow(window.window_handle, SW_SHOW) };

        window
    }

    pub fn new() -> Window {
        #[cfg(feature = "opengl")]
        let window = Window::new_opengl_window();

        #[cfg(not(feature = "opengl"))]
        panic!("Unsupported feature !");

        window
    }

    fn create_window_and_context(
        window_class: &HSTRING,
        window_name: &HSTRING,
        get_pixel_fmt: fn(HDC, pfd: &mut PIXELFORMATDESCRIPTOR) -> i32,
        wnd_proc: WNDPROC,
    ) -> Window {
        let instance = Window::get_instance();

        let window_data = Box::new(WindowData { destroyed: false });
        let window_handle =
            Window::create_window(instance, window_class, window_name, wnd_proc, &window_data);

        let dc = unsafe { GetDC(window_handle) };

        Window::set_pixel_format(dc, get_pixel_fmt);

        let context = Window::create_openl_context(dc);

        Window {
            window_handle: window_handle,
            dc: dc,
            context: context,
            window_data: window_data,
        }
    }

    unsafe extern "system" fn temp_wndproc(
        window: HWND,
        message: u32,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> LRESULT {
        match message as u32 {
            WM_DESTROY => LRESULT(0),
            WM_CREATE => LRESULT(0),
            _ => DefWindowProcW(window, message, wparam, lparam),
        }
    }

    fn get_instance() -> HINSTANCE {
        let mut instance: HINSTANCE = HINSTANCE::default();
        debug_assert!(unsafe { GetModuleHandleExW(0, None, &mut instance).as_bool() });
        debug_assert!(instance.0 != 0);
        instance
    }

    fn create_window(
        instance: HINSTANCE,
        window_class: &HSTRING,
        window_name: &HSTRING,
        wnd_proc: WNDPROC,
        window_data: &Box<WindowData>,
    ) -> HWND {
        let wc = WNDCLASSEXW {
            hCursor: unsafe { LoadCursorW(None, IDC_ARROW).unwrap() },
            hInstance: instance,
            lpszClassName: window_class.into(),
            cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
            style: CS_HREDRAW | CS_VREDRAW | CS_OWNDC | CS_DBLCLKS,
            lpfnWndProc: wnd_proc,
            ..Default::default()
        };

        debug_assert!(unsafe { RegisterClassExW(&wc) } != 0);
        unsafe {
            CreateWindowExW(
                WINDOW_EX_STYLE::default(),
                window_class,
                window_name,
                WS_OVERLAPPEDWINDOW,
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                None,
                None,
                instance,
                (&**window_data as *const WindowData).cast(),
            )
        }
    }

    fn set_pixel_format(dc: HDC, get_pixel_fmt: fn(HDC, &mut PIXELFORMATDESCRIPTOR) -> i32) {
        let mut pfd: PIXELFORMATDESCRIPTOR = PIXELFORMATDESCRIPTOR::default();
        let pixelfmt = get_pixel_fmt(dc, &mut pfd);
        if !unsafe { SetPixelFormat(dc, pixelfmt, &pfd).as_bool() } {
            panic!("Can't set pixel format !");
        }
    }

    fn create_openl_context(dc: HDC) -> HGLRC {
        let context = unsafe { wglCreateContext(dc).unwrap() };
        if !unsafe { wglMakeCurrent(dc, context).as_bool() } {
            panic!("Could not make context current !");
        }
        context
    }

    pub fn run(&self) {
        let mut msg = MSG::default();
        while unsafe { GetMessageW(&mut msg, HWND::default(), 0, 0) }.as_bool() {
            unsafe {
                TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }
        }
    }

    unsafe extern "system" fn wndproc(
        window: HWND,
        message: u32,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> LRESULT {
        match message as u32 {
            WM_PAINT => LRESULT(0),
            WM_CLOSE => {
                debug_assert!(unsafe { DestroyWindow(window) }.as_bool());
                LRESULT(0)
            }
            WM_DESTROY => {
                println!("WM_DESTROY");
                let window_data = GetWindowLongPtrW(window, GWLP_USERDATA) as *mut WindowData;
                (*window_data).destroyed = true;
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
                //DefWindowProcW(window, message, wparam, lparam)
                return LRESULT(1);
            }
            WM_CREATE => {
                println!("Create !");
                LRESULT(0)
            }
            _ => DefWindowProcW(window, message, wparam, lparam),
        }
    }
}

impl Drop for Window {
    fn drop(&mut self) {
        debug_assert!(unsafe { wglDeleteContext(self.context) }.as_bool());
        if !self.window_data.destroyed {
            debug_assert!(unsafe { ReleaseDC(self.window_handle, self.dc) } != 0);
            debug_assert!(unsafe { DestroyWindow(self.window_handle) }.as_bool());
        }
    }
}
