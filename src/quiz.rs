//! XML game definitions for the six-column, five-row board.

use std::{error::Error, fs::File, io::Read, path::Path};

use bevy::prelude::Resource;
use xml::{EventReader, attribute::OwnedAttribute, reader::XmlEvent};

pub const CATEGORY_COUNT: usize = 6;
pub const CLUE_COUNT: usize = 5;

type QuizResult<T> = Result<T, Box<dyn Error>>;

#[derive(Resource)]
pub struct Quiz {
    pub name: String,
    pub categories: Vec<Category>,
}

pub struct Category {
    pub name: String,
    pub clues: Vec<String>,
}

impl Quiz {
    /// Load a game definition from an XML file.
    pub fn load(path: impl AsRef<Path>) -> QuizResult<Self> {
        Self::read(File::open(path)?)
    }

    /// Parse and validate a game before creating the board.
    fn read(reader: impl Read) -> QuizResult<Self> {
        let mut reader = EventReader::new(reader);
        let XmlEvent::StartElement {
            name, attributes, ..
        } = next_tag(&mut reader)?
        else {
            return Err("expected a <game> element".into());
        };
        if name.local_name != "game" {
            return Err("expected a <game> element".into());
        }
        let name = attribute(&attributes, "name").unwrap_or_else(|| "Quiz!".into());
        let mut categories = Vec::new();
        loop {
            match next_tag(&mut reader)? {
                XmlEvent::StartElement {
                    name, attributes, ..
                } if name.local_name == "category" => {
                    categories.push(Category::read(&mut reader, &attributes)?);
                }
                XmlEvent::EndElement { name } if name.local_name == "game" => break,
                _ => return Err("expected <category> or </game>".into()),
            }
        }
        if next_tag(&mut reader)? != XmlEvent::EndDocument {
            return Err("unexpected content after </game>".into());
        }
        if categories.len() != CATEGORY_COUNT {
            return Err(format!(
                "expected {CATEGORY_COUNT} categories, found {}",
                categories.len()
            )
            .into());
        }
        Ok(Self { name, categories })
    }

    /// Look up a clue using the board's right-to-left box numbering.
    /// Each column begins with a non-clickable category header.
    pub fn clue_for_box(&self, index: usize) -> Option<&str> {
        let rows = CLUE_COUNT + 1;
        let category = (CATEGORY_COUNT - 1).checked_sub(index / rows)?;
        let clue = (index % rows).checked_sub(1)?;
        Some(self.categories.get(category)?.clues.get(clue)?.as_str())
    }
}

impl Category {
    fn read(
        reader: &mut EventReader<impl Read>,
        attributes: &[OwnedAttribute],
    ) -> QuizResult<Self> {
        let name = attribute(attributes, "name").ok_or("category is missing a name")?;
        let mut clues = Vec::new();
        loop {
            match next_tag(reader)? {
                XmlEvent::StartElement { name, .. } if name.local_name == "clue" => {
                    clues.push(read_clue(reader)?);
                }
                XmlEvent::EndElement { name } if name.local_name == "category" => break,
                _ => return Err("expected <clue> or </category>".into()),
            }
        }
        if clues.len() != CLUE_COUNT {
            return Err(format!(
                "category {name:?}: expected {CLUE_COUNT} clues, found {}",
                clues.len()
            )
            .into());
        }
        Ok(Self { name, clues })
    }
}

fn attribute(attributes: &[OwnedAttribute], name: &str) -> Option<String> {
    attributes
        .iter()
        .find(|a| a.name.local_name == name)
        .map(|a| a.value.clone())
}

/// Skip formatting whitespace between XML elements, but not in clues.
fn next_tag(reader: &mut EventReader<impl Read>) -> QuizResult<XmlEvent> {
    loop {
        match reader.next()? {
            XmlEvent::StartDocument { .. }
            | XmlEvent::Whitespace(_)
            | XmlEvent::Comment(_)
            | XmlEvent::ProcessingInstruction { .. } => continue,
            event => return Ok(event),
        }
    }
}

fn read_clue(reader: &mut EventReader<impl Read>) -> QuizResult<String> {
    let mut text = String::new();
    loop {
        match reader.next()? {
            XmlEvent::Characters(s) | XmlEvent::CData(s) | XmlEvent::Whitespace(s) => {
                text.push_str(&s)
            }
            XmlEvent::Comment(_) => continue,
            XmlEvent::EndElement { name } if name.local_name == "clue" => return Ok(text),
            _ => return Err("expected plain text inside <clue>".into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn example_xml() -> String {
        let mut source = String::from("<game name=\"Test &amp; Quiz\" author=\"Tester\">");
        for column in 0..CATEGORY_COUNT {
            source.push_str(&format!("<category name=\"Category {column}&#10;Name\">"));
            for row in 0..CLUE_COUNT {
                source.push_str(&format!(
                    "<clue>Clue {column}/{row}\n &lt; &amp; &gt;\n&#32;</clue>"
                ));
            }
            source.push_str("</category>");
        }
        source.push_str("</game>");
        source
    }

    #[test]
    fn parses_xml_and_maps_every_clue() {
        let quiz = Quiz::read(example_xml().as_bytes()).unwrap();
        assert_eq!(quiz.name, "Test & Quiz");
        for column in 0..CATEGORY_COUNT {
            assert_eq!(
                quiz.categories[column].name,
                format!("Category {column}\nName")
            );
            let box_base = (CATEGORY_COUNT - 1 - column) * (CLUE_COUNT + 1);
            assert_eq!(quiz.clue_for_box(box_base), None);
            for row in 0..CLUE_COUNT {
                let expected = format!("Clue {column}/{row}\n < & >\n ");
                assert_eq!(
                    quiz.clue_for_box(box_base + row + 1),
                    Some(expected.as_str())
                );
            }
        }
        assert_eq!(quiz.clue_for_box(36), None);
    }

    #[test]
    fn loads_bundled_quiz() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/gibli-quiz.xml");
        let quiz = Quiz::load(path).unwrap();
        assert_eq!(quiz.name, "GHIBLI JEOPARDY");
        assert_eq!(quiz.categories[0].name, "General\nMovies");
        assert_eq!(quiz.categories[2].name, "Creatures\n& Animals");
        assert_eq!(quiz.categories[5].name, "Random\n& Misc");
        assert!(
            quiz.clue_for_box(31)
                .unwrap()
                .starts_with("This Ghibli film\nfamously features")
        );
        assert!(
            quiz.clue_for_box(1)
                .unwrap()
                .starts_with("This “theme park”")
        );
    }

    #[test]
    fn rejects_invalid_games() {
        assert!(Quiz::read("<quiz/>".as_bytes()).is_err());
        assert!(Quiz::read("<game/>".as_bytes()).is_err());
        assert!(Quiz::read("<game><category name=\"Too short\"/></game>".as_bytes()).is_err());
        assert!(
            Quiz::read("<game><category><clue>Missing name</clue></category></game>".as_bytes())
                .is_err()
        );
        assert!(
            Quiz::read("<game><category name=\"Broken\"><clue></category></game>".as_bytes())
                .is_err()
        );
        let extra = example_xml().replace("</game>", "</game><game/>");
        assert!(Quiz::read(extra.as_bytes()).is_err());
    }
}
