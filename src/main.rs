use clap::Parser;
use reqwest::Error;
use serde::Deserialize;
use url::Url;

#[derive(Parser)]
struct Args {
    /// The programming language of GitHub repositories
    #[clap(long = "language")]
    language: String,

    /// The amount of projects to be read from GitHub
    #[clap(long = "project_count")]
    project_count: u32,
}

#[derive(Deserialize, Debug)]
struct RepoInfo {
    full_name: String,
    contributors_url: String,
}

#[derive(Deserialize, Debug)]
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

#[tokio::main]
async fn main() -> Result<(), Error> {
    // Parse command line arguments
    let args = Args::parse();
    let language = args.language;
    let project_count = args.project_count;

    // Prepare URL for first request
    // let req_url = "https://api.github.com/search/repositories";
    let req_url: Url = "https://api.github.com/search/repositories"
        .parse()
        .unwrap();
    println!("Request URI: {}", req_url);

    // Get response from the server
    let client = reqwest::Client::new();
    let res = client
        .get(req_url.to_string())
        .query(&[
            ("q", format!("language:{}", language)),
            ("sort", "stars".to_string()),
            ("order", "desc".to_string()),
            // ("page", 34.to_string()),
        ])
        .header("Accept", "application/vnd.github.v3+json")
        .header("User-Agent", "styczen")
        .send()
        .await?;

    // Get "next" link from the headers by parsing lines separated by comma
    let headers = res.headers();
    let links = headers.get("link").unwrap();
    let links = links.to_str().unwrap();
    println!("Links:\n{:#?}", links);

    let next_link = get_next_link(links);
    match next_link {
        Some(link) => println!("Next link: {}", link),
        None => println!("There is no more \"next\" link"),
    }
    let r: ReposResponse = res.json().await?;
    // println!("JSON:\n{:#?}", r.items);

    // let links = res.headers().get("Link").unwrap();

    // res.read_to_string(&mut body).unwrap();

    // // let links = res.headers().get(reqwest::header::LINK).unwrap();

    // println!("Status: {}", res.status());
    // println!("Headers: {:#?}", res.headers());
    // println!("Body: {}", body);
    // println!("Links: {:#?}", links);

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
