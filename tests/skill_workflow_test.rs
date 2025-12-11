//! Skill Workflow Tests
//!
//! These tests verify that the PR review skill workflow works correctly.
//! They test the same API calls that the skill would make.

use github_fetch::{GitHubFetcher, Repository};

/// Test that we can create a fetcher (requires GITHUB_TOKEN)
#[tokio::test]
#[ignore] // Run with: cargo test --ignored
async fn test_fetcher_creation() {
    let fetcher = GitHubFetcher::new(None);
    assert!(fetcher.is_ok(), "Should create fetcher with GITHUB_TOKEN env var");
}

/// Test the complete PR review workflow
/// This simulates what the skill does when reviewing a PR
#[tokio::test]
#[ignore] // Run with: cargo test --ignored -- --nocapture
async fn test_pr_review_workflow() {
    // Skip if no token
    if std::env::var("GITHUB_TOKEN").is_err() {
        eprintln!("Skipping: GITHUB_TOKEN not set");
        return;
    }

    let fetcher = GitHubFetcher::new(None).expect("Failed to create fetcher");

    // Use a well-known public repo and PR for testing
    // tokio-rs/axum PR #2865 is used in the skill examples
    let repo = Repository::new("tokio-rs", "axum");
    let pr_number = 2865;

    // Step 1: Fetch PR details
    println!("Step 1: Fetching PR #{}", pr_number);
    let pr = fetcher.fetch_pr(&repo, pr_number).await;
    assert!(pr.is_ok(), "Should fetch PR details");
    let pr = pr.unwrap();
    println!("  Title: {}", pr.title);
    println!("  State: {}", pr.state);

    // Step 2: Fetch file changes with diff
    println!("\nStep 2: Fetching file changes");
    let files = fetcher.fetch_pr_files(&repo, pr_number).await;
    assert!(files.is_ok(), "Should fetch PR files");
    let files = files.unwrap();
    println!("  Files changed: {}", files.len());
    for file in files.iter().take(3) {
        println!("    {} (+{} -{})", file.filename, file.additions, file.deletions);
    }

    // Step 3: Fetch reviews
    println!("\nStep 3: Fetching reviews");
    let reviews = fetcher.fetch_pr_reviews(&repo, pr_number).await;
    assert!(reviews.is_ok(), "Should fetch PR reviews");
    let reviews = reviews.unwrap();
    println!("  Reviews: {}", reviews.len());
    for review in &reviews {
        println!("    {}: {}", review.user.login, review.state);
    }

    // Step 4: Fetch review comments (inline diff comments)
    println!("\nStep 4: Fetching review comments");
    let review_comments = fetcher.fetch_pr_review_comments(&repo, pr_number).await;
    assert!(review_comments.is_ok(), "Should fetch PR review comments");
    let review_comments = review_comments.unwrap();
    println!("  Review comments: {}", review_comments.len());
    for comment in review_comments.iter().take(3) {
        println!("    {}: {} (line {:?})", comment.path, &comment.body[..50.min(comment.body.len())], comment.line);
    }

    println!("\nPR review workflow completed successfully!");
}

/// Test error handling for non-existent PR
#[tokio::test]
#[ignore]
async fn test_pr_not_found() {
    if std::env::var("GITHUB_TOKEN").is_err() {
        return;
    }

    let fetcher = GitHubFetcher::new(None).unwrap();
    let repo = Repository::new("tokio-rs", "axum");

    // Try to fetch a PR that doesn't exist
    let result = fetcher.fetch_pr(&repo, 9999999).await;
    assert!(result.is_err(), "Should return error for non-existent PR");
}

/// Test parsing PR references (unit test, no API call)
#[test]
fn test_parse_pr_url() {
    // Full URL format
    let url = "https://github.com/tokio-rs/axum/pull/2865";
    let parts: Vec<&str> = url.trim_start_matches("https://github.com/")
        .split('/')
        .collect();

    assert_eq!(parts.len(), 4);
    assert_eq!(parts[0], "tokio-rs"); // owner
    assert_eq!(parts[1], "axum");     // repo
    assert_eq!(parts[2], "pull");
    assert_eq!(parts[3], "2865");     // PR number
}

#[test]
fn test_parse_short_pr_reference() {
    // Short format: owner/repo#123
    let reference = "tokio-rs/axum#2865";

    let parts: Vec<&str> = reference.split('#').collect();
    assert_eq!(parts.len(), 2);

    let repo_parts: Vec<&str> = parts[0].split('/').collect();
    assert_eq!(repo_parts[0], "tokio-rs");
    assert_eq!(repo_parts[1], "axum");
    assert_eq!(parts[1], "2865");
}
