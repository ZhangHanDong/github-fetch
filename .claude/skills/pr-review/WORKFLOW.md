# PR Review Workflow

This document describes the step-by-step workflow for reviewing and fixing PR issues.

## Workflow Overview

```
┌─────────────────┐
│  1. Parse PR    │
│     Reference   │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│  2. Fetch PR    │
│     Data        │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│  3. Analyze     │
│     Reviews     │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│  4. Fix Issues  │
│                 │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│  5. Re-Review   │
│                 │
└─────────────────┘
```

## Detailed Steps

### 1. Parse PR Reference

Supported formats:

| Format | Example |
|--------|---------|
| Full URL | `https://github.com/owner/repo/pull/123` |
| Short form | `owner/repo#123` |
| Number only | `#123` (requires repo context) |

Regex for URL parsing:
```regex
https://github\.com/([^/]+)/([^/]+)/pull/(\d+)
```

### 2. Fetch PR Data

Required API calls:

```rust
// 1. PR metadata
let pr = fetcher.fetch_pr(&repo, pr_number).await?;

// 2. File changes with diff patches
let files = fetcher.fetch_pr_files(&repo, pr_number).await?;

// 3. Review decisions (APPROVED, CHANGES_REQUESTED, etc.)
let reviews = fetcher.fetch_pr_reviews(&repo, pr_number).await?;

// 4. Inline comments on specific lines
let review_comments = fetcher.fetch_pr_review_comments(&repo, pr_number).await?;

// 5. General conversation comments
let comments = fetcher.fetch_comments(&repo, pr_number).await?;
```

### 3. Analyze Reviews

Priority order for addressing feedback:

1. **CHANGES_REQUESTED** - Must fix before merge
2. **Inline review comments** - Specific code issues
3. **COMMENTED** - Suggestions to consider
4. **General comments** - Discussion points

Group inline comments by file for efficient processing:

```rust
let mut by_file: HashMap<&str, Vec<&PrReviewComment>> = HashMap::new();
for comment in &review_comments {
    by_file.entry(&comment.path).or_default().push(comment);
}
```

### 4. Fix Issues

For each issue:

1. **Locate the code**
   - Use `comment.path` to find the file
   - Use `comment.line` for the specific line
   - Use `comment.diff_hunk` for context

2. **Understand the context**
   ```
   diff_hunk shows:
   - Lines starting with '-' are removed
   - Lines starting with '+' are added
   - Lines starting with ' ' are context
   ```

3. **Apply the fix**
   - Read the full file for complete context
   - Make minimal, focused changes
   - Preserve existing code style

4. **Document the change**
   - Note what was fixed
   - Reference the review comment

### 5. Re-Review

Checklist after fixing:

- [ ] All CHANGES_REQUESTED reviews addressed
- [ ] All inline comments resolved
- [ ] No new issues introduced
- [ ] Code compiles without errors
- [ ] Tests pass (if applicable)
- [ ] Code style consistent

## Common Patterns

### Handling Multi-line Comments

Some review comments span multiple lines:

```rust
// Check both line and original_line
let start_line = comment.original_line.unwrap_or(0);
let end_line = comment.line.unwrap_or(start_line);
```

### Reply Threads

Comments can be replies to other comments:

```rust
// Filter to get only top-level comments
let top_level: Vec<_> = review_comments
    .iter()
    .filter(|c| c.in_reply_to_id.is_none())
    .collect();
```

### Outdated Comments

Comments on changed code may become "outdated":

```rust
// Check if position is None (often indicates outdated)
if comment.position.is_none() && comment.line.is_some() {
    // Comment may be on updated code
}
```

## Error Handling

| Error | Cause | Solution |
|-------|-------|----------|
| `AuthError` | Invalid/missing token | Set `GITHUB_TOKEN` env var |
| `NotFound` | PR doesn't exist | Verify PR number and repo |
| `RateLimitExceeded` | Too many requests | Wait and retry |

## Example Session

```
User: Review PR https://github.com/tokio-rs/axum/pull/2865

Claude:
1. Fetching PR #2865 from tokio-rs/axum...
2. Found 3 reviews, 2 with CHANGES_REQUESTED
3. Found 5 inline comments across 2 files
4. Analyzing issues...

Issues found:
1. src/router.rs:142 - "Add error handling for None case"
2. src/router.rs:156 - "Consider using ? operator"
3. src/lib.rs:45 - "Missing documentation"

Shall I fix these issues?

User: Yes, fix them

Claude:
[Reads src/router.rs]
[Applies fix for issue #1]
[Applies fix for issue #2]
[Reads src/lib.rs]
[Applies fix for issue #3]

All 3 issues have been fixed. Here's a summary:
- Added match expression for Option handling
- Replaced unwrap() with ? operator
- Added doc comment for public function

Re-reviewing... No new issues found.
```
