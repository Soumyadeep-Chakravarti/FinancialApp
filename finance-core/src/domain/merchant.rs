#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Merchant {
    pub name: String,
    pub upi_id: Option<String>,
}

impl Merchant {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            upi_id: None,
        }
    }

    pub fn with_upi_id(mut self, upi_id: impl Into<String>) -> Self {
        self.upi_id = Some(upi_id.into());
        self
    }
}

impl Default for Merchant {
    fn default() -> Self {
        Self {
            name: "Unknown".to_string(),
            upi_id: None,
        }
    }
}
