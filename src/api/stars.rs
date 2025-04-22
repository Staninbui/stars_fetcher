///
/// Functions for handling stars
/// This module contains functions to star and unstar repositories.
///

use std::error::Error;

trait Star {
    fn star_repo(&self, owner: &str, repo: &str) -> Result<(), Box<dyn Error>>;
    fn unstar_repo(&self, owner: &str, repo: &str) -> Result<(), Box<dyn Error>>;
}