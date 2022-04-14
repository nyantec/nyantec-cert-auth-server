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
use std::{env, fs, sync::Arc};

use hyper::{
    Request, Response, Server, service::{make_service_fn, service_fn}, StatusCode,
};
use nyantec_cert_auth::{CustomError, Permissions};

use crate::state::State;

mod state;

pub(crate) type Result<T> = std::result::Result<T, CustomError>;

fn usage() {
    println!(
        "Usage: snipe-it-cert-auth [ <path/to/permissions.json> ]\
        \
        Starts a web server for validating X.509 Client Certificates.\
        \
        Optionally accepts a path to a permissions.json file as first command line argument.\
        If such a file is provided, the cert-auth service will match a client certificate against\
        the provided list of allowed uids or email addresses."
    );
}

#[tokio::main]
async fn main() -> Result<()> {
    let mut permissions: Option<Permissions> = None;

    // command line argument parsing
    let args: Vec<String> = env::args().collect();
    match args.len() {
        1 => {
            // Without a supplied permissions.json as command line argument,
            // it will permit all requests providing a valid client certificate.
        }
        2 => {
            // Parse permissions file
            let path = args.get(1).expect("Unable parse the first CLI argument!");
            let contents = fs::read_to_string(path)
                .expect(&format!("Unable to read {} into a String!", path));
            permissions = Some(
                serde_json::from_str(&contents).expect("JSON File is not of the expected format!"),
            )
        }
        _ => usage(),
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
