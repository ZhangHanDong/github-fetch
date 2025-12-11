pub mod client;
pub mod config;
pub mod discussion;
pub mod error;
pub mod filters;
pub mod types;

pub use client::GitHubClient;
pub use config::{FetchConfig, GitHubConfig, RateLimitConfig};
pub use discussion::DiscussionClient;
pub use error::{GitHubFetchError, Result};
pub use filters::{
    extract_error_codes, has_code_blocks, has_rust_error_codes, DateRange, IssueFilters, IssueState,
};
pub use types::{
    CollectionResult, Discussion, DiscussionComment, GitHubComment, GitHubIssue, GitHubLabel,
    GitHubUser, PrFile, PrReview, PrReviewComment, Repository,
};

pub struct GitHubFetcher {
    client: GitHubClient,
    discussion_client: Option<DiscussionClient>,
}

impl GitHubFetcher {
    pub fn new(token: Option<String>) -> Result<Self> {
        let config = if let Some(token) = token {
            std::env::set_var("GITHUB_TOKEN", token);
            FetchConfig::default()
        } else {
            FetchConfig::default()
        };

        Self::with_config(config)
    }

    pub fn with_config(config: FetchConfig) -> Result<Self> {
        let client = GitHubClient::with_config(config.clone())?;
        let discussion_client = DiscussionClient::new(config.github).ok();

        Ok(Self {
            client,
            discussion_client,
        })
    }

    pub async fn fetch_issues(
        &self,
        repo: &Repository,
        filters: &IssueFilters,
    ) -> Result<Vec<GitHubIssue>> {
        let result = self.client.fetch_issues(repo, filters, None).await?;
        Ok(result.issues)
    }

    pub async fn fetch_issues_with_limit(
        &self,
        repo: &Repository,
        filters: &IssueFilters,
        max_issues: usize,
    ) -> Result<CollectionResult> {
        self.client
            .fetch_issues(repo, filters, Some(max_issues))
            .await
    }

    pub async fn fetch_issue(&self, repo: &Repository, number: u64) -> Result<GitHubIssue> {
        self.client.fetch_issue(repo, number).await
    }

    pub async fn fetch_pr(&self, repo: &Repository, number: u64) -> Result<GitHubIssue> {
        self.client.fetch_pr(repo, number).await
    }

    pub async fn fetch_comments(
        &self,
        repo: &Repository,
        issue_number: u64,
    ) -> Result<Vec<GitHubComment>> {
        self.client.fetch_comments(repo, issue_number).await
    }

    pub async fn fetch_pr_files(&self, repo: &Repository, pr_number: u64) -> Result<Vec<PrFile>> {
        self.client.fetch_pr_files(repo, pr_number).await
    }

    /// Fetch all reviews for a PR (approved, changes requested, etc.)
    pub async fn fetch_pr_reviews(
        &self,
        repo: &Repository,
        pr_number: u64,
    ) -> Result<Vec<PrReview>> {
        self.client.fetch_pr_reviews(repo, pr_number).await
    }

    /// Fetch all review comments (inline comments on diff) for a PR
    pub async fn fetch_pr_review_comments(
        &self,
        repo: &Repository,
        pr_number: u64,
    ) -> Result<Vec<PrReviewComment>> {
        self.client.fetch_pr_review_comments(repo, pr_number).await
    }

    pub async fn fetch_discussion(
        &self,
        repo: &Repository,
        discussion_number: u64,
    ) -> Result<Discussion> {
        let discussion_client = self.discussion_client.as_ref().ok_or_else(|| {
            GitHubFetchError::ConfigError("Discussion client not initialized".to_string())
        })?;

        discussion_client
            .fetch_discussion(repo, discussion_number)
            .await
    }

    pub async fn fetch_discussion_by_url(&self, discussion_url: &str) -> Result<Discussion> {
        let discussion_client = self.discussion_client.as_ref().ok_or_else(|| {
            GitHubFetchError::ConfigError("Discussion client not initialized".to_string())
        })?;

        discussion_client
            .fetch_discussion_by_url(discussion_url)
            .await
    }

    pub async fn test_connection(&self) -> Result<()> {
        self.client.test_connection().await
    }

    pub async fn get_rate_limit(&self) -> Result<String> {
        self.client.get_rate_limit().await
    }
}

pub struct GitHubFetcherBuilder {
    config: FetchConfig,
}

impl GitHubFetcherBuilder {
    pub fn new() -> Self {
        Self {
            config: FetchConfig::default(),
        }
    }

    pub fn token(self, token: impl Into<String>) -> Self {
        std::env::set_var(&self.config.github.token_env_var, token.into());
        self
    }

    pub fn token_env_var(mut self, var_name: impl Into<String>) -> Self {
        self.config.github.token_env_var = var_name.into();
        self
    }

    pub fn api_base_url(mut self, url: impl Into<String>) -> Self {
        self.config.github.api_base_url = url.into();
        self
    }

    pub fn user_agent(mut self, agent: impl Into<String>) -> Self {
        self.config.github.user_agent = agent.into();
        self
    }

    pub fn timeout(mut self, seconds: u64) -> Self {
        self.config.github.timeout_seconds = seconds;
        self
    }

    pub fn rate_limit(mut self, requests_per_minute: u32) -> Self {
        self.config.rate_limiting.requests_per_minute = requests_per_minute;
        self.config.rate_limiting.delay_between_requests_ms =
            60_000 / requests_per_minute.max(1) as u64;
        self
    }

    pub fn max_retries(mut self, retries: u32) -> Self {
        self.config.rate_limiting.max_retries = retries;
        self
    }

    pub fn build(self) -> Result<GitHubFetcher> {
        GitHubFetcher::with_config(self.config)
    }
}

impl Default for GitHubFetcherBuilder {
    fn default() -> Self {
        Self::new()
    }
}
