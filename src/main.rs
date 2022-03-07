use reqwest::{Error, header::HeaderMap};
use serde::Deserialize;
use url::Url;
use clap::Parser;

#[derive(Parser)]
pub struct ProgramOptions {
    /// Name of the programming language of GitHub repositories
    #[clap(long = "language")]
    pub language: String,

    /// The amount of projects to be read from GitHub
    #[clap(long = "project_count")]
    pub project_count: usize,
}

#[derive(Clone, Deserialize, Debug)]
struct RepoInfo {
    full_name: String,
    contributors_url: String,
}

#[derive(Clone, Deserialize, Debug)]
struct ReposResponse {
    items: Vec<RepoInfo>,
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

fn extract_links_from_header_map(headers: &HeaderMap) -> Option<&str> {
    let links = headers.get("link")?;
    match links.to_str() {
        Ok(link) => Some(link),
        Err(_) => None,
    }
}

fn get_contributors(client: &reqwest::Client) {
    
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    // Parse command line arguments
    let args = ProgramOptions::parse();
    let language = args.language;
    let project_count = args.project_count;

    let client = reqwest::Client::new();

    let mut loaded_projects_cnt: usize = 0;
    let mut req_url: Url = "https://api.github.com/search/repositories".parse().unwrap();
    while loaded_projects_cnt <= project_count {
        // Prepare URL for first request
        println!("Request URI: {}", req_url);

        // Get response from the server
        let res = client
            .get(req_url.to_string())
            .query(&[
                ("q", format!("language:{}", language)),
                ("sort", "stars".to_string()),
                ("order", "desc".to_string()),
                // ("per_page", 250.to_string()),
            ])
            .header("Accept", "application/vnd.github.v3+json")
            .header("User-Agent", "styczen")
            .send()
            .await?;

        // Get "next" link from the headers by parsing lines separated by comma
        let headers = res.headers();
        let links = extract_links_from_header_map(headers).unwrap();
        // println!("Links:\n{:#?}", links);
        
        let next_link = get_next_link(links);
        
        let mut r: ReposResponse = res.json().await?;
        println!("Len: {:#?}", r.items.len());
        
        for item in r.items {
            let url_res = client
            .get(item.contributors_url)
            .header("Accept", "application/vnd.github.v3+json")
            .header("User-Agent", "styczen")
            .send()
            .await?;

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
