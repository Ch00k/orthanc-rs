use serde::{Deserialize, Serialize};
use std::fmt;
use std::str;

/// Structure of Orthanc's API error
#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub struct ApiError {
    pub method: String,
    pub uri: String,
    pub message: String,
    pub details: Option<String>,
    pub http_status: u16,
    pub http_error: String,
    pub orthanc_status: u16,
    pub orthanc_error: String,
}

/// Error type
#[derive(Debug, Eq, PartialEq)]
pub struct Error {
    pub message: String,
    // TODO: This is pretty ugly
    pub details: Option<ApiError>,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}: {:#?}", self.message, self.details)
    }
}

impl Error {
    pub(crate) fn new(msg: &str, api_error: Option<ApiError>) -> Error {
        Error {
            message: msg.to_string(),
            details: api_error,
        }
    }
}

impl From<reqwest::Error> for Error {
    fn from(e: reqwest::Error) -> Self {
        Error::new(&e.to_string(), None)
    }
}

impl From<serde_json::error::Error> for Error {
    fn from(e: serde_json::error::Error) -> Self {
        Error::new(&e.to_string(), None)
    }
}

impl From<str::Utf8Error> for Error {
    fn from(e: str::Utf8Error) -> Self {
        Error::new(&e.to_string(), None)
    }
}
