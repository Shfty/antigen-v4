#[derive(Debug, Copy, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct OrthographicProjection {
    left: f32,
    right: f32,
    bottom: f32,
    top: f32,
}

impl OrthographicProjection {
    pub fn new(left: f32, right: f32, bottom: f32, top: f32) -> Self {
        OrthographicProjection {
            left,
            right,
            bottom,
            top,
        }
    }

    pub fn left(&self) -> f32 {
        self.left
    }

    pub fn right(&self) -> f32 {
        self.right
    }

    pub fn bottom(&self) -> f32 {
        self.bottom
    }

    pub fn top(&self) -> f32 {
        self.top
    }

    pub fn set_left(&mut self, left: f32) {
        self.left = left
    }

    pub fn set_right(&mut self, right: f32) {
        self.right = right
    }

    pub fn set_bottom(&mut self, bottom: f32) {
        self.bottom = bottom
    }

    pub fn set_top(&mut self, top: f32) {
        self.top = top
    }

    pub fn to_matrix(&self, near: f32, far: f32) -> cgmath::Matrix4<f32> {
        cgmath::ortho(self.left, self.right, self.bottom, self.top, near, far)
    }
}

legion_debugger::register_component!(OrthographicProjection);
