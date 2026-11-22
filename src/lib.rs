//! N3XR-ENGINE: Rust acceleration core for Minecraft Java Edition.
//! Java memegang game logic dan rendering; Rust mempercepat kerja CPU berat.

pub mod block;
pub mod chunk;
pub mod engine;
pub mod jni_api;
pub mod mesh;
pub mod player;
pub mod world;

#[cfg(feature = "legacy-gl")]
pub mod renderer;

use std::ffi::c_char;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::Mutex;

/// Versi engine, selalu sama dengan versi di Cargo.toml.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
const VERSION_C: &str = concat!(env!("CARGO_PKG_VERSION"), "\0");

pub struct EngineHandle {
    engine: Mutex<engine::Engine>,
}

/// Mengembalikan string versi bertipe C yang berakhir NUL.
/// Pointer bersifat statis dan tidak boleh di-free.
#[no_mangle]
pub extern "C" fn n3xr_engine_version() -> *const c_char {
    VERSION_C.as_ptr() as *const c_char
}

/// Membuat engine. Mengembalikan null jika gagal.
/// Pointer harus dilepas tepat satu kali dengan `n3xr_engine_destroy`.
#[no_mangle]
pub extern "C" fn n3xr_engine_create() -> *mut EngineHandle {
    let result = catch_unwind(|| match engine::Engine::new() {
        Ok(engine) => Box::into_raw(Box::new(EngineHandle {
            engine: Mutex::new(engine),
        })),
        Err(_) => std::ptr::null_mut(),
    });
    result.unwrap_or(std::ptr::null_mut())
}

/// Update satu frame. Mengembalikan false jika handle null,
/// delta_time tidak valid, mutex rusak, atau terjadi panic.
///
/// # Safety
/// `handle` harus null, atau pointer dari `n3xr_engine_create`
/// yang belum di-destroy.
#[no_mangle]
pub extern "C" fn n3xr_engine_update(handle: *mut EngineHandle, delta_time: f32) -> bool {
    if handle.is_null() || !delta_time.is_finite() || delta_time < 0.0 {
        return false;
    }
    // SAFETY: handle non-null; kontrak pemanggil di atas. Hanya referensi
    // bersama yang dibuat, penguncian dilakukan lewat Mutex.
    let handle = unsafe { &*handle };
    catch_unwind(AssertUnwindSafe(|| match handle.engine.lock() {
        Ok(mut engine) => {
            engine.update(delta_time);
            true
        }
        Err(_) => false,
    }))
    .unwrap_or(false)
}

/// Melepas engine. Aman dipanggil dengan null.
///
/// # Safety
/// `handle` harus berasal dari `n3xr_engine_create`, dan tidak boleh
/// dipakai atau di-destroy lagi sesudahnya (double free adalah UB).
#[no_mangle]
pub extern "C" fn n3xr_engine_destroy(handle: *mut EngineHandle) {
    if handle.is_null() {
        return;
    }
    // SAFETY: kontrak di atas menjamin pointer berasal dari Box::into_raw.
    let _ = catch_unwind(AssertUnwindSafe(|| unsafe {
        drop(Box::from_raw(handle));
    }));
}

// ---- Renderer OpenGL ES lama (opsional, bukan jalur utama Minecraft) ----

#[cfg(feature = "legacy-gl")]
#[no_mangle]
pub extern "C" fn n3xr_renderer_create() -> *mut renderer::Renderer {
    let mut r = renderer::Renderer::new();
    if !r.initialize_gl() {
        return std::ptr::null_mut();
    }
    Box::into_raw(Box::new(r))
}

#[cfg(feature = "legacy-gl")]
#[no_mangle]
pub extern "C" fn n3xr_renderer_resize(h: *mut renderer::Renderer, w: i32, hgt: i32) -> bool {
    if h.is_null() || w <= 0 || hgt <= 0 {
        return false;
    }
    // SAFETY: h non-null dan berasal dari n3xr_renderer_create.
    unsafe { &mut *h }.resize(w, hgt)
}

#[cfg(feature = "legacy-gl")]
#[no_mangle]
pub extern "C" fn n3xr_renderer_render(h: *mut renderer::Renderer) -> bool {
    if h.is_null() {
        return false;
    }
    // SAFETY: sama seperti di atas.
    unsafe { &mut *h }.render()
}

#[cfg(feature = "legacy-gl")]
#[no_mangle]
pub extern "C" fn n3xr_renderer_destroy(h: *mut renderer::Renderer) {
    if h.is_null() {
        return;
    }
    // SAFETY: h berasal dari Box::into_raw di n3xr_renderer_create.
    let mut r = unsafe { Box::from_raw(h) };
    r.shutdown();
}