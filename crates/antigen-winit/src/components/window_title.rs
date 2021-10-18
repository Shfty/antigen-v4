use std::borrow::Cow;
use on_change::OnChange;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct WindowTitle(pub OnChange<Cow<'static, str>>);

impl<T> From<T> for WindowTitle
where
    T: Into<Cow<'static, str>>,
{
    fn from(data: T) -> Self {
        let data = data.into();
        WindowTitle(OnChange::new_dirty(data))
    }
}

impl std::ops::Deref for WindowTitle {
    type Target = OnChange<Cow<'static, str>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for WindowTitle {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

legion_debugger::register_component!(WindowTitle);
