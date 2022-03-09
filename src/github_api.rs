use serde::Deserialize;

/// Single repository information with name and all contributors URL.
#[derive(Deserialize, Clone, Debug)]
pub struct RepoInfo {
    /// Name of the respository.
    pub name: String,
    /// URL address with all contributors.
    pub contributors_url: String,
}

/// All repositories which match search query.
#[derive(Deserialize)]
pub struct ReposResponse {
    /// Part of repositories response with all information.
    pub items: Vec<RepoInfo>,
}

/// Contributor information like login (username) and amount
/// of contributions for specific repository.
#[derive(Deserialize, Debug, Clone)]
pub struct Contributor {
    /// Username of contributor on GitHub.
    pub login: String,
    /// Amount of contributions to specific respository for this user.
    pub contributions: usize,
}
