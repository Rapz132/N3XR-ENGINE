
pub struct Player {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub yaw: f32,
    pub pitch: f32,
    pub speed: f32,
}

impl Player {
    pub fn new() -> Self {
        Self {
            x: 0.0,
            y: 10.0,
            z: 0.0,
            yaw: 0.0,
            pitch: 0.0,
            speed: 4.0,
        }
    }

    pub fn move_by(&mut self, x: f32, y: f32, z: f32) {
        self.x += x * self.speed;
        self.y += y * self.speed;
        self.z += z * self.speed;
    }

    pub fn rotate(&mut self, yaw: f32, pitch: f32) {
        self.yaw += yaw;
        self.pitch = (self.pitch + pitch).clamp(-89.0, 89.0);
    }
}
