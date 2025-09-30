use crate::types::common::Vec2;

impl Vec2 {
    pub fn new(x: f32, y: f32) -> Self {
        Vec2 { x, y }
    }

    pub fn sub(&self, other: &Vec2) -> Vec2 {
        Vec2 {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }

    pub fn length(&self) -> f32 {
        (self.x * self.x + self.y * self.y).sqrt()
    }

    pub fn signed_angle_to(&self, other: &Vec2) -> f32 {
        // Calculate the angle from self to other
        let cross = self.x * other.y - self.y * other.x;
        let dot = self.x * other.x + self.y * other.y;
        cross.atan2(dot)
    }
}

impl From<glam::Vec2> for Vec2 {
    fn from(glam_vec: glam::Vec2) -> Self {
        Vec2 {
            x: glam_vec.x,
            y: glam_vec.y,
        }
    }
}

impl From<Vec2> for glam::Vec2 {
    fn from(vec: Vec2) -> Self {
        glam::Vec2 { x: vec.x, y: vec.y }
    }
}
