# Project Guide for Claude

## Scope

- This is a Rust library for fetching GitHub issues, PRs, reviews, discussions, and diffs
- Work only within this repository
- Never modify files outside the project directory without explicit confirmation

## Common Commands

- Build: `cargo build`
- Run tests: `cargo test`
- Run examples: `cargo run --example <name>`
- Check: `cargo check`
- Format: `cargo fmt`
- Lint: `cargo clippy`

## Code Style

- Follow Rust idioms and the existing code style
- Use `anyhow::Result` for error handling in examples
- Use the custom `GitHubFetchError` for library errors
- Prefer async/await with tokio runtime
- Keep functions focused and well-documented

## Testing Expectations

- Use deterministic test data where possible
- Mock GitHub API calls in unit tests to avoid rate limiting
- Set `GITHUB_TOKEN` environment variable for integration tests

## Project Structure

```
src/
  lib.rs        - Main library entry, re-exports
  client.rs     - GitHubFetcher implementation
  config.rs     - Builder pattern configuration
  types.rs      - Data structures (Issue, PR, Review, etc.)
  filters.rs    - Issue filtering logic
  discussion.rs - GraphQL-based discussion fetching
  error.rs      - Error types
examples/       - Usage examples
.claude/skills/ - Claude Code Skills
```

## Skills

This project includes a Claude Code Skill for PR review workflows:

- Location: `.claude/skills/pr-review/`
- Purpose: Fetch PR data, analyze reviews, fix code issues, and re-review

## Environment Variables

- `GITHUB_TOKEN` - Required for GitHub API authentication

## CI/CD

The project uses GitHub Actions for CI. See `.github/workflows/ci.yml`:

- **test**: Build and run unit tests
- **lint**: Format and clippy checks
- **skill-validation**: Validates SKILL.md structure and frontmatter
- **integration-test**: API tests (requires GITHUB_TOKEN)
- **examples**: Verify all examples compile

Run locally:
```bash
cargo test                           # Unit tests
cargo test --ignored -- --nocapture  # Integration tests (needs GITHUB_TOKEN)
```
