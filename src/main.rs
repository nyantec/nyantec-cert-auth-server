//! This crate provides a web server for validating X.509 Client Certificates
//!
//! # X.509 Client Certificate Authentication with nginx
//!
//! Configure your reverse proxy to forward requests to `http://127.0.0.1:8124/cert-auth/`
//! and add the `X-SSL-Client-Dn` header.
//!
//! Example configuration[^note on ngx_http_auth_request_module]:
//! ```nginx
//! server {
//!     # ...
//!
//!     ssl_client_certificate CA.pem;
//!     ssl_verify_client on;
//!     ssl_verify_depth 1;
//!
//!     location /cert-auth/ {
//!         proxy_pass http://127.0.0.1:8124/;
//!         proxy_set_header X-SSL-Client-Dn $ssl_client_s_dn;
//!         proxy_set_header X-SSL-Verify $ssl_client_verify;
//!         proxy_set_header X-SSL-Client-Escaped-Cert $ssl_client_escaped_cert;
//!     }
//!
//!     location ~ \.php$ {
//!         auth_request /cert-auth/;
//!         fastcgi_param REMOTE_USER $ssl_client_s_dn_cn;
//!     }
//!
//!    # ...
//! }
//! ```
//!
//! [^note on ngx_http_auth_request_module] The [`ngx_http_auth_request_module`] is not built by
//! default and you might need to recompile nginx with the `--with-http_auth_request_module`
//! configuration parameter.
//!
//! ## Using this crate with GitLab
//! This crate can be used to sign in users via GitLab's [JWT OmniAuth provider].
//!
//! Invoke the application with the `--variant gitlab` argument and pass the JWT secret as
//! `GITLAB_JWT_SECRET` environment variable.
//!
//! ## Using this crate with Snipe-IT
//! [Snipe-IT] supports authentication via the REMOTE_USER header. However, a user must exist in
//! Snipe-IT first before being able to log in. As a step in between, this crate creates a new
//! user over the REST API if no such user exists in the database yet.
//!
//! For that, invoke the application with the `--variant snipe-it` argument and pass the API URL and
//! token as `SNIPE_IT_API_URL` and `SNIPE_IT_API_TOKEN` environment variable.
//!
//! ## Accepting only a subset of users with a valid client certificate
//!
//! It is also possible to permit only a subset of users, despite having a valid client
//! certificate. To use this feature, define a JSON file of the following structure
//!
//! ```json
//! {
//!     "allowed_uids": ["user1", "user2", "user3"]
//! }
//! ```
//! and pass the file as command line argument to `cert-auth`:
//!
//! ```shell
//! $ cert-auth --permissions permissions.json
//! ```
//!
//! [`ngx_http_auth_request_module`]: https://nginx.org/en/docs/http/ngx_http_auth_request_module.html
//! [JWT OmniAuth provider]: https://docs.gitlab.com/ee/administration/auth/jwt.html
//! [Snipe-IT]: https://snipeitapp.com

#![deny(missing_docs, unused_imports)]

use std::sync::Arc;
use std::{env, fs};

use clap::{ArgEnum, Parser};
use hyper::service::{make_service_fn, service_fn};
use hyper::{Request, Response, Server, StatusCode};

pub use nyantec_cert_auth;

use crate::snipe_it::SnipeItClient;
use crate::state::State;

#[doc(hidden)]
mod snipe_it;
#[doc(hidden)]
mod state;

#[doc(hidden)]
pub(crate) type Result<T> = std::result::Result<T, nyantec_cert_auth::CustomError>;

#[doc(hidden)]
pub const PORT: &'static str = "PORT";
#[doc(hidden)]
pub const SNIPE_IT_API_URL: &'static str = "SNIPE_IT_API_URL";
#[doc(hidden)]
pub const SNIPE_IT_API_TOKEN: &'static str = "SNIPE_IT_API_TOKEN";

/// Defines the supported authentication variants
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, ArgEnum)]
#[non_exhaustive]
#[doc(hidden)]
enum Variant {
	/// Starts the server with specialisation for authenticating against a Gitlab instance.
	Gitlab,

	/// Starts the server with specialisation for authenticating against a Snipe-IT instance.
	#[allow(non_camel_case_types)]
	Snipe_IT,
}

#[derive(Debug, Parser)]
#[clap(author, version, about, long_about = None)]
#[doc(hidden)]
struct ArgumentParser {
	/// Select an application variant
	#[clap(arg_enum, short, long)]
	variant: Option<Variant>,

	/// Path to a permissions.json containing a list of allowed UIDs
	#[clap(short, long)]
	permissions: Option<String>,
}

#[tokio::main]
#[doc(hidden)]
async fn main() -> Result<()> {
	let mut permissions: Option<nyantec_cert_auth::Permissions> = None;

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
