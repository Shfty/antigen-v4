use std::borrow::Cow;

#[derive(
    Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize,
)]
pub struct Name(pub Cow<'static, str>);

impl Name {
    pub fn new<T: Into<Cow<'static, str>>>(v: T) -> Self {
        Name(v.into())
    }
}

impl std::ops::Deref for Name {
    type Target = Cow<'static, str>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for Name {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

legion_debugger::register_component!(Name);
