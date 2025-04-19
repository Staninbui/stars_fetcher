use std::env;
use std::fs;
use std::error::Error;
use dirs;
use serde::{Deserialize, Serialize};
use toml;

// Config struct to hold the configuration
#[derive(Deserialize, Serialize, Debug)]
pub struct Config {
    pub github: GithubConfig,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct GithubConfig {
    pub token: String,
    pub email: String,
    pub api_url: String,
}

impl Config {
    // new function to create a new Config instance
    pub fn new() -> Result<Self, Box<dyn Error>> {
        match Self::load_from_file() {
            Ok(config) => {
                if config.github.token.is_empty() {
                    if let Ok(token) = env::var("GITHUB_TOKEN") {
                        let mut config = config;
                        config.github.token = token;

                        Ok(config)
                    } else {
                        Ok(config)
                    }
                } else {
                    Ok(config)
                }
            },
            Err(_) => {
                Self::create_default_config()
            }
        }
    }

    // load_from_file function to read the configuration from a file
    fn load_from_file() -> Result<Self, Box<dyn Error>> {
        let config_path = dirs::config_dir()
            .ok_or("Unable to find config directory")?
            .join("stars_fetcher");

        let config_file = config_path.join("config.toml");

        if config_file.exists() {
            let contents = fs::read_to_string(config_file).unwrap();
            let config: Config = toml::de::from_str(&contents)?;

            Ok(config)
        } else {
            Err("Config file not found".into())
        }
    }

    fn create_default_config() -> Result<Self, Box<dyn Error>> {
        let token = env::var("GITHUB_TOKEN").unwrap_or_default();
        let config = Config {
            github: GithubConfig {
                token,
                email: String::new(),
                api_url: String::from("https://api.github.com"),
            }
        };

        if let Some(config_dir) = dirs::config_dir() {
            let app_config_path = config_dir.join("stars_fetcher");
            fs::create_dir_all(&app_config_path)?;

            let config_file = app_config_path.join("config.toml");
            let toml_string = toml::to_string(&config)?;
            fs::write(config_file, toml_string)?;
        }

        Ok(config)
    }
}

// tests
#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;
    use std::env;

    fn get_test_config_path() -> PathBuf {
        dirs::config_dir()
            .unwrap()
            .join("stars_fetcher")
            .join("config.toml")
    }

    fn clean_test_config() {
        let path = get_test_config_path();
        if path.exists() {
            fs::remove_file(path).unwrap();
        }
    }

    #[test]
    fn test_new_creates_default_config() {
        clean_test_config();
        env::remove_var("GITHUB_TOKEN");

        let config = Config::new().unwrap();
        assert_eq!(config.github.token, "");
        assert_eq!(config.github.email, "");
        assert_eq!(config.github.api_url, "https://api.github.com");
        assert!(get_test_config_path().exists());

        clean_test_config();
    }

    #[test]
    fn test_new_uses_environment_variable() {
        clean_test_config();

        env::set_var("GITHUB_TOKEN", "test_token");
        let config = Config::new().unwrap();
        assert_eq!(config.github.token, "test_token");

        clean_test_config();
        env::remove_var("GITHUB_TOKEN");
    }

    #[test]
    fn test_new_loads_existing_config() {
        clean_test_config();

        let config_dir = dirs::config_dir().unwrap().join("stars_fetcher");
        fs::create_dir_all(&config_dir).unwrap();
        let test_config = r#"
[github]
token = "existing_token"
email = "test@example.com"
api_url = "https://test-api.github.com"
"#;
        fs::write(get_test_config_path(), test_config).unwrap();
        env::remove_var("GITHUB_TOKEN");
        let config = Config::new().unwrap();
        assert_eq!(config.github.token, "existing_token");
        assert_eq!(config.github.email, "test@example.com");
        assert_eq!(config.github.api_url, "https://test-api.github.com");

        clean_test_config();
    }

    #[test]
    fn test_env_var_overrides_empty_token() {
        clean_test_config();

        let config_dir = dirs::config_dir().unwrap().join("stars_fetcher");
        fs::create_dir_all(&config_dir).unwrap();

        let test_config = r#"
[github]
token = ""
email = "test@example.com"
api_url = "https://test-api.github.com"
"#;
        fs::write(get_test_config_path(), test_config).unwrap();
        env::set_var("GITHUB_TOKEN", "test_token");
        let config = Config::new().unwrap();
        assert_eq!(config.github.token, "test_token");
        assert_eq!(config.github.email, "test@example.com");

        clean_test_config();
        env::remove_var("GITHUB_TOKEN");
    }
}