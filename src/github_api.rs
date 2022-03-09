use serde::Deserialize;

#[derive(Deserialize, Clone, Debug)]
pub struct RepoInfo {
    /// Name of the respository.
    pub name: String,
    /// URL address with all contributors.
    pub contributors_url: String,
}

#[derive(Deserialize)]
pub struct ReposResponse {
    /// Part of repositories response with all information.
    pub items: Vec<RepoInfo>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Contributor {
    /// Username of contributor on GitHub.
    pub login: String,
    /// Amount of contributions to specific respository for this user.
    pub contributions: usize,
}
