use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io::{self, Write};
use dirs;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Config {
	pub api_url: String,
	#[serde(default = "default_api_type")]
	pub api_type: String,
	#[serde(default)]
	pub tag_names: HashMap<String, String>,
	#[serde(default = "default_border")]
	pub border: String,
}

fn default_api_type() -> String {
	"ruuvi_gateway".to_string()
}

fn default_border() -> String {
	"line".to_string()
}

/**
 * Load configuration with priority order:
 * 1. Config file at ~/.config/ruuvi-terminal-client/config.yml.
 * 2. Interactive prompt if no config file is present.
 */
pub fn load_config() -> Result<Config, Box<dyn std::error::Error>> {
	// Get config file path.
	let config_path = dirs::config_dir()
		.map(|dir| dir.join("ruuvi-terminal-client").join("config.yml"));

	// Check if config file exists.
	if let Some(ref path) = config_path {
		if path.exists() {
			let config_content = fs::read_to_string(&path)?;
			let config: Config = serde_yaml::from_str(&config_content)?;
			return Ok(config);
		}
	}

	// Ask user if no config file found.
	print!("Enter API URL: ");
	io::stdout().flush().unwrap();

	let mut api_url = String::new();
	io::stdin().read_line(&mut api_url)?;
	let api_url = api_url.trim().to_string();

	if api_url.is_empty() {
		return Err("API URL cannot be empty".into());
	}

	print!("Enter API type (ruuvi_custom_api/ruuvi_gateway) [default: ruuvi_gateway]: ");
	io::stdout().flush().unwrap();

	let mut api_type = String::new();
	io::stdin().read_line(&mut api_type)?;
	let api_type = api_type.trim().to_string();
	let api_type = if api_type.is_empty() {
		default_api_type()
	} else {
		api_type
	};

	print!("Enter border style (0: none, 1: line) [default: 1]: ");
	io::stdout().flush().unwrap();

	let mut border_choice = String::new();
	io::stdin().read_line(&mut border_choice)?;
	let border_choice = border_choice.trim().to_string();
	let border_style = match border_choice.as_str() {
		"0" => "none".to_string(),
		"" | "1" => "line".to_string(),
		_ => {
			println!("Invalid choice, using default (line)");
			"line".to_string()
		}
	};

	let config = Config {
		api_url: api_url.clone(),
		api_type: api_type.clone(),
		tag_names: HashMap::new(),
		border: border_style,
	};

	// Save to config file now that we have the settings.
	if let Some(path) = config_path {
		if let Some(parent) = path.parent() {
			fs::create_dir_all(parent)?;
		}
		let config_yaml = serde_yaml::to_string(&config)?;
		fs::write(&path, config_yaml)?;
	}

	return Ok(config);
}
