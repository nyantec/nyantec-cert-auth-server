//! # nyantec-cert-auth-server
//!
//!
//! ## Using the default variant
//! TODO document
//!
//!
//! ## Using the Gitlab variant
//! To use this variant, invoke the application with the `--variant snipe-it` command line flag.
//! TODO document
//!
//!
//! ## Using the Snipe-IT variant
//! To use this variant, invoke the application with the `--variant snipe-it` command line flag.
//!
//! If using the `snipe-it` variant, this crate parses a given client certificate for a matching
//! user in the Snipe-IT database and creates a new user over the REST API if such a user does not
//! exist yet.
//!
//! It expects a `SNIPE_IT_API_URL` and `SNIPE_IT_API_TOKEN` environment variable to be set.
//! Refer to the Snipe-IT documentation on how to obtain the API URL and API token.
//!
//!
//! ## Layout of the Permissions file
//! This crate expects the permissions.json file of the following structure:
//! ```json
//! {
//!     "allowed_uids": []
//! }
//! ```
use std::sync::Arc;
use std::{env, fs};

use clap::{ArgEnum, Parser};
use hyper::service::{make_service_fn, service_fn};
use hyper::{Request, Response, Server, StatusCode};
use nyantec_cert_auth::{CustomError, Permissions};

use crate::snipe_it::SnipeItClient;
use crate::state::State;

mod sftpgo;
mod snipe_it;
mod state;

pub(crate) type Result<T> = std::result::Result<T, CustomError>;

// define environment variables
pub(crate) const PORT: &'static str = "PORT";
pub(crate) const SNIPE_IT_API_URL: &'static str = "SNIPE_IT_API_URL";
pub(crate) const SNIPE_IT_API_TOKEN: &'static str = "SNIPE_IT_API_TOKEN";

/// Defines the supported variants of this Cert-Auth-Server
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, ArgEnum)]
#[non_exhaustive]
enum Variant {
	/// Starts the server with specialisation for authenticating against a Gitlab instance.
	Gitlab,

	///  Starts the server with specialisation for authenticating aginst a SFTPGo instance.
	SFTPGo,

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

	if let Some(path) = &clap.permissions {
		let contents =
			fs::read_to_string(&path).expect(&format!("Unable to read {} into a String!", path));
		permissions =
			Some(serde_json::from_str(&contents).expect("JSON File is not of the expected format!"))
	}

	let port = env::var(PORT)
		.or(Ok::<String, ()>("8124".to_string()))
		.unwrap()
		.parse::<u16>()
		.unwrap();
	let addr = ([127, 0, 0, 1], port).into();

	let state = Arc::new(State::new(permissions, clap.variant));
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
