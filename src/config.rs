use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FetchConfig {
    pub github: GitHubConfig,
    pub rate_limiting: RateLimitConfig,
}

impl Default for FetchConfig {
    fn default() -> Self {
        Self {
            github: GitHubConfig::default(),
            rate_limiting: RateLimitConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubConfig {
    pub token_env_var: String,
    pub api_base_url: String,
    pub user_agent: String,
    pub timeout_seconds: u64,
}

impl Default for GitHubConfig {
    fn default() -> Self {
        Self {
            token_env_var: "GITHUB_TOKEN".to_string(),
            api_base_url: "https://api.github.com".to_string(),
            user_agent: "github-fetch/0.1.0".to_string(),
            timeout_seconds: 30,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    pub requests_per_minute: u32,
    pub delay_between_requests_ms: u64,
    pub respect_github_rate_limits: bool,
    pub max_retries: u32,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            requests_per_minute: 60,
            delay_between_requests_ms: 1000,
            respect_github_rate_limits: true,
            max_retries: 3,
        }
    }
}

impl RateLimitConfig {
    pub fn delay_duration(&self) -> Duration {
        Duration::from_millis(self.delay_between_requests_ms)
    }
}
