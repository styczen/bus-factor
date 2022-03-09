use bus_factor::get_bus_factor;
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

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Parse command line arguments
    let args = ProgramOptions::parse();
    let language = args.language;
    let project_count = args.project_count;

    // Get all information with bus factor equal to 1
    let result = get_bus_factor(&language, project_count).await?;

    // Print result to console
    result.iter().for_each(|r| {
        println!(
            "project: {:<30} user: {:<30} percentage: {:.2}",
            r.0, r.1.login, r.2
        )
    });
    Ok(())
}
