#[derive(Clone)]
pub struct MyInstant {
    pub url: String,
    pub name: String,
}

impl MyInstant {
    pub fn from_url(url: &str) -> Self {
        Self {
            url: url.to_owned(),
            name: url.to_owned(),
        }
    }

    pub fn with_name(&self, name: &str) -> Self {
        Self {
            url: self.url.clone(),
            name: name.to_owned(),
        }
    }
}
