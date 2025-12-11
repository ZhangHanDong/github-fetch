use chrono::{DateTime, Utc};
use log::info;
use regex::Regex;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, USER_AGENT};
use serde_json::json;

use crate::config::GitHubConfig;
use crate::error::{GitHubFetchError, Result};
use crate::types::{Discussion, DiscussionComment, GitHubUser, Repository};

pub struct DiscussionClient {
    client: reqwest::Client,
    config: GitHubConfig,
}

impl DiscussionClient {
    pub fn new(config: GitHubConfig) -> Result<Self> {
        let client = reqwest::Client::new();
        Ok(Self { client, config })
    }

    pub async fn fetch_discussion(
        &self,
        repo: &Repository,
        discussion_number: u64,
    ) -> Result<Discussion> {
        info!(
            "Fetching discussion data for {}/{} #{}",
            repo.owner, repo.name, discussion_number
        );

        let token = std::env::var(&self.config.token_env_var).map_err(|_| {
            GitHubFetchError::AuthError(format!(
                "{} environment variable not set",
                self.config.token_env_var
            ))
        })?;

        let query = self.build_discussion_query(&repo.owner, &repo.name, discussion_number);

        let mut headers = HeaderMap::new();
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", token))
                .map_err(|e| GitHubFetchError::ConfigError(format!("Invalid token: {}", e)))?,
        );
        headers.insert(
            USER_AGENT,
            HeaderValue::from_str(&self.config.user_agent)
                .map_err(|e| GitHubFetchError::ConfigError(format!("Invalid user agent: {}", e)))?,
        );
        headers.insert(
            "Accept",
            HeaderValue::from_static("application/vnd.github+json"),
        );

        let request_body = json!({
            "query": query
        });

        let response = self
            .client
            .post("https://api.github.com/graphql")
            .headers(headers)
            .json(&request_body)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(GitHubFetchError::ApiError(format!(
                "GitHub GraphQL API request failed: {}",
                error_text
            )));
        }

        let response_json: serde_json::Value = response.json().await?;

        self.parse_discussion_response(response_json, repo, discussion_number)
    }

    pub async fn fetch_discussion_by_url(&self, discussion_url: &str) -> Result<Discussion> {
        let (owner, repo, discussion_number) = self.parse_discussion_url(discussion_url)?;
        let repository = Repository::new(owner, repo);
        self.fetch_discussion(&repository, discussion_number).await
    }

    fn parse_discussion_url(&self, url: &str) -> Result<(String, String, u64)> {
        let re = Regex::new(r"https://github\.com/([^/]+)/([^/]+)/discussions/(\d+)")
            .map_err(|e| GitHubFetchError::ConfigError(format!("Invalid regex: {}", e)))?;

        if let Some(captures) = re.captures(url) {
            let owner = captures.get(1).unwrap().as_str().to_string();
            let repo = captures.get(2).unwrap().as_str().to_string();
            let discussion_number: u64 =
                captures.get(3).unwrap().as_str().parse().map_err(|e| {
                    GitHubFetchError::InvalidRepository(format!("Invalid discussion number: {}", e))
                })?;
            Ok((owner, repo, discussion_number))
        } else {
            Err(GitHubFetchError::InvalidRepository(format!(
                "Invalid GitHub discussion URL format: {}",
                url
            )))
        }
    }

    fn build_discussion_query(&self, owner: &str, repo: &str, discussion_number: u64) -> String {
        format!(
            r#"
    {{
        repository(owner: "{}", name: "{}") {{
            discussion(number: {}) {{
                number
                title
                body
                url
                author {{
                    login
                    ... on User {{
                        id
                        avatarUrl
                    }}
                }}
                createdAt
                updatedAt
                comments(first: 100) {{
                    nodes {{
                        id
                        body
                        author {{
                            login
                            ... on User {{
                                id
                                avatarUrl
                            }}
                        }}
                        createdAt
                        updatedAt
                    }}
                }}
            }}
        }}
    }}"#,
            owner, repo, discussion_number
        )
    }

    fn parse_discussion_response(
        &self,
        response_json: serde_json::Value,
        repo: &Repository,
        discussion_number: u64,
    ) -> Result<Discussion> {
        let discussion_json = response_json
            .get("data")
            .and_then(|d| d.get("repository"))
            .and_then(|r| r.get("discussion"))
            .ok_or_else(|| {
                GitHubFetchError::NotFound(format!(
                    "Discussion #{} not found in {}/{}",
                    discussion_number, repo.owner, repo.name
                ))
            })?;

        let comments: Vec<DiscussionComment> = discussion_json
            .get("comments")
            .and_then(|c| c.get("nodes"))
            .and_then(|nodes| nodes.as_array())
            .map(|nodes| {
                nodes
                    .iter()
                    .filter_map(|comment| {
                        Some(DiscussionComment {
                            id: comment.get("id")?.as_str()?.to_string(),
                            body: comment.get("body")?.as_str()?.to_string(),
                            author: GitHubUser {
                                id: comment.get("author")?.get("id")?.as_str()?.parse().ok()?,
                                login: comment.get("author")?.get("login")?.as_str()?.to_string(),
                                avatar_url: comment
                                    .get("author")?
                                    .get("avatarUrl")?
                                    .as_str()?
                                    .to_string(),
                            },
                            created_at: comment
                                .get("createdAt")?
                                .as_str()?
                                .parse::<DateTime<Utc>>()
                                .ok()?,
                            updated_at: comment
                                .get("updatedAt")?
                                .as_str()?
                                .parse::<DateTime<Utc>>()
                                .ok()?,
                        })
                    })
                    .collect()
            })
            .unwrap_or_default();

        let number = discussion_json
            .get("number")
            .and_then(|n| n.as_u64())
            .unwrap_or(discussion_number);

        let author_json = discussion_json.get("author");
        let author = if let Some(author) = author_json {
            GitHubUser {
                id: author
                    .get("id")
                    .and_then(|id| id.as_str())
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(0),
                login: author
                    .get("login")
                    .and_then(|l| l.as_str())
                    .unwrap_or("unknown")
                    .to_string(),
                avatar_url: author
                    .get("avatarUrl")
                    .and_then(|u| u.as_str())
                    .unwrap_or("")
                    .to_string(),
            }
        } else {
            GitHubUser {
                id: 0,
                login: "unknown".to_string(),
                avatar_url: "".to_string(),
            }
        };

        Ok(Discussion {
            number,
            title: discussion_json
                .get("title")
                .and_then(|t| t.as_str())
                .unwrap_or("Unknown Discussion")
                .to_string(),
            body: discussion_json
                .get("body")
                .and_then(|b| b.as_str())
                .unwrap_or("")
                .to_string(),
            url: discussion_json
                .get("url")
                .and_then(|u| u.as_str())
                .unwrap_or("")
                .to_string(),
            author,
            created_at: discussion_json
                .get("createdAt")
                .and_then(|c| c.as_str())
                .and_then(|s| s.parse::<DateTime<Utc>>().ok())
                .unwrap_or_else(Utc::now),
            updated_at: discussion_json
                .get("updatedAt")
                .and_then(|u| u.as_str())
                .and_then(|s| s.parse::<DateTime<Utc>>().ok())
                .unwrap_or_else(Utc::now),
            comments,
        })
    }
}
