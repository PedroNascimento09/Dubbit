use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Segment {
    pub index: usize,
    pub start: f64,
    pub end: f64,
    #[serde(default)]
    pub text: String,
    #[serde(default)]
    pub translation: String,
}

impl Segment {
    pub fn new(index: usize, start: f64, end: f64, text: impl Into<String>) -> Self {
        Self {
            index,
            start,
            end,
            text: text.into(),
            translation: String::new(),
        }
    }

    pub fn duration(&self) -> f64 {
        (self.end - self.start).max(0.0)
    }

    pub fn line(&self, translated: bool) -> &str {
        if translated {
            &self.translation
        } else {
            &self.text
        }
    }
}
