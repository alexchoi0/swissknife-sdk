use clap::{Parser, ValueEnum};

#[derive(Debug, Clone, ValueEnum)]
pub enum ServerMode {
    Stdio,
    Http,
}

#[derive(Debug, Clone, ValueEnum)]
pub enum ToolCategory {
    All,
    Payments,
    Crm,
    Communication,
    Social,
    Hr,
    Banking,
    Auth,
    Search,
    Devtools,
    Productivity,
    Pm,
    Vectordb,
    Database,
    Ecommerce,
    Observability,
    Cloud,
    Memory,
    Scraping,
    Queue,
    Automation,
    File,
    Markets,
    Research,
    Llm,
}

#[derive(Parser, Debug)]
#[command(name = "swissknife-mcp")]
#[command(author = "Swissknife")]
#[command(version = "0.1.0")]
#[command(about = "MCP server for Swissknife AI tools", long_about = None)]
pub struct Cli {
    #[arg(short, long, value_enum, default_value = "http")]
    pub mode: ServerMode,

    #[arg(short, long, default_value = "127.0.0.1")]
    pub host: String,

    #[arg(short, long, default_value = "3000")]
    pub port: u16,

    #[arg(short, long, value_enum, default_value = "all")]
    pub tools: Vec<ToolCategory>,

    #[arg(long, default_value = "swissknife-mcp")]
    pub name: String,

    #[arg(long, default_value = "0.1.0")]
    pub version: String,

    #[arg(long)]
    pub cors_origins: Vec<String>,

    #[arg(short, long)]
    pub verbose: bool,
}

impl Cli {
    pub fn addr(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }

    pub fn has_category(&self, category: ToolCategory) -> bool {
        self.tools.iter().any(|t| matches!(t, ToolCategory::All))
            || self.tools.iter().any(|t| std::mem::discriminant(t) == std::mem::discriminant(&category))
    }
}
