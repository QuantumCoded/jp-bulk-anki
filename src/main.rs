use crate::error::Error;
use reqwest::blocking;
use scraper::{Html, Selector};
use std::io::{stdin, BufRead};
use urlencoding::encode;

mod cli;
mod deck_builder;
mod error;

fn main() -> Result<(), main_error::MainError> {
    let options = cli::main();
    let mut deck =
        crate::deck_builder::DeckBuilder::new(options.deck_name, options.deck_description);
    let stdin = stdin();

    stdin
        .lock()
        .lines()
        .filter_map(
            |line: Result<String, std::io::Error>| -> Option<Result<String, Error>> {
                match line {
                    Ok(line) if line.trim() == "" => None,
                    Err(error) => Some(Err(Error::StdinError(error))),
                    Ok(data) => {
                        use crate::cli::RomanizationMethod;

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

                    deck.add(
                        element.inner_html().trim().to_owned(),
                        card.select(&word_selector)
                            .next()
                            .ok_or(Error::IchiMoeError(url.clone()))?
                            .inner_html()
                            .trim()
                            .to_owned(),
                    )?;

                    Ok(())
                })
                .collect::<Result<(), _>>()?;

            Ok(())
        })
        .collect::<Result<(), _>>()?;

    deck.save(
        options
            .output
            .to_str()
            .ok_or(Error::InvalidOutput(options.output.clone()))?,
    )?;

    Ok(())
}
