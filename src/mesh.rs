//! Face culling dan pembuatan vertex untuk satu section 16x16x16.

use crate::block::{FLAG_OPAQUE, FLAG_VISIBLE};

pub const SECTION_SIZE: usize = 16;
pub const SECTION_VOLUME: usize = SECTION_SIZE * SECTION_SIZE * SECTION_SIZE;

/// Urutan face: 0=-X, 1=+X, 2=-Y, 3=+Y, 4=-Z, 5=+Z.
const DIRS: [(i32, i32, i32); 6] = [
    (-1, 0, 0),
    (1, 0, 0),
    (0, -1, 0),
    (0, 1, 0),
    (0, 0, -1),
    (0, 0, 1),
];

/// Empat sudut per face (BL, BR, TR, TL), berlawanan arah jarum jam
/// jika dilihat dari luar block.
const CORNERS: [[[u8; 3]; 4]; 6] = [
    [[0, 0, 0], [0, 0, 1], [0, 1, 1], [0, 1, 0]], // -X
    [[1, 0, 1], [1, 0, 0], [1, 1, 0], [1, 1, 1]], // +X
    [[0, 0, 0], [1, 0, 0], [1, 0, 1], [0, 0, 1]], // -Y
    [[0, 1, 1], [1, 1, 1], [1, 1, 0], [0, 1, 0]], // +Y
    [[1, 0, 0], [0, 0, 0], [0, 1, 0], [1, 1, 0]], // -Z
    [[0, 0, 1], [1, 0, 1], [1, 1, 1], [0, 1, 1]], // +Z
];

/// Vertex ringkas, 8 byte. Koordinat 0..=16 relatif terhadap section.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Vertex {
    pub x: u8,
    pub y: u8,
    pub z: u8,
    pub face: u8,
    pub corner: u8,
    pub pad: u8,
    pub block: u16,
}

#[derive(Debug, Default)]
pub struct Mesh {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
}

impl Mesh {
    pub fn clear(&mut self) {
        self.vertices.clear();
        self.indices.clear();
    }

    pub fn face_count(&self) -> usize {
        self.indices.len() / 6
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum MeshError {
    BadSectionLength(usize),
    BadNeighborLength { face: usize, len: usize },
}

impl std::fmt::Display for MeshError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MeshError::BadSectionLength(n) => {
                write!(f, "section length {n}, expected {SECTION_VOLUME}")
            }
            MeshError::BadNeighborLength { face, len } => {
                write!(f, "neighbor {face} length {len}, expected {SECTION_VOLUME}")
            }
        }
    }
}

impl std::error::Error for MeshError {}

/// Section tetangga per arah (urutan sama dengan DIRS). None = belum dimuat;
/// face di perbatasan tetap dibuat (aman secara visual).
#[derive(Clone, Copy, Default)]
pub struct Neighbors<'a> {
    pub faces: [Option<&'a [u16]>; 6],
}

#[inline]
fn index(x: usize, y: usize, z: usize) -> usize {
    y * SECTION_SIZE * SECTION_SIZE + z * SECTION_SIZE + x
}

#[inline]
fn wrap(v: i32) -> usize {
    v.rem_euclid(SECTION_SIZE as i32) as usize
}

/// Membuat mesh satu section dan menambahkannya ke `out`
/// (panggil `out.clear()` dulu bila buffer dipakai ulang).
pub fn build_section_mesh(
    blocks: &[u16],
    flags: &[u8],
    neighbors: &Neighbors<'_>,
    out: &mut Mesh,
) -> Result<(), MeshError> {
    if blocks.len() != SECTION_VOLUME {
        return Err(MeshError::BadSectionLength(blocks.len()));
    }
    for (face, n) in neighbors.faces.iter().enumerate() {
        if let Some(s) = n {
            if s.len() != SECTION_VOLUME {
                return Err(MeshError::BadNeighborLength { face, len: s.len() });
            }
        }
    }

    let flag_of = |id: u16| flags.get(id as usize).copied().unwrap_or(0);
    let s = SECTION_SIZE as i32;

    for y in 0..SECTION_SIZE {
        for z in 0..SECTION_SIZE {
            for x in 0..SECTION_SIZE {
                let id = blocks[index(x, y, z)];
                if flag_of(id) & FLAG_VISIBLE == 0 {
                    continue;
                }
                for (face, (dx, dy, dz)) in DIRS.iter().enumerate() {
                    let (nx, ny, nz) = (x as i32 + dx, y as i32 + dy, z as i32 + dz);
                    let inside = (0..s).contains(&nx)
                        && (0..s).contains(&ny)
                        && (0..s).contains(&nz);
                    let neighbor_id = if inside {
                        Some(blocks[index(nx as usize, ny as usize, nz as usize)])
                    } else {
                        neighbors.faces[face].map(|sec| sec[index(wrap(nx), wrap(ny), wrap(nz))])
                    };
                    let hidden = match neighbor_id {
                        Some(n) => flag_of(n) & FLAG_OPAQUE != 0 || n == id,
                        None => false,
                    };
                    if hidden {
                        continue;
                    }
                    let base = out.vertices.len() as u32;
                    for (corner, c) in CORNERS[face].iter().enumerate() {
                        out.vertices.push(Vertex {
                            x: x as u8 + c[0],
                            y: y as u8 + c[1],
                            z: z as u8 + c[2],
                            face: face as u8,
                            corner: corner as u8,
                            pad: 0,
                            block: id,
                        });
                    }
                    out.indices
                        .extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
                }
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const FLAGS: [u8; 3] = [0, FLAG_VISIBLE | FLAG_OPAQUE, FLAG_VISIBLE];

    fn empty() -> Vec<u16> {
        vec![0; SECTION_VOLUME]
    }

    #[test]
    fn empty_section_has_no_faces() {
        let mut m = Mesh::default();
        build_section_mesh(&empty(), &FLAGS, &Neighbors::default(), &mut m).unwrap();
        assert_eq!(m.face_count(), 0);
    }

    #[test]
    fn single_block_has_six_faces() {
        let mut b = empty();
        b[index(5, 5, 5)] = 1;
        let mut m = Mesh::default();
        build_section_mesh(&b, &FLAGS, &Neighbors::default(), &mut m).unwrap();
        assert_eq!(m.face_count(), 6);
        assert_eq!(m.vertices.len(), 24);
        assert_eq!(m.indices.len(), 36);
    }

    #[test]
    fn adjacent_opaque_blocks_hide_shared_faces() {
        let mut b = empty();
        b[index(5, 5, 5)] = 1;
        b[index(6, 5, 5)] = 1;
        let mut m = Mesh::default();
        build_section_mesh(&b, &FLAGS, &Neighbors::default(), &mut m).unwrap();
        assert_eq!(m.face_count(), 10);
    }

    #[test]
    fn same_transparent_blocks_hide_shared_faces() {
        let mut b = empty();
        b[index(2, 2, 2)] = 2;
        b[index(3, 2, 2)] = 2;
        let mut m = Mesh::default();
        build_section_mesh(&b, &FLAGS, &Neighbors::default(), &mut m).unwrap();
        assert_eq!(m.face_count(), 10);
    }

    #[test]
    fn opaque_neighbor_section_hides_border_face() {
        let mut b = empty();
        b[index(0, 5, 5)] = 1;
        let solid = vec![1u16; SECTION_VOLUME];
        let mut n = Neighbors::default();
        n.faces[0] = Some(&solid);
        let mut m = Mesh::default();
        build_section_mesh(&b, &FLAGS, &n, &mut m).unwrap();
        assert_eq!(m.face_count(), 5);
    }

    #[test]
    fn wrong_length_is_rejected() {
        let mut m = Mesh::default();
        let r = build_section_mesh(&[0u16; 10], &FLAGS, &Neighbors::default(), &mut m);
        assert_eq!(r, Err(MeshError::BadSectionLength(10)));
    }

    #[test]
    fn unknown_id_is_treated_as_invisible() {
        let mut b = empty();
        b[0] = 999;
        let mut m = Mesh::default();
        build_section_mesh(&b, &FLAGS, &Neighbors::default(), &mut m).unwrap();
        assert_eq!(m.face_count(), 0);
    }
}