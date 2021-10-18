use on_change::OnChange;
use winit::dpi::Position;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct OuterPosition(pub OnChange<Position>);

impl<T> From<T> for OuterPosition
where
    T: Into<Position>,
{
    fn from(data: T) -> Self {
        let data = data.into();
        OuterPosition(OnChange::new_clean(data))
    }
}

impl std::ops::Deref for OuterPosition {
    type Target = OnChange<Position>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for OuterPosition {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

legion_debugger::register_component!(OuterPosition);
