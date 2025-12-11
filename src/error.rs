use thiserror::Error;

#[derive(Error, Debug)]
pub enum GitHubFetchError {
    #[error("GitHub API error: {0}")]
    ApiError(String),

    #[error("Authentication error: {0}")]
    AuthError(String),

    #[error("Rate limit exceeded")]
    RateLimitExceeded,

    #[error("Invalid repository format: {0}")]
    InvalidRepository(String),

    #[error("Resource not found: {0}")]
    NotFound(String),

    #[error("Network error: {0}")]
    NetworkError(#[from] reqwest::Error),

    #[error("Octocrab error: {0}")]
    OctocrabError(#[from] octocrab::Error),

    #[error("JSON parsing error: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Other error: {0}")]
    Other(#[from] anyhow::Error),
}

pub type Result<T> = std::result::Result<T, GitHubFetchError>;
