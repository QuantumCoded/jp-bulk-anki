use std::{collections::HashSet};

use genanki_rs::{Deck, Error, Field, Model, Note, Template};

pub struct DeckBuilder {
    deck: Deck,
    model: Model,
    used: HashSet<String>,
}

impl DeckBuilder {
    pub fn new(name: String, description: String) -> DeckBuilder {
        DeckBuilder {
            deck: Deck::new(
                2059400110,
                &name,
                &description,
            ),
            model: Model::new(
                1607392319,
                "Simple Model",
                vec![Field::new("Question"), Field::new("Answer")],
                vec![Template::new("Card")
                    .qfmt("{{Question}}")
                    .afmt(r#"<link rel="stylesheet" href="https://ichi.moe/css/ichiran.css?v=3850663369"><div class="gloss">{{Answer}}</div>"#)],
            ),
            used: HashSet::new()
        }
    }

    pub fn add(&mut self, question: String, answer: String) -> Result<(), Error> {
        if !self.used.contains(&answer) {
            self.deck
                .add_note(Note::new(self.model.clone(), vec![&question, &answer])?);

            self.used.insert(answer);
        }

        Ok(())
    }

    pub fn save(&self, path: impl AsRef<str>) -> Result<(), Error> {
        self.deck.write_to_file(path.as_ref())?;

        Ok(())
    }
}
