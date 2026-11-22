/// Flag render per block ID. Dipakai oleh mesh generator.
pub const FLAG_VISIBLE: u8 = 1; // block digambar (bukan udara)
pub const FLAG_OPAQUE: u8 = 2; // block menutup face tetangganya

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

    /// Flag render untuk block ini.
    pub fn flags(self) -> u8 {
        match self {
            Block::Air => 0,
            Block::Water => FLAG_VISIBLE,
            _ => FLAG_VISIBLE | FLAG_OPAQUE,
        }
    }
}