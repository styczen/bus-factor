use clap::Parser;
// use futures::executor::block_on;
use http::{Request, Response};

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

    // Prepare URI for first request 
    let req = format!(
        "https://api.github.com/search/repositories?q=language:{}&sort=stars&order=desc",
        language
    );
    println!("{}", req);

    // let mut request = Request::builder()
    //     .uri("https://api.github.com/search/repositories?q=language:rust&sort=stars&order=desc")
    //     .header("Accept", "application/vnd.github.v3+json");

    // let request = Request::get(
    //     "https://api.github.com/search/repositories?q=language:rust&sort=stars&order=desc",
    // )
    // .body(())
    // .unwrap();
}
