use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubIssue {
    pub id: u64,
    pub number: u64,
    pub title: String,
    pub body: Option<String>,
    pub state: String,
    pub labels: Vec<GitHubLabel>,
    pub user: GitHubUser,
    pub assignees: Vec<GitHubUser>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub closed_at: Option<DateTime<Utc>>,
    pub merged_at: Option<DateTime<Utc>>,
    pub html_url: String,
    pub is_pull_request: bool,
    pub comments: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubLabel {
    pub id: u64,
    pub name: String,
    pub color: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubUser {
    pub id: u64,
    pub login: String,
    pub avatar_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubComment {
    pub id: u64,
    pub user: GitHubUser,
    pub body: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub html_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Repository {
    pub owner: String,
    pub name: String,
    pub full_name: String,
}

impl Repository {
    pub fn new(owner: impl Into<String>, name: impl Into<String>) -> Self {
        let owner = owner.into();
        let name = name.into();
        let full_name = format!("{}/{}", owner, name);
        Self {
            owner,
            name,
            full_name,
        }
    }

    pub fn from_url(url: &str) -> anyhow::Result<Self> {
        let parts: Vec<&str> = url.trim_end_matches('/').split('/').collect();

        if parts.len() < 2 {
            return Err(anyhow::anyhow!("Invalid repository URL: {}", url));
        }

        let owner = parts[parts.len() - 2].to_string();
        let name = parts[parts.len() - 1].to_string();

        Ok(Self::new(owner, name))
    }

    pub fn from_full_name(full_name: &str) -> anyhow::Result<Self> {
        let parts: Vec<&str> = full_name.split('/').collect();
        if parts.len() != 2 {
            return Err(anyhow::anyhow!(
                "Invalid repository full name format. Expected 'owner/name', got: {}",
                full_name
            ));
        }
        Ok(Self::new(parts[0], parts[1]))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Discussion {
    pub number: u64,
    pub title: String,
    pub body: String,
    pub url: String,
    pub author: GitHubUser,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub comments: Vec<DiscussionComment>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscussionComment {
    pub id: String,
    pub body: String,
    pub author: GitHubUser,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrFile {
    pub filename: String,
    pub status: String,
    pub additions: u32,
    pub deletions: u32,
    pub changes: u32,
    pub patch: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectionResult {
    pub repository: Repository,
    pub issues: Vec<GitHubIssue>,
    pub total_collected: usize,
    pub collection_time: DateTime<Utc>,
    pub filters_applied: Vec<String>,
}

/// PR Review information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrReview {
    pub id: u64,
    pub user: GitHubUser,
    pub body: Option<String>,
    /// Review state: APPROVED, CHANGES_REQUESTED, COMMENTED, DISMISSED, PENDING
    pub state: String,
    pub submitted_at: Option<DateTime<Utc>>,
    pub html_url: String,
    pub commit_id: Option<String>,
}

/// PR Review comment (inline comment on diff)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrReviewComment {
    pub id: u64,
    pub review_id: Option<u64>,
    pub user: GitHubUser,
    pub body: String,
    /// File path the comment is on
    pub path: String,
    /// Line number in the diff
    pub line: Option<u32>,
    /// Original line number (for multi-line comments)
    pub original_line: Option<u32>,
    /// Diff hunk context
    pub diff_hunk: String,
    /// Side of the diff: LEFT or RIGHT
    pub side: Option<String>,
    pub commit_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub html_url: String,
    /// Position in the diff (deprecated, use line instead)
    pub position: Option<u32>,
    /// In reply to another comment
    pub in_reply_to_id: Option<u64>,
}
