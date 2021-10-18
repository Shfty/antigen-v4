use on_change::OnChange;

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize)]
pub enum RedrawMode {
    None,
    MainEventsClearedRequest,
    MainEventsClearedLoop,
}

impl Default for RedrawMode {
    fn default() -> Self {
        RedrawMode::None
    }
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct RedrawModeComponent(pub OnChange<RedrawMode>);

impl<T> From<T> for RedrawModeComponent
where
    T: Into<RedrawMode>,
{
    fn from(data: T) -> Self {
        let data = data.into();
        RedrawModeComponent(OnChange::new_dirty(data))
    }
}

impl std::ops::Deref for RedrawModeComponent {
    type Target = OnChange<RedrawMode>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for RedrawModeComponent {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

legion_debugger::register_component!(RedrawModeComponent);
