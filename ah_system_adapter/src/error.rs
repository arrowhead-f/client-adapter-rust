use crate::dtos::ArrowheadServerException;

use std::error;
use std::fmt;
use std::result;

pub type Result<T> = result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    HttpError(String),
    ArrowheadError(ArrowheadServerException),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::HttpError(message) => write!(f, "Http error: {}", message),
            Self::ArrowheadError(ah_server_exception) => {
                write!(f, "Arrowhead error: {}", ah_server_exception.error_message)
            }
        }
    }
}

impl error::Error for Error {}

impl From<reqwest::Error> for Error {
    fn from(err: reqwest::Error) -> Self {
        Self::HttpError(format!("{}", err))
    }
}

impl From<url::ParseError> for Error {
    fn from(err: url::ParseError) -> Self {
        Self::HttpError(format!("{}", err))
    }
}
