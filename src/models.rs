#[derive(Debug, Clone)]
pub struct SensorData {
	pub current: f64,
	#[allow(dead_code)]
	pub min: Option<f64>,
	#[allow(dead_code)]
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
