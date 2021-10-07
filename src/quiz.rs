use std::error::Error;

use string_error;
use yaserde::de;
use yaserde_derive::YaDeserialize;

#[derive(YaDeserialize)]
pub struct Quiz {
    #[yaserde(attribute)]
    pub name: Option<String>,
    #[yaserde(attribute)]
    pub author: Option<String>,
    #[yaserde(child)]
    pub category: Vec<Category>,
}

#[derive(YaDeserialize)]
pub struct Category {
    #[yaserde(attribute)]
    pub name: String,
    #[yaserde(child)]
    pub clue: Vec<Clue>,
}

#[derive(YaDeserialize)]
pub struct Clue {
    #[yaserde(child)]
    pub text: String,
}

impl Quiz {
    pub fn new(quiz_path: impl AsRef<std::path::Path>) -> Result<Self, Box<dyn Error>> {
        let quiz_file = std::fs::File::open(quiz_path)?;
        de::from_reader::<_, Quiz>(quiz_file).or_else(
            |s| Err(string_error::into_err(s))
        )
    }

    pub fn get_clue(&self, index: usize) -> &str {
        &self.category[index / 7].clue[index % 6 - 1].text
    }
}
