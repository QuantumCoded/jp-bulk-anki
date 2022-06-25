use crate::error::Error;
use card::{Card, CardParser};
use deck::DeckBuilder;
use std::fs::File;
use std::io::{stdin, BufRead, BufReader};
use urlencoding::encode;

mod card;
mod cli;
mod deck;
mod error;

fn main() -> Result<(), main_error::MainError> {
    let options = cli::main()?;

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
    let card_parser = CardParser::new();
    let mut deck_builder =
        DeckBuilder::new(&options.deck_name, &options.deck_description, &options)?;

    input
        .filter_map(
            |line: Result<String, std::io::Error>| -> Option<Result<Vec<Card>, Error>> {
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
                .map(|url: Result<String, Error>| -> Result<Vec<Card>, Error> {
                    Ok(card_parser.parse(url?)?)
                })
            },
        )
        .map(|cards| -> Result<(), Error> {
            for card in cards? {
                deck_builder.add(card)?;
            }

            Ok(())
        })
        .collect::<Result<(), _>>()?;

    println!(
        "created deck {:?} with {} notes\nexporting as apkg...",
        options.deck_name,
        deck_builder.len()
    );

    deck_builder.save()?;

    Ok(())
}
