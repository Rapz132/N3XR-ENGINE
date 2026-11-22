use crate::block::Block;

pub const CHUNK_SIZE: usize = 16;
pub const CHUNK_HEIGHT: usize = 64;
pub const CHUNK_VOLUME: usize = CHUNK_SIZE * CHUNK_HEIGHT * CHUNK_SIZE;

pub struct Chunk {
    pub blocks: Vec<Block>,
}

impl Default for Chunk {
    fn default() -> Self {
        Self::new()
    }
}

impl Chunk {
    pub fn new() -> Self {
        Self {
            blocks: vec![Block::Air; CHUNK_VOLUME],
        }
    }

    /// Membuat chunk dari data yang sudah ada.
    /// Mengembalikan None jika panjang data tidak sama dengan CHUNK_VOLUME.
    pub fn from_blocks(blocks: Vec<Block>) -> Option<Self> {
        if blocks.len() != CHUNK_VOLUME {
            return None;
        }
        Some(Self { blocks })
    }

    pub fn get_index(x: usize, y: usize, z: usize) -> Option<usize> {
        if x >= CHUNK_SIZE || y >= CHUNK_HEIGHT || z >= CHUNK_SIZE {
            return None;
        }

        Some(y * CHUNK_SIZE * CHUNK_SIZE + z * CHUNK_SIZE + x)
    }

    pub fn get_block(&self, x: usize, y: usize, z: usize) -> Option<Block> {
        let index = Self::get_index(x, y, z)?;
        self.blocks.get(index).copied()
    }

    pub fn set_block(&mut self, x: usize, y: usize, z: usize, block: Block) -> bool {
        let Some(index) = Self::get_index(x, y, z) else {
            return false;
        };

        self.blocks[index] = block;
        true
    }
}