use on_change::OnChange;
use winit::dpi::Size;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct InnerSize(pub OnChange<Size>);

impl<T> From<T> for InnerSize
where
    T: Into<Size>,
{
    fn from(data: T) -> Self {
        let data = data.into();
        InnerSize(OnChange::new_dirty(data))
    }
}

impl std::ops::Deref for InnerSize {
    type Target = OnChange<Size>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for InnerSize {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

legion_debugger::register_component!(InnerSize);
