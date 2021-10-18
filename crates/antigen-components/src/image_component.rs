use crate::Image;
use on_change::OnChange;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct ImageComponent(pub OnChange<Image>);

impl From<Image> for ImageComponent {
    fn from(image: Image) -> Self {
        ImageComponent(OnChange::new_dirty(image))
    }
}

legion_debugger::register_component!(ImageComponent);
