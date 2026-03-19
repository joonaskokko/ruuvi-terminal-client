// These are here so Cargo won't nag about parenthesis in IFs or unused code.
#![allow(unused_parens)]
#![allow(warnings)]
#[allow(dead_code)]

mod config;
mod ruuvi_custom_api;
mod ruuvi_gateway;
mod trends;
mod renderer;

use pancurses::{Input, Window, COLOR_PAIR, COLOR_GREEN, COLOR_WHITE, COLOR_RED, A_BOLD};
use reqwest;
use chrono::{DateTime, Utc};
use std::{thread, time};
use std::process::ExitCode;

use config::load_config;
use trends::TrendTracker;
use renderer::render;

#[derive(Debug, Clone)]
pub struct SensorData {
	pub current: f64,
	pub min: Option<f64>,
	pub max: Option<f64>,
	pub trend: Option<i8>,
}

#[derive(Debug, Clone)]
pub struct Tag {
	pub tag_id: String,
	pub tag_name: String,
	pub temperature: SensorData,
	pub humidity: SensorData,
	pub battery_low: bool,
	pub unreachable: bool,
	pub datetime: String,
}

type ApiResponse = Vec<Tag>;

/**
 * Main.
 */
fn main() -> ExitCode {
	let config = load_config()
		.expect("Failed to load configuration");
	let api_url = config.api_url;
	let api_type = config.api_type;
	let tag_names = config.tag_names;
	let border_style = config.border;
	let mut network_error = false;
	let mut last_refresh = Utc::now() - chrono::Duration::minutes(1);
	let mut data: ApiResponse = Vec::new();
	let mut trend_tracker = TrendTracker::new();

	let window = setup_terminal();

	// Main loop.
	loop {
		let now = Utc::now();
		if (now - last_refresh).num_seconds() >= 60 {
			match fetch_data(&api_url, &api_type, &tag_names) {
				Ok(mut new_data) => {
					trend_tracker.update_trends(&mut new_data);
					data = new_data;
					last_refresh = now;
					network_error = false;
				},
				Err(_) => {
					network_error = true;
				}
			}
		}

		render(&window, &data, network_error, &border_style);

		match window.getch() {
			Some(Input::Character('q')) => break,
			_ => {}
		}

		thread::sleep(time::Duration::from_secs(1));
	}

	pancurses::endwin();

	// Explicit return code 0.
	return ExitCode::SUCCESS;
}

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
fn fetch_data(api_url: &str, api_type: &str, tag_names: &std::collections::HashMap<String, String>) -> Result<ApiResponse, Box<dyn std::error::Error>> {
	let response = reqwest::blocking::get(api_url)?;
	let data = response.text()?;

	match api_type {
		"ruuvi_custom_api" => ruuvi_custom_api::parse_ruuvi_custom_api(&data, tag_names),
		"ruuvi_gateway" => ruuvi_gateway::parse_ruuvi_gateway(&data, tag_names),
		_ => Err(format!("Unknown API type: {}", api_type).into()),
	}
}
