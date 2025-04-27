use pancurses::{initscr, endwin, Input, Window, COLOR_PAIR, COLOR_GREEN, COLOR_WHITE, COLOR_RED, start_color, init_pair, has_colors, A_BOLD, use_default_colors};
use reqwest;
use serde::Deserialize;
use serde_json::Value;
use chrono::{DateTime, Utc};
use std::{thread, time};
use std::env;

mod formatters;
use formatters::custom_ruuvi_api::parse_custom_ruuvi_api_format;
use formatters::ruuvi_gateway::parse_ruuvi_gateway_format;

#[derive(Debug, Deserialize)]
struct Metric {
	current: f64,
	min: f64,
	max: f64,
	trend: i8,
}

#[derive(Debug, Deserialize)]
struct Tag {
	id: u32,
	tag_id: u32,
	datetime: String,
	temperature: Metric,
	humidity: Metric,
	battery_low: bool,
	tag_name: String,
}

type ApiResponse = Vec<Tag>;

enum BackendFormat {
	RuuviGatewayHistory,
	CustomRuuviApi,
}

struct Config {
	api_url: String,
	backend_format: BackendFormat,
	name_mapping: HashMap<String, String>,
}

/**
 * Get data from the API.
*/
fn fetch_data(config: &Config) -> Result<ApiResponse, Box<dyn std::error::Error>> {
	let response = reqwest::blocking::get(&config.api_url)?;
	let data: serde_json::Value = response.json()?;
	//return Ok(data);
	
	match &config.backend_format {
		BackendFormat::RuuviGatewayHistory => parse_ruuvi_gateway_format(&data),
		BackendFormat::CustomRuuviApi => parse_custom_ruuvi_api_format(&data),
	}
}

/**
 * The main render function.
*/
fn render(window: &Window, data: &ApiResponse, network_error: bool) {
	window.clear();
	
	for tag in data {
		// Title row.
		window.attron(COLOR_PAIR(2) | A_BOLD);
		window.addstr(&tag.tag_name);
		window.attroff(COLOR_PAIR(2) | A_BOLD);
		
		// Battery low indicator.
		if (tag.battery_low) {
			window.addstr(" ");
			window.attron(COLOR_PAIR(3) | A_BOLD);
			window.addstr("Battery low");
			window.attroff(COLOR_PAIR(3) | A_BOLD);
		}
		
		window.addstr("\n");

		// Temperature and humidity.
		window.attron(COLOR_PAIR(1) | A_BOLD);
		window.addstr(&format!("{:+.2}°C", tag.temperature.current));
		window.attroff(COLOR_PAIR(1) | A_BOLD);

		window.attron(COLOR_PAIR(2) | A_BOLD);
		window.addstr(trend_arrow(tag.temperature.trend));
		window.attroff(COLOR_PAIR(2) | A_BOLD);

		window.attron(COLOR_PAIR(1) | A_BOLD);
		window.addstr(&format!(" {:.2}%", tag.humidity.current));
		window.attroff(COLOR_PAIR(1) | A_BOLD);

		window.attron(COLOR_PAIR(2) | A_BOLD);
		window.addstr(trend_arrow(tag.humidity.trend));
		window.attroff(COLOR_PAIR(2) | A_BOLD);

		window.addstr("\n");

		// Temperature min/max.
		window.addstr(&format!(
			"{:+.2}…{:+.2}°C\n",
			tag.temperature.min,
			tag.temperature.max
		));
		
		// Updated string.
		window.addstr("Updated: ");
		window.addstr(&format_time_ago(&tag.datetime));
		
		window.addstr("\n\n");
	}
	
	if (network_error) {
		window.attron(COLOR_PAIR(3) | A_BOLD);
		window.addstr("Network error\n");
		window.attroff(COLOR_PAIR(3) | A_BOLD);
	}

	window.refresh();
}

/**
 * Helper function for helper arrow mapping.
*/
fn trend_arrow(trend: i8) -> &'static str {
	match trend {
		1 => "▴",
		-1 => "▾",
		_ => "▸",
	}
}

/**
 * Get human readable time ago.
*/
fn format_time_ago(datetime: &str) -> String {
	if let Ok(parsed) = datetime.parse::<DateTime<Utc>>() {
		let now = Utc::now();
		let seconds = (now - parsed).num_seconds();

		if (seconds < 60) {
			format!("{} seconds ago", seconds)
		}
		else if seconds < 3600 {
			format!("{} minutes ago", seconds / 60)
		}
		else if seconds < 86400 {
			format!("{} hours ago", seconds / 3600)
		}
		else {
			format!("{} days ago", seconds / 86400)
		}
	}
	else {
		"unknown".into() // &str to String.
	}
}

/**
 * Create config.
*/
fn load_config() -> Config {
	let api_url = env::var("API_URL")
		.expect("Environment variable API_URL must be set");

	let backend_format = match env::var("BACKEND_FORMAT")
		.unwrap_or_else(|_| "custom".to_string())
		.to_lowercase()
		.as_str() {
			"ruuvi-gateway-history" => BackendFormat::RuuviGatewayHistory,
			"custom" | _ => BackendFormat::CustomRuuviApi,
	};

	/*
	// Load names from names.json if exists (optional)
	let name_mapping = match std::fs::read_to_string("names.json") {
		Ok(content) => serde_json::from_str(&content).unwrap_or_default(),
		Err(_) => HashMap::new(),
	};
	*/

	return Config {
		api_url,
		backend_format,
		name_mapping,
	}
}

/**
 * Main.
*/
fn main() {
	// Do configuration.
	let config = load_config();
	let mut network_error = false;
	let mut last_refresh = Utc::now() - chrono::Duration::minutes(1);
	let mut data: ApiResponse = Vec::new();
	
	let window = initscr();
	
	start_color();

	if (!has_colors()) {
		println!("Your terminal does not support colors.");
		return;
	}

	// This is needed. If not set, the background color will be forced black instead of terminal color.
	use_default_colors();
	window.nodelay(true);
	
	// Set color pairs. -1 is transparent background.
	init_pair(1, COLOR_WHITE, -1);
	init_pair(2, COLOR_GREEN, -1);
	init_pair(3, COLOR_RED, -1);

	// Main loop.
	loop {
		let now = Utc::now();
		if (now - last_refresh).num_seconds() >= 60 {
			match fetch_data(&config) {
				Ok(new_data) => {
					data = new_data;
					last_refresh = now;
					network_error = false;
				},
				Err(_) => {
					network_error = true;
				}
			}
		}

		render(&window, &data, network_error);

		match window.getch() {
			Some(Input::Character('q')) => break,
			_ => {}
		}
		
		thread::sleep(time::Duration::from_secs(1));
	}

	endwin();
}