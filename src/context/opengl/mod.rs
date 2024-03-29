pub static WGL_NUMBER_PIXEL_FORMATS_ARB: i32 = 0x2000;
pub static WGL_DRAW_TO_WINDOW_ARB: i32 = 0x2001;
pub static WGL_DRAW_TO_BITMAP_ARB: i32 = 0x2002;
pub static WGL_ACCELERATION_ARB: i32 = 0x2003;
pub static WGL_NEED_PALETTE_ARB: i32 = 0x2004;
pub static WGL_NEED_SYSTEM_PALETTE_ARB: i32 = 0x2005;
pub static WGL_SWAP_LAYER_BUFFERS_ARB: i32 = 0x2006;
pub static WGL_SWAP_METHOD_ARB: i32 = 0x2007;
pub static WGL_NUMBER_OVERLAYS_ARB: i32 = 0x2008;
pub static WGL_NUMBER_UNDERLAYS_ARB: i32 = 0x2009;
pub static WGL_TRANSPARENT_ARB: i32 = 0x200A;
pub static WGL_TRANSPARENT_RED_VALUE_ARB: i32 = 0x2037;
pub static WGL_TRANSPARENT_GREEN_VALUE_ARB: i32 = 0x2038;
pub static WGL_TRANSPARENT_BLUE_VALUE_ARB: i32 = 0x2039;
pub static WGL_TRANSPARENT_ALPHA_VALUE_ARB: i32 = 0x203A;
pub static WGL_TRANSPARENT_INDEX_VALUE_ARB: i32 = 0x203B;
pub static WGL_SHARE_DEPTH_ARB: i32 = 0x200C;
pub static WGL_SHARE_STENCIL_ARB: i32 = 0x200D;
pub static WGL_SHARE_ACCUM_ARB: i32 = 0x200E;
pub static WGL_SUPPORT_GDI_ARB: i32 = 0x200F;
pub static WGL_SUPPORT_OPENGL_ARB: i32 = 0x2010;
pub static WGL_DOUBLE_BUFFER_ARB: i32 = 0x2011;
pub static WGL_STEREO_ARB: i32 = 0x2012;
pub static WGL_PIXEL_TYPE_ARB: i32 = 0x2013;
pub static WGL_COLOR_BITS_ARB: i32 = 0x2014;
pub static WGL_RED_BITS_ARB: i32 = 0x2015;
pub static WGL_RED_SHIFT_ARB: i32 = 0x2016;
pub static WGL_GREEN_BITS_ARB: i32 = 0x2017;
pub static WGL_GREEN_SHIFT_ARB: i32 = 0x2018;
pub static WGL_BLUE_BITS_ARB: i32 = 0x2019;
pub static WGL_BLUE_SHIFT_ARB: i32 = 0x201A;
pub static WGL_ALPHA_BITS_ARB: i32 = 0x201B;
pub static WGL_ALPHA_SHIFT_ARB: i32 = 0x201C;
pub static WGL_ACCUM_BITS_ARB: i32 = 0x201D;
pub static WGL_ACCUM_RED_BITS_ARB: i32 = 0x201E;
pub static WGL_ACCUM_GREEN_BITS_ARB: i32 = 0x201F;
pub static WGL_ACCUM_BLUE_BITS_ARB: i32 = 0x2020;
pub static WGL_ACCUM_ALPHA_BITS_ARB: i32 = 0x2021;
pub static WGL_DEPTH_BITS_ARB: i32 = 0x2022;
pub static WGL_STENCIL_BITS_ARB: i32 = 0x2023;
pub static WGL_AUX_BUFFERS_ARB: i32 = 0x2024;
pub static WGL_FULL_ACCELERATION_ARB: i32 = 0x2027;
pub static WGL_TYPE_RGBA_ARB: i32 = 0x202B;
pub static WGL_SAMPLE_BUFFERS_ARB: i32 = 0x2041;
pub static WGL_SAMPLES_ARB: i32 = 0x2042;

use std::{ffi::CString, os::raw::c_void};

use windows::{
    core::*,
    Win32::{Graphics::OpenGL::wglGetProcAddress, System::LibraryLoader::*},
};

#[cfg(target_os = "windows")]
pub fn load_context() {
    let open_gl_handle = unsafe { GetModuleHandleA(s!("opengl32.dll")).unwrap() };

    gl::load_with(|s| {
        // Here we receive glGetnColorTable
        let proc_name = CString::new(s).unwrap();
        match unsafe { GetProcAddress(open_gl_handle, PCSTR(proc_name.as_ptr() as *const u8)) } {
            Some(func) => func as *const c_void,
            None => match unsafe { wglGetProcAddress(PCSTR(proc_name.as_ptr() as *const u8)) } {
                Some(func) => func as *const c_void,
                None => {
                    println!("Could not load func : {:?}!", proc_name);
                    std::ptr::null()
                }
            },
        }
    });
}
