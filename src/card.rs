use crate::error::Error;
use regex::Regex;
use scraper::{Html, Selector};

pub struct Card {
    pub html: String,
    pub reading: String,
    pub writing: String,
}

pub struct CardParser {
    card_selector: Selector,
    reading_selector: Selector,
    writing_selector: Selector,
    writing_regex: Regex,
}

impl CardParser {
    pub fn new() -> CardParser {
        CardParser {
            card_selector: Selector::parse("div.gloss-row:not(.hidden)>ul>li>div").unwrap(),
            reading_selector: Selector::parse(".info-link").unwrap(),
            writing_selector: Selector::parse(".alternatives>dt").unwrap(),
            writing_regex: Regex::new(r#"([^0-9【】\s.]+?)(\s【|$)"#).unwrap(),
        }
    }

    pub fn parse(&self, url: impl AsRef<str>) -> Result<Vec<Card>, Error> {
        Ok(
            Html::parse_document(&reqwest::blocking::get(url.as_ref())?.text()?)
                .select(&self.card_selector)
                .map(|element| -> Result<Card, Error> {
                    let html = element.inner_html();
                    let fragment = Html::parse_fragment(&html);
                    let reading = fragment
                        .select(&self.reading_selector)
                        .next()
                        .ok_or(Error::IchiMoeError(
                            "failed to find reading".to_owned(),
                            url.as_ref().to_owned(),
                        ))?
                        .inner_html()
                        .trim()
                        .to_owned();
                    let writing = fragment
                        .select(&self.writing_selector)
                        .find_map(|element| {
                            let text = element.inner_html().replace('\u{200b}', "");

                            let reading = reading.replace(' ', "").replace('·', "");
                            let reading = if let Some(idx) = reading.find('/') {
                                &reading[0..idx]
                            } else {
                                &reading
                            };

                            println!(
                                "checking if text {:?} matches reading {:?}",
                                &text, &reading
                            );

                            if text.replace(' ', "").contains(&reading) {
                                Some(
                                    self.writing_regex
                                        .captures(&text)
                                        .ok_or(Error::IchiMoeError(
                                            "found no matches to writing regex".to_owned(),
                                            url.as_ref().to_owned(),
                                        ))
                                        .map(|matched| matched.get(1).unwrap().as_str().to_owned()),
                                )
                            } else {
                                None
                            }
                        })
                        .ok_or(Error::IchiMoeError(
                            url.as_ref().to_owned(),
                            "found no writings that matched the reading".to_owned(),
                        ))??;

                    Ok(Card {
                        html,
                        reading,
                        writing,
                    })
                })
                .collect::<Result<Vec<_>, _>>()?,
        )
    }
}
