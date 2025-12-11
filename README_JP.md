# github-fetch

[![Crates.io](https://img.shields.io/crates/v/github-fetch.svg)](https://crates.io/crates/github-fetch)
[![Documentation](https://docs.rs/github-fetch/badge.svg)](https://docs.rs/github-fetch)
[![License](https://img.shields.io/crates/l/github-fetch.svg)](LICENSE-MIT)

[English](README.md) | [中文](README_CN.md)

GitHub API を使用して Issues、Pull Requests、Discussions、Reviews、Diff 情報を取得するための Rust ライブラリです。

## 機能

- 柔軟なフィルタリングで Issues と Pull Requests を取得
- PR Reviews（承認、変更リクエストなど）を取得
- PR Review Comments（Diff 上のインラインコメント）を取得
- PR ファイル変更と Diff/Patch 内容を取得
- GraphQL API で GitHub Discussions を取得
- レート制限とリトライをサポート
- Builder パターンによる設定

## インストール

`Cargo.toml` に追加：

```toml
[dependencies]
github-fetch = "0.1"
tokio = { version = "1", features = ["full"] }
anyhow = "1.0"
```

## 認証

GitHub Token を環境変数に設定：

```bash
export GITHUB_TOKEN=ghp_your_token_here
```

## クイックスタート

```rust
use github_fetch::{GitHubFetcher, Repository};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let fetcher = GitHubFetcher::new(None)?;
    let repo = Repository::new("tokio-rs", "tokio");

    // PR と reviews、diff を取得
    let pr = fetcher.fetch_pr(&repo, 1234).await?;
    let reviews = fetcher.fetch_pr_reviews(&repo, 1234).await?;
    let files = fetcher.fetch_pr_files(&repo, 1234).await?;

    println!("PR: {}", pr.title);
    println!("Reviews 数: {}", reviews.len());
    println!("変更ファイル数: {}", files.len());

    Ok(())
}
```

## 使用方法

### Builder パターン

```rust
use github_fetch::GitHubFetcherBuilder;

let fetcher = GitHubFetcherBuilder::new()
    .token("ghp_your_token")
    .user_agent("my-app/1.0.0")
    .rate_limit(30)
    .max_retries(5)
    .build()?;
```

### フィルタ付きで Issues を取得

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

### PR Reviews を取得

```rust
let reviews = fetcher.fetch_pr_reviews(&repo, 2865).await?;

for review in &reviews {
    println!("{}: {}", review.user.login, review.state);
    // state: APPROVED, CHANGES_REQUESTED, COMMENTED, DISMISSED, PENDING
}
```

### PR Review Comments（Diff インラインコメント）を取得

```rust
let review_comments = fetcher.fetch_pr_review_comments(&repo, 2865).await?;

for comment in &review_comments {
    println!("ファイル: {} 行: {:?}", comment.path, comment.line);
    println!("コメント: {}", comment.body);
    println!("Diff コンテキスト:\n{}", comment.diff_hunk);
}
```

### PR ファイル変更（Diff）を取得

```rust
let files = fetcher.fetch_pr_files(&repo, 2865).await?;

for file in &files {
    println!("{}: +{} -{}", file.filename, file.additions, file.deletions);
    if let Some(patch) = &file.patch {
        println!("{}", patch);
    }
}
```

### Discussions を取得

```rust
let discussion = fetcher.fetch_discussion(&repo, 3766).await?;
println!("タイトル: {}", discussion.title);

// URL で取得
let discussion = fetcher.fetch_discussion_by_url(
    "https://github.com/actix/actix-web/discussions/3766"
).await?;
```

## データ構造

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
    pub path: String,           // ファイルパス
    pub line: Option<u32>,      // 行番号
    pub diff_hunk: String,      // Diff コンテキスト
    pub side: Option<String>,   // LEFT または RIGHT
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

## サンプル実行

```bash
export GITHUB_TOKEN=ghp_your_token

cargo run --example fetch_tokio_issue
cargo run --example fetch_axum_pr
cargo run --example fetch_pr_review
cargo run --example fetch_with_filters
cargo run --example fetch_actix_discussion
cargo run --example advanced_usage
```

## エラーハンドリング

```rust
use github_fetch::{GitHubFetcher, GitHubFetchError, Repository};

match fetcher.fetch_issue(&repo, 999999).await {
    Ok(issue) => println!("見つかりました: {}", issue.title),
    Err(GitHubFetchError::NotFound(msg)) => println!("見つかりません: {}", msg),
    Err(GitHubFetchError::AuthError(msg)) => println!("認証エラー: {}", msg),
    Err(GitHubFetchError::RateLimitExceeded) => println!("レート制限超過!"),
    Err(e) => println!("エラー: {}", e),
}
```

## Claude Code Skill

このプロジェクトには、PR レビューと修正のための Claude Code Skill が含まれています。この Skill は PR 情報の取得、レビューコメントの分析、コード問題の修正、再レビューを自動的に行います。

### Skill の機能

1. **PR データの取得** - PR 詳細、ファイル Diff、Reviews、インラインコメントを取得
2. **Reviews の分析** - APPROVED/CHANGES_REQUESTED ステータスとアクションアイテムを要約
3. **問題の修正** - レビューフィードバックに基づいてコードを修正
4. **再レビュー** - 修正を検証し、新しい問題をチェック

### インストール方法

#### 方法1：プロジェクトレベルインストール（推奨）

Skill をプロジェクトの `skills/` ディレクトリにクローンまたはコピー：

```bash
# リポジトリをクローンしてプロジェクトに skill をコピー
git clone https://github.com/ZhangHanDong/github-fetch.git
cp -r github-fetch/skills/pr-review your-project/skills/

# このライブラリを使用している場合、skills は skills/pr-review/ に含まれています
```

Claude Code はプロジェクトで作業する際に自動的に Skill を検出します。

#### 方法2：Claude Code プラグインマーケットプレイス経由

1. Claude Code で Skills を有効化：Settings > Capabilities > "Skills" をオン
2. マーケットプレイスソースを追加：
   ```
   /plugin marketplace add ZhangHanDong/github-fetch
   ```
3. Skill をインストール：
   ```
   /plugin install pr-review@github-fetch
   ```

### 使用方法

インストール後、自然言語で Claude に依頼：

```
Review PR https://github.com/tokio-rs/axum/pull/2865
```

```
owner/repo#123 のレビューコメントを分析して問題を修正してください
```

```
この PR で修正が必要な箇所は？ https://github.com/owner/repo/pull/456
```

### 使用例

```
あなた: Review PR https://github.com/owner/repo/pull/123

Claude:
## PR #123: 認証バグの修正

**作成者:** @developer | **状態:** Open

### Reviews
| レビュアー | 状態 | コメント |
|-----------|------|----------|
| @reviewer1 | CHANGES_REQUESTED | エラーハンドリングを追加してください |
| @reviewer2 | COMMENTED | Result 型の使用を検討してください |

### 変更ファイル (2 ファイル, +45 -12)
- `src/auth.rs` (+30 -10) Modified
- `src/lib.rs` (+15 -2) Modified

### レビューコメント

#### src/auth.rs:42
> @reviewer1: ここで None のケースを処理する必要があります

### アクションアイテム
1. [ ] authenticate() で None ケースを処理 (src/auth.rs:42)
2. [ ] Result 戻り型を追加 (src/auth.rs:45)

これらの問題を修正しますか？

あなた: はい、修正してください

Claude: [ファイルを読み取り、修正を適用、再レビュー...]

すべての問題が修正されました：
- Option の match 式を追加
- 戻り型を Result<(), AuthError> に変更

再レビュー中... 新しい問題は見つかりませんでした。
```

### 前提条件

- `GITHUB_TOKEN` 環境変数を設定する必要があります
- コード修正には：リポジトリがローカルにクローンされている必要があります

### Skill ファイル構成

```
skills/pr-review/
├── SKILL.md      # Claude への主要な指示
├── WORKFLOW.md   # 詳細なワークフローリファレンス
└── scripts/
    └── fetch_pr.rs  # サンプル取得スクリプト
```

## ライセンス

MIT OR Apache-2.0
