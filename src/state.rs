use hyper::Body;
use nyantec_cert_auth::{CustomError, get_claims, is_allowed_by_uid, Permissions};

use crate::{Request, Response, StatusCode};

pub struct State {
    permissions: Option<Permissions>,
}

impl State {
    pub(crate) fn new(permissions: Option<Permissions>) -> Self {
        Self { permissions }
    }

    /// Retrieves the username and other relevant values from the client certificate
    /// and checks whether a matching user exists in the Snipe-IT database.
    ///
    /// If such a user does not yet exist, it creates a new user via the Snipe-IT REST API.
    async fn process_request(&self, req: Request<Body>) -> crate::Result<Response<Body>> {
        let claims = get_claims(req)?;

        if let Some(permissions) = &self.permissions {
            is_allowed_by_uid(&claims, &permissions)?;
        }

        Ok(Response::builder()
            .status(StatusCode::OK)
            .body(Body::from("success"))?)
    }

    /// Returns a HTTP response depending on the result of `process_request`.
    pub(crate) async fn handle(&self, req: Request<hyper::Body>) -> crate::Result<Response<hyper::Body>> {
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
                        .body(hyper::Body::from("error"))
                        .unwrap(), // error handling error
                )
            }
        }
    }
}
