use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Connection<'a> {
	pub from: &'a str,
	pub to: &'a str,
}
