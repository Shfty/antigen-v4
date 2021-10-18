use crate::Image;
use on_change::{OnChange, OnChangeTrait};

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct ImageComponent(pub OnChange<Image>);

impl From<Image> for ImageComponent {
    fn from(image: Image) -> Self {
        ImageComponent(OnChange::new_dirty(image))
    }
}

impl OnChangeTrait<Image> for ImageComponent {
    fn take_change(&self) -> Option<&Image> {
        self.0.take_change()
    }
}

legion_debugger::register_component!(ImageComponent);
