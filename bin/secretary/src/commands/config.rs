use crate::cli::ConfigCommands;
use crate::config::{self, Config};

pub fn handle_config_command(command: &ConfigCommands, config: &Config) {
    match command {
        ConfigCommands::Show => match toml::to_string_pretty(config) {
            Ok(s) => println!("{}", s),
            Err(e) => {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        },
        ConfigCommands::Path => println!("{}", Config::config_path().display()),
        ConfigCommands::Init => {
            let path = Config::config_path();
            if path.exists() {
                eprintln!("Config file already exists at: {}", path.display());
                std::process::exit(1);
            }
            match Config::default().save() {
                Ok(_) => println!("Created config file at: {}", path.display()),
                Err(e) => {
                    eprintln!("Error: {}", e);
                    std::process::exit(1);
                }
            }
        }
        ConfigCommands::Set { key, value } => match config::set_config_value(key, value) {
            Ok(_) => println!("Set {} = {}", key, value),
            Err(e) => {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        },
        ConfigCommands::Get { key } => match config::get_config_value(key) {
            Some(value) => println!("{}", value),
            None => {
                eprintln!("Key not found: {}", key);
                std::process::exit(1);
            }
        },
        ConfigCommands::Unset { key } => match config::unset_config_value(key) {
            Ok(_) => println!("Unset {}", key),
            Err(e) => {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        },
    }
}
