use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "secretary")]
#[command(about = "A conversational CLI assistant powered by Claude", long_about = None)]
#[command(version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,

    /// Enable extended thinking
    #[arg(long, global = true)]
    pub think: bool,

    /// Disable extended thinking
    #[arg(long, global = true)]
    pub no_think: bool,

    /// Disable SDK MCP tools
    #[arg(long, global = true)]
    pub no_sdk: bool,

    /// Disable builtin tools
    #[arg(long, global = true)]
    pub no_builtin: bool,

    /// Custom config file path
    #[arg(short, long, global = true)]
    pub config: Option<PathBuf>,

    /// Model to use (e.g., haiku, sonnet, opus, or full model ID)
    #[arg(short, long, global = true)]
    pub model: Option<String>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Interactive chat sessions
    Chat {
        #[command(subcommand)]
        command: Option<ChatCommands>,
    },
    /// Session management
    Sessions {
        #[command(subcommand)]
        command: SessionsCommands,
    },
    /// Configuration management
    Config {
        #[command(subcommand)]
        command: ConfigCommands,
    },
    /// MCP server management
    Mcp {
        #[command(subcommand)]
        command: McpCommands,
    },
    /// Import Claude Code history into memory
    Import {
        #[command(subcommand)]
        command: ImportCommands,
    },
    /// Query Claude Code history
    History {
        #[command(subcommand)]
        command: HistoryCommands,
    },
}

#[derive(Subcommand)]
pub enum ChatCommands {
    /// Start a new session
    New,
    /// Resume a specific session
    Resume {
        /// Session ID (or prefix)
        id: String,
    },
}

#[derive(Subcommand)]
pub enum SessionsCommands {
    /// List recent sessions
    List {
        /// Number of sessions to show
        #[arg(short, long, default_value = "10")]
        limit: usize,
    },
    /// Delete a session
    Delete {
        /// Session ID
        id: String,
    },
    /// Show session details
    Show {
        /// Session ID
        id: String,
    },
}

#[derive(Subcommand)]
pub enum ConfigCommands {
    /// Show current config (TOML format)
    Show,
    /// Show config file path
    Path,
    /// Create default config file
    Init,
    /// Set a config value
    Set {
        /// Config key (e.g., model.name)
        key: String,
        /// Value to set
        value: String,
    },
    /// Get a config value
    Get {
        /// Config key (e.g., model.name)
        key: String,
    },
    /// Remove a config value
    Unset {
        /// Config key to remove
        key: String,
    },
}

#[derive(Subcommand)]
pub enum McpCommands {
    /// List configured MCP servers
    List,
    /// Add an MCP server command
    Add {
        /// Command to start the MCP server
        command: String,
    },
    /// Remove an MCP server
    Remove {
        /// Name or partial match of server to remove
        name: String,
    },
}

#[derive(Subcommand)]
pub enum ImportCommands {
    /// Import from ~/.claude/ history
    Claude {
        /// Only import from specific project path
        #[arg(short, long)]
        project: Option<String>,

        /// Limit number of prompts to import (omit for unlimited)
        #[arg(short, long)]
        limit: Option<usize>,
    },
}

#[derive(Subcommand)]
pub enum HistoryCommands {
    /// Search prompts by text
    Search {
        /// Search query
        query: String,

        /// Limit results
        #[arg(short, long, default_value = "20")]
        limit: usize,
    },
    /// List recent prompts
    Prompts {
        /// Filter by project path
        #[arg(short, long)]
        project: Option<String>,

        /// Limit results
        #[arg(short, long, default_value = "20")]
        limit: usize,
    },
    /// List messages from a session
    Messages {
        /// Session ID (or prefix)
        session_id: String,

        /// Limit results
        #[arg(short, long, default_value = "50")]
        limit: usize,
    },
    /// Show statistics
    Stats,
    /// Run raw SQL query
    Sql {
        /// SQL query to execute
        query: String,
    },
}
