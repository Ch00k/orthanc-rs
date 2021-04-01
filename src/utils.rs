use crate::{Error, Result};
use bytes::Bytes;

pub(crate) fn check_http_error(status: reqwest::StatusCode, body: Bytes) -> Result<Bytes> {
    if status >= reqwest::StatusCode::BAD_REQUEST {
        let message = format!("API error: {}", status);
        if body.is_empty() {
            return Err(Error::new(&message, None));
        };
        return Err(Error::new(&message, serde_json::from_slice(&body)?));
    }
    Ok(body)
}

pub(crate) mod serde_datetime {
    use chrono::NaiveDateTime;
    use serde::{self, Deserialize, Deserializer, Serializer};

    const FORMAT: &str = "%Y%m%dT%H%M%S";
    const FORMAT_MKS: &str = "%Y%m%dT%H%M%S.%6f";

    pub fn serialize<S>(date: &NaiveDateTime, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        return serializer.serialize_str(&date.format(FORMAT).to_string());
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<NaiveDateTime, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        NaiveDateTime::parse_from_str(&s, FORMAT).or_else(|_| {
            NaiveDateTime::parse_from_str(&s, FORMAT_MKS).map_err(serde::de::Error::custom)
        })
    }
}

pub(crate) mod serde_datetime_optional {
    use chrono::NaiveDateTime;
    use serde::{self, Deserialize, Deserializer, Serializer};

    const FORMAT: &str = "%Y%m%dT%H%M%S.%6f";

    pub fn serialize<S>(
        date: &Option<NaiveDateTime>,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if let Some(ref d) = *date {
            return serializer.serialize_str(&d.format(FORMAT).to_string());
        }
        serializer.serialize_none()
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<NaiveDateTime>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: Option<String> = Option::deserialize(deserializer)?;
        if let Some(s) = s {
            return Ok(Some(
                NaiveDateTime::parse_from_str(&s, FORMAT)
                    .map_err(serde::de::Error::custom)?,
            ));
        }
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ApiError;

    #[test]
    fn test_check_http_error_ok() {
        let res =
            check_http_error(reqwest::StatusCode::PERMANENT_REDIRECT, Bytes::from("foo"));
        assert!(res.is_ok());
    }

    #[test]
    fn test_check_http_error_error() {
        let res = check_http_error(
            reqwest::StatusCode::BAD_REQUEST,
            Bytes::from(
                r#"
                    {
                        "Details" : "Cannot parse an invalid DICOM file (size: 12 bytes)",
                        "HttpError" : "Bad Request",
                        "HttpStatus" : 400,
                        "Message" : "Bad file format",
                        "Method" : "POST",
                        "OrthancError" : "Bad file format",
                        "OrthancStatus" : 15,
                        "Uri" : "/instances"
                    }
                "#,
            ),
        );
        assert_eq!(
            res.unwrap_err(),
            Error {
                message: "API error: 400 Bad Request".to_string(),
                details: Some(ApiError {
                    method: "POST".to_string(),
                    uri: "/instances".to_string(),
                    message: "Bad file format".to_string(),
                    details: Some(
                        "Cannot parse an invalid DICOM file (size: 12 bytes)".to_string()
                    ),
                    http_status: 400,
                    http_error: "Bad Request".to_string(),
                    orthanc_status: 15,
                    orthanc_error: "Bad file format".to_string(),
                },),
            },
        );
    }

    #[test]
    fn test_check_http_error_error_empty_body() {
        let res = check_http_error(reqwest::StatusCode::UNAUTHORIZED, Bytes::from(""));
        assert_eq!(
            res.unwrap_err(),
            Error {
                message: "API error: 401 Unauthorized".to_string(),
                details: None
            },
        );
    }

    // TODO: Firgure out how to handle this
    #[test]
    fn test_check_http_error_error_random_body() {
        let res = check_http_error(
            reqwest::StatusCode::GATEWAY_TIMEOUT,
            Bytes::from("foo bar baz"),
        );
        assert_eq!(
            res.unwrap_err(),
            Error {
                message: "expected ident at line 1 column 2".to_string(),
                details: None
            },
        );
    }
}
