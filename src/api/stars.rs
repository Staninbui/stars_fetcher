///
/// Functions for handling stars
/// This module contains functions to star and unstar repositories.
///

use std::error::Error;
use crate::api::client::GitHubClient;
use reqwest::StatusCode;

pub trait Star {
    async fn star_repo(&self, owner: &str, repo: &str) -> Result<(), Box<dyn Error>>;
    async fn unstar_repo(&self, owner: &str, repo: &str) -> Result<(), Box<dyn Error>>;
    async fn is_starred(&self, owner: &str, repo: &str) -> Result<bool, Box<dyn Error>>;
}

impl Star for GitHubClient {
    async fn star_repo(&self, owner: &str, repo: &str) -> Result<(), Box<dyn Error>> {
        let url = format!("{}/user/starred/{}/{}", self.api_url, owner, repo);
        let response = self.client
            .put(&url)
            .bearer_auth(&self.token)
            .header("Content-Length", "0")
            .send()
            .await?;

        match response.status() {
            StatusCode::NO_CONTENT => Ok(()),
            StatusCode::OK => Ok(()),
            _ => Err(format!("Failed to star repository: {}",
                             response.text().await.unwrap_or_default()).into())
        }
    }

    async fn unstar_repo(&self, owner: &str, repo: &str) -> Result<(), Box<dyn Error>> {
        let url = format!("{}/user/starred/{}/{}", self.api_url, owner, repo);
        let response = self.client
            .delete(&url)
            .bearer_auth(&self.token)
            .send()
            .await?;

        match response.status() {
            StatusCode::NO_CONTENT => Ok(()),
            StatusCode::OK => Ok(()),
            _ => Err(format!("Failed to unstar repository: {}",
                             response.text().await.unwrap_or_default()).into())
        }
    }

    async fn is_starred(&self, owner: &str, repo: &str) -> Result<bool, Box<dyn Error>> {
        let url = format!("{}/user/starred/{}/{}", self.api_url, owner, repo);
        let response = self.client
            .get(&url)
            .bearer_auth(&self.token)
            .send()
            .await?;

        match response.status() {
            StatusCode::NO_CONTENT => Ok(true),
            StatusCode::NOT_FOUND => Ok(false),
            _ => Err(format!("Failed to check starred status: {}",
                             response.text().await.unwrap_or_default()).into())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockito::Server;

    #[tokio::test]
    async fn test_star_repo() {
        let mut server = Server::new_async().await;

        let mock = server
            .mock("PUT", "/user/starred/octocat/hello-world")
            .with_status(204)
            .create_async()
            .await;

        let client = GitHubClient::new(
            server.url().to_string(),
            "test_token".to_string()
        ).await;

        let result = client.star_repo("octocat", "hello-world").await;

        assert!(result.is_ok());
        mock.assert_async().await;
    }

    #[tokio::test]
    async fn test_unstar_repo() {
        let mut server = Server::new_async().await;

        let mock = server
            .mock("DELETE", "/user/starred/octocat/hello-world")
            .with_status(204)
            .create_async()
            .await;

        let client = GitHubClient::new(
            server.url().to_string(),
            "test_token".to_string()
        ).await;

        let result = client.unstar_repo("octocat", "hello-world").await;

        assert!(result.is_ok());
        mock.assert_async().await;
    }

    #[tokio::test]
    async fn test_is_starred_true() {
        let mut server = Server::new_async().await;

        let mock = server
            .mock("GET", "/user/starred/octocat/hello-world")
            .with_status(204)
            .create_async()
            .await;

        let client = GitHubClient::new(
            server.url().to_string(),
            "test_token".to_string()
        ).await;

        let result = client.is_starred("octocat", "hello-world").await;

        assert!(result.is_ok());
        assert!(result.unwrap());
        mock.assert_async().await;
    }

    #[tokio::test]
    async fn test_is_starred_false() {
        let mut server = Server::new_async().await;

        let mock = server
            .mock("GET", "/user/starred/octocat/hello-world")
            .with_status(404)
            .create_async()
            .await;

        let client = GitHubClient::new(
            server.url().to_string(),
            "test_token".to_string()
        ).await;

        let result = client.is_starred("octocat", "hello-world").await;

        assert!(result.is_ok());
        assert!(!result.unwrap());
        mock.assert_async().await;
    }
}