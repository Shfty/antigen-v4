use on_change::OnChange;
use winit::dpi::Position;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct ImePosition(pub OnChange<Position>);

impl<T> From<T> for ImePosition
where
    T: Into<Position>,
{
    fn from(data: T) -> Self {
        let data = data.into();
        ImePosition(OnChange::new_clean(data))
    }
}

impl std::ops::Deref for ImePosition {
    type Target = OnChange<Position>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for ImePosition {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

legion_debugger::register_component!(ImePosition);
