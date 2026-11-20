
pub mod engine;
pub mod renderer;
pub mod world;
pub mod chunk;
pub mod block;
pub mod player;

use std::ffi::CString;
use std::os::raw::c_char;

#[no_mangle]
pub extern "C" fn n3xr_engine_version() -> *mut c_char {
    CString::new("0.1.0")
        .unwrap()
        .into_raw()
}

#[no_mangle]
pub extern "C" fn n3xr_engine_initialize() -> bool {
    engine::Engine::new().is_ok()
}

#[no_mangle]
pub extern "C" fn n3xr_engine_destroy(ptr: *mut c_char) {
    if !ptr.is_null() {
        unsafe {
            drop(CString::from_raw(ptr));
        }
    }
}
