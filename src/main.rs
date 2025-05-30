use pancurses::{Input, Window, COLOR_PAIR, COLOR_GREEN, COLOR_WHITE, COLOR_RED, A_BOLD};
use reqwest;
use serde::Deserialize;
use chrono::{DateTime, Utc};
use std::{thread, time};
use std::env;

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
	unreachable: bool,
	tag_name: String,
}

type ApiResponse = Vec<Tag>;

/**
 * Wrapper for setting up the terminal.
*/
fn setup_terminal() -> Window {
	let window = pancurses::initscr();
	pancurses::start_color();
	pancurses::use_default_colors(); 	// This is needed. If not set, the background color will be forced black instead of terminal color.
	pancurses::init_pair(1, COLOR_WHITE, -1);
	pancurses::init_pair(2, COLOR_GREEN, -1);
	pancurses::init_pair(3, COLOR_RED, -1);
	
	window.nodelay(true);
	
	return window;
}

/**
 * Get data from the API.
*/
fn fetch_data(api_url: &str) -> Result<ApiResponse, Box<dyn std::error::Error>> {
	let response = reqwest::blocking::get(api_url)?;
	let data: ApiResponse = response.json()?;
	return Ok(data);
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
		else if (tag.unreachable) {
			window.addstr(" ");
			window.attron(COLOR_PAIR(3) | A_BOLD);
			window.addstr("Unreachable");
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
			return format!("{} seconds ago", seconds)
		}
		else if seconds < 3600 {
			return format!("{} minutes ago", seconds / 60)
		}
		else if seconds < 86400 {
			return format!("{} hours ago", seconds / 3600)
		}
		else {
			return format!("{} days ago", seconds / 86400)
		}
	}
	else {
		return "unknown".into() // &str to String.
	}
}

/**
 * Main.
*/
fn main() {
	// First check the API URL ENV.
	let api_url = env::var("API_URL")
		.expect("Environment variable API_URL must be set");
	let mut network_error = false;
	let mut last_refresh = Utc::now() - chrono::Duration::minutes(1);
	let mut data: ApiResponse = Vec::new();
	
	let window = setup_terminal();

	// Main loop.
	loop {
		let now = Utc::now();
		if (now - last_refresh).num_seconds() >= 60 {
			match fetch_data(&api_url) {
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

	pancurses::endwin();
}