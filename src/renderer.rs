use pancurses::{Window, COLOR_PAIR, COLOR_GREEN, COLOR_WHITE, COLOR_RED, A_BOLD};
use chrono::{DateTime, Utc};
use crate::ApiResponse;

/**
 * Main render function - orchestrates the entire UI rendering.
 */
pub fn render(window: &Window, data: &ApiResponse, network_error: bool, border_style: &str) {
	window.clear();

	// Terminal width and if we should draw a border or not.
	let terminal_width = window.get_max_x();
	let draw_border = border_style != "none";

	for tag in data {
		render_tag_card(window, tag, terminal_width, draw_border);
	}

	if (network_error) {
		render_error_message(window);
	}

	window.refresh();
	return;
}

/**
 * Render a complete tag card with all sections.
 */
fn render_tag_card(window: &Window, tag: &crate::Tag, terminal_width: i32, draw_border: bool) {
	if (draw_border) {
		let top_border_line = draw_horizontal_line("top", terminal_width);
		window.addstr(&format!("{}", top_border_line));
	}

	render_tag_header(window, tag, terminal_width, draw_border);
	render_sensor_row(window, tag, terminal_width, draw_border);

	if let (Some(min), Some(max)) = (tag.temperature.min, tag.temperature.max) {
		render_minmax_row(window, min, max, terminal_width, draw_border);
	}

	render_updated_row(window, &tag.datetime, terminal_width, draw_border);

	if (draw_border) {
		let bottom_border_line = draw_horizontal_line("bottom", terminal_width);
		window.addstr(&format!("{}", bottom_border_line));
	} else {
		window.addstr("\n");
	}
}

/**
 * Render the tag header with name and status indicators.
 */
fn render_tag_header(window: &Window, tag: &crate::Tag, terminal_width: i32, draw_border: bool) {
	let left_border_string = if draw_border { "│ " } else { "" };
	window.addstr(left_border_string);
	let mut title_line_length = if draw_border { 2 } else { 0 };

	window.attron(COLOR_PAIR(2) | A_BOLD);
	window.addstr(&tag.tag_name);
	window.attroff(COLOR_PAIR(2) | A_BOLD);
	title_line_length += display_width(&tag.tag_name);

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
		window.addstr(&get_box_padding(title_line_length, terminal_width));
	}
	else {
		window.addstr("\n");
	}
}

/**
 * Render the sensor values row (temperature and humidity with trends).
 */
fn render_sensor_row(window: &Window, tag: &crate::Tag, terminal_width: i32, draw_border: bool) {
	let left_border_string = if draw_border { "│ " } else { "" };
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
		window.addstr(&get_box_padding(sensor_line_length, terminal_width));
	}
	else {
		window.addstr("\n");
	}
}

/**
 * Render the min/max temperature row (only shown if data is available).
 */
fn render_minmax_row(window: &Window, min: f64, max: f64, terminal_width: i32, draw_border: bool) {
	let left_border_string = if draw_border { "│ " } else { "" };
	window.addstr(left_border_string);

	let minmax_string = format!("{:+.2}…{:+.2}°C", min, max);
	window.addstr(&minmax_string);

	if (draw_border) {
		let minmax_line_length = if draw_border { 2 } else { 0 } + display_width(&minmax_string);
		window.addstr(&get_box_padding(minmax_line_length, terminal_width));
	}
	else {
		window.addstr("\n");
	}
}

/**
 * Render the last updated timestamp row.
 */
fn render_updated_row(window: &Window, datetime: &str, terminal_width: i32, draw_border: bool) {
	let left_border_string = if draw_border { "│ " } else { "" };
	window.addstr(left_border_string);
	let mut update_line_length = if draw_border { 2 } else { 0 };

	window.addstr("Updated: ");
	update_line_length += display_width("Updated: ");

	let time_ago_string = format_time_ago(datetime);
	window.addstr(&time_ago_string);
	update_line_length += display_width(&time_ago_string);

	if (draw_border) {
		window.addstr(&get_box_padding(update_line_length, terminal_width));
	}
	else {
		window.addstr("\n");
	}
}

/**
 * Render network error message.
 */
fn render_error_message(window: &Window) {
	window.attron(COLOR_PAIR(3) | A_BOLD);
	window.addstr("Network error\n");
	window.attroff(COLOR_PAIR(3) | A_BOLD);
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
	return string_value.chars().count()
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

	// - 2 so we can fit the turn symbol.
	let fill_width = (width - 2).max(0) as usize;
	let fill_string = fill.repeat(fill_width);
	return format!("{}{}{}", left, fill_string, right);
}

/**
 * Add spaces to fill to a certain column position, then add the right border.
 * Returns just the padding and border as a string.
 */
fn get_box_padding(content_length: usize, box_width: i32) -> String {
	let spaces_needed = (box_width as usize) - content_length - 1;
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
		return "unknown".into()
	}
}
