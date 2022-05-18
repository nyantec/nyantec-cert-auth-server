//! # nyantec-cert-auth-server
//!
//! This crate parses a given client certificate for a matching user in the Snipe-IT database and
//! creates a new user over the REST API if such a user does not exist yet.
//!
//! ## Connecting to the Snipe-IT API
//! This crate expects an `API_URL` and an `API_TOKEN` environment variable.
//! Refer to the Snipe-IT documentation on how to obtain the API URL and API token.
//!
//! ## Layout of the Permissions file
//! This crate expects the permissions.json file of the following structure:
//! ```json
//! {
//!     "allowed_emails": [],
//!     "allowed_uids": []
//! }
//! ```
use clap::{ArgEnum, Parser};
use hyper::service::{make_service_fn, service_fn};
use hyper::{Request, Response, Server, StatusCode};
use nyantec_cert_auth::{CustomError, Permissions};

use std::sync::Arc;
use std::{env, fs};

use crate::state::State;

mod state;

pub(crate) type Result<T> = std::result::Result<T, CustomError>;

/// Defines the supported variants of this Cert-Auth-Server
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, ArgEnum)]
#[non_exhaustive]
enum Variant {
	/// Starts the server with specialisation for authenticating against a Gitlab instance.
	Gitlab,

	/// Starts the server with specialisation for authenticating against a Snipe-IT instance.
	#[allow(non_camel_case_types)]
	Snipe_IT,
}

#[derive(Debug, Parser)]
#[clap(author, version, about, long_about = None)]
struct ArgumentParser {
	/// Select an application variant.
	#[clap(arg_enum, short, long)]
	variant: Option<Variant>,

	/// Path to a permissions.json containing a list of allowed user ids
	#[clap(short, long)]
	permissions: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
	let mut permissions: Option<Permissions> = None;

	let clap = ArgumentParser::parse();

	if let Some(path) = clap.permissions {
		let contents =
			fs::read_to_string(&path).expect(&format!("Unable to read {} into a String!", path));
		permissions =
			Some(serde_json::from_str(&contents).expect("JSON File is not of the expected format!"))
	}

	let port = env::var("PORT")
		.or(Ok::<String, ()>("8124".to_string()))
		.unwrap()
		.parse::<u16>()
		.unwrap();
	let addr = ([127, 0, 0, 1], port).into();

	let state = Arc::new(State::new(permissions));
	let make_svc = make_service_fn(|_conn| {
		let state = Arc::clone(&state);
		async move {
			Ok::<_, hyper::Error>(service_fn(move |req| {
				let state = Arc::clone(&state);
				async move { state.handle(req).await }
			}))
		}
	});

	let server = Server::bind(&addr).serve(make_svc);

	println!("listening on port {}", port);

	if let Err(e) = server.await {
		eprintln!("server error: {}", e);
	}
	Ok(())
}
