//! Shared helpers

pub mod math;
pub mod renderer;

use windows::Win32::System::Console::{AllocConsole, FreeConsole};
use egui_opengl_internal::OpenGLApp;

pub fn alloc_console() {
    unsafe { AllocConsole().unwrap() };
}

pub fn free_console() {
    unsafe { FreeConsole().unwrap() };
}

pub fn get_proc_address(_app: &mut OpenGLApp<i32>) -> *const std::ffi::c_void {
    // Get wglSwapBuffers function pointer from opengl32.dll
    use windows::Win32::System::LibraryLoader::{GetModuleHandleA, GetProcAddress};
    use std::ffi::CString;
    
    unsafe {
        let module = match GetModuleHandleA(windows::core::PCSTR(b"opengl32.dll\0".as_ptr())) {
            Ok(m) => m,
            Err(_) => return std::ptr::null(),
        };
        
        let func_name = CString::new("wglSwapBuffers").unwrap();
        let func_ptr = GetProcAddress(module, windows::core::PCSTR(func_name.as_ptr().cast()));
        match func_ptr {
            Some(ptr) => ptr as *const std::ffi::c_void,
            None => std::ptr::null(),
        }
    }
}

pub fn unload(_app: &mut OpenGLApp<i32>) {
    egui_opengl_internal::utils::unload();
}
