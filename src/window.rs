use std::{ffi::CString, os::raw::c_void};

use windows::{
    core::*,
    Win32::{
        Foundation::*,
        Graphics::{
            Gdi::GetDC,
            OpenGL::{
                wglCreateContext, wglGetProcAddress, wglMakeCurrent, ChoosePixelFormat,
                SetPixelFormat, PFD_DOUBLEBUFFER, PFD_DRAW_TO_WINDOW, PFD_MAIN_PLANE,
                PFD_SUPPORT_OPENGL, PFD_TYPE_RGBA, PIXELFORMATDESCRIPTOR,
            },
        },
        System::LibraryLoader::*,
        UI::WindowsAndMessaging::*,
    },
};

pub struct Window {
    window: HWND,
    created: bool,
}

impl Window {
    fn create_context() {
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

        let mut instance: HINSTANCE = HINSTANCE::default();
        debug_assert!(unsafe { GetModuleHandleExW(0, None, &mut instance).as_bool() });
        debug_assert!(instance.0 != 0);

        let window_class = w!("Temporary window class");

        let wc = WNDCLASSEXW {
            hCursor: unsafe { LoadCursorW(None, IDC_ARROW).unwrap() },
            hInstance: instance,
            lpszClassName: window_class.into(),
            cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
            style: CS_HREDRAW | CS_VREDRAW | CS_OWNDC | CS_DBLCLKS,
            lpfnWndProc: Some(temp_wndproc),
            ..Default::default()
        };

        debug_assert!(unsafe { RegisterClassExW(&wc) } != 0);
        let window_name = w!("This is a sample window");
        let window = unsafe {
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
        };

        let temp_dc = unsafe { GetDC(window) };
        let pfd = PIXELFORMATDESCRIPTOR {
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
        let pixelfmt = unsafe { ChoosePixelFormat(temp_dc, &pfd) };
        if !unsafe { SetPixelFormat(temp_dc, pixelfmt, &pfd).as_bool() } {
            panic!("Can't set pixel format !");
        }

        let temp_context = unsafe { wglCreateContext(temp_dc).unwrap() };
        if !unsafe { wglMakeCurrent(temp_dc, temp_context).as_bool() } {
            panic!("Could not make context current !");
        }

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

    pub unsafe fn new() {
        let temporary_window = Self {
            window: HWND::default(),
            created: false,
        };
        Window::create_context();

        /*let mut message = MSG::default();
        while !temporary_window.created
            && PeekMessageW(&mut message, None, 0, 0, PM_NOYIELD | PM_REMOVE).as_bool()
        {
            DispatchMessageW(&message);
        }*/

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
