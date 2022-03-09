use clap::Parser;
use reqwest::header::{HeaderMap, HeaderValue};
use serde::Deserialize;
use url::Url;
use futures::future::join_all;

const AMOUNT_OF_CONTRIBUTORS: usize = 25;
const BUS_FACTOR_THRESHOLD: f32 = 0.75;

#[derive(Parser)]
pub struct ProgramOptions {
    /// Name of the programming language of GitHub repositories
    #[clap(long = "language")]
    pub language: String,

    /// The amount of projects to be read from GitHub
    #[clap(long = "project_count")]
    pub project_count: usize,
}

#[derive(Deserialize, Clone, Debug)]
struct RepoInfo {
    full_name: String,
    contributors_url: String,
}

#[derive(Deserialize)]
struct ReposResponse {
    items: Vec<RepoInfo>,
}

#[derive(Deserialize, Debug, Clone)]
struct Contributor {
    login: String,
    contributions: usize,
    // repository_name: String,
}

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

// enum Error
async fn get_contributors(
    client: &reqwest::Client,
    repo: &RepoInfo,
    amount: usize
) -> Result<Vec<Contributor>, Box<dyn std::error::Error>> {
    let url = Url::parse(&format!("{}?per_page={}", repo.contributors_url, amount))?;
    // println!("contributors_url: {}", url);
    let url_res = client.get(url).send().await?;
    let contributors: Vec<Contributor> = url_res.json().await?;
    Ok(contributors)
}

async fn search_top_star_repos(
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
        // println!("Request URI: {}", req_url);

        // Get response from the server
        let res = client.get(req_url.to_string()).send().await?;

        // Get "next" link from the headers by parsing lines separated by comma
        let headers = res.headers();
        let links = extract_links_from_header_map(headers)?;

        // Getting "next" link here to avoid borrow move error
        let next_link = get_next_link(links);

        let r: ReposResponse = res.json().await?;
        // println!("Response items Len: {:#?}", r.items.len());

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

async fn get_bus_factor(language: &str, project_count: usize) -> Result<Vec<(Contributor, f32)>, Box<dyn std::error::Error>> {
    // Initialize HTTP client
    let mut headers = HeaderMap::new();
    headers.insert(
        "Accept",
        HeaderValue::from_static("application/vnd.github.v3+json"),
    );
    let client = reqwest::Client::builder()
        .user_agent("styczen") // TODO: This has to be removed and changed to token
        .default_headers(headers)
        .build()?;

    let loaded_repos = search_top_star_repos(&client, &language, project_count).await?;
    println!("loaded_repos len: {}", loaded_repos.len());

    // Fetch most active 25 contributors for loaded repositories
    let contributors_per_repo = join_all(
        loaded_repos
            .iter()
            .map(|repo| get_contributors(&client, repo, AMOUNT_OF_CONTRIBUTORS))
        ).await;

    println!("contributors_per_repo len: {}", contributors_per_repo.len());
    // println!("contributors_per_repo: {:#?}", contributors_per_repo);
    println!("BUS_FACTOR_THRESHOLD: {}", BUS_FACTOR_THRESHOLD);

    // Filter out repositories which bus factor is equal to 1
    // and return tuple with most active contributor, 
    // percentage of theirs contributions and name of the repo 
    let result: Vec<(Contributor, f32)> = contributors_per_repo
        .iter()
        .filter_map(|contributors| {
            match contributors {
                Ok(contributors) => {
                    let all_contributions = contributors
                        .iter()
                        .fold(0, |curr_sum, contributor| curr_sum + contributor.contributions);
                    
                    // println!("most active: {}, all: {}", contributors[0].contributions, all_contributions);
                    let percentage = contributors[0].contributions as f32 / all_contributions as f32;
                    if percentage >= BUS_FACTOR_THRESHOLD {
                        return Some((contributors[0].clone(), percentage));
                    }

                    None
                },
                Err(_) => None
            }
        })
        .collect();

    println!("result len: {}", result.len());

    Ok(result)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Parse command line arguments
    let args = ProgramOptions::parse();
    let language = args.language;
    let project_count = args.project_count;

    let result = get_bus_factor(&language, project_count).await?;

    println!("{}", result.len());
    // Print result to console
    result.iter().for_each(|r| println!("project: {:<20} user: {:<20} percentage: {}", "PROJECT", r.0.login, r.1));
    Ok(())
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
