use github_fetch::{GitHubFetcher, Repository};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();

    let fetcher = GitHubFetcher::new(None)?;

    let repo = Repository::new("tokio-rs", "tokio");
    let issue_number = 6800;

    println!("Fetching Tokio issue #{}...", issue_number);

    let issue = fetcher.fetch_issue(&repo, issue_number).await?;

    println!("\n=== Issue Information ===");
    println!("Number: #{}", issue.number);
    println!("Title: {}", issue.title);
    println!("State: {}", issue.state);
    println!("Author: {}", issue.user.login);
    println!("Created: {}", issue.created_at);
    println!("Updated: {}", issue.updated_at);
    println!("Comments: {}", issue.comments);
    println!(
        "Labels: {:?}",
        issue.labels.iter().map(|l| &l.name).collect::<Vec<_>>()
    );
    println!("URL: {}", issue.html_url);

    if let Some(body) = &issue.body {
        println!("\n=== Body ===");
        println!("{}", body);
    }

    println!("\n=== Fetching Comments ===");
    let comments = fetcher.fetch_comments(&repo, issue_number).await?;
    println!("Found {} comments", comments.len());

    for (i, comment) in comments.iter().take(3).enumerate() {
        println!("\n--- Comment {} by {} ---", i + 1, comment.user.login);
        println!("{}", comment.body.chars().take(200).collect::<String>());
        if comment.body.len() > 200 {
            println!("...(truncated)");
        }
    }

    Ok(())
}
