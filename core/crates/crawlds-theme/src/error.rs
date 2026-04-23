use thiserror::Error;

#[derive(Error, Debug)]
pub enum ThemeError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("TOML parse error: {0}")]
    TomlParse(#[from] toml::de::Error),
    #[error("Theme not found: {0}")]
    NotFound(String),
    #[error("Invalid theme: {0}")]
    Invalid(String),
    #[error("Invalid hex color: {0}")]
    InvalidColor(String),
    #[error("Matugen error: {0}")]
    Matugen(String),
    #[error("Matugen not found")]
    MatugenNotFound,
    #[error("JSON parse error: {0}")]
    JsonParse(#[from] serde_json::Error),
}

pub type ThemeResult<T> = Result<T, ThemeError>;
