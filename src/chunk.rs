
use crate::block::Block;

pub const CHUNK_SIZE: usize = 16;
pub const CHUNK_HEIGHT: usize = 64;

pub struct Chunk {
    pub blocks: Vec<Block>,
}

impl Chunk {
    pub fn new() -> Self {
        Self {
            blocks: vec![
                Block::Air;
                CHUNK_SIZE * CHUNK_HEIGHT * CHUNK_SIZE
            ],
        }
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
