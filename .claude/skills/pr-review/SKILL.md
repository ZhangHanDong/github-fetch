---
name: pr-review-and-fix
description: Fetch PR code diff and existing review comments from GitHub, then fix code issues and re-review. Use when the user wants to review a PR, fix code based on review feedback, address PR comments, or analyze PR changes.
---

# PR Review and Fix

A skill for fetching GitHub PR information including code diffs and review comments, then helping fix issues and re-review the code.

## Prerequisites

- `GITHUB_TOKEN` environment variable must be set
- The `github-fetch` library must be available in the project

## Instructions

When the user provides a PR URL or repository/PR number:

### Step 1: Parse the PR Reference

Extract owner, repo, and PR number from:
- Full URL: `https://github.com/owner/repo/pull/123`
- Short form: `owner/repo#123`
- Just number if repo context is clear: `#123`

### Step 2: Fetch PR Information

Use the `github-fetch` library to retrieve:

```rust
use github_fetch::{GitHubFetcher, Repository};

let fetcher = GitHubFetcher::new(None)?;
let repo = Repository::new("owner", "repo");

// Get PR details
let pr = fetcher.fetch_pr(&repo, pr_number).await?;

// Get file changes with diff
let files = fetcher.fetch_pr_files(&repo, pr_number).await?;

// Get reviews (approved, changes requested, etc.)
let reviews = fetcher.fetch_pr_reviews(&repo, pr_number).await?;

// Get inline review comments on the diff
let review_comments = fetcher.fetch_pr_review_comments(&repo, pr_number).await?;

// Get general PR comments
let comments = fetcher.fetch_comments(&repo, pr_number).await?;
```

### Step 3: Analyze and Present

1. **PR Overview**: Show title, author, state, and labels
2. **Review Status**: List all reviews with their state (APPROVED, CHANGES_REQUESTED, etc.)
3. **Files Changed**: Show file list with additions/deletions count
4. **Review Comments**: Group inline comments by file and line number
5. **Issues Found**: Summarize all requested changes and feedback

### Step 4: Fix Code Issues

For each issue identified in review comments:

1. Read the relevant file using the `Read` tool
2. Understand the context from `diff_hunk` in review comments
3. Apply the fix using the `Edit` tool
4. Explain what was changed and why

### Step 5: Re-Review

After fixes are applied:

1. Review the modified code for correctness
2. Check if fixes address all review comments
3. Look for any new issues introduced
4. Verify code style and best practices
5. Provide a summary of changes made

## Data Structures Reference

### PrReview
```rust
pub struct PrReview {
    pub id: u64,
    pub user: GitHubUser,
    pub body: Option<String>,
    pub state: String,  // APPROVED, CHANGES_REQUESTED, COMMENTED, DISMISSED, PENDING
    pub submitted_at: Option<DateTime<Utc>>,
    pub commit_id: Option<String>,
}
```

### PrReviewComment (inline diff comment)
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
    pub status: String,       // Added, Modified, Removed
    pub additions: u32,
    pub deletions: u32,
    pub patch: Option<String>, // The actual diff content
}
```

## Output Format

When presenting PR review information:

```
## PR #123: <title>

**Author:** @username | **State:** Open/Closed/Merged
**Labels:** bug, enhancement

---

### Reviews

| Reviewer | State | Comment |
|----------|-------|---------|
| @user1 | APPROVED | LGTM |
| @user2 | CHANGES_REQUESTED | Please fix the error handling |

---

### Files Changed (3 files, +45 -12)

- `src/main.rs` (+20 -5) Modified
- `src/lib.rs` (+15 -7) Modified
- `tests/test.rs` (+10 -0) Added

---

### Review Comments

#### src/main.rs:42
> @reviewer: This function should handle the None case

```diff
@@ -40,6 +40,8 @@
 fn process(data: Option<String>) {
-    println!("{}", data.unwrap());
+    // TODO: Handle None case
 }
```

---

### Action Items

1. [ ] Handle None case in `process()` function (src/main.rs:42)
2. [ ] Add error handling for network failures
```

## Example Usage

User: "Review PR https://github.com/tokio-rs/axum/pull/2865 and fix the issues"

Response:
1. Fetch all PR data using github-fetch
2. Present the review summary
3. For each issue, show the code context and apply fixes
4. Re-review the changes

## Tips

- Always show the diff context when discussing review comments
- Group related comments together
- Prioritize CHANGES_REQUESTED reviews over COMMENTED
- When fixing code, explain the rationale
- After fixes, verify no regressions are introduced
