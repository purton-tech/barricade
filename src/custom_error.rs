use actix_web::{HttpResponse, ResponseError};
use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub enum CustomError {
    FaultySetup(String),
    DatabaseError(String),
    Unauthorized,
}

// Allow the use of "{}" format specifier
impl fmt::Display for CustomError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            CustomError::FaultySetup(ref cause) => write!(f, "Setup Error: {}", cause),
            CustomError::DatabaseError(ref cause) => write!(f, "Setup Error: {}", cause),
            CustomError::Unauthorized => write!(f, "User doesn't have access"),
        }
    }
}

/// Actix web uses `ResponseError` for conversion of errors to a response
impl ResponseError for CustomError {
    fn error_response(&self) -> HttpResponse {
        match self {
            CustomError::FaultySetup(err) => {
                HttpResponse::InternalServerError().body(err.to_string())
            }
            CustomError::DatabaseError(err) => {
                HttpResponse::InternalServerError().body(err.to_string())
            }
            CustomError::Unauthorized => HttpResponse::Unauthorized().body("Unauthorized"),
        }
    }
}

// rust bitcoin Allow this type to be treated like an error
impl Error for CustomError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        // Generic error, underlying cause isn't tracked.
        None
    }
}

// Age using a buffered writer
impl From<sqlx::Error> for CustomError {
    fn from(err: sqlx::Error) -> CustomError {
        CustomError::DatabaseError(err.to_string())
    }
}

// Age using a buffered writer
impl From<std::io::Error> for CustomError {
    fn from(err: std::io::Error) -> CustomError {
        CustomError::FaultySetup(err.to_string())
    }
}

impl From<std::str::Utf8Error> for CustomError {
    fn from(err: std::str::Utf8Error) -> CustomError {
        CustomError::FaultySetup(err.to_string())
    }
}

impl From<std::num::ParseIntError> for CustomError {
    fn from(err: std::num::ParseIntError) -> CustomError {
        CustomError::FaultySetup(err.to_string())
    }
}

impl From<std::num::ParseFloatError> for CustomError {
    fn from(err: std::num::ParseFloatError) -> CustomError {
        CustomError::FaultySetup(err.to_string())
    }
}
