use serde::{Deserialize, Serialize};
use std::{fmt, str};

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Client;
    use serde_json::{Error as SerdeError, Value};
    use std::str;

    #[test]
    fn test_error_formatting() {
        let error = Error {
            message: "400".to_string(),
            details: Some(ApiError {
                method: "POST".to_string(),
                uri: "/instances".to_string(),
                message: "Bad file format".to_string(),
                details: Some(
                    "Cannot parse an invalid DICOM file (size: 12 bytes)".to_string(),
                ),
                http_status: 400,
                http_error: "Bad Request".to_string(),
                orthanc_status: 15,
                orthanc_error: "Bad file format".to_string(),
            }),
        };

        // TODO: Any way to make the formatting nicer?
        let expected_error_str = r#"400: Some(
    ApiError {
        method: "POST",
        uri: "/instances",
        message: "Bad file format",
        details: Some(
            "Cannot parse an invalid DICOM file (size: 12 bytes)",
        ),
        http_status: 400,
        http_error: "Bad Request",
        orthanc_status: 15,
        orthanc_error: "Bad file format",
    },
)"#;
        assert_eq!(format!("{}", error), expected_error_str);
    }

    #[test]
    fn test_error_from_serde_json() {
        let serde_error: Result<Value, SerdeError> = serde_json::from_str("foobar");
        assert_eq!(
            Error::from(serde_error.unwrap_err()),
            Error {
                message: "expected ident at line 1 column 2".to_string(),
                details: None,
            },
        )
    }

    #[test]
    fn test_error_from_reqwest() {
        let cl = Client::new("http://foo").auth("foo", "bar");
        let resp = cl.patients();

        let expected_err = concat!(
            r#"error sending request for url (http://foo/patients): "#,
            r#"error trying to connect: dns error: "#,
            r#"failed to lookup address information: "#,
            r#"Temporary failure in name resolution"#,
        );
        assert_eq!(
            resp.unwrap_err(),
            Error {
                message: expected_err.to_string(),
                details: None,
            },
        );
    }

    #[test]
    fn test_error_from_utf8() {
        let sparkle_heart = vec![0, 159, 146, 150];
        let utf8_error = str::from_utf8(&sparkle_heart).unwrap_err();
        let orthanc_error = Error::from(utf8_error);
        assert_eq!(
            orthanc_error,
            Error {
                message: "invalid utf-8 sequence of 1 bytes from index 1".to_string(),
                details: None,
            }
        );
    }
}
