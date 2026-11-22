//! JNI untuk mask face per batch. Tanpa state, tidak butuh nativeInit.

use std::panic::{catch_unwind, AssertUnwindSafe};

use jni::objects::{JByteArray, JClass, JIntArray};
use jni::sys::{jbyteArray, jint};
use jni::JNIEnv;

use crate::facemask::build_face_masks;
use crate::jni_api::{MAX_PALETTE_SIZE, MAX_SECTIONS_PER_BATCH};
use crate::mesh::SECTION_VOLUME;

/// Logika murni (tanpa JNI) agar bisa dites di host.
pub fn build_mask_batch(ids: &[i32], flags: &[u8], count: usize) -> Result<Vec<u8>, String> {
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

    let mut out = vec![0u8; count * SECTION_VOLUME];
    let mut section = vec![0u16; SECTION_VOLUME];

    for (s, chunk) in ids.chunks_exact(SECTION_VOLUME).enumerate() {
        for (dst, &src) in section.iter_mut().zip(chunk) {
            *dst = u16::try_from(src).map_err(|_| format!("block id {src} out of range"))?;
        }
        let range = s * SECTION_VOLUME..(s + 1) * SECTION_VOLUME;
        build_face_masks(&section, flags, &mut out[range])?;
    }
    Ok(out)
}

fn build_masks_impl(
    env: &mut JNIEnv,
    ids: &JIntArray,
    flags: &JByteArray,
    count: jint,
) -> Result<jbyteArray, String> {
    if count <= 0 {
        return Err(format!("invalid section count {count}"));
    }
    let n = count as usize;
    if n > MAX_SECTIONS_PER_BATCH {
        return Err(format!("section count {n} exceeds limit"));
    }

    let ids_len = env.get_array_length(ids).map_err(|e| e.to_string())? as usize;
    if ids_len != n * SECTION_VOLUME {
        return Err(format!(
            "ids length {ids_len}, expected {}",
            n * SECTION_VOLUME
        ));
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

    let out = build_mask_batch(&id_buf, &flag_bytes, n)?;
    let arr = env.byte_array_from_slice(&out).map_err(|e| e.to_string())?;
    Ok(arr.into_raw())
}

/// Mask face untuk `count` section. Null + exception Java jika input tidak valid.
#[no_mangle]
pub extern "system" fn Java_com_rapz_n3xr_FaceMaskBridge_nativeBuildFaceMasks<'l>(
    mut env: JNIEnv<'l>,
    _class: JClass<'l>,
    ids: JIntArray<'l>,
    flags: JByteArray<'l>,
    count: jint,
) -> jbyteArray {
    let result = catch_unwind(AssertUnwindSafe(|| {
        build_masks_impl(&mut env, &ids, &flags, count)
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
    fn batch_has_one_mask_per_block() {
        let mut ids = vec![0i32; SECTION_VOLUME * 2];
        ids[SECTION_VOLUME + 5 * 256 + 5 * 16 + 5] = 1;
        let out = build_mask_batch(&ids, &FLAGS, 2).unwrap();
        assert_eq!(out.len(), SECTION_VOLUME * 2);
        assert_eq!(out[5 * 256 + 5 * 16 + 5], 0);
        assert_eq!(out[SECTION_VOLUME + 5 * 256 + 5 * 16 + 5], 0b11_1111);
    }

    #[test]
    fn rejects_bad_input() {
        assert!(build_mask_batch(&[0; 10], &FLAGS, 1).is_err());
        assert!(build_mask_batch(&[], &FLAGS, 0).is_err());
        let mut ids = vec![0i32; SECTION_VOLUME];
        ids[0] = -1;
        assert!(build_mask_batch(&ids, &FLAGS, 1).is_err());
    }
}