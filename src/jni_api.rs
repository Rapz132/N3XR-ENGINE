//! Antarmuka JNI untuk Java/Fabric. Pemrosesan dilakukan per batch
//! (banyak section dalam satu panggilan) untuk menekan overhead JNI.

use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::atomic::{AtomicBool, Ordering};

use jni::objects::{JByteArray, JClass, JIntArray};
use jni::sys::{jbyteArray, jint, jstring};
use jni::JNIEnv;

use crate::mesh::{build_section_mesh, Mesh, Neighbors, SECTION_VOLUME};

/// Batas pengaman agar input aneh tidak menghabiskan memori.
pub const MAX_SECTIONS_PER_BATCH: usize = 512;
pub const MAX_PALETTE_SIZE: usize = 65_536;

static INITIALIZED: AtomicBool = AtomicBool::new(false);

/// Logika murni (tanpa JNI) agar bisa dites di host.
/// `ids` berisi `count * 4096` ID block; hasil diserialisasi seperti dijelaskan di atas.
pub fn build_batch(ids: &[i32], flags: &[u8], count: usize) -> Result<Vec<u8>, String> {
    if count == 0 || count > MAX_SECTIONS_PER_BATCH {
        return Err(format!("invalid section count {count}"));
    }
    if ids.len() != count * SECTION_VOLUME {
        return Err(format!(
            "ids length {}, expected {}",
            ids.len(),
            count * SECTION_VOLUME
        ));
    }

    let mut section = vec![0u16; SECTION_VOLUME];
    let mut mesh = Mesh::default();
    let mut out: Vec<u8> = Vec::new();

    for chunk in ids.chunks_exact(SECTION_VOLUME) {
        for (dst, &src) in section.iter_mut().zip(chunk) {
            *dst = u16::try_from(src).map_err(|_| format!("block id {src} out of range"))?;
        }

        mesh.clear();
        build_section_mesh(&section, flags, &Neighbors::default(), &mut mesh)
            .map_err(|e| e.to_string())?;

        out.extend_from_slice(&(mesh.vertices.len() as u32).to_le_bytes());
        out.extend_from_slice(&(mesh.indices.len() as u32).to_le_bytes());
        for v in &mesh.vertices {
            out.extend_from_slice(&[v.x, v.y, v.z, v.face, v.corner, v.pad]);
            out.extend_from_slice(&v.block.to_le_bytes());
        }
        for i in &mesh.indices {
            out.extend_from_slice(&i.to_le_bytes());
        }
    }
    Ok(out)
}

fn build_sections_impl(
    env: &mut JNIEnv,
    ids: &JIntArray,
    flags: &JByteArray,
    count: jint,
) -> Result<jbyteArray, String> {
    if !INITIALIZED.load(Ordering::Acquire) {
        return Err("engine not initialized".to_string());
    }
    if count <= 0 {
        return Err(format!("invalid section count {count}"));
    }
    let n = count as usize;
    if n > MAX_SECTIONS_PER_BATCH {
        return Err(format!("section count {n} exceeds limit"));
    }

    let ids_len = env.get_array_length(ids).map_err(|e| e.to_string())? as usize;
    if ids_len != n * SECTION_VOLUME {
        return Err(format!("ids length {ids_len}, expected {}", n * SECTION_VOLUME));
    }
    let flags_len = env.get_array_length(flags).map_err(|e| e.to_string())? as usize;
    if flags_len == 0 || flags_len > MAX_PALETTE_SIZE {
        return Err(format!("invalid flags length {flags_len}"));
    }

    let mut id_buf = vec![0i32; ids_len];
    env.get_int_array_region(ids, 0, &mut id_buf)
        .map_err(|e| e.to_string())?;
    let mut flag_buf = vec![0i8; flags_len];
    env.get_byte_array_region(flags, 0, &mut flag_buf)
        .map_err(|e| e.to_string())?;
    let flag_bytes: Vec<u8> = flag_buf.iter().map(|&b| b as u8).collect();

    let out = build_batch(&id_buf, &flag_bytes, n)?;
    let arr = env.byte_array_from_slice(&out).map_err(|e| e.to_string())?;
    Ok(arr.into_raw())
}

/// String versi engine.
#[no_mangle]
pub extern "system" fn Java_com_rapz_n3xr_NativeBridge_nativeVersion<'l>(
    mut env: JNIEnv<'l>,
    _class: JClass<'l>,
) -> jstring {
    match env.new_string(crate::VERSION) {
        Ok(s) => s.into_raw(),
        Err(_) => std::ptr::null_mut(),
    }
}

/// Inisialisasi. Mengembalikan 0 jika berhasil. Aman dipanggil berulang.
#[no_mangle]
pub extern "system" fn Java_com_rapz_n3xr_NativeBridge_nativeInit<'l>(
    _env: JNIEnv<'l>,
    _class: JClass<'l>,
) -> jint {
    INITIALIZED.store(true, Ordering::Release);
    0
}

/// Shutdown. Setelah ini nativeBuildSections menolak panggilan sampai init lagi.
#[no_mangle]
pub extern "system" fn Java_com_rapz_n3xr_NativeBridge_nativeShutdown<'l>(
    _env: JNIEnv<'l>,
    _class: JClass<'l>,
) {
    INITIALIZED.store(false, Ordering::Release);
}

/// Membuat mesh untuk `count` section sekaligus.
/// Mengembalikan null dan melempar exception Java jika input tidak valid.
#[no_mangle]
pub extern "system" fn Java_com_rapz_n3xr_NativeBridge_nativeBuildSections<'l>(
    mut env: JNIEnv<'l>,
    _class: JClass<'l>,
    ids: JIntArray<'l>,
    flags: JByteArray<'l>,
    count: jint,
) -> jbyteArray {
    // Catatan: pada profil release `panic = "abort"`, catch_unwind tidak
    // menangkap apa pun; jalur ini aktif di build debug/test.
    let result = catch_unwind(AssertUnwindSafe(|| {
        build_sections_impl(&mut env, &ids, &flags, count)
    }));
    match result {
        Ok(Ok(ptr)) => ptr,
        Ok(Err(msg)) => {
            let _ = env.throw_new("java/lang/IllegalArgumentException", msg);
            std::ptr::null_mut()
        }
        Err(_) => {
            let _ = env.throw_new("java/lang/IllegalStateException", "native panic");
            std::ptr::null_mut()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::block::{FLAG_OPAQUE, FLAG_VISIBLE};

    const FLAGS: [u8; 2] = [0, FLAG_VISIBLE | FLAG_OPAQUE];

    #[test]
    fn single_block_serializes_expected_size() {
        let mut ids = vec![0i32; SECTION_VOLUME];
        ids[5 * 256 + 5 * 16 + 5] = 1;
        let out = build_batch(&ids, &FLAGS, 1).unwrap();
        // 8 byte header + 24 vertex * 8 + 36 indeks * 4
        assert_eq!(out.len(), 8 + 24 * 8 + 36 * 4);
        assert_eq!(u32::from_le_bytes(out[0..4].try_into().unwrap()), 24);
        assert_eq!(u32::from_le_bytes(out[4..8].try_into().unwrap()), 36);
    }

    #[test]
    fn two_sections_are_concatenated() {
        let ids = vec![0i32; SECTION_VOLUME * 2];
        let out = build_batch(&ids, &FLAGS, 2).unwrap();
        assert_eq!(out.len(), 16); // dua header kosong
    }

    #[test]
    fn rejects_wrong_length() {
        assert!(build_batch(&[0; 10], &FLAGS, 1).is_err());
    }

    #[test]
    fn rejects_zero_count() {
        assert!(build_batch(&[], &FLAGS, 0).is_err());
    }

    #[test]
    fn rejects_negative_id() {
        let mut ids = vec![0i32; SECTION_VOLUME];
        ids[0] = -1;
        assert!(build_batch(&ids, &FLAGS, 1).is_err());
    }

    #[test]
    fn rejects_id_above_u16() {
        let mut ids = vec![0i32; SECTION_VOLUME];
        ids[0] = 70_000;
        assert!(build_batch(&ids, &FLAGS, 1).is_err());
    }
}