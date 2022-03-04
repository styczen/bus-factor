use std::io::Read;

use clap::Parser;

#[derive(Parser)]
struct Args {
    /// The programming language of GitHub repositories
    #[clap(long = "language")]
    language: String,

    /// The amount of projects to be read from GitHub
    #[clap(long = "project_count")]
    project_count: u32,
}

fn main() {
    let args = Args::parse();
    let language = args.language;
    let project_count = args.project_count;

    // Prepare URL for first request
    let req_url = format!(
        "https://api.github.com/search/repositories?q=language:{}&sort=stars&order=desc",
        language
    );
    println!("Request URI: {}", req_url);

    let client = reqwest::blocking::Client::new();
    let mut res = client
        .get(req_url)
        .header("Accept", "application/vnd.github.v3+json")
        // .header("User-Agent", "styczen")
        // .header(
        //     "Authorization",
        //     "token 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",
        // )
        .bearer_auth("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")
        .send()
        .unwrap();

    // let mut res = reqwest::blocking::get(req_uri).unwrap();
    let mut body = String::new();
    res.read_to_string(&mut body).unwrap();

    // let links = res.headers().get(reqwest::header::LINK).unwrap();

    println!("Status: {}", res.status());
    println!("Headers: {:#?}", res.headers());
    // println!("Body: {}", body);
    // println!("Links: {:#?}", links);
}
