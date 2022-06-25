use crate::{card::Card, cli::Options, error::Error};
use genanki_rs::{Deck, Field, Model, Note, Template};
use std::collections::HashSet;

pub struct DeckBuilder<'a> {
    used: HashSet<String>,
    deck: Deck,
    fields: Vec<Field>,
    templates: Vec<Template>,
    css: String,
    options: &'a Options,
}

impl<'a> DeckBuilder<'a> {
    pub fn new(
        name: impl AsRef<str>,
        description: impl AsRef<str>,
        options: &'a Options,
    ) -> Result<DeckBuilder, Error> {
        let deck = Deck::new(2059400110, name.as_ref(), description.as_ref());
        let fields = vec![Field::new("Question"), Field::new("Answer")];
        let templates = vec![Template::new("Card")
            .qfmt("{{Question}}")
            .afmt(r#"<div class="gloss">{{Answer}}</div>"#)];
        let css = format!(
            r#"<style>* {{ color: black }}</style>{}"#,
            reqwest::blocking::get("https://ichi.moe/css/ichiran.css")?.text()?
        );

        Ok(DeckBuilder {
            used: HashSet::new(),
            deck,
            fields,
            templates,
            css,
            options,
        })
    }

    fn note(&self, card: &Card) -> Result<Note, Error> {
        let front = if self.options.writing {
            &card.writing
        } else {
            &card.reading
        };

        let fields: Vec<&str> = if self.options.writing {
            vec![front, &card.html]
        } else {
            vec![front, &card.html]
        };

        println!("creating note {:?}", front);

        Ok(Note::new(
            Model::new_with_options(
                1607392319,
                "Simple Model",
                self.fields.clone(),
                self.templates.clone(),
                Some(&self.css),
                None,
                None,
                None,
                Some(self.used.len() as i64),
            ),
            fields,
        )?)
    }

    pub fn add(&mut self, card: Card) -> Result<(), Error> {
        if !self.used.contains(&card.writing) {
            self.deck.add_note(self.note(&card)?);
            self.used.insert(card.writing);
        } else {
            println!("rejecting duplicate note {:?}", card.writing);
        }

        Ok(())
    }

    pub fn save(&self) -> Result<(), Error> {
        self.deck.write_to_file(&self.options.output)?;

        Ok(())
    }

    pub fn len(&self) -> usize {
        self.used.len()
    }
}
