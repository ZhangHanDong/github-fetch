use github_fetch::{GitHubFetcher, Repository};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();

    let fetcher = GitHubFetcher::new(None)?;

    let repo = Repository::new("actix", "actix-web");
    let discussion_number = 3766;

    println!("Fetching Actix-web discussion #{}...", discussion_number);

    let discussion = fetcher.fetch_discussion(&repo, discussion_number).await?;

    println!("\n=== Discussion Information ===");
    println!("Number: #{}", discussion.number);
    println!("Title: {}", discussion.title);
    println!("Author: {}", discussion.author.login);
    println!("Created: {}", discussion.created_at);
    println!("Updated: {}", discussion.updated_at);
    println!("URL: {}", discussion.url);
    println!("Comments: {}", discussion.comments.len());

    println!("\n=== Original Post ===");
    println!("{}", discussion.body);

    println!("\n=== Comments ===");
    for (i, comment) in discussion.comments.iter().take(5).enumerate() {
        println!("\n--- Comment {} by {} ---", i + 1, comment.author.login);
        println!("Created: {}", comment.created_at);
        println!("{}", comment.body.chars().take(300).collect::<String>());
        if comment.body.len() > 300 {
            println!("...(truncated)");
        }
    }

    if discussion.comments.len() > 5 {
        println!("\n... and {} more comments", discussion.comments.len() - 5);
    }

    println!("\n=== Test: Fetch by URL ===");
    let discussion_url = format!(
        "https://github.com/{}/discussions/{}",
        repo.full_name, discussion_number
    );
    let discussion2 = fetcher.fetch_discussion_by_url(&discussion_url).await?;
    println!(
        "Successfully fetched discussion by URL: {}",
        discussion2.title
    );

    Ok(())
}
