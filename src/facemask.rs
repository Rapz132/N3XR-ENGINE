//! Mask face terlihat per blok untuk satu section 16x16x16.

use crate::block::{FLAG_OPAQUE, FLAG_VISIBLE};
use crate::mesh::{SECTION_SIZE, SECTION_VOLUME};

const DIRS: [(i32, i32, i32); 6] = [
    (-1, 0, 0),
    (1, 0, 0),
    (0, -1, 0),
    (0, 1, 0),
    (0, 0, -1),
    (0, 0, 1),
];

/// Mengisi `out` (panjang 4096) dengan mask face per blok.
pub fn build_face_masks(blocks: &[u16], flags: &[u8], out: &mut [u8]) -> Result<(), String> {
    if blocks.len() != SECTION_VOLUME {
        return Err(format!(
            "section length {}, expected {SECTION_VOLUME}",
            blocks.len()
        ));
    }
    if out.len() != SECTION_VOLUME {
        return Err(format!(
            "output length {}, expected {SECTION_VOLUME}",
            out.len()
        ));
    }

    let flag_of = |id: u16| flags.get(id as usize).copied().unwrap_or(0);
    let s = SECTION_SIZE as i32;

    for y in 0..SECTION_SIZE {
        for z in 0..SECTION_SIZE {
            for x in 0..SECTION_SIZE {
                let i = y * 256 + z * 16 + x;
                let id = blocks[i];
                if flag_of(id) & FLAG_VISIBLE == 0 {
                    out[i] = 0;
                    continue;
                }
                let mut mask = 0u8;
                for (face, (dx, dy, dz)) in DIRS.iter().enumerate() {
                    let (nx, ny, nz) = (x as i32 + dx, y as i32 + dy, z as i32 + dz);
                    let inside =
                        (0..s).contains(&nx) && (0..s).contains(&ny) && (0..s).contains(&nz);
                    let hidden = if inside {
                        let n = blocks[ny as usize * 256 + nz as usize * 16 + nx as usize];
                        flag_of(n) & FLAG_OPAQUE != 0 || n == id
                    } else {
                        false
                    };
                    if !hidden {
                        mask |= 1 << face;
                    }
                }
                out[i] = mask;
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const FLAGS: [u8; 3] = [0, FLAG_VISIBLE | FLAG_OPAQUE, FLAG_VISIBLE];

    fn run(blocks: &[u16]) -> Vec<u8> {
        let mut out = vec![0u8; SECTION_VOLUME];
        build_face_masks(blocks, &FLAGS, &mut out).unwrap();
        out
    }

    #[test]
    fn air_has_empty_mask() {
        let out = run(&vec![0u16; SECTION_VOLUME]);
        assert!(out.iter().all(|&m| m == 0));
    }

    #[test]
    fn single_block_has_all_six_bits() {
        let mut b = vec![0u16; SECTION_VOLUME];
        b[5 * 256 + 5 * 16 + 5] = 1;
        let out = run(&b);
        assert_eq!(out[5 * 256 + 5 * 16 + 5], 0b11_1111);
    }

    #[test]
    fn shared_face_is_hidden() {
        let mut b = vec![0u16; SECTION_VOLUME];
        b[5 * 256 + 5 * 16 + 5] = 1;
        b[5 * 256 + 5 * 16 + 6] = 1;
        let out = run(&b);
        // blok kiri kehilangan face +X (bit 1), blok kanan kehilangan -X (bit 0)
        assert_eq!(out[5 * 256 + 5 * 16 + 5], 0b11_1101);
        assert_eq!(out[5 * 256 + 5 * 16 + 6], 0b11_1110);
    }

    #[test]
    fn same_transparent_blocks_hide_each_other() {
        let mut b = vec![0u16; SECTION_VOLUME];
        b[2 * 256 + 2 * 16 + 2] = 2;
        b[2 * 256 + 2 * 16 + 3] = 2;
        let out = run(&b);
        assert_eq!(out[2 * 256 + 2 * 16 + 2] & 0b10, 0);
    }

    #[test]
    fn wrong_length_is_rejected() {
        let mut out = vec![0u8; SECTION_VOLUME];
        assert!(build_face_masks(&[0u16; 10], &FLAGS, &mut out).is_err());
    }
}