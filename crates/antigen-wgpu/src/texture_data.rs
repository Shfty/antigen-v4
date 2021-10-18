pub trait TextureData {
    fn is_dirty(&self) -> bool;
    fn set_dirty(&self, dirty: bool);
    fn texture_data(&self) -> &[u8];
}

impl TextureData for antigen_components::ImageComponent {
    fn is_dirty(&self) -> bool {
        self.0.is_dirty()
    }

    fn set_dirty(&self, dirty: bool) {
        self.0.set_dirty(dirty)
    }

    fn texture_data(&self) -> &[u8] {
        self.0.get().as_ref()
    }
}
