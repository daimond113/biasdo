use actix_web::body::BoxBody;
use actix_web::{HttpResponse, ResponseError};
use log::error;
use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("an error occurred while querying the database")]
    DB(#[from] sqlx::Error),

    #[error("an error occurred while verifying the password")]
    Password(#[from] password_auth::VerifyError),

    #[error("an error occurred while validating the request data")]
    Validation(#[from] validator::ValidationErrors),
}

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
}

impl ResponseError for Error {
    fn error_response(&self) -> HttpResponse<BoxBody> {
        match self {
            Error::DB(e) => error!("Database error: {e:?}"),
            Error::Password(e) => {
                return match e {
                    password_auth::VerifyError::Parse(err) => {
                        error!("Password parse error: {err:?}");

                        HttpResponse::InternalServerError().finish()
                    }
                    // the passwords didn't match, no need to log that
                    _ => HttpResponse::Unauthorized().json(ErrorResponse {
                        error: "Invalid username or password".to_string(),
                    }),
                };
            }
            _ => (),
        }

        match self {
            Error::Validation(err) => {
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
            _ => HttpResponse::InternalServerError().finish(),
        }
    }
}
