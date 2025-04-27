use crate::ApiResponse;

fn parse_custom_ruuvi_api_format(value: &serde_json::Value) -> Result<ApiResponse, Box<dyn std::error::Error>> {
	let tags: ApiResponse = serde_json::from_value(value.clone())?;
	return Ok(tags);
}