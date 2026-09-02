// crate-wide error enum (thiserror or hand-rolled)

use std::fs;

pub type Result<T> = std::result::Result<T, LeetCodeError>;

#[derive(thiserror::Error, Debug)]
pub enum LeetCodeError {
    /* Package related errors */
    #[error("network request failed: {0}")]
    Request(#[from] reqwest::Error),

    #[error("failed to parse JSON: {0}")]
    Json(#[from] serde_json::Error),

    #[error("failed to deserialize TOML: {0}")]
    TomlDeserialize(#[from] toml::de::Error),

    #[error("failed to serialize TOML: {0}")]
    TomlSerialize(#[from] toml::ser::Error),

    #[error("failed to enact IO: {0}")]
    IO(#[from] std::io::Error),

    #[error("failed to parse spec HTML: {0}")]
    HtmlConversion(#[from] html_to_markdown_rs::ConversionError),

    /* Handrolled Errors */
    #[error("failed to create dir: {0}")]
    CargoInitFailed(String),

    #[error("no storage directory initialised")]
    NoStorageDir,

    #[error("failed to find config dir")]
    ConfigDir,

    #[error("problem already pulled: use 'pull <slug> --force' for a hard reset")]
    AlreadyPulled(String),

    #[error("html parse succeeded, but still had no content")]
    NoMdContent,

    #[error("problem does not have a solution snippet for language: {0}")]
    UnsupportedLanguage(String),

    #[error("Took too long waiting for testcases to complete")]
    TestingTooLong,

    /* Status Error Series */
    // 401/403
    #[error("failed to authenticate token")]
    NotAuthenticated,

    // 404
    #[error("failed to find requested problem: {0}")]
    ProblemNotFound(String),

    // 429
    #[error("failed: rate limited")]
    TooManyRequests,

    // generic catch
    #[error("failed: http status error {0}")]
    StatusError(reqwest::StatusCode),
}
