use crate::error::Error;
use clap::{App, Arg};
use std::{ffi::OsStr, path::PathBuf};

pub enum RomanizationMethod {
    Kana,
    HepburnBasic,
    HepburnTraditional,
    HepburnModified,
    Kunreisiki,
}

pub struct Options {
    pub romanization_method: RomanizationMethod,
    pub convert_katakana_to_hiragana: bool,
    pub deck_name: String,
    pub deck_description: String,
    pub output: String,
    pub input: Option<PathBuf>,
    pub writing: bool,
}

pub fn main() -> Result<Options, Error> {
    use RomanizationMethod::*;

    let args = cli().get_matches();

    Ok(Options {
        romanization_method: match args
            .get_one::<String>("romanization_method")
            .expect("RomanizationMethod has default")
            .as_str()
        {
            "kana" => Kana,
            "hepburnbasic" => HepburnBasic,
            "hepburntraditional" => HepburnTraditional,
            "hepburnmodified" => HepburnModified,
            "kunreisiki" => Kunreisiki,
            _ => unreachable!(),
        },
        convert_katakana_to_hiragana: args.contains_id("convert_katakana"),
        deck_name: args
            .get_one::<String>("deck_name")
            .expect("DeckName is required")
            .to_owned(),
        deck_description: args
            .get_one::<String>("deck_name")
            .expect("DeckDescription has default")
            .to_owned(),
        output: {
            let mut path = PathBuf::from(
                args.get_one::<String>("output")
                    .expect("OUTPUT is required"),
            );

            match path.extension() {
                Some(extension) if extension == OsStr::new("apkg") => path
                    .to_str()
                    .ok_or(Error::InvalidOutput(path.clone()))?
                    .to_owned(),
                Some(extension) => {
                    let mut extension = extension.to_os_string();
                    extension.push(".apkg");
                    path.set_extension(extension);
                    path.to_str()
                        .ok_or(Error::InvalidOutput(path.clone()))?
                        .to_owned()
                }
                None => {
                    path.set_extension("apkg");
                    path.to_str()
                        .ok_or(Error::InvalidOutput(path.clone()))?
                        .to_owned()
                }
            }
        },
        input: args
            .get_one::<String>("input")
            .map(|path| PathBuf::from(path)),
        writing: args.contains_id("writing"),
    })
}

fn cli() -> App<'static> {
    clap::command!()
        .arg(
            Arg::new("romanization_method")
                .help("The romanization method, determines what's on the front of cards when in reading mode")
                .value_name("RomanizationMethod")
                .short('r')
                .long("romanization-method")
                .default_value("kana")
                .value_parser([
                    "kana",
                    "hepburnbasic",
                    "hepburntraditional",
                    "hepburnmodified",
                    "kunreisiki",
                ]),
        )
        .arg(
            Arg::new("convert_katakana")
                .help("Covert katakana to hiragana")
                .value_name("ConvertKatakana")
                .short('c')
                .long("convert-katakana"),
        )
        .arg(
            Arg::new("deck_name")
                .help("The name to assign to the generated anki deck")
                .value_name("DeckName")
                .index(1)
                .required(true),
        )
        .arg(
            Arg::new("deck_description")
                .help("The description to assign to the generated anki deck")
                .value_name("DeckDescription")
                .short('d')
                .long("description")
                .default_value("JP Bulk Anki auto-generated deck"),
        )
        .arg(
            Arg::new("output")
                .help("Output anki package filename")
                .value_name("OUTPUT")
                .index(2)
                .required(true),
        )
        .arg(
            Arg::new("input")
                .help("Input JP text file (disables stdin)")
                .value_name("INPUT")
                .short('i')
                .long("input"),
        )
        .arg(
            Arg::new("writing")
                .help("Changes the front of cards from the words reading to its writing")
                .short('w')
                .long("writing")
                .conflicts_with("romanization_method")
        )
}
