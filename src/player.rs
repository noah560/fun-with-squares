pub struct Player {
    pub x: f32,
    pub y: f32,
    pub z: f32
}

impl Player {
    pub fn new(
        x: f32,
        y: f32,
        z: f32
    ) -> Player {
        Player {
            x,
            y,
            z
        }
    }
}
