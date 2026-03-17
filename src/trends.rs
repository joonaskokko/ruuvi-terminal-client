use crate::models::Tag;

pub struct TrendTracker {
	previous_tags: Option<Vec<Tag>>,
}

impl TrendTracker {
	pub fn new() -> Self {
		return TrendTracker {
			previous_tags: None,
		};
	}

	pub fn update_trends(&mut self, tags: &mut Vec<Tag>) {
		if let Some(ref prev_tags) = self.previous_tags {
			for tag in tags.iter_mut() {
				if let Some(prev_tag) = prev_tags.iter().find(|t| t.tag_id == tag.tag_id) {
					// Only calculate trend if it wasn't provided in the payload (is None)
					if tag.temperature.trend.is_none() {
						tag.temperature.trend = Some(compare(tag.temperature.current, prev_tag.temperature.current));
					}
					if tag.humidity.trend.is_none() {
						tag.humidity.trend = Some(compare(tag.humidity.current, prev_tag.humidity.current));
					}
				}
			}
		}

		self.previous_tags = Some(tags.clone());
	}
}

fn compare(current: f64, previous: f64) -> i8 {
	if current > previous {
		return 1;
	} else if current < previous {
		return -1;
	} else {
		return 0;
	}
}

