//! Fetch complete PR information including reviews, comments, and diffs
//!
//! Usage: cargo run --example fetch_pr_review -- <owner> <repo> <pr_number>
//!
//! Example: cargo run --example fetch_pr_review -- tokio-rs axum 2865

use github_fetch::{GitHubFetcher, Repository};
use std::env;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();

    let args: Vec<String> = env::args().collect();
    if args.len() != 4 {
        eprintln!("Usage: {} <owner> <repo> <pr_number>", args[0]);
        eprintln!("Example: {} tokio-rs axum 2865", args[0]);
        std::process::exit(1);
    }

    let owner = &args[1];
    let repo_name = &args[2];
    let pr_number: u64 = args[3].parse().expect("PR number must be a valid integer");

    let fetcher = GitHubFetcher::new(None)?;
    let repo = Repository::new(owner, repo_name);

    println!("Fetching PR #{} from {}/{}...\n", pr_number, owner, repo_name);

    // Fetch PR details
    let pr = fetcher.fetch_pr(&repo, pr_number).await?;

    println!("## PR #{}: {}\n", pr.number, pr.title);
    println!("**Author:** @{} | **State:** {}", pr.user.login, pr.state);
    if !pr.labels.is_empty() {
        println!(
            "**Labels:** {}",
            pr.labels.iter().map(|l| &l.name).collect::<Vec<_>>().join(", ")
        );
    }
    println!("**URL:** {}", pr.html_url);
    println!();

    if let Some(body) = &pr.body {
        println!("### Description\n");
        println!("{}\n", body);
    }

    // Fetch reviews
    println!("---\n### Reviews\n");
    let reviews = fetcher.fetch_pr_reviews(&repo, pr_number).await?;

    if reviews.is_empty() {
        println!("No reviews yet.\n");
    } else {
        println!("| Reviewer | State | Comment |");
        println!("|----------|-------|---------|");
        for review in &reviews {
            let body_preview = review
                .body
                .as_ref()
                .map(|b| {
                    let preview: String = b.chars().take(50).collect();
                    if b.len() > 50 {
                        format!("{}...", preview)
                    } else {
                        preview
                    }
                })
                .unwrap_or_else(|| "-".to_string());
            println!("| @{} | {} | {} |", review.user.login, review.state, body_preview);
        }
        println!();
    }

    // Fetch file changes
    println!("---\n### Files Changed\n");
    let files = fetcher.fetch_pr_files(&repo, pr_number).await?;

    let total_additions: u32 = files.iter().map(|f| f.additions).sum();
    let total_deletions: u32 = files.iter().map(|f| f.deletions).sum();
    println!(
        "**{} files changed**, +{} -{}\n",
        files.len(),
        total_additions,
        total_deletions
    );

    for file in &files {
        println!(
            "- `{}` (+{} -{}) {}",
            file.filename, file.additions, file.deletions, file.status
        );
    }
    println!();

    // Show diffs for each file
    println!("---\n### Diffs\n");
    for file in &files {
        if let Some(patch) = &file.patch {
            println!("#### {}\n", file.filename);
            println!("```diff\n{}\n```\n", patch);
        }
    }

    // Fetch review comments (inline comments on diff)
    println!("---\n### Review Comments (Inline)\n");
    let review_comments = fetcher.fetch_pr_review_comments(&repo, pr_number).await?;

    if review_comments.is_empty() {
        println!("No inline review comments.\n");
    } else {
        // Group by file
        let mut comments_by_file: std::collections::HashMap<&str, Vec<_>> =
            std::collections::HashMap::new();
        for comment in &review_comments {
            comments_by_file
                .entry(&comment.path)
                .or_default()
                .push(comment);
        }

        for (path, comments) in comments_by_file {
            println!("#### {}\n", path);
            for comment in comments {
                let line_info = comment
                    .line
                    .map(|l| format!("Line {}", l))
                    .unwrap_or_else(|| "".to_string());

                println!("**@{}** ({})", comment.user.login, line_info);
                println!("> {}\n", comment.body.replace('\n', "\n> "));

                if !comment.diff_hunk.is_empty() {
                    println!("```diff\n{}\n```\n", comment.diff_hunk);
                }
            }
        }
    }

    // Fetch general comments
    println!("---\n### General Comments\n");
    let comments = fetcher.fetch_comments(&repo, pr_number).await?;

    if comments.is_empty() {
        println!("No general comments.\n");
    } else {
        for comment in comments.iter().take(10) {
            println!("**@{}** ({})", comment.user.login, comment.created_at);
            println!("{}\n", comment.body);
        }
        if comments.len() > 10 {
            println!("... and {} more comments\n", comments.len() - 10);
        }
    }

    // Summary of action items
    println!("---\n### Action Items\n");
    let changes_requested: Vec<_> = reviews
        .iter()
        .filter(|r| r.state == "ChangesRequested" || r.state == "CHANGES_REQUESTED")
        .collect();

    if changes_requested.is_empty() && review_comments.is_empty() {
        println!("No action items - PR looks good!\n");
    } else {
        let mut item_num = 1;
        for review in &changes_requested {
            if let Some(body) = &review.body {
                println!("{}. [ ] {}", item_num, body.lines().next().unwrap_or(""));
                item_num += 1;
            }
        }
        for comment in &review_comments {
            if comment.in_reply_to_id.is_none() {
                // Only top-level comments
                let preview: String = comment.body.chars().take(80).collect();
                println!(
                    "{}. [ ] {} ({}:{})",
                    item_num,
                    preview,
                    comment.path,
                    comment.line.unwrap_or(0)
                );
                item_num += 1;
            }
        }
    }

    Ok(())
}
