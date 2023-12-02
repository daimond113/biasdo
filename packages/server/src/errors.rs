use actix_web::http::StatusCode;
use actix_web::{cookie::Cookie, HttpResponse, ResponseError};
use derive_more::{Display, Error};
use log::error;
use serde::Serialize;
use sqlx::Error as DbError;
use validator::ValidationErrors;

use crate::consts;

#[derive(Serialize)]
pub struct ErrorResponse {
    pub errors: String,
}

#[derive(Debug, Display, Error)]
pub enum Errors {
    Validation(ValidationErrors),
    Db(DbError),
}

pub fn remove_tokens_from_response<T>(mut response: HttpResponse<T>) -> HttpResponse<T> {
    response
        .add_removal_cookie(
            &Cookie::build(consts::SESSION_COOKIE_NAME, "")
                .path("/")
                .finish(),
        )
        .expect("HTTP-spec defying session cookie name");

    response
}

impl ResponseError for Errors {
    fn error_response(&self) -> HttpResponse {
        match self {
            Errors::Validation(err) => {
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
                    errors: error_string,
                })
            }
            Errors::Db(err) => {
                error!("{}", err);
                HttpResponse::InternalServerError().json(ErrorResponse {
                    errors: "DB error".to_string(),
                })
            } // _ => panic!("Unexpected error, {}", self),
        }
    }
}

#[derive(Debug, Display)]
pub enum RouteError {
    Errors(Errors),
    Status(StatusCode),
}

impl ResponseError for RouteError {
    fn error_response(&self) -> HttpResponse {
        match self {
            RouteError::Errors(err) => err.error_response(),
            RouteError::Status(status) => HttpResponse::build(*status).finish(),
        }
    }
}
