
pub mod engine;
pub mod renderer;
pub mod world;
pub mod chunk;
pub mod block;
pub mod player;

use std::ffi::c_char;

pub struct EngineHandle {
    engine: engine::Engine,
}

#[no_mangle]
pub extern "C" fn n3xr_engine_version() -> *const c_char {
    b"0.1.0\0".as_ptr() as *const c_char
}

#[no_mangle]
pub extern "C" fn n3xr_engine_create() -> *mut EngineHandle {
    match engine::Engine::new() {
        Ok(engine) => Box::into_raw(Box::new(EngineHandle { engine })),
        Err(_) => std::ptr::null_mut(),
    }
}

#[no_mangle]
pub extern "C" fn n3xr_engine_update(
    handle: *mut EngineHandle,
    delta_time: f32,
) -> bool {
    if handle.is_null() || !delta_time.is_finite() {
        return false;
    }

    let handle = unsafe { &mut *handle };
    handle.engine.update(delta_time);
    true
}

#[no_mangle]
pub extern "C" fn n3xr_engine_destroy(
    handle: *mut EngineHandle,
) {
    if !handle.is_null() {
        unsafe {
            drop(Box::from_raw(handle));
        }
    }
}
