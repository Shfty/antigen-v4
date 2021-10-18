/// A type that can cast itself to a slice of some [`Pod`] type
pub trait CastSlice<R>
where
    R: bytemuck::Pod,
{
    fn cast_slice(&self) -> &[R];
}

impl<R, T, const C: usize> CastSlice<R> for [T; C]
where
    R: bytemuck::Pod,
    T: bytemuck::Pod,
{
    fn cast_slice(&self) -> &[R] {
        bytemuck::cast_slice(self)
    }
}

impl<R, T> CastSlice<R> for &[T]
where
    R: bytemuck::Pod,
    T: bytemuck::Pod,
{
    fn cast_slice(&self) -> &[R] {
        bytemuck::cast_slice(self)
    }
}

impl<R, T> CastSlice<R> for Vec<T>
where
    R: bytemuck::Pod,
    T: bytemuck::Pod,
{
    fn cast_slice(&self) -> &[R] {
        bytemuck::cast_slice(self)
    }
}

impl CastSlice<u8> for antigen_cgmath::cgmath::Matrix4<f32> {
    fn cast_slice(&self) -> &[u8] {
        let mx: &[f32; 16] = self.as_ref();
        mx.cast_slice()
    }
}

impl CastSlice<u8> for antigen_components::Image {
    fn cast_slice(&self) -> &[u8] {
        self.data()
    }
}
