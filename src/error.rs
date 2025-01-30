use actix_web::{body::BoxBody, HttpResponse, ResponseError};
use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum BackendError {
	#[error("error querying the database")]
	DB(#[from] sqlx::Error),

	#[error("error verifying password")]
	Password(#[from] password_auth::VerifyError),

	#[error("error occurred validating request data")]
	Validation(#[from] validator::ValidationErrors),

	#[error("webauthn error")]
	Webauthn(#[from] webauthn_rs::prelude::WebauthnError),

	#[error("serde_json error")]
	SerdeJson(#[from] serde_json::Error),
}

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
	pub error: String,
}

impl ResponseError for BackendError {
	fn error_response(&self) -> HttpResponse<BoxBody> {
		match self {
			BackendError::Validation(err) => {
				let mut error_string = String::new();
				let field_errors = err.field_errors();

				for (field_name, error_list) in field_errors.iter() {
					let mut field_error_messages = Vec::new();

					for error in error_list.iter() {
						field_error_messages.push(error.to_string());
					}

					let field_error_string =
						format!("{}: {}", field_name, field_error_messages.join(", "));
					error_string.push_str(&field_error_string);
					error_string.push('\n');
				}

				HttpResponse::BadRequest().json(ErrorResponse {
					error: error_string,
				})
			}
			BackendError::Password(e) => {
				match e {
					password_auth::VerifyError::Parse(_) => {
						HttpResponse::InternalServerError().finish()
					}
					// the passwords didn't match, no need to log that
					_ => HttpResponse::Unauthorized().json(ErrorResponse {
						error: "Invalid username or password".to_string(),
					}),
				}
			}
			BackendError::Webauthn(e) => HttpResponse::BadRequest().json(ErrorResponse {
				error: e.to_string(),
			}),
			_ => HttpResponse::InternalServerError().finish(),
		}
	}
}

pub type ApiResult = Result<HttpResponse, BackendError>;
