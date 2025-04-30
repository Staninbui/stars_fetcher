use dialoguer::{theme::ColorfulTheme, Select, MultiSelect};
use std::fmt::Display;

/// A simple wrapper for repository data to display in selector
pub struct RepoDisplayItem {
    pub id: u64,
    pub name: String,
    pub owner: String,
    pub description: Option<String>,
    pub html_url: String,

    // Store the original repo to return it later
    repo: serde_json::Value,
}

impl RepoDisplayItem {
    /// Create a new RepoDisplayItem from a repository JSON
    pub fn from_repo(repo: serde_json::Value) -> Option<Self> {
        let id = repo.get("id")?.as_u64()?;
        let name = repo.get("name")?.as_str()?.to_string();
        let owner = repo.get("owner")?.get("login")?.as_str()?.to_string();
        let description = repo.get("description").and_then(|d| d.as_str()).map(|s| s.to_string());
        let html_url = repo.get("html_url")?.as_str()?.to_string();

        Some(Self {
            id,
            name,
            owner,
            description,
            html_url,
            repo,
        })
    }

    /// Get the original repo data
    pub fn into_repo(self) -> serde_json::Value {
        self.repo
    }
    
    /// Get the repository ID
    pub fn repo(&self) -> serde_json::Value {
        self.repo.clone()
    }
}

impl Display for RepoDisplayItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}/{}: {}",
            self.owner,
            self.name,
            self.description.as_deref().unwrap_or("No description")
        )
    }
}

/// A utility for displaying and selecting repositories in an interactive terminal UI
pub struct RepoSelector;

impl RepoSelector {
    /// Display a list of repositories and allow the user to select one
    pub fn select_repo(repos: Vec<serde_json::Value>) -> Option<serde_json::Value> {
        if repos.is_empty() {
            println!("No repositories to display.");
            return None;
        }

        // Convert to display items
        let display_items: Vec<RepoDisplayItem> = repos
            .into_iter()
            .filter_map(RepoDisplayItem::from_repo)
            .collect();

        if display_items.is_empty() {
            println!("Failed to parse repository data.");
            return None;
        }

        // Display selection dialog
        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Select a repository")
            .items(&display_items)
            .default(0)
            .interact_opt()
            .unwrap_or(None);

        // Use non-consuming repo() method
        selection.map(|index| display_items[index].repo())
    }

    /// Display a list of repositories and allow the user to select multiple
    pub fn select_multiple_repos(repos: Vec<serde_json::Value>) -> Vec<serde_json::Value> {
        if repos.is_empty() {
            println!("No repositories to display.");
            return Vec::new();
        }

        // Convert to display items
        let display_items: Vec<RepoDisplayItem> = repos
            .into_iter()
            .filter_map(RepoDisplayItem::from_repo)
            .collect();

        if display_items.is_empty() {
            println!("Failed to parse repository data.");
            return Vec::new();
        }

        // Display multi-selection dialog
        let selection = MultiSelect::with_theme(&ColorfulTheme::default())
            .with_prompt("Select repositories (space to select, enter to confirm)")
            .items(&display_items)
            .interact_opt()
            .unwrap_or(None);

        // Use non-consuming repo() method
        match selection {
            Some(indices) => indices
                .into_iter()
                .map(|i| display_items[i].repo())
                .collect(),
            None => Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    // Create test repo data
    fn create_test_repos() -> Vec<serde_json::Value> {
        vec![
            json!({
                "id": 1,
                "name": "repo1",
                "owner": {
                    "login": "user1"
                },
                "description": "Description for repo1",
                "html_url": "https://github.com/user1/repo1"
            }),
            json!({
                "id": 2,
                "name": "repo2",
                "owner": {
                    "login": "user2"
                },
                "description": "Description for repo2",
                "html_url": "https://github.com/user2/repo2"
            }),
        ]
    }

    #[test]
    #[ignore = "requires user interaction"]
    fn test_select_repo() {
        let repos = create_test_repos();
        let _selected = RepoSelector::select_repo(repos);
    }

    #[test]
    #[ignore = "requires user interaction"]
    fn test_select_multiple_repos() {
        let repos = create_test_repos();
        let _selected = RepoSelector::select_multiple_repos(repos);
    }

    #[test]
    fn test_empty_repos() {
        let empty_repos: Vec<serde_json::Value> = vec![];
        assert!(RepoSelector::select_repo(empty_repos.clone()).is_none());
        assert!(RepoSelector::select_multiple_repos(empty_repos).is_empty());
    }

    #[test]
    fn test_repo_display_item() {
        let repo = json!({
            "id": 1,
            "name": "test-repo",
            "owner": {
                "login": "test-user"
            },
            "description": "Test description",
            "html_url": "https://github.com/test-user/test-repo"
        });

        let item = RepoDisplayItem::from_repo(repo.clone()).unwrap();
        assert_eq!(item.id, 1);
        assert_eq!(item.name, "test-repo");
        assert_eq!(item.owner, "test-user");
        assert_eq!(item.description, Some("Test description".to_string()));
        assert_eq!(item.html_url, "https://github.com/test-user/test-repo");

        // Test display formatting
        assert_eq!(format!("{}", item), "test-user/test-repo: Test description");

        // Test repo conversion
        assert_eq!(item.into_repo(), repo);
    }
}