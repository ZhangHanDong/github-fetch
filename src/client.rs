use chrono::Utc;
use log::{debug, info, warn};
use octocrab::models::issues::Issue;
use octocrab::{Octocrab, Page};
use tokio::time::{sleep, Duration};

use crate::config::{FetchConfig, GitHubConfig};
use crate::error::{GitHubFetchError, Result};
use crate::filters::{IssueFilters, IssueState};
use crate::types::{
    CollectionResult, GitHubComment, GitHubIssue, GitHubLabel, GitHubUser, PrFile, PrReview,
    PrReviewComment, Repository,
};

pub struct GitHubClient {
    octocrab: Octocrab,
    rate_limit_delay: Duration,
    #[allow(dead_code)]
    config: GitHubConfig,
}

impl GitHubClient {
    pub fn new() -> Result<Self> {
        Self::with_config(FetchConfig::default())
    }

    pub fn with_config(config: FetchConfig) -> Result<Self> {
        let mut builder = Octocrab::builder();

        let token = std::env::var(&config.github.token_env_var).map_err(|_| {
            GitHubFetchError::AuthError(format!(
                "{} environment variable not set",
                config.github.token_env_var
            ))
        })?;

        builder = builder.personal_token(token);

        if !config.github.api_base_url.is_empty()
            && config.github.api_base_url != "https://api.github.com"
        {
            builder = builder
                .base_uri(&config.github.api_base_url)
                .map_err(|e| GitHubFetchError::ConfigError(format!("Invalid base URI: {}", e)))?;
        }

        let octocrab = builder.build()?;
        let rate_limit_delay = config.rate_limiting.delay_duration();

        Ok(Self {
            octocrab,
            rate_limit_delay,
            config: config.github,
        })
    }

    fn convert_state(state: &IssueState) -> Option<octocrab::params::State> {
        match state {
            IssueState::Open => Some(octocrab::params::State::Open),
            IssueState::Closed => Some(octocrab::params::State::Closed),
            IssueState::All => None,
        }
    }

    pub async fn fetch_issues(
        &self,
        repo: &Repository,
        filters: &IssueFilters,
        max_issues: Option<usize>,
    ) -> Result<CollectionResult> {
        info!("Collecting issues from {}", repo.full_name);

        let mut all_issues = Vec::new();
        let mut page = 1u32;
        let per_page = 100u8;
        let mut collected_count = 0;

        loop {
            debug!("Fetching page {} for {}", page, repo.full_name);

            let issues_handler = self.octocrab.issues(&repo.owner, &repo.name);
            let mut list_builder = issues_handler
                .list()
                .sort(octocrab::params::issues::Sort::Updated)
                .direction(octocrab::params::Direction::Descending)
                .per_page(per_page)
                .page(page);

            if let Some(state) = Self::convert_state(&filters.state) {
                list_builder = list_builder.state(state);
            }

            if !filters.include_labels.is_empty() {
                list_builder = list_builder.labels(&filters.include_labels);
            }

            if let Some(date_range) = &filters.date_range {
                if let Some(since) = date_range.start {
                    list_builder = list_builder.since(since);
                }
            }

            let issues_page: Page<Issue> = list_builder.send().await.map_err(|e| {
                GitHubFetchError::ApiError(format!("Failed to fetch issues: {}", e))
            })?;

            sleep(self.rate_limit_delay).await;

            if issues_page.items.is_empty() {
                break;
            }

            for issue in issues_page.items {
                let github_issue = self.convert_issue(issue).await?;

                if filters.matches(&github_issue) {
                    all_issues.push(github_issue);
                    collected_count += 1;

                    if let Some(max) = max_issues {
                        if collected_count >= max {
                            info!("Reached maximum issue limit: {}", max);
                            break;
                        }
                    }
                }
            }

            if let Some(max) = max_issues {
                if collected_count >= max {
                    break;
                }
            }

            page += 1;

            if page > 100 {
                warn!("Reached maximum page limit (100) for {}", repo.full_name);
                break;
            }
        }

        info!(
            "Collected {} issues from {}",
            all_issues.len(),
            repo.full_name
        );

        Ok(CollectionResult {
            repository: repo.clone(),
            issues: all_issues,
            total_collected: collected_count,
            collection_time: Utc::now(),
            filters_applied: self.describe_filters(filters),
        })
    }

    pub async fn fetch_issue(&self, repo: &Repository, issue_number: u64) -> Result<GitHubIssue> {
        sleep(self.rate_limit_delay).await;

        let issue = self
            .octocrab
            .issues(&repo.owner, &repo.name)
            .get(issue_number)
            .await
            .map_err(|e| {
                GitHubFetchError::NotFound(format!("Issue #{} not found: {}", issue_number, e))
            })?;

        self.convert_issue(issue).await
    }

    pub async fn fetch_pr(&self, repo: &Repository, pr_number: u64) -> Result<GitHubIssue> {
        sleep(self.rate_limit_delay).await;

        let pr = self
            .octocrab
            .pulls(&repo.owner, &repo.name)
            .get(pr_number)
            .await
            .map_err(|e| {
                GitHubFetchError::NotFound(format!("PR #{} not found: {}", pr_number, e))
            })?;

        let merged_at = pr.merged_at;
        let closed_at = pr.closed_at.or(merged_at);

        Ok(GitHubIssue {
            id: pr.id.0,
            number: pr.number,
            title: pr.title.unwrap_or_default(),
            body: pr.body,
            state: pr
                .state
                .map(|s| format!("{:?}", s))
                .unwrap_or_else(|| "open".to_string()),
            labels: pr
                .labels
                .unwrap_or_default()
                .into_iter()
                .map(|label| GitHubLabel {
                    id: label.id.0,
                    name: label.name,
                    color: label.color,
                    description: label.description,
                })
                .collect(),
            user: if let Some(user) = pr.user {
                GitHubUser {
                    id: user.id.0,
                    login: user.login,
                    avatar_url: user.avatar_url.to_string(),
                }
            } else {
                GitHubUser {
                    id: 0,
                    login: "unknown".to_string(),
                    avatar_url: "".to_string(),
                }
            },
            assignees: pr
                .assignees
                .unwrap_or_default()
                .into_iter()
                .map(|assignee| GitHubUser {
                    id: assignee.id.0,
                    login: assignee.login,
                    avatar_url: assignee.avatar_url.to_string(),
                })
                .collect(),
            created_at: pr.created_at.unwrap_or_else(|| Utc::now()),
            updated_at: pr.updated_at.unwrap_or_else(|| Utc::now()),
            closed_at,
            merged_at,
            html_url: pr.html_url.map(|url| url.to_string()).unwrap_or_default(),
            is_pull_request: true,
            comments: pr.comments.unwrap_or(0) as u32,
        })
    }

    pub async fn fetch_comments(
        &self,
        repo: &Repository,
        issue_number: u64,
    ) -> Result<Vec<GitHubComment>> {
        debug!(
            "Fetching comments for issue #{} in {}",
            issue_number, repo.full_name
        );

        let mut comments = Vec::new();
        let mut page = 1u32;

        loop {
            let comments_page = self
                .octocrab
                .issues(&repo.owner, &repo.name)
                .list_comments(issue_number)
                .per_page(100)
                .page(page)
                .send()
                .await
                .map_err(|e| {
                    GitHubFetchError::ApiError(format!("Failed to fetch comments: {}", e))
                })?;

            sleep(self.rate_limit_delay).await;

            if comments_page.items.is_empty() {
                break;
            }

            for comment in comments_page.items {
                comments.push(GitHubComment {
                    id: comment.id.0,
                    user: GitHubUser {
                        id: comment.user.id.0,
                        login: comment.user.login,
                        avatar_url: comment.user.avatar_url.to_string(),
                    },
                    body: comment.body.unwrap_or_default(),
                    created_at: comment.created_at,
                    updated_at: comment.updated_at.unwrap_or(comment.created_at),
                    html_url: comment.html_url.to_string(),
                });
            }

            page += 1;
        }

        Ok(comments)
    }

    pub async fn fetch_pr_files(&self, repo: &Repository, pr_number: u64) -> Result<Vec<PrFile>> {
        sleep(self.rate_limit_delay).await;

        let files = self
            .octocrab
            .pulls(&repo.owner, &repo.name)
            .list_files(pr_number)
            .await
            .map_err(|e| GitHubFetchError::ApiError(format!("Failed to fetch PR files: {}", e)))?;

        Ok(files
            .items
            .into_iter()
            .map(|file| PrFile {
                filename: file.filename,
                status: format!("{:?}", file.status),
                additions: file.additions as u32,
                deletions: file.deletions as u32,
                changes: file.changes as u32,
                patch: file.patch,
            })
            .collect())
    }

    /// Fetch all reviews for a PR
    pub async fn fetch_pr_reviews(
        &self,
        repo: &Repository,
        pr_number: u64,
    ) -> Result<Vec<PrReview>> {
        debug!(
            "Fetching reviews for PR #{} in {}",
            pr_number, repo.full_name
        );

        sleep(self.rate_limit_delay).await;

        let reviews = self
            .octocrab
            .pulls(&repo.owner, &repo.name)
            .list_reviews(pr_number)
            .send()
            .await
            .map_err(|e| GitHubFetchError::ApiError(format!("Failed to fetch PR reviews: {}", e)))?;

        Ok(reviews
            .items
            .into_iter()
            .map(|review| PrReview {
                id: review.id.0,
                user: GitHubUser {
                    id: review.user.as_ref().map(|u| u.id.0).unwrap_or(0),
                    login: review
                        .user
                        .as_ref()
                        .map(|u| u.login.clone())
                        .unwrap_or_else(|| "unknown".to_string()),
                    avatar_url: review
                        .user
                        .as_ref()
                        .map(|u| u.avatar_url.to_string())
                        .unwrap_or_default(),
                },
                body: review.body,
                state: review
                    .state
                    .map(|s| format!("{:?}", s))
                    .unwrap_or_else(|| "UNKNOWN".to_string()),
                submitted_at: review.submitted_at,
                html_url: review.html_url.to_string(),
                commit_id: review.commit_id,
            })
            .collect())
    }

    /// Fetch all review comments (inline comments on diff) for a PR
    pub async fn fetch_pr_review_comments(
        &self,
        repo: &Repository,
        pr_number: u64,
    ) -> Result<Vec<PrReviewComment>> {
        debug!(
            "Fetching review comments for PR #{} in {}",
            pr_number, repo.full_name
        );

        let mut comments = Vec::new();
        let mut page = 1u32;

        loop {
            sleep(self.rate_limit_delay).await;

            let url = format!(
                "/repos/{}/{}/pulls/{}/comments?per_page=100&page={}",
                repo.owner, repo.name, pr_number, page
            );

            let response: Vec<serde_json::Value> = self
                .octocrab
                .get(&url, None::<&()>)
                .await
                .map_err(|e| {
                    GitHubFetchError::ApiError(format!("Failed to fetch review comments: {}", e))
                })?;

            if response.is_empty() {
                break;
            }

            for comment in response {
                if let Some(parsed) = self.parse_review_comment(&comment) {
                    comments.push(parsed);
                }
            }

            page += 1;
        }

        Ok(comments)
    }

    fn parse_review_comment(&self, comment: &serde_json::Value) -> Option<PrReviewComment> {
        let user = comment.get("user")?;

        Some(PrReviewComment {
            id: comment.get("id")?.as_u64()?,
            review_id: comment
                .get("pull_request_review_id")
                .and_then(|v| v.as_u64()),
            user: GitHubUser {
                id: user.get("id")?.as_u64()?,
                login: user.get("login")?.as_str()?.to_string(),
                avatar_url: user.get("avatar_url")?.as_str()?.to_string(),
            },
            body: comment.get("body")?.as_str()?.to_string(),
            path: comment.get("path")?.as_str()?.to_string(),
            line: comment.get("line").and_then(|v| v.as_u64()).map(|v| v as u32),
            original_line: comment
                .get("original_line")
                .and_then(|v| v.as_u64())
                .map(|v| v as u32),
            diff_hunk: comment
                .get("diff_hunk")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            side: comment
                .get("side")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            commit_id: comment
                .get("commit_id")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            created_at: comment
                .get("created_at")?
                .as_str()?
                .parse()
                .ok()?,
            updated_at: comment
                .get("updated_at")?
                .as_str()?
                .parse()
                .ok()?,
            html_url: comment.get("html_url")?.as_str()?.to_string(),
            position: comment
                .get("position")
                .and_then(|v| v.as_u64())
                .map(|v| v as u32),
            in_reply_to_id: comment.get("in_reply_to_id").and_then(|v| v.as_u64()),
        })
    }

    pub async fn test_connection(&self) -> Result<()> {
        debug!("Testing GitHub API connection");

        self.octocrab
            .ratelimit()
            .get()
            .await
            .map_err(|e| GitHubFetchError::ApiError(format!("Connection test failed: {}", e)))?;

        info!("GitHub API connection successful");
        Ok(())
    }

    pub async fn get_rate_limit(&self) -> Result<String> {
        let rate_limit =
            self.octocrab.ratelimit().get().await.map_err(|e| {
                GitHubFetchError::ApiError(format!("Failed to get rate limit: {}", e))
            })?;

        Ok(format!(
            "Rate limit: {}/{} remaining, resets at {}",
            rate_limit.resources.core.remaining,
            rate_limit.resources.core.limit,
            rate_limit.resources.core.reset
        ))
    }

    async fn convert_issue(&self, issue: Issue) -> Result<GitHubIssue> {
        let is_pull_request = issue.pull_request.is_some();

        let merged_at = if is_pull_request {
            self.get_pr_merged_at(&issue).await?
        } else {
            None
        };

        Ok(GitHubIssue {
            id: issue.id.0,
            number: issue.number,
            title: issue.title,
            body: issue.body,
            state: format!("{:?}", issue.state),
            labels: issue
                .labels
                .into_iter()
                .map(|label| GitHubLabel {
                    id: label.id.0,
                    name: label.name,
                    color: label.color,
                    description: label.description,
                })
                .collect(),
            user: GitHubUser {
                id: issue.user.id.0,
                login: issue.user.login,
                avatar_url: issue.user.avatar_url.to_string(),
            },
            assignees: issue
                .assignees
                .into_iter()
                .map(|assignee| GitHubUser {
                    id: assignee.id.0,
                    login: assignee.login,
                    avatar_url: assignee.avatar_url.to_string(),
                })
                .collect(),
            created_at: issue.created_at,
            updated_at: issue.updated_at,
            closed_at: issue.closed_at,
            merged_at,
            html_url: issue.html_url.to_string(),
            is_pull_request,
            comments: issue.comments,
        })
    }

    async fn get_pr_merged_at(
        &self,
        issue: &Issue,
    ) -> Result<Option<chrono::DateTime<chrono::Utc>>> {
        if let Some(ref _pr_url) = issue.pull_request {
            let repo_url_str = issue.repository_url.to_string();
            let parts: Vec<&str> = repo_url_str.trim_end_matches('/').split('/').collect();

            if parts.len() >= 2 {
                let owner = parts[parts.len() - 2];
                let repo = parts[parts.len() - 1];

                match self.octocrab.pulls(owner, repo).get(issue.number).await {
                    Ok(pr) => {
                        sleep(self.rate_limit_delay).await;
                        Ok(pr.merged_at)
                    }
                    Err(e) => {
                        warn!("Failed to fetch PR #{} merged_at: {}", issue.number, e);
                        Ok(None)
                    }
                }
            } else {
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }

    fn describe_filters(&self, filters: &IssueFilters) -> Vec<String> {
        let mut descriptions = Vec::new();

        if !filters.include_labels.is_empty() {
            descriptions.push(format!("include_labels: {:?}", filters.include_labels));
        }
        if !filters.exclude_labels.is_empty() {
            descriptions.push(format!("exclude_labels: {:?}", filters.exclude_labels));
        }
        if filters.rust_errors_only {
            descriptions.push("rust_errors_only: true".to_string());
        }
        if filters.code_blocks_only {
            descriptions.push("code_blocks_only: true".to_string());
        }
        if let Some(min_length) = filters.min_body_length {
            descriptions.push(format!("min_body_length: {}", min_length));
        }
        if !filters.include_pull_requests {
            descriptions.push("exclude_pull_requests: true".to_string());
        }
        if let Some(min_comments) = filters.min_comments {
            descriptions.push(format!("min_comments: {}", min_comments));
        }

        descriptions
    }
}
