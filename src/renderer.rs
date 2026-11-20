
pub struct Renderer {
    pub frame_count: u64,
    pub delta_time: f32,
}

impl Renderer {
    pub fn new() -> Self {
        Self {
            frame_count: 0,
            delta_time: 0.0,
        }
    }

    pub fn update(&mut self, delta_time: f32) {
        self.delta_time = delta_time.max(0.0);
        self.frame_count += 1;
    }

    pub fn frame_count(&self) -> u64 {
        self.frame_count
    }
}
