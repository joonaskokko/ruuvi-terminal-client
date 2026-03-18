use serde::Deserialize;
use crate::{Tag, SensorData};
use std::collections::HashMap;

#[derive(Debug, Deserialize, Clone)]
struct Metric {
	current: f64,
	min: f64,
	max: f64,
	trend: i8,
}

#[derive(Debug, Deserialize)]
struct RuuviApiTag {
	tag_id: u32,
	datetime: String,
	temperature: Metric,
	humidity: Metric,
	battery_low: bool,
	unreachable: bool,
	tag_name: String,
}

pub fn parse_ruuvi_custom_api(data: &str, _tag_names: &HashMap<String, String>) -> Result<Vec<Tag>, Box<dyn std::error::Error>> {
	let tags: Vec<RuuviApiTag> = serde_json::from_str(data)?;

	let unified = tags.into_iter().map(|tag| Tag {
		tag_id: tag.tag_id.to_string(),
		tag_name: if tag.tag_name.is_empty() { "Unknown tag".to_string() } else { tag.tag_name },
		temperature: SensorData {
			current: tag.temperature.current,
			min: Some(tag.temperature.min),
			max: Some(tag.temperature.max),
			trend: Some(tag.temperature.trend),
		},
		humidity: SensorData {
			current: tag.humidity.current,
			min: Some(tag.humidity.min),
			max: Some(tag.humidity.max),
			trend: Some(tag.humidity.trend),
		},
		battery_low: tag.battery_low,
		unreachable: tag.unreachable,
		datetime: tag.datetime,
	}).collect();

	return Ok(unified);
}
