mod github_api;

use futures::future::join_all;
use github_api::{Contributor, RepoInfo, ReposResponse};
use reqwest::header::{HeaderMap, HeaderValue};
use std::env;
use url::Url;

const AMOUNT_OF_CONTRIBUTORS: usize = 25;
const BUS_FACTOR_THRESHOLD: f32 = 0.75;

fn get_next_link(links: &str) -> Option<Url> {
    let next_line = links
        .split(',')
        .find(|line| match line.find(r#"rel="next""#) {
            Some(_) => true,
            None => false,
        })?;
    let addr_start = next_line.find('<')?;
    let addr_end = next_line.find('>')?;
    match Url::parse(next_line.get(addr_start + 1..addr_end)?) {
        Ok(url) => Some(url),
        Err(_) => None,
    }
}

fn extract_links_from_header_map(headers: &HeaderMap) -> Result<&str, Box<dyn std::error::Error>> {
    Ok(headers
        .get("link")
        .ok_or(r#"no "link" key in headers"#.to_string())?
        .to_str()?)
}

/// Function responsible for fetching contributors information
/// for specific repository.
///
/// This function panics when HTTP response has status code
/// different than 200 (OK)
pub async fn get_contributors(
    client: &reqwest::Client,
    repo: &RepoInfo,
    amount: usize,
) -> Result<Vec<Contributor>, Box<dyn std::error::Error>> {
    let url = Url::parse(&format!("{}?per_page={}", repo.contributors_url, amount))?;
    let url_res = client.get(url).send().await?;

    // This error should be handled appropriately
    let status_code = url_res.status();
    if status_code != reqwest::StatusCode::OK {
        let content = url_res.text().await?;
        panic!(
            "Func: get_contributors. Invalid status code ({}): {:?}",
            status_code, content
        );
    }

    let contributors: Vec<Contributor> = url_res.json().await?;
    Ok(contributors)
}

/// Function which is responsible for fetching repositories with most stars
/// in descending order. User specifies programming language and amount
/// of repositories to fetch from GitHub API.
///
/// This function panics when HTTP response has status code
/// different than 200 (OK)
pub async fn search_top_star_repos(
    client: &reqwest::Client,
    language: &str,
    project_count: usize,
) -> Result<Vec<RepoInfo>, Box<dyn std::error::Error>> {
    let mut req_url: Url = format!(
        "https://api.github.com/search/repositories?q=language:{}&sort=stars&order=desc",
        language
    )
    .parse()
    .unwrap();

    let mut loaded_projects: Vec<RepoInfo> = Vec::new();
    while loaded_projects.len() < project_count {
        // Get response from the server
        let res = client.get(req_url.to_string()).send().await?;

        // This error should be handled appropriately
        let status_code = res.status();
        if status_code != reqwest::StatusCode::OK {
            let content = res.text().await?;
            panic!(
                "Func: search_top_star_repos. Invalid status code ({}): {:?}",
                status_code, content
            );
        }

        // Get "next" link from the headers by parsing lines separated by comma
        let headers = res.headers();
        let links = extract_links_from_header_map(headers)?;

        // Getting "next" link here to avoid borrow move error
        let next_link = get_next_link(links);

        // Deserialize response content to JSON
        let r: ReposResponse = res.json().await?;

        // Check whether extending by all of new repositories increases
        // length of repos vector above set project_count
        if loaded_projects.len() + r.items.len() > project_count {
            let nr_elements_to_take = project_count - loaded_projects.len();
            for i in 0..nr_elements_to_take {
                loaded_projects.push(r.items[i].clone());
            }
        } else {
            loaded_projects.extend(r.items);
        }

        // Check next link and break loop if there is no more "next" link
        match next_link {
            Some(link) => req_url = link,
            None => {
                println!("There is no more \"next\" link");
                break;
            }
        }
    }

    Ok(loaded_projects)
}

/// Function responsible for fetching vector of repositories'
/// most active contributor and percentage of contributions
/// of the most active contributor.
pub async fn get_bus_factor(
    language: &str,
    project_count: usize,
) -> Result<Vec<(String, Contributor, f32)>, Box<dyn std::error::Error>> {
    // Initialize HTTP client
    let mut headers = HeaderMap::new();
    headers.insert(
        "accept",
        HeaderValue::from_static("application/vnd.github.v3+json"),
    );
    // Personal access token (GitHub API token read from environment variable GITHUB_ACCESS_TOKEN)
    let personal_access_token = format!("token {}", env::var("GITHUB_ACCESS_TOKEN")?);
    let authorization_header_key = HeaderValue::from_str(&personal_access_token)?;
    headers.insert("authorization", authorization_header_key);
    let client = reqwest::Client::builder()
        .user_agent("bus_factor")
        .default_headers(headers)
        .build()?;

    let loaded_repos = search_top_star_repos(&client, &language, project_count).await?;

    // Fetch most active 25 contributors for loaded repositories
    let contributors_per_repo = join_all(
        loaded_repos
            .iter()
            .map(|repo| get_contributors(&client, repo, AMOUNT_OF_CONTRIBUTORS)),
    )
    .await;

    // Filter out repositories which bus factor is equal to 1
    // and return tuple with most active contributor,
    // percentage of theirs contributions and name of the repo
    let result: Vec<(String, Contributor, f32)> = contributors_per_repo
        .iter()
        .enumerate()
        .filter_map(|(i, contributors)| match contributors {
            Ok(contributors) => {
                let all_contributions = contributors.iter().fold(0, |curr_sum, contributor| {
                    curr_sum + contributor.contributions
                });

                let percentage = contributors[0].contributions as f32 / all_contributions as f32;
                if percentage >= BUS_FACTOR_THRESHOLD {
                    return Some((
                        loaded_repos[i].name.clone(),
                        contributors[0].clone(),
                        percentage,
                    ));
                }

                None
            }
            Err(_) => None,
        })
        .collect();

    Ok(result)
}

#[cfg(test)]
mod tests {
    use crate::*;

    #[test]
    fn links_with_valid_next() {
        let links = "<https://api.github.com/search/repositories?q=language%3Arust&sort=stars&order=desc&page=2>; rel=\"next\", 
                          <https://api.github.com/search/repositories?q=language%3Arust&sort=stars&order=desc&page=34>; rel=\"last\"";
        assert_eq!(get_next_link(links),
                   Some(Url::parse("https://api.github.com/search/repositories?q=language%3Arust&sort=stars&order=desc&page=2").unwrap()));
    }

    #[test]
    fn links_empty_links() {
        assert_eq!(get_next_link(""), None);
    }

    #[test]
    fn links_all_rel_links() {
        let links = "<https://api.github.com/search/repositories?q=language%3Arust&sort=stars&order=desc&page=1>; rel=\"prev\", 
                          <https://api.github.com/search/repositories?q=language%3Arust&sort=stars&order=desc&page=3>; rel=\"next\", 
                          <https://api.github.com/search/repositories?q=language%3Arust&sort=stars&order=desc&page=34>; rel=\"last\", 
                          <https://api.github.com/search/repositories?q=language%3Arust&sort=stars&order=desc&page=1>; rel=\"first\"";
        assert_eq!(get_next_link(links), Some(Url::parse("https://api.github.com/search/repositories?q=language%3Arust&sort=stars&order=desc&page=3").unwrap()));
    }

    #[test]
    fn links_no_next_link() {
        let links = "<https://api.github.com/search/repositories?q=language%3Arust&sort=stars&order=desc&page=33>; rel=\"prev\", 
                          <https://api.github.com/search/repositories?q=language%3Arust&sort=stars&order=desc&page=1>; rel=\"first\"";
        assert_eq!(get_next_link(links), None);
    }

    #[test]
    fn links_empty_url() {
        let links = "<>; rel=\"next\", 
                          <https://api.github.com/search/repositories?q=language%3Arust&sort=stars&order=desc&page=34>; rel=\"last\"";
        assert_eq!(get_next_link(links), None);
    }
}
