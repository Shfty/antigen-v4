use on_change::OnChange;

use winit::window::CursorIcon as WinitCursorIcon;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct CursorIcon(pub OnChange<WinitCursorIcon>);

impl<T> From<T> for CursorIcon
where
    T: Into<WinitCursorIcon>,
{
    fn from(data: T) -> Self {
        let data = data.into();
        CursorIcon(OnChange::new_dirty(data))
    }
}

impl std::ops::Deref for CursorIcon {
    type Target = OnChange<WinitCursorIcon>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for CursorIcon {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

legion_debugger::register_component!(CursorIcon);
