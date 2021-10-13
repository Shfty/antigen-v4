/// Owned manufacturer name / product name pair
#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DeviceName {
    manufacturer: String,
    product: String,
}

impl DeviceName {
    pub fn new(manufacturer: String, product: String) -> Self {
        DeviceName {
            manufacturer,
            product,
        }
    }

    pub fn manufacturer(&self) -> &str {
        self.manufacturer.as_str()
    }

    pub fn product(&self) -> &str {
        self.product.as_str()
    }
}
