// crate-wide error enum (thiserror or hand-rolled)

pub type Result<T> = std::result::Result<T, LeetCodeError>;

#[derive(thiserror::Error, Debug)]
pub enum LeetCodeError {
    #[error("network request failed: {0}")]
    Request(#[from] reqwest::Error),

    #[error("failed to parse JSON: {0}")]
    Json(#[from] serde_json::Error),

    #[error("failed to parse TOML: {0}")]
    Toml(#[from] toml::de::Error),

    #[error("failed to find config dir")]
    ConfigDir,

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
