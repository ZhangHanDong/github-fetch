# github-fetch

[![Crates.io](https://img.shields.io/crates/v/github-fetch.svg)](https://crates.io/crates/github-fetch)
[![Documentation](https://docs.rs/github-fetch/badge.svg)](https://docs.rs/github-fetch)
[![License](https://img.shields.io/crates/l/github-fetch.svg)](LICENSE-MIT)

[English](README.md) | [日本語](README_JP.md)

一个用于通过 GitHub API 获取 Issues、Pull Requests、Discussions、Reviews 和 Diff 信息的 Rust 库。

## 功能特性

- 灵活过滤获取 Issues 和 Pull Requests
- 获取 PR Reviews（批准、请求修改等）
- 获取 PR Review Comments（Diff 上的行内评论）
- 获取 PR 文件变更及 Diff/Patch 内容
- 通过 GraphQL API 获取 GitHub Discussions
- 支持速率限制和重试
- 支持 Builder 模式配置

## 安装

在 `Cargo.toml` 中添加：

```toml
[dependencies]
github-fetch = "0.1"
tokio = { version = "1", features = ["full"] }
anyhow = "1.0"
```

## 认证

设置 GitHub Token 环境变量：

```bash
export GITHUB_TOKEN=ghp_your_token_here
```

## 快速开始

```rust
use github_fetch::{GitHubFetcher, Repository};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let fetcher = GitHubFetcher::new(None)?;
    let repo = Repository::new("tokio-rs", "tokio");

    // 获取 PR 及其 reviews 和 diff
    let pr = fetcher.fetch_pr(&repo, 1234).await?;
    let reviews = fetcher.fetch_pr_reviews(&repo, 1234).await?;
    let files = fetcher.fetch_pr_files(&repo, 1234).await?;

    println!("PR: {}", pr.title);
    println!("Reviews 数量: {}", reviews.len());
    println!("变更文件数: {}", files.len());

    Ok(())
}
```

## 使用方法

### Builder 模式

```rust
use github_fetch::GitHubFetcherBuilder;

let fetcher = GitHubFetcherBuilder::new()
    .token("ghp_your_token")
    .user_agent("my-app/1.0.0")
    .rate_limit(30)
    .max_retries(5)
    .build()?;
```

### 带过滤条件获取 Issues

```rust
use github_fetch::{GitHubFetcher, Repository, IssueFilters, IssueState};

let filters = IssueFilters {
    state: IssueState::Closed,
    include_labels: vec!["bug".to_string()],
    min_comments: Some(3),
    code_blocks_only: true,
    ..Default::default()
};

let result = fetcher.fetch_issues_with_limit(&repo, &filters, 10).await?;
```

### 获取 PR Reviews

```rust
let reviews = fetcher.fetch_pr_reviews(&repo, 2865).await?;

for review in &reviews {
    println!("{}: {}", review.user.login, review.state);
    // state: APPROVED, CHANGES_REQUESTED, COMMENTED, DISMISSED, PENDING
}
```

### 获取 PR Review Comments（Diff 行内评论）

```rust
let review_comments = fetcher.fetch_pr_review_comments(&repo, 2865).await?;

for comment in &review_comments {
    println!("文件: {} 行号: {:?}", comment.path, comment.line);
    println!("评论: {}", comment.body);
    println!("Diff 上下文:\n{}", comment.diff_hunk);
}
```

### 获取 PR 文件变更（Diff）

```rust
let files = fetcher.fetch_pr_files(&repo, 2865).await?;

for file in &files {
    println!("{}: +{} -{}", file.filename, file.additions, file.deletions);
    if let Some(patch) = &file.patch {
        println!("{}", patch);
    }
}
```

### 获取 Discussions

```rust
let discussion = fetcher.fetch_discussion(&repo, 3766).await?;
println!("标题: {}", discussion.title);

// 或通过 URL 获取
let discussion = fetcher.fetch_discussion_by_url(
    "https://github.com/actix/actix-web/discussions/3766"
).await?;
```

## 数据结构

### PrReview

```rust
pub struct PrReview {
    pub id: u64,
    pub user: GitHubUser,
    pub body: Option<String>,
    pub state: String,  // APPROVED, CHANGES_REQUESTED, COMMENTED, DISMISSED, PENDING
    pub submitted_at: Option<DateTime<Utc>>,
    pub html_url: String,
    pub commit_id: Option<String>,
}
```

### PrReviewComment

```rust
pub struct PrReviewComment {
    pub id: u64,
    pub user: GitHubUser,
    pub body: String,
    pub path: String,           // 文件路径
    pub line: Option<u32>,      // 行号
    pub diff_hunk: String,      // Diff 上下文
    pub side: Option<String>,   // LEFT 或 RIGHT
    pub in_reply_to_id: Option<u64>,
}
```

### PrFile

```rust
pub struct PrFile {
    pub filename: String,
    pub status: String,        // Added, Modified, Removed, Renamed
    pub additions: u32,
    pub deletions: u32,
    pub patch: Option<String>, // Diff 内容
}
```

## 运行示例

```bash
export GITHUB_TOKEN=ghp_your_token

cargo run --example fetch_tokio_issue
cargo run --example fetch_axum_pr
cargo run --example fetch_pr_review
cargo run --example fetch_with_filters
cargo run --example fetch_actix_discussion
cargo run --example advanced_usage
```

## 错误处理

```rust
use github_fetch::{GitHubFetcher, GitHubFetchError, Repository};

match fetcher.fetch_issue(&repo, 999999).await {
    Ok(issue) => println!("找到: {}", issue.title),
    Err(GitHubFetchError::NotFound(msg)) => println!("未找到: {}", msg),
    Err(GitHubFetchError::AuthError(msg)) => println!("认证错误: {}", msg),
    Err(GitHubFetchError::RateLimitExceeded) => println!("超出速率限制!"),
    Err(e) => println!("错误: {}", e),
}
```

## Claude Code Skill

本项目包含一个用于 PR 审查和修复的 Claude Code Skill。该 Skill 可以自动获取 PR 信息、分析 Review 评论、修复代码问题并重新审查。

### Skill 功能

1. **获取 PR 数据** - 获取 PR 详情、文件 Diff、Reviews 和行内评论
2. **分析 Reviews** - 汇总 APPROVED/CHANGES_REQUESTED 状态和待办事项
3. **修复问题** - 根据 Review 反馈修复代码
4. **重新审查** - 验证修复并检查新问题

### 安装方式

#### 方式一：项目级安装（推荐）

将 Skill 克隆或复制到项目的 `.claude/skills/` 目录：

```bash
# 克隆仓库并复制 skill 到你的项目
git clone https://github.com/ZhangHanDong/github-fetch.git
mkdir -p your-project/.claude/skills
cp -r github-fetch/.claude/skills/pr-review your-project/.claude/skills/

# 如果已经在使用此库，skills 已包含在 .claude/skills/pr-review/
```

Claude Code 在项目中工作时会自动发现该 Skill。

#### 方式二：通过 Claude Code 插件市场安装

1. 添加市场源：
   ```
   /plugin marketplace add ZhangHanDong/github-fetch
   ```
2. 安装插件：
   ```
   /plugin install pr-review@github-fetch
   ```
3. 验证安装：
   ```
   /plugin marketplace list
   ```

### 使用方法

安装后，用自然语言对 Claude 说：

```
Review PR https://github.com/tokio-rs/axum/pull/2865
```

```
帮我分析 owner/repo#123 的 review 意见并修复问题
```

```
这个 PR 有哪些需要修改的地方？https://github.com/owner/repo/pull/456
```

### 使用示例

```
你: Review PR https://github.com/owner/repo/pull/123

Claude:
## PR #123: 修复认证 bug

**作者:** @developer | **状态:** Open

### Reviews
| 审查者 | 状态 | 评论 |
|--------|------|------|
| @reviewer1 | CHANGES_REQUESTED | 请添加错误处理 |
| @reviewer2 | COMMENTED | 建议使用 Result 类型 |

### 文件变更 (2 个文件, +45 -12)
- `src/auth.rs` (+30 -10) Modified
- `src/lib.rs` (+15 -2) Modified

### Review 评论

#### src/auth.rs:42
> @reviewer1: 这里应该处理 None 的情况

### 待办事项
1. [ ] 处理 authenticate() 中的 None 情况 (src/auth.rs:42)
2. [ ] 添加 Result 返回类型 (src/auth.rs:45)

需要我修复这些问题吗？

你: 好的，请修复

Claude: [读取文件，应用修复，重新审查...]

所有问题已修复：
- 添加了 Option 的 match 表达式处理
- 将返回类型改为 Result<(), AuthError>

重新审查中... 未发现新问题。
```

### 前置条件

- 必须设置 `GITHUB_TOKEN` 环境变量
- 修复代码需要：仓库已克隆到本地

### Skill 文件结构

```
.claude/skills/pr-review/
├── SKILL.md      # Claude 的主要指令
├── WORKFLOW.md   # 详细工作流参考
└── scripts/
    └── fetch_pr.rs  # 示例获取脚本
```

## 许可证

MIT OR Apache-2.0
