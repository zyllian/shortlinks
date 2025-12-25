use std::collections::BTreeMap;

use axum::{
	Router,
	extract::{Path, State},
	http::StatusCode,
	response::{Html, Redirect},
	routing::get,
};
use serde::Deserialize;
use tokio::net::TcpListener;

const ROOT_KEY: &str = "$root";

#[tokio::main]
async fn main() -> eyre::Result<()> {
	color_eyre::install()?;

	let config: &'static Config = Box::leak(Box::new(serde_json::from_str(
		&std::fs::read_to_string("config.json")?,
	)?));

	let app: Router = Router::new()
		.route("/", get(root))
		.route("/{*shortlink}", get(shortlink_handler))
		.with_state(config);

	let listener = TcpListener::bind("0.0.0.0:80").await?;
	axum::serve(listener, app).await?;

	Ok(())
}

async fn root() -> &'static str {
	"shortlink handler is running!"
}

async fn shortlink_handler(
	State(config): State<&'static Config>,
	Path(shortlink): Path<String>,
) -> Result<Redirect, (StatusCode, Html<&'static str>)> {
	match find_shortlink(&shortlink, &config.links) {
		Some(link) => Ok(Redirect::to(link)),
		None => Err((StatusCode::NOT_FOUND, Html(&config.not_found_message))),
	}
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum ShortlinkEntry {
	Link(String),
	Nested(ShortlinkMap),
}

type ShortlinkMap = BTreeMap<String, ShortlinkEntry>;

fn find_shortlink<'m>(shortlink: &str, map: &'m ShortlinkMap) -> Option<&'m str> {
	let mut segments = shortlink.split('/');
	let mut selection = map.get(segments.next()?)?;

	for segment in segments {
		match selection {
			ShortlinkEntry::Nested(map) => {
				selection = map.get(segment)?;
			}
			_ => return None,
		}
	}

	match selection {
		ShortlinkEntry::Nested(map) => map.get(ROOT_KEY).and_then(|e| match e {
			ShortlinkEntry::Link(link) => Some(link.as_str()),
			_ => None,
		}),
		ShortlinkEntry::Link(link) => Some(link),
	}
}

#[derive(Debug, Deserialize)]
struct Config {
	not_found_message: String,
	links: ShortlinkMap,
}
