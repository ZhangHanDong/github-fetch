use chrono::{DateTime, Utc};
use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::types::GitHubIssue;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum IssueState {
    Open,
    Closed,
    All,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssueFilters {
    pub state: IssueState,
    pub include_labels: Vec<String>,
    pub exclude_labels: Vec<String>,
    pub rust_errors_only: bool,
    pub code_blocks_only: bool,
    pub min_body_length: Option<usize>,
    pub date_range: Option<DateRange>,
    pub include_pull_requests: bool,
    pub min_comments: Option<u32>,
    pub required_keywords: Vec<String>,
    pub excluded_keywords: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DateRange {
    pub start: Option<DateTime<Utc>>,
    pub end: Option<DateTime<Utc>>,
}

impl Default for IssueFilters {
    fn default() -> Self {
        Self {
            state: IssueState::All,
            include_labels: vec![],
            exclude_labels: vec![
                "duplicate".to_string(),
                "invalid".to_string(),
                "wontfix".to_string(),
                "question".to_string(),
            ],
            rust_errors_only: false,
            code_blocks_only: false,
            min_body_length: Some(50),
            date_range: None,
            include_pull_requests: false,
            min_comments: None,
            required_keywords: vec![],
            excluded_keywords: vec![
                "discussion".to_string(),
                "RFC".to_string(),
                "tracking".to_string(),
            ],
        }
    }
}

impl IssueFilters {
    pub fn rust_error_focused() -> Self {
        Self {
            state: IssueState::All,
            include_labels: vec![
                "E-help-wanted".to_string(),
                "A-diagnostics".to_string(),
                "A-borrowck".to_string(),
                "E-easy".to_string(),
                "E-medium".to_string(),
            ],
            rust_errors_only: true,
            code_blocks_only: true,
            min_body_length: Some(100),
            ..Default::default()
        }
    }

    pub fn matches(&self, issue: &GitHubIssue) -> bool {
        match self.state {
            IssueState::Open => {
                if issue.state != "Open" {
                    return false;
                }
            }
            IssueState::Closed => {
                if issue.state != "Closed" {
                    return false;
                }
            }
            IssueState::All => {}
        }

        if !self.include_pull_requests && issue.is_pull_request {
            return false;
        }

        if !self.include_labels.is_empty() {
            let has_included_label = issue.labels.iter().any(|label| {
                self.include_labels
                    .iter()
                    .any(|include_label| include_label.to_lowercase() == label.name.to_lowercase())
            });
            if !has_included_label {
                return false;
            }
        }

        if issue.labels.iter().any(|label| {
            self.exclude_labels
                .iter()
                .any(|exclude_label| exclude_label.to_lowercase() == label.name.to_lowercase())
        }) {
            return false;
        }

        if let Some(min_length) = self.min_body_length {
            if issue.body.as_ref().map_or(0, |b| b.len()) < min_length {
                return false;
            }
        }

        if let Some(min_comments) = self.min_comments {
            if issue.comments < min_comments {
                return false;
            }
        }

        if let Some(date_range) = &self.date_range {
            if let Some(start) = date_range.start {
                if issue.created_at < start {
                    return false;
                }
            }
            if let Some(end) = date_range.end {
                if issue.created_at > end {
                    return false;
                }
            }
        }

        let content =
            format!("{} {}", issue.title, issue.body.as_deref().unwrap_or("")).to_lowercase();

        if !self.required_keywords.is_empty() {
            if !self
                .required_keywords
                .iter()
                .any(|keyword| content.contains(&keyword.to_lowercase()))
            {
                return false;
            }
        }

        if self
            .excluded_keywords
            .iter()
            .any(|keyword| content.contains(&keyword.to_lowercase()))
        {
            return false;
        }

        if self.rust_errors_only {
            if !has_rust_error_codes(&content) {
                return false;
            }
        }

        if self.code_blocks_only {
            if !has_code_blocks(issue.body.as_deref().unwrap_or("")) {
                return false;
            }
        }

        true
    }
}

pub fn has_rust_error_codes(text: &str) -> bool {
    let error_regex = Regex::new(r"E0\d{3,4}").unwrap();
    error_regex.is_match(text)
}

pub fn has_code_blocks(text: &str) -> bool {
    if text.contains("```") {
        return true;
    }

    text.lines()
        .any(|line| line.starts_with("    ") && !line.trim().is_empty())
}

pub fn extract_error_codes(text: &str) -> Vec<String> {
    let error_regex = Regex::new(r"E0\d{3,4}").unwrap();
    error_regex
        .find_iter(text)
        .map(|m| m.as_str().to_string())
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rust_error_detection() {
        assert!(has_rust_error_codes("Error E0382: use of moved value"));
        assert!(has_rust_error_codes("Getting E0502 and E0499 errors"));
        assert!(!has_rust_error_codes("No errors here"));

        let codes = extract_error_codes("E0382 and E0502 errors occurred");
        assert_eq!(codes.len(), 2);
        assert!(codes.contains(&"E0382".to_string()));
        assert!(codes.contains(&"E0502".to_string()));
    }

    #[test]
    fn test_code_block_detection() {
        assert!(has_code_blocks("```rust\nfn main() {}\n```"));
        assert!(has_code_blocks("    let x = 5;\n    println!(\"{}\", x);"));
        assert!(!has_code_blocks("Just regular text without code"));
    }
}
