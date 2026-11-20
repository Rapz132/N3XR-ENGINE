
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
    b"0.2.0\0".as_ptr() as *const c_char
}

#[no_mangle]
pub extern "C" fn n3xr_engine_create() -> *mut EngineHandle {
    match engine::Engine::new() {
        Ok(engine) => {
            Box::into_raw(Box::new(EngineHandle { engine }))
        }
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

// OpenGL ES Renderer API

#[no_mangle]
pub extern "C" fn n3xr_renderer_create() -> *mut renderer::Renderer {
    let mut renderer = renderer::Renderer::new();

    if !renderer.initialize_gl() {
        return std::ptr::null_mut();
    }

    Box::into_raw(Box::new(renderer))
}

#[no_mangle]
pub extern "C" fn n3xr_renderer_resize(
    handle: *mut renderer::Renderer,
    width: i32,
    height: i32,
) -> bool {
    if handle.is_null() {
        return false;
    }

    let renderer = unsafe { &mut *handle };

    renderer.resize(width, height)
}

#[no_mangle]
pub extern "C" fn n3xr_renderer_render(
    handle: *mut renderer::Renderer,
) -> bool {
    if handle.is_null() {
        return false;
    }

    let renderer = unsafe { &mut *handle };

    renderer.render()
}

#[no_mangle]
pub extern "C" fn n3xr_renderer_destroy(
    handle: *mut renderer::Renderer,
) {
    if !handle.is_null() {
        unsafe {
            let mut renderer = Box::from_raw(handle);
            renderer.shutdown();
        }
    }
}
