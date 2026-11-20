
use crate::chunk::Chunk;

pub struct World {
    pub chunks: Vec<Chunk>,
    pub tick: u64,
}

impl World {
    pub fn new() -> Self {
        Self {
            chunks: vec![Chunk::new()],
            tick: 0,
        }
    }

    pub fn update(&mut self, _delta_time: f32) {
        self.tick += 1;
    }

    pub fn chunk_count(&self) -> usize {
        self.chunks.len()
    }
}
