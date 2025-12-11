use github_fetch::{GitHubFetcher, IssueFilters, IssueState, Repository};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();

    let fetcher = GitHubFetcher::new(None)?;

    let repo = Repository::new("tokio-rs", "tokio");

    println!("=== Example 1: Fetch closed issues with 'bug' label ===");
    let filters = IssueFilters {
        state: IssueState::Closed,
        include_labels: vec!["bug".to_string()],
        min_comments: Some(3),
        ..Default::default()
    };

    let result = fetcher.fetch_issues_with_limit(&repo, &filters, 5).await?;

    println!(
        "Found {} issues (filters: {:?})",
        result.issues.len(),
        result.filters_applied
    );
    for issue in &result.issues {
        println!(
            "  #{}: {} (comments: {})",
            issue.number, issue.title, issue.comments
        );
    }

    println!("\n=== Example 2: Fetch Rust error-focused issues ===");
    let rust_filters = IssueFilters::rust_error_focused();

    let rust_result = fetcher
        .fetch_issues_with_limit(&repo, &rust_filters, 3)
        .await?;

    println!("Found {} issues with Rust errors", rust_result.issues.len());
    for issue in &rust_result.issues {
        println!("  #{}: {}", issue.number, issue.title);
        if let Some(body) = &issue.body {
            let error_codes = github_fetch::extract_error_codes(body);
            if !error_codes.is_empty() {
                println!("    Error codes: {:?}", error_codes);
            }
        }
    }

    println!("\n=== Example 3: Check rate limit ===");
    let rate_limit = fetcher.get_rate_limit().await?;
    println!("{}", rate_limit);

    Ok(())
}
