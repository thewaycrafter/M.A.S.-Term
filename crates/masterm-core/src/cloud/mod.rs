//! Cloud synchronization using GitHub Gists

use anyhow::{Context, Result};
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, USER_AGENT, CONTENT_TYPE};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize)]
struct CreateGistRequest {
    description: String,
    public: bool,
    files: HashMap<String, GistFileContent>,
}

#[derive(Serialize)]
struct GistFileContent {
    content: String,
}

#[derive(Deserialize)]
struct GistResponse {
    html_url: String,
}

#[derive(Deserialize)]
struct GetGistResponse {
    files: HashMap<String, GetGistFile>,
}

#[derive(Deserialize)]
struct GetGistFile {
    content: String,
}

pub struct GistSync {
    client: reqwest::Client,
    token: Option<String>,
}

impl GistSync {
    pub fn new(token: Option<String>) -> Result<Self> {
        let mut headers = HeaderMap::new();
        headers.insert(USER_AGENT, HeaderValue::from_static("masterm-cli"));
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

        if let Some(t) = &token {
            let auth_value = format!("token {}", t);
            if let Ok(val) = HeaderValue::from_str(&auth_value) {
                headers.insert(AUTHORIZATION, val);
            }
        }

        let client = reqwest::Client::builder()
            .default_headers(headers)
            .build()
            .context("Failed to build HTTP client")?;

        Ok(Self { client, token })
    }

    pub async fn upload(&self, filename: &str, content: String, description: &str) -> Result<String> {
        if self.token.is_none() {
            return Err(anyhow::anyhow!("GitHub token required for upload"));
        }

        let mut files = HashMap::new();
        files.insert(
            filename.to_string(),
            GistFileContent { content },
        );

        let request = CreateGistRequest {
            description: description.to_string(),
            public: false, // Default to private
            files,
        };

        let response = self.client
            .post("https://api.github.com/gists")
            .json(&request)
            .send()
            .await
            .context("Failed to send Gist creation request")?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!("Gist creation failed: {}", error_text));
        }

        let gist: GistResponse = response
            .json()
            .await
            .context("Failed to parse Gist response")?;

        Ok(gist.html_url)
    }

    pub async fn download(&self, gist_id: &str, filename: &str) -> Result<String> {
        let url = format!("https://api.github.com/gists/{}", gist_id);
        
        let response = self.client
            .get(&url)
            .send()
            .await
            .context("Failed to fetch Gist")?;

        if !response.status().is_success() {
            return Err(anyhow::anyhow!("Failed to fetch Gist: {}", response.status()));
        }

        let gist: GetGistResponse = response
            .json()
            .await
            .context("Failed to parse Gist response")?;

        let file = gist.files.get(filename)
            .ok_or_else(|| anyhow::anyhow!("File '{}' not found in Gist", filename))?;

        Ok(file.content.clone())
    }
}
