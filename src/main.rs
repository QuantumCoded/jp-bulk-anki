use crate::error::Error;
use genanki_rs::{Deck, Field, Model, Note, Template};
use reqwest::blocking;
use scraper::{Html, Selector};
use std::fs::File;
use std::io::{stdin, BufRead, BufReader};
use urlencoding::encode;

mod cli;
mod error;

fn main() -> Result<(), main_error::MainError> {
    let options = cli::main();

    println!("generating anki deck, this may take a few minutes...");

    match &options.input {
        Some(file) => build_deck(BufReader::new(File::open(file)?).lines(), options),
        None => {
            let stdin = stdin();
            build_deck(stdin.lock().lines(), options)
        }
    }?;

    println!("finished!");

    Ok(())
}

fn build_deck(
    input: impl Iterator<Item = std::io::Result<String>>,
    options: crate::cli::Options,
) -> Result<(), Error> {
    let mut deck = Deck::new(2059400110, &options.deck_name, &options.deck_description);
    let model = Model::new(
        1607392319,
        "Simple Model",
        vec![Field::new("Question"), Field::new("Answer")],
        vec![Template::new("Card").qfmt("{{Question}}").afmt(
            r#"<link rel="stylesheet" href="https://ichi.moe/css/ichiran.css?v=3850663369">
<style>* { color: black }</style>                    
<div class="gloss">{{Answer}}</div>"#,
        )],
    );

    input
        .filter_map(
            |line: Result<String, std::io::Error>| -> Option<Result<String, Error>> {
                match line {
                    Ok(line) if line.trim() == "" => None,
                    Err(error) => Some(Err(error.into())),
                    Ok(data) => {
                        use crate::cli::RomanizationMethod;

                        println!("preforming lookup for {:?}", &data);

                        Some(Ok(format!(
                            "https://ichi.moe/cl/qr/?q={}&r={}{}",
                            encode(&data),
                            match options.romanization_method {
                                RomanizationMethod::Kana => "kana",
                                RomanizationMethod::HepburnBasic => "hb",
                                RomanizationMethod::HepburnTraditional => "ht",
                                RomanizationMethod::HepburnModified => "hm",
                                RomanizationMethod::Kunreisiki => "ks",
                            },
                            if options.convert_katakana_to_hiragana {
                                "&hira=on"
                            } else {
                                ""
                            }
                        )))
                    }
                }
            },
        )
        .map(|url| -> Result<(), Error> {
            let url = url?;

            let card_selector = Selector::parse("div.gloss-row:not(.hidden)>ul>li>div").unwrap();
            let word_selector = Selector::parse(".info-link").unwrap();

            let html = blocking::get(&url)?.text()?;
            let document = Html::parse_document(&html);

            document
                .select(&card_selector)
                .map(|element| -> Result<(), Error> {
                    let card = Html::parse_fragment(&element.inner_html());

                    let question = card
                        .select(&word_selector)
                        .next()
                        .ok_or(Error::IchiMoeError(url.clone()))?
                        .inner_html()
                        .trim()
                        .to_owned();
                    let answer = element.inner_html().trim().to_owned();

                    deck.add_note(Note::new(model.clone(), vec![&question, &answer])?);

                    println!("adding card {:?}", &question);

                    Ok(())
                })
                .collect::<Result<(), _>>()?;

            Ok(())
        })
        .collect::<Result<(), _>>()?;

    println!("done!\nexporting...");

    deck.write_to_file(
        options
            .output
            .to_str()
            .ok_or(Error::InvalidOutput(options.output.clone()))?,
    )?;

    Ok(())
}
