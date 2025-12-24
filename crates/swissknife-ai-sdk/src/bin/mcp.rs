use clap::Parser;
use swissknife_ai_sdk::mcp::cli::{run, Cli};

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    if let Err(e) = run(cli).await {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
