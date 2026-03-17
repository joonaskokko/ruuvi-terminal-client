use serde::Deserialize;
use chrono::{DateTime, Utc};
use crate::models::{Tag, SensorData};
use std::collections::HashMap;

#[derive(Debug, Deserialize)]
struct RuuviGatewayResponse {
	data: RuuviGatewayData,
}

#[derive(Debug, Deserialize)]
struct RuuviGatewayData {
	tags: HashMap<String, RuuviGatewayTag>,
}

#[derive(Debug, Deserialize)]
struct RuuviGatewayTag {
	temperature: f64,
	humidity: f64,
	voltage: f64,
	timestamp: u64,
}

pub fn parse_ruuvi_gateway(data: &str, tag_names: &HashMap<String, String>) -> Result<Vec<Tag>, Box<dyn std::error::Error>> {
	let response: RuuviGatewayResponse = serde_json::from_str(data)?;
	let now = Utc::now();
	const TWELVE_HOURS_SECS: i64 = 12 * 60 * 60;

	let mut tags: Vec<Tag> = response.data.tags.into_iter().map(|(mac, tag_data)| {
		let tag_datetime = DateTime::<Utc>::from_timestamp(tag_data.timestamp as i64, 0)
			.unwrap_or(now);
		let datetime_str = tag_datetime.to_rfc3339();
		
		let is_unreachable = (now - tag_datetime).num_seconds() > TWELVE_HOURS_SECS;

		Tag {
			tag_id: mac.clone(),
			tag_name: tag_names.get(&mac).cloned().unwrap_or_else(|| mac.clone()),
			temperature: SensorData {
				current: tag_data.temperature,
				min: None,
				max: None,
				trend: None,
			},
			humidity: SensorData {
				current: tag_data.humidity,
				min: None,
				max: None,
				trend: None,
			},
			battery_low: tag_data.voltage < 2.0,
			unreachable: is_unreachable,
			datetime: datetime_str,
		}
	}).collect();

	// Sort by tag_id for consistent ordering
	tags.sort_by(|a, b| a.tag_id.cmp(&b.tag_id));

	return Ok(tags);
}
