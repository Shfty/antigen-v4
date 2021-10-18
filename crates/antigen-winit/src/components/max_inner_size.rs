use on_change::OnChange;
use winit::dpi::Size;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct MaxInnerSize(pub OnChange<Option<Size>>);

impl<T> From<T> for MaxInnerSize
where
    T: Into<Option<Size>>,
{
    fn from(data: T) -> Self {
        let data = data.into();
        MaxInnerSize(OnChange::new_clean(data))
    }
}

impl std::ops::Deref for MaxInnerSize {
    type Target = OnChange<Option<Size>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for MaxInnerSize {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

legion_debugger::register_component!(MaxInnerSize);
