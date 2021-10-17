use cgmath::Deg;

#[derive(Debug, Copy, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct FieldOfView {
    fov: Deg<f32>,
}

impl Default for FieldOfView {
    fn default() -> Self {
        FieldOfView { fov: Deg(90.0) }
    }
}

impl FieldOfView {
    const FOV_EPSILON: f32 = 1.0;
    const FOV_MIN: f32 = Self::FOV_EPSILON;
    const FOV_MAX: f32 = 180.0 - Self::FOV_EPSILON;

    pub fn new(fov: Deg<f32>) -> Self {
        FieldOfView {
            fov: Self::clamp_fov(fov),
        }
    }

    fn clamp_fov(Deg(fov): Deg<f32>) -> Deg<f32> {
        Deg(fov.min(Self::FOV_MAX).max(Self::FOV_MIN))
    }

    pub fn fov(&self) -> Deg<f32> {
        self.fov
    }

    pub fn set_fov(&mut self, fov: Deg<f32>) {
        self.fov = Self::clamp_fov(fov)
    }

    pub fn to_matrix(&self, aspect_ratio: f32, near: f32, far: f32) -> cgmath::Matrix4<f32> {
        cgmath::perspective(self.fov, aspect_ratio, near, far)
    }
}

legion_debugger::register_component!(FieldOfView);
