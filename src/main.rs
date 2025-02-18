use clap::{App, Arg, SubCommand};
use prettytable::{Table, row, cell};
use reqwest::{Client, header};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::env;

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

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let github_token = env::var("GITHUB_TOKEN").expect("GITHUB_TOKEN must be set");
    let mut headers = header::HeaderMap::new();
    headers.insert(header::USER_AGENT, header::HeaderValue::from_static("reqwest"));
    headers.insert(header::AUTHORIZATION, header::HeaderValue::from_str(&format!("token {}", github_token))?);

    let client = Client::builder()
        .default_headers(headers)
        .build()?;

    let matches = App::new("GitHub CLI")
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
        .get_matches();

    match matches.subcommand() {
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
        _ => {}
    }

    Ok(())
}