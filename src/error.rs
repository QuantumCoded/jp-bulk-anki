#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("invalid output path {0:?}, must contain only UTF-8")]
    InvalidOutput(std::path::PathBuf),

    #[error("error reading input")]
    InputError(#[from] std::io::Error),

    #[error("failed to download html from ichi.moe")]
    HttpError(#[from] reqwest::Error),

    #[error("malformed ichi.moe page {0:?}")]
    IchiMoeError(String),

    #[error("failed to build anki package")]
    GenankiError(#[from] genanki_rs::Error)
}