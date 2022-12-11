#[path = "opengl_defines.rs"]
mod opengl_defines;

use std::{ffi::CString, os::raw::c_void};
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

use crate::window::opengl_defines::*;

enum Context {
    OPENGL,
}

struct WindowsWindow {
    window_handle: HWND,
    dc: HDC,
    context: HGLRC,
}

impl WindowsWindow {
    fn new(context: Context) -> WindowsWindow {
        if !matches!(context, Context::OPENGL) {
            panic!("Unsupported context !");
        }

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

        let temporary_window = WindowsWindow::create_window_and_context(
            w!("Temporary window class"),
            w!("Temporary window"),
            choose_pixel_format,
        );

        WindowsWindow::load_opengl_funcs();

        fn choose_pixel_format_ogl46(dc: HDC, pfd: &mut PIXELFORMATDESCRIPTOR) -> i32 {
            let wglChoosePixelFormatStr = CString::new("wglChoosePixelFormatARB").unwrap();
            let proc =
                unsafe { wglGetProcAddress(PCSTR(wglChoosePixelFormatStr.as_ptr() as *const u8)) }
                    .unwrap();
            let wglChoosePixelFormatARB = unsafe {
                std::mem::transmute::<
                    unsafe extern "system" fn() -> isize,
                    fn(HDC, *const i32, *const f32, u32, *mut i32, *mut u32) -> BOOL,
                >(proc)
            };

            let mut pixel_fmt: i32 = 0;
            let mut num_pixel_fmt: u32 = 0;
            static pixel_attribs: [i32; 21] = [
                WGL_SUPPORT_OPENGL_ARB,
                GL_TRUE as i32,
                WGL_ACCELERATION_ARB,
                WGL_FULL_ACCELERATION_ARB,
                WGL_DRAW_TO_WINDOW_ARB,
                GL_TRUE as i32,
                WGL_DOUBLE_BUFFER_ARB,
                GL_TRUE as i32,
                WGL_PIXEL_TYPE_ARB,
                WGL_TYPE_RGBA_ARB,
                WGL_COLOR_BITS_ARB,
                24,
                WGL_DEPTH_BITS_ARB,
                24,
                WGL_STENCIL_BITS_ARB,
                8,
                WGL_SAMPLE_BUFFERS_ARB,
                GL_TRUE as i32,
                WGL_SAMPLES_ARB,
                4,
                0,
            ];

            debug_assert!(wglChoosePixelFormatARB(
                dc,
                pixel_attribs.as_ptr(),
                std::ptr::null(),
                1,
                &mut pixel_fmt,
                &mut num_pixel_fmt,
            )
            .as_bool());
            let err = unsafe {
                DescribePixelFormat(
                    dc,
                    PFD_PIXEL_TYPE(pixel_fmt as i8),
                    std::mem::size_of::<PIXELFORMATDESCRIPTOR>() as u32,
                    pfd,
                )
            };
            let err_win = unsafe { GetLastError() };
            println!("test");
            pixel_fmt
        }

        let window = WindowsWindow::create_window_and_context(
            w!("Renderer window class"),
            w!("Rendered window"),
            choose_pixel_format_ogl46,
        );
        window
    }

    fn create_window_and_context(
        window_class: &HSTRING,
        window_name: &HSTRING,
        get_pixel_fmt: fn(HDC, pfd: &mut PIXELFORMATDESCRIPTOR) -> i32,
    ) -> WindowsWindow {
        let instance = WindowsWindow::get_instance();

        let window_handle = WindowsWindow::create_window(instance, window_class, window_name);
        unsafe { ShowWindow(window_handle, SW_SHOW) };

        let dc = unsafe { GetDC(window_handle) };

        WindowsWindow::set_pixel_format(dc, get_pixel_fmt);

        let context = WindowsWindow::create_openl_context(dc);

        WindowsWindow {
            window_handle: window_handle,
            dc: dc,
            context: context,
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

    fn create_window(instance: HINSTANCE, window_class: &HSTRING, window_name: &HSTRING) -> HWND {
        let wc = WNDCLASSEXW {
            hCursor: unsafe { LoadCursorW(None, IDC_ARROW).unwrap() },
            hInstance: instance,
            lpszClassName: window_class.into(),
            cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
            style: CS_HREDRAW | CS_VREDRAW | CS_OWNDC | CS_DBLCLKS,
            lpfnWndProc: Some(WindowsWindow::temp_wndproc),
            ..Default::default()
        };

        debug_assert!(unsafe { RegisterClassExW(&wc) } != 0);
        unsafe {
            CreateWindowExW(
                WINDOW_EX_STYLE::default(),
                window_class,
                window_name,
                WS_OVERLAPPEDWINDOW | WS_VISIBLE,
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                None,
                None,
                instance,
                std::ptr::null(),
            )
        }
    }

    fn load_opengl_funcs() {
        let open_gl_handle = unsafe { GetModuleHandleA(s!("opengl32.dll")).unwrap() };

        gl::load_with(|s| {
            // Here we receive glGetnColorTable
            let proc_name = CString::new(s).unwrap();
            match unsafe { GetProcAddress(open_gl_handle, PCSTR(proc_name.as_ptr() as *const u8)) }
            {
                Some(func) => func as *const c_void,
                None => {
                    match unsafe { wglGetProcAddress(PCSTR(proc_name.as_ptr() as *const u8)) } {
                        Some(func) => func as *const c_void,
                        None => {
                            println!("Could not load func : {:?}!", proc_name);
                            std::ptr::null()
                        }
                    }
                }
            }
        });
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
}

impl Drop for WindowsWindow {
    fn drop(&mut self) {
        debug_assert!(unsafe { wglDeleteContext(self.context) }.as_bool());
        debug_assert!(unsafe { ReleaseDC(self.window_handle, self.dc) } != 0);
        debug_assert!(unsafe { DestroyWindow(self.window_handle) }.as_bool());
    }
}

pub struct Window {
    window: HWND,
    created: bool,
}

impl Window {
    pub unsafe fn new() {
        WindowsWindow::new(Context::OPENGL);
        println!("End !");
        loop {}
    }

    unsafe extern "system" fn wndproc(
        window: HWND,
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
    }
}
