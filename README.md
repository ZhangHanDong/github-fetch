# github-fetch

[![Crates.io](https://img.shields.io/crates/v/github-fetch.svg)](https://crates.io/crates/github-fetch)
[![Documentation](https://docs.rs/github-fetch/badge.svg)](https://docs.rs/github-fetch)
[![License](https://img.shields.io/crates/l/github-fetch.svg)](LICENSE-MIT)

[中文](README_CN.md) | [日本語](README_JP.md)

A Rust library for fetching GitHub issues, pull requests, discussions, reviews, and diff information via the GitHub API.

## Features

- Fetch issues and pull requests with flexible filtering
- Fetch PR reviews (approved, changes requested, etc.)
- Fetch PR review comments (inline comments on diff)
- Fetch PR file changes with diff/patch content
- Fetch GitHub Discussions via GraphQL API
- Rate limiting and retry support
- Builder pattern for configuration

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
github-fetch = "0.1"
tokio = { version = "1", features = ["full"] }
anyhow = "1.0"
```

## Authentication

Set your GitHub token as an environment variable:

```bash
export GITHUB_TOKEN=ghp_your_token_here
```

## Quick Start

```rust
use github_fetch::{GitHubFetcher, Repository};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let fetcher = GitHubFetcher::new(None)?;
    let repo = Repository::new("tokio-rs", "tokio");

    // Fetch a PR with reviews and diff
    let pr = fetcher.fetch_pr(&repo, 1234).await?;
    let reviews = fetcher.fetch_pr_reviews(&repo, 1234).await?;
    let files = fetcher.fetch_pr_files(&repo, 1234).await?;

    println!("PR: {}", pr.title);
    println!("Reviews: {}", reviews.len());
    println!("Files changed: {}", files.len());

    Ok(())
}
```

## Usage

### Builder Pattern

```rust
use github_fetch::GitHubFetcherBuilder;

let fetcher = GitHubFetcherBuilder::new()
    .token("ghp_your_token")
    .user_agent("my-app/1.0.0")
    .rate_limit(30)
    .max_retries(5)
    .build()?;
```

### Fetching Issues with Filters

```rust
use github_fetch::{GitHubFetcher, Repository, IssueFilters, IssueState};

let filters = IssueFilters {
    state: IssueState::Closed,
    include_labels: vec!["bug".to_string()],
    min_comments: Some(3),
    code_blocks_only: true,
    ..Default::default()
};

let result = fetcher.fetch_issues_with_limit(&repo, &filters, 10).await?;
```

### Fetching PR Reviews

```rust
let reviews = fetcher.fetch_pr_reviews(&repo, 2865).await?;

for review in &reviews {
    println!("{}: {}", review.user.login, review.state);
    // state: APPROVED, CHANGES_REQUESTED, COMMENTED, DISMISSED, PENDING
}
```

### Fetching PR Review Comments (Inline Diff Comments)

```rust
let review_comments = fetcher.fetch_pr_review_comments(&repo, 2865).await?;

for comment in &review_comments {
    println!("File: {} Line: {:?}", comment.path, comment.line);
    println!("Comment: {}", comment.body);
    println!("Diff:\n{}", comment.diff_hunk);
}
```

### Fetching PR File Changes (Diff)

```rust
let files = fetcher.fetch_pr_files(&repo, 2865).await?;

for file in &files {
    println!("{}: +{} -{}", file.filename, file.additions, file.deletions);
    if let Some(patch) = &file.patch {
        println!("{}", patch);
    }
}
```

### Fetching Discussions

```rust
let discussion = fetcher.fetch_discussion(&repo, 3766).await?;
println!("Title: {}", discussion.title);

// Or fetch by URL
let discussion = fetcher.fetch_discussion_by_url(
    "https://github.com/actix/actix-web/discussions/3766"
).await?;
```

## Data Structures

### PrReview

```rust
pub struct PrReview {
    pub id: u64,
    pub user: GitHubUser,
    pub body: Option<String>,
    pub state: String,  // APPROVED, CHANGES_REQUESTED, COMMENTED, DISMISSED, PENDING
    pub submitted_at: Option<DateTime<Utc>>,
    pub html_url: String,
    pub commit_id: Option<String>,
}
```

### PrReviewComment

```rust
pub struct PrReviewComment {
    pub id: u64,
    pub user: GitHubUser,
    pub body: String,
    pub path: String,           // File path
    pub line: Option<u32>,      // Line number
    pub diff_hunk: String,      // Diff context
    pub side: Option<String>,   // LEFT or RIGHT
    pub in_reply_to_id: Option<u64>,
}
```

### PrFile

```rust
pub struct PrFile {
    pub filename: String,
    pub status: String,        // Added, Modified, Removed, Renamed
    pub additions: u32,
    pub deletions: u32,
    pub patch: Option<String>, // Diff content
}
```

## Examples

```bash
export GITHUB_TOKEN=ghp_your_token

cargo run --example fetch_tokio_issue
cargo run --example fetch_axum_pr
cargo run --example fetch_pr_review
cargo run --example fetch_with_filters
cargo run --example fetch_actix_discussion
cargo run --example advanced_usage
```

## Error Handling

```rust
use github_fetch::{GitHubFetcher, GitHubFetchError, Repository};

match fetcher.fetch_issue(&repo, 999999).await {
    Ok(issue) => println!("Found: {}", issue.title),
    Err(GitHubFetchError::NotFound(msg)) => println!("Not found: {}", msg),
    Err(GitHubFetchError::AuthError(msg)) => println!("Auth error: {}", msg),
    Err(GitHubFetchError::RateLimitExceeded) => println!("Rate limited!"),
    Err(e) => println!("Error: {}", e),
}
```

## Claude Code Skill

This project includes a Claude Code Skill for PR review and fixing. The skill automatically fetches PR information, analyzes review comments, fixes code issues, and re-reviews changes.

### What the Skill Does

1. **Fetch PR Data** - Gets PR details, file diffs, reviews, and inline comments
2. **Analyze Reviews** - Summarizes APPROVED/CHANGES_REQUESTED status and action items
3. **Fix Issues** - Applies fixes based on review feedback
4. **Re-review** - Verifies fixes and checks for new issues

### Installation

#### Option 1: Project-Level Installation (Recommended)

Clone or copy the skill into your project's `.claude/skills/` directory:

```bash
# Clone repo and copy skill to your project
git clone https://github.com/ZhangHanDong/github-fetch.git
mkdir -p your-project/.claude/skills
cp -r github-fetch/.claude/skills/pr-review your-project/.claude/skills/

# Or if already using this library, skills are included at .claude/skills/pr-review/
```

The skill will be automatically discovered by Claude Code when working in the project.

#### Option 2: Via Claude Code Plugin Marketplace

1. Add the marketplace:
   ```
   /plugin marketplace add ZhangHanDong/github-fetch
   ```
2. Install the plugin:
   ```
   /plugin install pr-review@github-fetch
   ```
3. Verify installation:
   ```
   /plugin marketplace list
   ```

### Usage

After installation, just ask Claude naturally:

```
Review PR https://github.com/tokio-rs/axum/pull/2865
```

```
Analyze the review comments on owner/repo#123 and fix the issues
```

```
What changes are requested in this PR? https://github.com/owner/repo/pull/456
```

### Example Session

```
You: Review PR https://github.com/owner/repo/pull/123

Claude:
## PR #123: Fix authentication bug

**Author:** @developer | **State:** Open

### Reviews
| Reviewer | State | Comment |
|----------|-------|---------|
| @reviewer1 | CHANGES_REQUESTED | Please add error handling |
| @reviewer2 | COMMENTED | Consider using Result type |

### Files Changed (2 files, +45 -12)
- `src/auth.rs` (+30 -10) Modified
- `src/lib.rs` (+15 -2) Modified

### Review Comments

#### src/auth.rs:42
> @reviewer1: This should handle the None case

### Action Items
1. [ ] Handle None case in authenticate() (src/auth.rs:42)
2. [ ] Add Result return type (src/auth.rs:45)

Would you like me to fix these issues?

You: Yes, fix them

Claude: [Reads files, applies fixes, re-reviews...]

All issues have been fixed:
- Added match expression for Option handling
- Changed return type to Result<(), AuthError>

Re-reviewing... No new issues found.
```

### Requirements

- `GITHUB_TOKEN` environment variable must be set
- For fixing code: the repository must be cloned locally

### Skill Files

```
.claude/skills/pr-review/
├── SKILL.md      # Main instructions for Claude
├── WORKFLOW.md   # Detailed workflow reference
└── scripts/
    └── fetch_pr.rs  # Example fetch script
```

## License

MIT OR Apache-2.0
