///
/// Functions to interact with the GitHub API for repositories
/// This module contains functions to get, list, get details of repositories, star, and unstar repositories.
///

use std::error::Error;
use crate::api::client::GitHubClient;
use serde::{Deserialize, Serialize};
use reqwest::StatusCode;

pub trait Repo {
    async fn get_repo(&self, owner: &str, repo: &str) -> Result<RepoResponse, Box<dyn Error>>;
    async fn list_repos(&self) -> Result<Vec<RepoResponse>, Box<dyn Error>>;
    async fn get_repo_details(&self, owner: &str, repo: &str) -> Result<RepoDetailsResponse, Box<dyn Error>>;
}

#[derive(Debug, Deserialize, Serialize)]
struct RepoResponse {
    pub id: u64,
    pub name: String,
    pub owner: OwnerResponse,
    #[serde(rename = "stargazers_count")]
    pub stars: u64,
}

#[derive(Debug, Deserialize, Serialize)]
struct OwnerResponse {
    pub login: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct RepoDetailsResponse {
    pub id: u64,
    pub name: String,
    pub owner: OwnerResponse,  // Changed from String to OwnerResponse
    #[serde(rename = "stargazers_count")]
    pub stars: u64,
    pub description: Option<String>,
    pub html_url: String,
}

impl Repo for GitHubClient {
    async fn get_repo(&self, owner: &str, repo: &str) -> Result<RepoResponse, Box<dyn Error>> {
        let url = format!("{}/repos/{}/{}", self.api_url, owner, repo);
        let response = self.client
            .get(&url)
            .bearer_auth(&self.token)
            .send()
            .await?;

        if response.status() == StatusCode::OK {
            let repo_response = response.json::<RepoResponse>().await?;
            Ok(repo_response)
        } else {
            Err("Failed to fetch repository".into())
        }
    }

    async fn list_repos(&self) -> Result<Vec<RepoResponse>, Box<dyn Error>> {
        let url = format!("{}/user/starred", self.api_url);
        let response = self.client
            .get(&url)
            .bearer_auth(&self.token)
            .send()
            .await?;

        if response.status() == StatusCode::OK {
            let repos = response.json::<Vec<RepoResponse>>().await?;
            Ok(repos)
        } else {
            Err("Failed to list repositories".into())
        }
    }

    async fn get_repo_details(&self, owner: &str, repo: &str) -> Result<RepoDetailsResponse, Box<dyn Error>> {
        let url = format!("{}/repos/{}/{}", self.api_url, owner, repo);
        let response = self.client
            .get(&url)
            .bearer_auth(&self.token)
            .send()
            .await?;

        if response.status() == StatusCode::OK {
            let repo_details = response.json::<RepoDetailsResponse>().await?;
            Ok(repo_details)
        } else {
            Err("Failed to fetch repository details".into())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use mockito::Server;

    #[tokio::test]
    async fn test_get_repo() {
        // Create a new mock server
        let mut server = Server::new_async().await;

        // Create the mock endpoint
        let mock = server
            .mock("GET", "/repos/octocat/hello-world")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(json!({
                "id": 1296269,
                "name": "hello-world",
                "owner": {
                    "login": "octocat"
                },
                "stargazers_count": 80
            }).to_string())
            .create_async()
            .await;

        // Use the server's URL
        let client = GitHubClient::new(
            server.url().to_string(),
            "test_token".to_string()
        ).await;

        let result = client.get_repo("octocat", "hello-world").await;

        assert!(result.is_ok());
        let repo = result.unwrap();
        assert_eq!(repo.id, 1296269);
        assert_eq!(repo.name, "hello-world");
        assert_eq!(repo.owner.login, "octocat");
        assert_eq!(repo.stars, 80);

        // Verify that the mock was called
        mock.assert_async().await;
    }

    #[tokio::test]
    async fn test_get_repo_not_found() {
        let mut server = Server::new_async().await;

        let mock = server
            .mock("GET", "/repos/octocat/not-found")
            .with_status(404)
            .create_async()
            .await;

        let client = GitHubClient::new(
            server.url().to_string(),
            "test_token".to_string()
        ).await;

        let result = client.get_repo("octocat", "not-found").await;

        assert!(result.is_err());
        mock.assert_async().await;
    }

    #[tokio::test]
    async fn test_list_repos() {
        let mut server = Server::new_async().await;

        let mock = server
            .mock("GET", "/user/starred")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(json!([
                {
                    "id": 1,
                    "name": "repo1",
                    "owner": {
                        "login": "user1"
                    },
                    "stargazers_count": 10
                },
                {
                    "id": 2,
                    "name": "repo2",
                    "owner": {
                        "login": "user2"
                    },
                    "stargazers_count": 20
                }
            ]).to_string())
            .create_async()
            .await;

        let client = GitHubClient::new(
            server.url().to_string(),
            "test_token".to_string()
        ).await;

        let result = client.list_repos().await;

        assert!(result.is_ok());
        let repos = result.unwrap();
        assert_eq!(repos.len(), 2);
        assert_eq!(repos[0].id, 1);
        assert_eq!(repos[0].name, "repo1");
        assert_eq!(repos[0].owner.login, "user1");
        assert_eq!(repos[0].stars, 10);
        assert_eq!(repos[1].id, 2);
        assert_eq!(repos[1].name, "repo2");
        assert_eq!(repos[1].stars, 20);

        mock.assert_async().await;
    }

    #[tokio::test]
    async fn test_list_repos_error() {
        let mut server = Server::new_async().await;

        let mock = server
            .mock("GET", "/user/starred")
            .with_status(401)
            .create_async()
            .await;

        let client = GitHubClient::new(
            server.url().to_string(),
            "invalid_token".to_string()
        ).await;

        let result = client.list_repos().await;

        assert!(result.is_err());
        mock.assert_async().await;
    }

    #[tokio::test]
    async fn test_get_repo_details() {
        let mut server = Server::new_async().await;

        let mock = server
            .mock("GET", "/repos/octocat/hello-world")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(json!({
                "id": 1296269,
                "name": "hello-world",
                "owner": {
                    "login": "octocat"
                },
                "stargazers_count": 80,
                "description": "My first repository",
                "html_url": "https://github.com/octocat/hello-world"
            }).to_string())
            .create_async()
            .await;

        let client = GitHubClient::new(
            server.url().to_string(),
            "test_token".to_string()
        ).await;

        let result = client.get_repo_details("octocat", "hello-world").await;

        assert!(result.is_ok());
        let details = result.unwrap();
        assert_eq!(details.id, 1296269);
        assert_eq!(details.name, "hello-world");
        assert_eq!(details.owner.login, "octocat");
        assert_eq!(details.stars, 80);
        assert_eq!(details.description, Some("My first repository".to_string()));
        assert_eq!(details.html_url, "https://github.com/octocat/hello-world");

        mock.assert_async().await;
    }

    #[tokio::test]
    async fn test_get_repo_details_not_found() {
        let mut server = Server::new_async().await;

        let mock = server
            .mock("GET", "/repos/octocat/not-found")
            .with_status(404)
            .create_async()
            .await;

        let client = GitHubClient::new(
            server.url().to_string(),
            "test_token".to_string()
        ).await;

        let result = client.get_repo_details("octocat", "not-found").await;

        assert!(result.is_err());
        mock.assert_async().await;
    }
}