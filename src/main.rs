use clap::{App, Arg, SubCommand};
use dialoguer::{theme::ColorfulTheme, Select};
use prettytable::{Table, row, cell};
use reqwest::{Client, header};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::env;
use starts_fetcher::ui::selector::RepoSelector;
use serde_json::Value;

#[derive(Serialize, Deserialize, Debug)]
struct Repo {
    id: u64,
    name: String,
    full_name: String,
    description: Option<String>,
    html_url: String,
}

async fn get_repo(client: &Client, owner: &str, repo: &str) -> Result<Repo, Box<dyn Error>> {
    let url = format!("https://api.github.com/repos/{}/{}", owner, repo);
    let response = client.get(url).send().await?.json::<Repo>().await?;
    Ok(response)
}

async fn list_repos(client: &Client) -> Result<Vec<Repo>, Box<dyn Error>> {
    let url = "https://api.github.com/user/starred";
    let response = client.get(url).send().await?.json::<Vec<Repo>>().await?;
    Ok(response)
}

async fn star_repo(client: &Client, owner: &str, repo: &str) -> Result<(), Box<dyn Error>> {
    let url = format!("https://api.github.com/user/starred/{}/{}", owner, repo);
    client.put(url).send().await?;
    Ok(())
}

async fn unstar_repo(client: &Client, owner: &str, repo: &str) -> Result<(), Box<dyn Error>> {
    let url = format!("https://api.github.com/user/starred/{}/{}", owner, repo);
    client.delete(url).send().await?;
    Ok(())
}

async fn get_repo_detail(client: &Client, owner: &str, repo: &str) -> Result<Repo, Box<dyn Error>> {
    get_repo(client, owner, repo).await
}

// Convert Repo structs to Value for selector
async fn convert_repos_to_values(repos: Vec<Repo>) -> Vec<Value> {
    repos
        .into_iter()
        .map(|repo| serde_json::to_value(repo).unwrap_or_default())
        .collect()
}

// Display help information
fn show_help() {
    println!("GitHub CLI Tool - Commands:");
    println!("  get <owner> <repo>      - Fetch information about a repository");
    println!("  list                    - List all starred repositories");
    println!("  star <owner> <repo>     - Star a repository");
    println!("  unstar <owner> <repo>   - Unstar a repository");
    println!("  detail <owner> <repo>   - Get detailed information about a repository");
    println!("  --interactive           - Launch interactive mode with menu selection");
    println!("");
    println!("Example usage:");
    println!("  github-cli list");
    println!("  github-cli star octocat hello-world");
    println!("");
    println!("Note: GITHUB_TOKEN environment variable must be set");
}

// Interactive mode showing menu options
async fn interactive_mode(client: &Client) -> Result<(), Box<dyn Error>> {
    let items = vec![
        "List starred repositories",
        "Get repository details",
        "Star a repository",
        "Unstar a repository",
        "Exit",
    ];

    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Select action")
        .default(0)
        .items(&items)
        .interact()?;

    match selection {
        0 => {
            // List repositories
            let repos = list_repos(client).await?;
            println!("Found {} starred repositories", repos.len());

            // Convert to Value objects for the selector
            let repos_json = convert_repos_to_values(repos).await;

            if let Some(selected) = RepoSelector::select_repo(repos_json) {
                println!("\nSelected repository:");
                println!("Name: {}", selected["name"]);
                println!("Full name: {}", selected["full_name"]);
                println!("URL: {}", selected["html_url"]);
                if let Some(desc) = selected["description"].as_str() {
                    println!("Description: {}", desc);
                }
            }
        }
        1 => {
            // Get repository details (first list, then show details)
            let repos = list_repos(client).await?;
            let repos_json = convert_repos_to_values(repos).await;

            if let Some(selected) = RepoSelector::select_repo(repos_json) {
                let owner = selected["owner"]["login"].as_str().unwrap_or("unknown");
                let repo_name = selected["name"].as_str().unwrap_or("unknown");

                let repo_details = get_repo_detail(client, owner, repo_name).await?;
                let mut table = Table::new();
                table.add_row(row!["ID", "Name", "Full Name", "Description", "URL"]);
                table.add_row(row![
                    repo_details.id,
                    repo_details.name,
                    repo_details.full_name,
                    repo_details.description.unwrap_or_default(),
                    repo_details.html_url
                ]);
                table.printstd();
            }
        }
        2 => {
            // Star a repository - need manual input
            println!("Enter repository owner:");
            let mut owner = String::new();
            std::io::stdin().read_line(&mut owner)?;
            let owner = owner.trim();

            println!("Enter repository name:");
            let mut repo_name = String::new();
            std::io::stdin().read_line(&mut repo_name)?;
            let repo_name = repo_name.trim();

            star_repo(client, owner, repo_name).await?;
            println!("Starred repository {}/{}", owner, repo_name);
        }
        3 => {
            // Unstar a repository - select from currently starred
            let repos = list_repos(client).await?;
            let repos_json = convert_repos_to_values(repos).await;

            if let Some(selected) = RepoSelector::select_repo(repos_json) {
                let owner = selected["owner"]["login"].as_str().unwrap_or("unknown");
                let repo_name = selected["name"].as_str().unwrap_or("unknown");

                unstar_repo(client, owner, repo_name).await?;
                println!("Unstarred repository {}/{}", owner, repo_name);
            }
        }
        4 | _ => {
            println!("Exiting");
            return Ok(());
        }
    }

    // Recursively call interactive mode to keep the menu going
    Box::pin(interactive_mode(client)).await
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // If no arguments provided, show help
    if std::env::args().len() <= 1 {
        show_help();
        return Ok(());
    }

    let github_token = match env::var("GITHUB_TOKEN") {
        Ok(token) => token,
        Err(_) => {
            eprintln!("Error: GITHUB_TOKEN environment variable must be set");
            return Ok(());
        }
    };

    let mut headers = header::HeaderMap::new();
    headers.insert(header::USER_AGENT, header::HeaderValue::from_static("reqwest"));
    headers.insert(header::AUTHORIZATION, header::HeaderValue::from_str(&format!("token {}", github_token))?);

    let client = Client::builder()
        .default_headers(headers)
        .build()?;

    let app = App::new("GitHub CLI")
        .version("1.0")
        .author("Your Name <your.email@example.com>")
        .about("CLI tool to interact with GitHub")
        .subcommand(SubCommand::with_name("get")
            .about("Fetch a repository")
            .arg(Arg::with_name("owner")
                .help("Owner of the repository")
                .required(true)
                .index(1))
            .arg(Arg::with_name("repo")
                .help("Name of the repository")
                .required(true)
                .index(2)))
        .subcommand(SubCommand::with_name("list")
            .about("List all starred repositories"))
        .subcommand(SubCommand::with_name("star")
            .about("Star a repository")
            .arg(Arg::with_name("owner")
                .help("Owner of the repository")
                .required(true)
                .index(1))
            .arg(Arg::with_name("repo")
                .help("Name of the repository")
                .required(true)
                .index(2)))
        .subcommand(SubCommand::with_name("unstar")
            .about("Unstar a repository")
            .arg(Arg::with_name("owner")
                .help("Owner of the repository")
                .required(true)
                .index(1))
            .arg(Arg::with_name("repo")
                .help("Name of the repository")
                .required(true)
                .index(2)))
        .subcommand(SubCommand::with_name("detail")
            .about("Get repository details")
            .arg(Arg::with_name("owner")
                .help("Owner of the repository")
                .required(true)
                .index(1))
            .arg(Arg::with_name("repo")
                .help("Name of the repository")
                .required(true)
                .index(2)))
        .arg(Arg::with_name("interactive")
            .long("interactive")
            .help("Start interactive mode"))
        .get_matches();

    // Check if --interactive flag is used
    if app.is_present("interactive") {
        return interactive_mode(&client).await;
    }

    match app.subcommand() {
        Some(("get", sub_m)) => {
            let owner = sub_m.value_of("owner").unwrap();
            let repo = sub_m.value_of("repo").unwrap();
            let repo = get_repo(&client, owner, repo).await?;
            let mut table = Table::new();
            table.add_row(row!["ID", "Name", "Full Name", "Description", "URL"]);
            table.add_row(row![
                repo.id,
                repo.name,
                repo.full_name,
                repo.description.unwrap_or_default(),
                repo.html_url
            ]);
            table.printstd();
        }
        Some(("list", _)) => {
            let repos = list_repos(&client).await?;
            let mut table = Table::new();
            table.add_row(row!["ID", "Name", "Full Name", "Description", "URL"]);
            for repo in repos {
                table.add_row(row![
                    repo.id,
                    repo.name,
                    repo.full_name,
                    repo.description.unwrap_or_default(),
                    repo.html_url
                ]);
            }
            table.printstd();
        }
        Some(("star", sub_m)) => {
            let owner = sub_m.value_of("owner").unwrap();
            let repo = sub_m.value_of("repo").unwrap();
            star_repo(&client, owner, repo).await?;
            println!("Starred repository {}/{}", owner, repo);
        }
        Some(("unstar", sub_m)) => {
            let owner = sub_m.value_of("owner").unwrap();
            let repo = sub_m.value_of("repo").unwrap();
            unstar_repo(&client, owner, repo).await?;
            println!("Unstarred repository {}/{}", owner, repo);
        }
        Some(("detail", sub_m)) => {
            let owner = sub_m.value_of("owner").unwrap();
            let repo = sub_m.value_of("repo").unwrap();
            let repo = get_repo_detail(&client, owner, repo).await?;
            let mut table = Table::new();
            table.add_row(row!["ID", "Name", "Full Name", "Description", "URL"]);
            table.add_row(row![
                repo.id,
                repo.name,
                repo.full_name,
                repo.description.unwrap_or_default(),
                repo.html_url
            ]);
            table.printstd();
        }
        _ => {
            // No matching subcommand, show help
            show_help();
        }
    }

    Ok(())
}