use on_change::OnChange;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct CursorVisible(pub OnChange<bool>);

impl<T> From<T> for CursorVisible
where
    T: Into<bool>,
{
    fn from(data: T) -> Self {
        let data = data.into();
        CursorVisible(OnChange::new_dirty(data))
    }
}

impl std::ops::Deref for CursorVisible {
    type Target = OnChange<bool>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for CursorVisible {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

legion_debugger::register_component!(CursorVisible);
