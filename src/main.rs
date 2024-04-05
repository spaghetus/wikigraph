use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::io::{stdin, stdout, BufReader, BufWriter};
use wikigraph::Connection;
use xml::reader::XmlEvent;

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename = "page")]
struct Page {
	pub title: String,
	pub revision: Vec<Revision>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename = "revision")]
struct Revision {
	pub text: String,
	pub timestamp: chrono::DateTime<Utc>,
}

fn main() {
	let mut wiki = serde_xml_rs::EventReader::new(BufReader::new(stdin().lock()));

	while let Ok(event) = wiki.next() {
		match event {
			XmlEvent::EndElement { name } if name.local_name == "siteinfo" => {
				break;
			}
			_ => {}
		}
	}

	eprintln!("Finished siteinfo");

	let mut deser = serde_xml_rs::Deserializer::new(wiki);
	let mut ser = csv::Writer::from_writer(BufWriter::new(stdout().lock()));

	while let Ok(page) = Page::deserialize(&mut deser) {
		let name = page.title.as_str();
		let Some(revision) = page.revision.iter().max_by_key(|r| &r.timestamp) else {
			continue;
		};
		let mut text = after_brackets(&revision.text);
		while !text.is_empty() {
			let (content, rest) = split_at_right_brackets(text);
			let dest = link_destination(content);
			let connection = Connection {
				from: name,
				to: dest,
			};
			ser.serialize(&connection).unwrap();
			text = after_brackets(rest);
		}
	}
}

fn after_brackets(it: &str) -> &str {
	it.split_once("[[").unwrap_or(("", "")).1
}

fn split_at_right_brackets(it: &str) -> (&str, &str) {
	it.split_once("]]").unwrap_or(("", it))
}

fn link_destination(it: &str) -> &str {
	it.split_once('|').unwrap_or((it, "")).0
}
