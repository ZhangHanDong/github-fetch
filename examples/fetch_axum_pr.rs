use github_fetch::{GitHubFetcher, Repository};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();

    let fetcher = GitHubFetcher::new(None)?;

    let repo = Repository::new("tokio-rs", "axum");
    let pr_number = 2865;

    println!("Fetching Axum PR #{}...", pr_number);

    let pr = fetcher.fetch_pr(&repo, pr_number).await?;

    println!("\n=== Pull Request Information ===");
    println!("Number: #{}", pr.number);
    println!("Title: {}", pr.title);
    println!("State: {}", pr.state);
    println!("Author: {}", pr.user.login);
    println!("Created: {}", pr.created_at);
    println!("Updated: {}", pr.updated_at);
    if let Some(closed_at) = pr.closed_at {
        println!("Closed: {}", closed_at);
    }
    if let Some(merged_at) = pr.merged_at {
        println!("Merged: {}", merged_at);
    }
    println!("Comments: {}", pr.comments);
    println!(
        "Labels: {:?}",
        pr.labels.iter().map(|l| &l.name).collect::<Vec<_>>()
    );
    println!("URL: {}", pr.html_url);

    if let Some(body) = &pr.body {
        println!("\n=== Description ===");
        println!("{}", body);
    }

    println!("\n=== Fetching PR Files ===");
    let files = fetcher.fetch_pr_files(&repo, pr_number).await?;
    println!("Found {} files changed", files.len());

    for file in files.iter().take(5) {
        println!("\n{}", file.filename);
        println!("  Status: {}", file.status);
        println!(
            "  Changes: +{} -{} ({})",
            file.additions, file.deletions, file.changes
        );

        if let Some(patch) = &file.patch {
            let lines: Vec<&str> = patch.lines().take(10).collect();
            println!("  Patch (first 10 lines):");
            for line in lines {
                println!("    {}", line);
            }
            if patch.lines().count() > 10 {
                println!("    ...(truncated)");
            }
        }
    }

    if files.len() > 5 {
        println!("\n... and {} more files", files.len() - 5);
    }

    Ok(())
}
