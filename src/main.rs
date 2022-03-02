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
    println!(
        "Programming language: {}, project count: {}",
        language, project_count
    );
}
