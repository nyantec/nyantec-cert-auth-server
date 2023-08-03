use std::env;

use hyper::{header, Body};
use jsonwebtoken::{encode, EncodingKey, Header};
use rand::distributions::Alphanumeric;
use rand::Rng;

use crate::nyantec_cert_auth::{get_claims, is_allowed_by_uid, CustomError, Permissions};
use crate::{snipe_it, Request, Response, SnipeItClient, StatusCode, Variant};

pub struct State {
	permissions: Option<Permissions>,
	variant: Option<Variant>,
	snipe_it_client: SnipeItClient,
}

impl State {
	pub(crate) fn new(permissions: Option<Permissions>, variant: Option<Variant>) -> Self {
		let snipe_it_client = match variant {
			Some(Variant::Snipe_IT) => SnipeItClient::new(
				Some(env::var("SNIPE_IT_API_URL").expect("missing SNIPE_IT_API_URL, exiting")),
				Some(
					env::var("SNIPE_IT_API_TOKEN")
						.expect("missing SNIPE_IT_API_TOKEN, exiting")
						.to_string(),
				),
			),
			_ => SnipeItClient::new(None, None),
		};

		Self {
			permissions,
			variant,
			snipe_it_client,
		}
	}

	/// Processes an authentication request for the selected application variant
	async fn process_request(&self, req: Request<Body>) -> crate::Result<Response<Body>> {
		let claims = get_claims(req)?;

		if let Some(permissions) = &self.permissions {
			is_allowed_by_uid(&claims, &permissions)?;
		}

		match self.variant {
			Some(Variant::Gitlab) => {
				let jwt_secret = env::var("GITLAB_JWT_SECRET")
					.expect("Missing GITLAB_JWT_SECRET environment variable!")
					.to_string();
				let token = encode(
					&Header::default(),
					&claims,
					&EncodingKey::from_secret(jwt_secret.as_str().as_ref()),
				)?;

				Ok(Response::builder()
					.status(StatusCode::TEMPORARY_REDIRECT)
					.header(
						header::LOCATION,
						format!("/users/auth/jwt/callback?jwt={}", token),
					)
					.body(Body::from("success"))?)
			}
			Some(Variant::Snipe_IT) => {
				let users = self.snipe_it_client.get_users().await?;
				if !self.snipe_it_client.contains_username(&claims.uid, &users) {
					let password: String = rand::thread_rng()
						.sample_iter(&Alphanumeric)
						.take(64)
						.map(char::from)
						.collect();

					let user = snipe_it::User {
						username: claims.uid.to_string(),
						first_name: claims.name.split(" ").take(1).collect(),
						last_name: claims.name.split(" ").skip(1).collect(),
						email: claims.email.to_string(),
						activated: true,
						password: password.clone(),
						password_confirmation: password,
					};

					let _response = self.snipe_it_client.post_users(&user).await?;
				}

				Ok(Response::builder()
					.status(StatusCode::OK)
					.header("X-Remote-User", claims.uid)
					.body(Body::from("success"))?)
			}
			_ => Ok(Response::builder()
				.status(StatusCode::OK)
				.header("X-Remote-User", &claims.uid)
				.header("X-Remote-Name", &claims.name)
				.header("X-Remote-Email", &claims.email)
				.body(Body::from("success"))?),
		}
	}

	/// Returns a HTTP response depending on the result of `process_request`.
	pub(crate) async fn handle(
		&self,
		req: Request<hyper::Body>,
	) -> crate::Result<Response<hyper::Body>> {
		match self.process_request(req).await {
			Ok(resp) => Ok(resp),
			Err(e) => {
				println!("Error while processing request: {}", e);
				Ok(
					Response::builder()
						.status(match e {
							CustomError::PermissionNotMatchedError => hyper::StatusCode::FORBIDDEN,
							_ => hyper::StatusCode::INTERNAL_SERVER_ERROR,
						})
						.body(hyper::Body::from(format!(
							"error: {}",
							match e {
								CustomError::PermissionNotMatchedError => "Permission denied",
								_ => "Internal Server Error",
							},
						)))
						.unwrap(), // error handling error
				)
			}
		}
	}
}
