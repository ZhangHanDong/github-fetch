use github_fetch::{GitHubFetcherBuilder, IssueFilters, IssueState, Repository};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();

    println!("=== Advanced Usage Example ===\n");

    println!("1. Using Builder Pattern with Custom Configuration");
    let fetcher = GitHubFetcherBuilder::new()
        .user_agent("corust-golden-dataset/0.1.0")
        .rate_limit(30)
        .max_retries(5)
        .build()?;

    println!("   ✓ Fetcher created with custom config\n");

    println!("2. Testing Connection");
    fetcher.test_connection().await?;
    let rate_limit = fetcher.get_rate_limit().await?;
    println!("   ✓ Connection successful");
    println!("   {}\n", rate_limit);

    println!("3. Fetching from Multiple Repositories");
    let repos = vec![
        Repository::new("tokio-rs", "tokio"),
        Repository::new("tokio-rs", "axum"),
        Repository::new("actix", "actix-web"),
    ];

    for repo in &repos {
        println!("\n   Fetching from {}...", repo.full_name);

        let filters = IssueFilters {
            state: IssueState::Closed,
            include_labels: vec!["bug".to_string()],
            code_blocks_only: true,
            min_body_length: Some(100),
            ..Default::default()
        };

        match fetcher.fetch_issues_with_limit(&repo, &filters, 2).await {
            Ok(result) => {
                println!("   ✓ Found {} issues", result.issues.len());
                for issue in &result.issues {
                    println!("     - #{}: {}", issue.number, issue.title);
                    println!(
                        "       Labels: {:?}",
                        issue.labels.iter().map(|l| &l.name).collect::<Vec<_>>()
                    );

                    if let Some(body) = &issue.body {
                        let has_code = github_fetch::has_code_blocks(body);
                        let has_errors = github_fetch::has_rust_error_codes(body);
                        println!(
                            "       Code blocks: {}, Rust errors: {}",
                            has_code, has_errors
                        );

                        if has_errors {
                            let codes = github_fetch::extract_error_codes(body);
                            println!("       Error codes: {:?}", codes);
                        }
                    }
                }
            }
            Err(e) => {
                println!("   ✗ Error: {}", e);
            }
        }
    }

    println!("\n4. Fetching Specific Items by Number");
    let tokio_repo = Repository::new("tokio-rs", "tokio");

    println!("\n   Fetching issue #6800...");
    match fetcher.fetch_issue(&tokio_repo, 6800).await {
        Ok(issue) => {
            println!("   ✓ {}", issue.title);
            println!(
                "     Author: {}, State: {}, Comments: {}",
                issue.user.login, issue.state, issue.comments
            );
        }
        Err(e) => println!("   ✗ Error: {}", e),
    }

    println!("\n5. Fetching Discussion");
    let actix_repo = Repository::new("actix", "actix-web");

    println!("\n   Fetching discussion #3766...");
    match fetcher.fetch_discussion(&actix_repo, 3766).await {
        Ok(discussion) => {
            println!("   ✓ {}", discussion.title);
            println!(
                "     Author: {}, Comments: {}",
                discussion.author.login,
                discussion.comments.len()
            );
        }
        Err(e) => println!("   ✗ Error: {}", e),
    }

    println!("\n6. Repository Creation Methods");
    let repo1 = Repository::new("owner", "repo");
    let repo2 = Repository::from_full_name("owner/repo")?;
    let repo3 = Repository::from_url("https://github.com/owner/repo")?;

    println!("   ✓ Created repositories:");
    println!("     - {}", repo1.full_name);
    println!("     - {}", repo2.full_name);
    println!("     - {}", repo3.full_name);

    println!("\n=== All Examples Completed ===");

    Ok(())
}
