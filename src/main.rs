use clap::Parser;
use reqwest::Error;
use serde::Deserialize;

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

#[tokio::main]
async fn main() -> Result<(), Error> {
    let args = Args::parse();
    let language = args.language;
    let project_count = args.project_count;

    // Prepare URL for first request
    let req_url = format!(
        "https://api.github.com/search/repositories?q=language:{}&sort=stars&order=desc",
        language
    );
    println!("Request URI: {}", req_url);

    let client = reqwest::Client::new();
    let res = client
        .get(req_url)
        // .header("Accept", "application/vnd.github.v3+json")
        .header("User-Agent", "styczen")
        // .header(
        //     "Authorization",
        //     "Basic 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",
        // )
        // .bearer_auth("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")
        .send()
        .await?;

    // let content = res.text().await?;
    // println!("{}", content);

    let r: Vec<RepoInfo> = res.json().await?;
    println!("{:#?}", r);

    // res.read_to_string(&mut body).unwrap();

    // // let links = res.headers().get(reqwest::header::LINK).unwrap();

    // println!("Status: {}", res.status());
    // println!("Headers: {:#?}", res.headers());
    // println!("Body: {}", body);
    // println!("Links: {:#?}", links);

    Ok(())
}
