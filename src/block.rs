
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Block {
    Air = 0,
    Grass = 1,
    Dirt = 2,
    Stone = 3,
    Sand = 4,
    Water = 5,
}

impl Block {
    pub fn is_solid(self) -> bool {
        !matches!(self, Block::Air | Block::Water)
    }
}
