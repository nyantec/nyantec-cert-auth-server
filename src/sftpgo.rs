use serde_derive::Serialize;

#[derive(Serialize)]
pub(crate) struct SFTPGoUser {
	pub(crate) username: String,
	pub(crate) email: String,
	pub(crate) password: String,
}
