fn parse_ruuvi_gateway_format(value: &Value) -> Result<ApiResponse, Box<dyn std::error::Error>> {
	let mut tags = Vec::new();

	let tags_obj = &value["data"]["tags"];
	
	if !tags_obj.is_object() {
		return Err("Expected 'tags' to be an object".into());
	}

	for (mac, tag_data) in tags_obj.as_object().unwrap() {
		let temperature = tag_data.get("temperature").and_then(|v| v.as_f64()).unwrap_or(0.0);
		let humidity = tag_data.get("humidity").and_then(|v| v.as_f64()).unwrap_or(0.0);
		let voltage = tag_data.get("voltage").and_then(|v| v.as_f64()).unwrap_or(3.0); // assume 3.0V if missing
		let timestamp = tag_data.get("timestamp").and_then(|v| v.as_i64()).unwrap_or(Utc::now().timestamp());
		
		let battery_low = voltage <= 2.0;
		
		let tag = Tag {
			id: 0,
			tag_id: 0,
			datetime: DateTime::<Utc>::from_utc(
				chrono::NaiveDateTime::from_timestamp_opt(timestamp, 0)
					.ok_or("Invalid timestamp")?,
				Utc,
			).to_rfc3339(),

			temperature: Metric {
				current: temperature,
				min: temperature, // no real min/max in this format
				max: temperature,
				trend: 0, // fake for now
			},
			humidity: Metric {
				current: humidity,
				min: humidity,
				max: humidity,
				trend: 0, // fake for now
			},
			battery_low,
			tag_name: mac.clone(), // show MAC for now
		};

		tags.push(tag);
	}

	return Ok(tags);
}
