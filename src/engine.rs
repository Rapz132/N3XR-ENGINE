#[cfg(feature = "legacy-gl")]
use crate::renderer::Renderer;
use crate::world::World;

pub struct Engine {
    pub initialized: bool,
    #[cfg(feature = "legacy-gl")]
    pub renderer: Renderer,
    pub world: World,
}

impl Engine {
    pub fn new() -> Result<Self, String> {
        Ok(Self {
            initialized: true,
            #[cfg(feature = "legacy-gl")]
            renderer: Renderer::new(),
            world: World::new(),
        })
    }

    pub fn update(&mut self, delta_time: f32) {
        if !self.initialized {
            return;
        }

        #[cfg(feature = "legacy-gl")]
        self.renderer.update(delta_time);
        self.world.update(delta_time);
    }

    pub fn is_initialized(&self) -> bool {
        self.initialized
    }
}