///
/// This module contains the client for the GitHub API.
///

use crate::config::Config;
use reqwest::{Client, ClientBuilder};
use std::error::Error;
use std::time::Duration;

pub struct GitHubClient {
    pub(crate) client: Client,
    pub api_url: String,
    pub token: String,
}

impl GitHubClient {
    fn create_http_client() -> Client {
        ClientBuilder::new()
            .timeout(Duration::from_secs(30))
            .user_agent("stars-fetcher")
            .build()
            .expect("Failed to create HTTP client")
    }

    pub async fn new(api_url: String, token: String) -> Self {
        let client = Self::create_http_client();
        Self {
            client,
            api_url,
            token
        }
    }

    async fn validate_auth(&self) -> Result<bool, Box<dyn Error>> {
        let url = format!("{}/user", self.api_url);
        let response = self.client
            .get(&url)
            .bearer_auth(&self.token)
            .send()
            .await?;

        Ok(response.status().is_success())
    }

    pub async fn from_config(config: &Config) -> Result<Self, Box<dyn Error>> {
        let api_url = config.github.api_url.clone();
        let token = config.github.token.clone();

        if api_url.is_empty() {
            return Err("API URL is empty".into());
        }
        if token.is_empty() {
            return Err("GitHub API token is empty".into());
        }

        Ok(Self::new(api_url, token).await)
    }

    pub async fn new_validated(config: &Config) -> Result<Self, Box<dyn Error>> {
        let client = Self::from_config(config).await?;

        if !client.validate_auth().await? {
            return Err("Invalid GitHub API token".into());
        }

        Ok(client)
    }
}

pub async fn validate_github_config() -> Result<(), Box<dyn Error>> {
    let config = Config::new()?;
    // Create client without validation first
    let api_url = config.github.api_url.clone();
    let token = config.github.token.clone();

    if api_url.is_empty() || token.is_empty() {
        return Err("GitHub configuration is incomplete".into());
    }

    let client = GitHubClient::new(api_url, token).await;

    match client.validate_auth().await {
        Ok(true) => {
            println!("GitHub API authentication successful");
            Ok(())
        }
        Ok(false) => {
            println!("GitHub API authentication failed");
            Err("Invalid GitHub API token".into())
        }
        Err(e) => {
            println!("Error validating GitHub API token: {}", e);
            Err(e)
        }
    }
}