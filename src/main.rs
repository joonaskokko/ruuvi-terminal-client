// These are here so Cargo won't nag about parenthesis in IFs or unused code.
#![allow(unused_parens)]
#![allow(warnings)]
#[allow(dead_code)]

mod config;
mod ruuvi_custom_api;
mod ruuvi_gateway;
mod trends;

use pancurses::{Input, Window, COLOR_PAIR, COLOR_GREEN, COLOR_WHITE, COLOR_RED, A_BOLD};
use reqwest;
use chrono::{DateTime, Utc};
use std::{thread, time};
use std::process::ExitCode;

use config::load_config;
use trends::TrendTracker;

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

/**
 * The main render function.
 */
fn render(window: &Window, data: &ApiResponse, network_error: bool, border_style: &str) {
	window.clear();

	let maximum_width = window.get_max_x() - 1;
	let draw_border = border_style == "line";

	for tag in data {
		// Top border.
		if (draw_border) {
			let top_border_line = draw_horizontal_line("top", maximum_width);
			window.addstr(&format!("{}\n", top_border_line));
		}

		// Title row.
		let left_border_string = if draw_border { "│ " } else { "" };
		window.addstr(left_border_string);
		let mut title_line_length = if draw_border { 2 } else { 0 };
		window.attron(COLOR_PAIR(2) | A_BOLD);
		window.addstr(&tag.tag_name);
		window.attroff(COLOR_PAIR(2) | A_BOLD);
		title_line_length += display_width(&tag.tag_name);

		// Battery low indicator.
		if (tag.battery_low) {
			window.addstr(" ");
			title_line_length += 1;
			window.attron(COLOR_PAIR(3) | A_BOLD);
			window.addstr("Battery low");
			window.attroff(COLOR_PAIR(3) | A_BOLD);
			title_line_length += display_width("Battery low");
		}
		else if (tag.unreachable) {
			window.addstr(" ");
			title_line_length += 1;
			window.attron(COLOR_PAIR(3) | A_BOLD);
			window.addstr("Unreachable");
			window.attroff(COLOR_PAIR(3) | A_BOLD);
			title_line_length += display_width("Unreachable");
		}

		if (draw_border) {
			window.addstr(&get_box_padding(title_line_length, maximum_width));
		}
		window.addstr("\n");

		// Temperature and humidity.
		window.addstr(left_border_string);
		let mut sensor_line_length = if draw_border { 2 } else { 0 };
		window.attron(COLOR_PAIR(1) | A_BOLD);
		let temperature_string = format!("{:+.2}°C", tag.temperature.current);
		window.addstr(&temperature_string);
		window.attroff(COLOR_PAIR(1) | A_BOLD);
		sensor_line_length += display_width(&temperature_string);

		window.attron(COLOR_PAIR(2) | A_BOLD);
		let temperature_trend_symbol = trend_arrow(tag.temperature.trend);
		window.addstr(temperature_trend_symbol);
		window.attroff(COLOR_PAIR(2) | A_BOLD);
		sensor_line_length += display_width(temperature_trend_symbol);

		window.addstr(" ");
		sensor_line_length += 1;
		window.attron(COLOR_PAIR(1) | A_BOLD);
		let humidity_string = format!("{:.2}%", tag.humidity.current);
		window.addstr(&humidity_string);
		window.attroff(COLOR_PAIR(1) | A_BOLD);
		sensor_line_length += display_width(&humidity_string);

		window.attron(COLOR_PAIR(2) | A_BOLD);
		let humidity_trend_symbol = trend_arrow(tag.humidity.trend);
		window.addstr(humidity_trend_symbol);
		window.attroff(COLOR_PAIR(2) | A_BOLD);
		sensor_line_length += display_width(humidity_trend_symbol);

		if (draw_border) {
			window.addstr(&get_box_padding(sensor_line_length, maximum_width));
		}
		window.addstr("\n");

		// Temperature min/max (only if available).
		if let (Some(min), Some(max)) = (tag.temperature.min, tag.temperature.max) {
			window.addstr(left_border_string);
			let minmax_string = format!(
				"{:+.2}…{:+.2}°C",
				min,
				max
			);
			window.addstr(&minmax_string);

			if (draw_border) {
				let minmax_line_length = if draw_border { 2 } else { 0 } + display_width(&minmax_string);
				window.addstr(&get_box_padding(minmax_line_length, maximum_width));
			}
			window.addstr("\n");
		}

		// Updated string.
		window.addstr(left_border_string);
		let mut update_line_length = if draw_border { 2 } else { 0 };
		window.addstr("Updated: ");
		update_line_length += display_width("Updated: ");
		let time_ago_string = format_time_ago(&tag.datetime);
		window.addstr(&time_ago_string);
		update_line_length += display_width(&time_ago_string);
		if (draw_border) {
			window.addstr(&get_box_padding(update_line_length, maximum_width));
		}
		window.addstr("\n");

		// Bottom border.
		if (draw_border) {
			let bottom_border_line = draw_horizontal_line("bottom", maximum_width);
			window.addstr(&format!("{}\n", bottom_border_line));
		} else {
			window.addstr("\n");
		}
	}

	if (network_error) {
		window.attron(COLOR_PAIR(3) | A_BOLD);
		window.addstr("Network error\n");
		window.attroff(COLOR_PAIR(3) | A_BOLD);
	}

	window.refresh();
	return;
}

/**
 * Helper function for trend arrow mapping.
 */
fn trend_arrow(trend: Option<i8>) -> &'static str {
	match trend {
		Some(1) => "▴",
		Some(-1) => "▾",
		Some(_) | None => "▸",
	}
}

/**
 * Calculate the display width of a string (handling multi-byte UTF-8 characters).
 * Most characters are 1 width, but some Unicode characters might be different.
 * For our use case, we'll count grapheme clusters properly.
 */
fn display_width(string_value: &str) -> usize {
	string_value.chars().count()
}

/**
 * Draw a horizontal line for a box given width.
 */
fn draw_horizontal_line(line_type: &str, width: i32) -> String {
	let (left, right, fill) = match line_type {
		"top" => ("┌", "┐", "─"),
		"bottom" => ("└", "┘", "─"),
		_ => ("", "", ""),
	};

	let fill_width = (width - 2).max(0) as usize;
	let fill_string = fill.repeat(fill_width);
	return format!("{}{}{}", left, fill_string, right);
}

/**
 * Add spaces to fill to a certain column position, then add the right border.
 * Returns just the padding and border as a string.
 */
fn get_box_padding(content_length: usize, box_width: i32) -> String {
	let spaces_needed = if (box_width as usize) > content_length + 1 {
		(box_width as usize) - content_length - 1
	} else {
		0
	};
	return format!("{}│", " ".repeat(spaces_needed));
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
