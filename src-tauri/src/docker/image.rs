pub struct BcImage {
    id: String,
}

impl BcImage {
    pub fn new(id: String) -> Self {
        Self { id }
    }

    pub fn id(&self) -> &str {
        &self.id
    }
}
