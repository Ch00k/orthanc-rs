use bytes::Bytes;
use chrono::NaiveDateTime;
use reqwest::blocking::{Client, RequestBuilder};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::fmt;
use std::result;
use std::str;

type Result<T> = result::Result<T, OrthancError>;

#[derive(Debug, Eq, PartialEq)]
pub struct OrthancError {
    pub details: String,
    // TODO: This is pretty ugly
    pub error_response: Option<ErrorResponse>,
}

impl fmt::Display for OrthancError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}: {:#?}", self.details, self.error_response)
    }
}

impl OrthancError {
    pub fn new(msg: &str, error_response: Option<ErrorResponse>) -> OrthancError {
        OrthancError {
            details: msg.to_string(),
            error_response,
        }
    }
}

impl From<reqwest::Error> for OrthancError {
    fn from(e: reqwest::Error) -> Self {
        OrthancError::new(&e.to_string(), None)
    }
}

impl From<serde_json::error::Error> for OrthancError {
    fn from(e: serde_json::error::Error) -> Self {
        OrthancError::new(&e.to_string(), None)
    }
}

impl From<str::Utf8Error> for OrthancError {
    fn from(e: str::Utf8Error) -> Self {
        OrthancError::new(&e.to_string(), None)
    }
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
pub enum EntityType {
    Patient,
    Study,
    Series,
    Instance,
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub struct Modality {
    #[serde(rename = "AET")]
    pub aet: String,
    pub host: String,
    pub port: u32,
    pub manufacturer: String,
    pub allow_echo: bool,
    pub allow_find: bool,
    pub allow_get: bool,
    pub allow_move: bool,
    pub allow_store: bool,
    pub allow_n_action: bool,
    pub allow_event_report: bool,
    pub allow_transcoding: bool,
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub struct Patient {
    #[serde(rename = "ID")]
    pub id: String,
    pub is_stable: bool,
    #[serde(with = "datetime_format")]
    pub last_update: NaiveDateTime,
    pub main_dicom_tags: HashMap<String, String>,
    pub studies: Vec<String>,
    #[serde(rename = "Type")]
    pub entity_type: EntityType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub anonymized_from: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub struct Study {
    #[serde(rename = "ID")]
    pub id: String,
    pub is_stable: bool,
    #[serde(with = "datetime_format")]
    pub last_update: NaiveDateTime,
    pub main_dicom_tags: HashMap<String, String>,
    pub parent_patient: String,
    pub patient_main_dicom_tags: HashMap<String, String>,
    pub series: Vec<String>,
    #[serde(rename = "Type")]
    pub entity_type: EntityType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub anonymized_from: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub struct Series {
    #[serde(rename = "ID")]
    pub id: String,
    pub status: String,
    pub is_stable: bool,
    #[serde(with = "datetime_format")]
    pub last_update: NaiveDateTime,
    pub main_dicom_tags: HashMap<String, String>,
    pub parent_study: String,
    pub expected_number_of_instances: Option<u32>,
    pub instances: Vec<String>,
    #[serde(rename = "Type")]
    pub entity_type: EntityType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub anonymized_from: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub struct Instance {
    #[serde(rename = "ID")]
    pub id: String,
    pub main_dicom_tags: HashMap<String, String>,
    pub parent_series: String,
    pub index_in_series: u32,
    pub file_uuid: String,
    pub file_size: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modified_from: Option<String>,
    #[serde(rename = "Type")]
    pub entity_type: EntityType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub anonymized_from: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub struct UploadStatusResponse {
    #[serde(rename = "ID")]
    pub id: String,
    pub status: String,
    pub path: String,
    pub parent_patient: String,
    pub parent_study: String,
    pub parent_series: String,
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub struct RemainingAncestor {
    #[serde(rename = "ID")]
    pub id: String,
    pub path: String,
    #[serde(rename = "Type")]
    pub entity_type: EntityType,
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub struct RemainingAncestorResponse {
    pub remaining_ancestor: Option<RemainingAncestor>,
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub struct StoreResponse {
    description: String,
    local_aet: String,
    remote_aet: String,
    parent_resources: Vec<String>,
    instances_count: u64,
    failed_instances_count: u64,
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub struct ErrorResponse {
    pub method: String,
    pub uri: String,
    pub message: String,
    pub details: Option<String>,
    pub http_status: u16,
    pub http_error: String,
    pub orthanc_status: u16,
    pub orthanc_error: String,
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub struct ModifyResponse {
    #[serde(rename = "ID")]
    pub id: String,
    #[serde(rename = "PatientID")]
    pub patient_id: String,
    pub path: String,
    #[serde(rename = "Type")]
    pub entity_type: EntityType,
}

#[derive(Serialize, Debug, Eq, PartialEq)]
#[serde(rename_all = "PascalCase")]
struct Anonymization {
    #[serde(skip_serializing_if = "Option::is_none")]
    replace: Option<HashMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    keep: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    keep_private_tags: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    dicom_version: Option<String>,
}

#[derive(Serialize, Debug, Eq, PartialEq)]
#[serde(rename_all = "PascalCase")]
struct Modification {
    #[serde(skip_serializing_if = "Option::is_none")]
    replace: Option<HashMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    remove: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    force: Option<bool>,
}

pub struct OrthancClient {
    server_address: String,
    username: Option<String>,
    password: Option<String>,
    client: Client,
}

impl OrthancClient {
    pub fn new(
        server_address: &str,
        username: Option<&str>,
        password: Option<&str>,
    ) -> OrthancClient {
        OrthancClient {
            server_address: server_address.to_string(),
            username: username.map(|u| u.to_string()),
            password: password.map(|p| p.to_string()),
            client: Client::new(),
        }
    }

    fn add_auth(&self, request: RequestBuilder) -> RequestBuilder {
        match (&self.username, &self.password) {
            (Some(u), Some(p)) => request.basic_auth(u, Some(p)),
            _ => request,
        }
    }

    fn get(&self, path: &str) -> Result<String> {
        let url = format!("{}/{}", self.server_address, &path);
        let mut request = self.client.get(&url);
        request = self.add_auth(request);
        let resp = request.send()?;
        let status = resp.status();
        let body = resp.text()?;

        if let Err(err) = check_http_error(status, &body) {
            return Err(err);
        }
        Ok(body)
    }

    fn get_bytes(&self, path: &str) -> Result<Bytes> {
        let url = format!("{}/{}", self.server_address, &path);
        let mut request = self.client.get(&url);
        request = self.add_auth(request);
        let resp = request.send()?;
        let status = resp.status();
        let body = resp.bytes()?;

        // TODO: This is not (unit-)testable due to the fact that
        // `Mock.return_body()` only accepts `str`, which is unicode.
        // Probably need to rethink HTTP error handling.
        let text = String::from_utf8_lossy(&body);

        if let Err(err) = check_http_error(status, &text) {
            return Err(err);
        }
        Ok(body)
    }

    // TODO: Can I make one function out of these two?
    fn post(&self, path: &str, data: Value) -> Result<String> {
        let url = format!("{}/{}", self.server_address, path);
        let mut request = self.client.post(&url).json(&data);
        request = self.add_auth(request);
        let resp = request.send()?;
        let status = resp.status();
        let body = resp.text()?;

        if let Err(err) = check_http_error(status, &body) {
            return Err(err);
        }
        Ok(body)
    }

    fn post_receive_bytes(&self, path: &str, data: Value) -> Result<Bytes> {
        let url = format!("{}/{}", self.server_address, path);
        let mut request = self.client.post(&url).json(&data);
        request = self.add_auth(request);
        let resp = request.send()?;
        let status = resp.status();
        let body = resp.bytes()?;
        let text = String::from_utf8_lossy(&body);

        if let Err(err) = check_http_error(status, &text) {
            return Err(err);
        }
        Ok(body)
    }

    fn post_bytes(&self, path: &str, data: &[u8]) -> Result<String> {
        let url = format!("{}/{}", self.server_address, path);
        // TODO: .to_vec() here is probably not a good idea
        let mut request = self.client.post(&url).body(data.to_vec());
        request = self.add_auth(request);
        let resp = request.send()?;
        let status = resp.status();
        let body = resp.text()?;

        if let Err(err) = check_http_error(status, &body) {
            return Err(err);
        }
        Ok(body)
    }

    fn delete(&self, path: &str) -> Result<String> {
        let url = format!("{}/{}", self.server_address, &path);
        let mut request = self.client.delete(&url);
        request = self.add_auth(request);
        let resp = request.send()?;
        let status = resp.status();
        let body = resp.text()?;

        if let Err(err) = check_http_error(status, &body) {
            return Err(err);
        }
        Ok(body)
    }

    fn list(&self, entity: &str) -> Result<Vec<String>> {
        let resp = self.get(entity)?;
        let json: Vec<String> = serde_json::from_str(&resp)?;
        Ok(json)
    }

    pub fn list_modalities(&self) -> Result<Vec<String>> {
        self.list("modalities")
    }

    pub fn list_patients(&self) -> Result<Vec<String>> {
        self.list("patients")
    }

    pub fn list_studies(&self) -> Result<Vec<String>> {
        self.list("studies")
    }

    pub fn list_series(&self) -> Result<Vec<String>> {
        self.list("series")
    }

    pub fn list_instances(&self) -> Result<Vec<String>> {
        self.list("instances")
    }

    pub fn list_modalities_expanded(&self) -> Result<HashMap<String, Modality>> {
        let resp = self.get("modalities?expand")?;
        let json: HashMap<String, Modality> = serde_json::from_str(&resp)?;
        Ok(json)
    }

    pub fn list_patients_expanded(&self) -> Result<Vec<Patient>> {
        let resp = self.get("patients?expand")?;
        let json: Vec<Patient> = serde_json::from_str(&resp)?;
        Ok(json)
    }

    pub fn list_studies_expanded(&self) -> Result<Vec<Study>> {
        let resp = self.get("studies?expand")?;
        let json: Vec<Study> = serde_json::from_str(&resp)?;
        Ok(json)
    }

    pub fn list_series_expanded(&self) -> Result<Vec<Series>> {
        let resp = self.get("series?expand")?;
        let json: Vec<Series> = serde_json::from_str(&resp)?;
        Ok(json)
    }

    pub fn list_instances_expanded(&self) -> Result<Vec<Instance>> {
        let resp = self.get("instances?expand")?;
        let json: Vec<Instance> = serde_json::from_str(&resp)?;
        Ok(json)
    }

    pub fn get_patient(&self, id: &str) -> Result<Patient> {
        let resp = self.get(&format!("patients/{}", id))?;
        let json: Patient = serde_json::from_str(&resp)?;
        Ok(json)
    }

    pub fn get_study(&self, id: &str) -> Result<Study> {
        let resp = self.get(&format!("studies/{}", id))?;
        let json: Study = serde_json::from_str(&resp)?;
        Ok(json)
    }

    pub fn get_series(&self, id: &str) -> Result<Series> {
        let resp = self.get(&format!("series/{}", id))?;
        let json: Series = serde_json::from_str(&resp)?;
        Ok(json)
    }

    pub fn get_instance(&self, id: &str) -> Result<Instance> {
        let resp = self.get(&format!("instances/{}", id))?;
        let json: Instance = serde_json::from_str(&resp)?;
        Ok(json)
    }

    pub fn get_instance_tags(&self, id: &str) -> Result<Value> {
        let resp = self.get(&format!("instances/{}/simplified-tags", id))?;
        let json: Value = serde_json::from_str(&resp)?;
        Ok(json)
    }

    pub fn get_instance_tags_expanded(&self, id: &str) -> Result<Value> {
        let resp = self.get(&format!("instances/{}/tags", id))?;
        let json: Value = serde_json::from_str(&resp)?;
        Ok(json)
    }

    pub fn get_patient_dicom(&self, id: &str) -> Result<Bytes> {
        let path = format!("patients/{}/archive", id);
        self.get_bytes(&path)
    }

    pub fn get_study_dicom(&self, id: &str) -> Result<Bytes> {
        let path = format!("studies/{}/archive", id);
        self.get_bytes(&path)
    }

    pub fn get_series_dicom(&self, id: &str) -> Result<Bytes> {
        let path = format!("series/{}/archive", id);
        self.get_bytes(&path)
    }

    pub fn get_instance_dicom(&self, id: &str) -> Result<Bytes> {
        let path = format!("instances/{}/file", id);
        self.get_bytes(&path)
    }

    pub fn delete_patient(&self, id: &str) -> Result<RemainingAncestorResponse> {
        let resp = self.delete(&format!("patients/{}", id))?;
        let json: RemainingAncestorResponse = serde_json::from_str(&resp)?;
        Ok(json)
    }

    pub fn delete_study(&self, id: &str) -> Result<RemainingAncestorResponse> {
        let resp = self.delete(&format!("studies/{}", id))?;
        let json: RemainingAncestorResponse = serde_json::from_str(&resp)?;
        Ok(json)
    }

    pub fn delete_series(&self, id: &str) -> Result<RemainingAncestorResponse> {
        let resp = self.delete(&format!("series/{}", id))?;
        let json: RemainingAncestorResponse = serde_json::from_str(&resp)?;
        Ok(json)
    }

    pub fn delete_instance(&self, id: &str) -> Result<RemainingAncestorResponse> {
        let resp = self.delete(&format!("instances/{}", id))?;
        let json: RemainingAncestorResponse = serde_json::from_str(&resp)?;
        Ok(json)
    }

    pub fn echo(&self, modality: &str, timeout: Option<u32>) -> Result<()> {
        let mut data = HashMap::new();
        // TODO: This does not seem idiomatic
        if timeout != None {
            data.insert("Timeout", timeout);
        }
        self.post(
            &format!("modalities/{}/echo", modality),
            serde_json::json!(data),
        )
        .map(|_| ())
    }

    pub fn store(&self, modality: &str, ids: &[&str]) -> Result<StoreResponse> {
        let resp = self.post(
            &format!("modalities/{}/store", modality),
            serde_json::json!(ids),
        )?;
        let json: StoreResponse = serde_json::from_str(&resp)?;
        Ok(json)
    }

    fn anonymize(
        &self,
        entity: &str,
        id: &str,
        replace: Option<HashMap<String, String>>,
        keep: Option<Vec<String>>,
        keep_private_tags: Option<bool>,
        dicom_version: Option<String>,
    ) -> Result<ModifyResponse> {
        let data = Anonymization {
            replace,
            keep,
            keep_private_tags,
            dicom_version,
        };
        let resp = self.post(
            &format!("{}/{}/anonymize", entity, id),
            serde_json::to_value(data)?,
        )?;
        let json: ModifyResponse = serde_json::from_str(&resp)?;
        Ok(json)
    }

    fn modify(
        &self,
        entity: &str,
        id: &str,
        replace: Option<HashMap<String, String>>,
        remove: Option<Vec<String>>,
        force: Option<bool>,
    ) -> Result<ModifyResponse> {
        let data = Modification {
            replace,
            remove,
            force,
        };
        let resp = self.post(
            &format!("{}/{}/modify", entity, id),
            serde_json::to_value(data)?,
        )?;
        let json: ModifyResponse = serde_json::from_str(&resp)?;
        Ok(json)
    }

    pub fn anonymize_patient(
        &self,
        id: &str,
        replace: Option<HashMap<String, String>>,
        keep: Option<Vec<String>>,
        keep_private_tags: Option<bool>,
        dicom_version: Option<String>,
    ) -> Result<ModifyResponse> {
        self.anonymize(
            "patients",
            id,
            replace,
            keep,
            keep_private_tags,
            dicom_version,
        )
    }

    pub fn anonymize_study(
        &self,
        id: &str,
        replace: Option<HashMap<String, String>>,
        keep: Option<Vec<String>>,
        keep_private_tags: Option<bool>,
        dicom_version: Option<String>,
    ) -> Result<ModifyResponse> {
        self.anonymize(
            "studies",
            id,
            replace,
            keep,
            keep_private_tags,
            dicom_version,
        )
    }

    pub fn anonymize_series(
        &self,
        id: &str,
        replace: Option<HashMap<String, String>>,
        keep: Option<Vec<String>>,
        keep_private_tags: Option<bool>,
        dicom_version: Option<String>,
    ) -> Result<ModifyResponse> {
        self.anonymize(
            "series",
            id,
            replace,
            keep,
            keep_private_tags,
            dicom_version,
        )
    }

    pub fn anonymize_instance(
        &self,
        id: &str,
        replace: Option<HashMap<String, String>>,
        keep: Option<Vec<String>>,
        keep_private_tags: Option<bool>,
        dicom_version: Option<String>,
    ) -> Result<Bytes> {
        let data = Anonymization {
            replace,
            keep,
            keep_private_tags,
            dicom_version,
        };
        let resp = self.post_receive_bytes(
            &format!("instances/{}/anonymize", id),
            serde_json::to_value(data)?,
        )?;
        Ok(resp)
    }

    pub fn modify_patient(
        &self,
        id: &str,
        replace: Option<HashMap<String, String>>,
        remove: Option<Vec<String>>,
    ) -> Result<ModifyResponse> {
        self.modify("patients", id, replace, remove, Some(true))
    }

    pub fn modify_study(
        &self,
        id: &str,
        replace: Option<HashMap<String, String>>,
        remove: Option<Vec<String>>,
    ) -> Result<ModifyResponse> {
        self.modify("studies", id, replace, remove, None)
    }

    pub fn modify_series(
        &self,
        id: &str,
        replace: Option<HashMap<String, String>>,
        remove: Option<Vec<String>>,
    ) -> Result<ModifyResponse> {
        self.modify("series", id, replace, remove, None)
    }

    pub fn modify_instance(
        &self,
        id: &str,
        replace: Option<HashMap<String, String>>,
        remove: Option<Vec<String>>,
    ) -> Result<Bytes> {
        let data = Modification {
            replace,
            remove,
            force: None,
        };
        let resp = self.post_receive_bytes(
            &format!("instances/{}/modify", id),
            serde_json::to_value(data)?,
        )?;
        Ok(resp)
    }

    pub fn upload_dicom(&self, data: &[u8]) -> Result<UploadStatusResponse> {
        let resp = self.post_bytes("instances", data)?;
        let json: UploadStatusResponse = serde_json::from_str(&resp)?;
        Ok(json)
    }
}

fn check_http_error(
    response_status: reqwest::StatusCode,
    response_body: &str,
) -> Result<()> {
    if response_status >= reqwest::StatusCode::BAD_REQUEST {
        if response_body.is_empty() {
            return Err(OrthancError::new(response_status.as_str(), None));
        };
        return Err(OrthancError::new(
            response_status.as_str(),
            serde_json::from_str(response_body)?,
        ));
    }
    Ok(())
}

mod datetime_format {
    use chrono::NaiveDateTime;
    use serde::{self, Deserialize, Deserializer, Serializer};

    const FORMAT: &str = "%Y%m%dT%H%M%S";

    pub fn serialize<S>(date: &NaiveDateTime, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = format!("{}", date.format(FORMAT));
        serializer.serialize_str(&s)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<NaiveDateTime, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        NaiveDateTime::parse_from_str(&s, FORMAT).map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;
    use httpmock::{Method, Mock, MockServer};
    use maplit::hashmap;

    #[test]
    fn test_error_formatting() {
        let error = OrthancError {
            details: "400".to_string(),
            error_response: Some(ErrorResponse {
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
    ErrorResponse {
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
    fn test_default_fields() {
        let cl = OrthancClient::new("http://localhost:8042", None, None);
        assert_eq!(cl.server_address, "http://localhost:8042");
        assert_eq!(cl.username, None);
        assert_eq!(cl.password, None);
    }

    #[test]
    fn test_auth() {
        let cl = OrthancClient::new("http://localhost:8042", Some("foo"), Some("bar"));
        assert_eq!(cl.username, Some("foo".to_string()));
        assert_eq!(cl.password, Some("bar".to_string()));
    }

    #[test]
    fn test_get() {
        let mock_server = MockServer::start();
        let url = mock_server.url("");

        let m = Mock::new()
            .expect_method(Method::GET)
            .expect_path("/foo")
            .expect_header("Authorization", "Basic Zm9vOmJhcg==")
            .return_status(200)
            .return_body("bar")
            .create_on(&mock_server);

        let cl = OrthancClient::new(&url, Some("foo"), Some("bar"));
        let resp = cl.get("foo").unwrap();

        assert_eq!(resp, "bar");
        assert_eq!(m.times_called(), 1);
    }

    #[test]
    fn test_get_bytes() {
        let mock_server = MockServer::start();
        let url = mock_server.url("");

        let m = Mock::new()
            .expect_method(Method::GET)
            .expect_path("/foo")
            .expect_header("Authorization", "Basic Zm9vOmJhcg==")
            .return_status(200)
            .return_body("bar")
            .create_on(&mock_server);

        let cl = OrthancClient::new(&url, Some("foo"), Some("bar"));
        let resp = cl.get_bytes("foo").unwrap();

        assert_eq!(resp, "bar");
        assert_eq!(m.times_called(), 1);
    }

    #[test]
    fn test_post() {
        let mock_server = MockServer::start();
        let url = mock_server.url("");

        let m = Mock::new()
            .expect_method(Method::POST)
            .expect_path("/foo")
            .expect_body("\"bar\"")
            .expect_header("Authorization", "Basic Zm9vOmJhcg==")
            .return_header("Content-Type", "application/json")
            .return_status(200)
            .return_body("baz")
            .create_on(&mock_server);

        let cl = OrthancClient::new(&url, Some("foo"), Some("bar"));
        let resp = cl.post("foo", serde_json::json!("bar")).unwrap();

        assert_eq!(resp, "baz");
        assert_eq!(m.times_called(), 1);
    }

    #[test]
    fn test_post_bytes() {
        let mock_server = MockServer::start();
        let url = mock_server.url("");

        let m = Mock::new()
            .expect_method(Method::POST)
            .expect_path("/foo")
            .expect_body("bar")
            .expect_header("Authorization", "Basic Zm9vOmJhcg==")
            .return_header("Content-Type", "application/json")
            .return_status(200)
            .return_body("baz")
            .create_on(&mock_server);

        let cl = OrthancClient::new(&url, Some("foo"), Some("bar"));
        let resp = cl.post_bytes("foo", "bar".as_bytes()).unwrap();

        assert_eq!(resp, "baz");
        assert_eq!(m.times_called(), 1);
    }

    #[test]
    fn test_delete() {
        let mock_server = MockServer::start();
        let url = mock_server.url("");

        let m = Mock::new()
            .expect_method(Method::DELETE)
            .expect_path("/foo")
            .expect_header("Authorization", "Basic Zm9vOmJhcg==")
            .return_status(200)
            .create_on(&mock_server);

        let cl = OrthancClient::new(&url, Some("foo"), Some("bar"));
        let resp = cl.delete("foo").unwrap();

        assert_eq!(resp, "");
        assert_eq!(m.times_called(), 1);
    }

    #[test]
    fn test_get_error_response() {
        let mock_server = MockServer::start();
        let url = mock_server.url("");

        let m = Mock::new()
            .expect_method(Method::GET)
            .expect_path("/foo")
            .return_status(400)
            .return_body(
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
            )
            .create_on(&mock_server);

        let cl = OrthancClient::new(&url, Some("foo"), Some("bar"));
        let resp = cl.get("foo");

        assert_eq!(
            resp.unwrap_err(),
            OrthancError {
                details: "400".to_string(),
                error_response: Some(ErrorResponse {
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
        assert_eq!(m.times_called(), 1);
    }

    #[test]
    fn test_get_bytes_error_response() {
        let mock_server = MockServer::start();
        let url = mock_server.url("");

        let m = Mock::new()
            .expect_method(Method::GET)
            .expect_path("/foo")
            .return_status(400)
            .return_body(
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
            )
            .create_on(&mock_server);

        let cl = OrthancClient::new(&url, Some("foo"), Some("bar"));
        let resp = cl.get_bytes("foo");

        assert_eq!(
            resp.unwrap_err(),
            OrthancError {
                details: "400".to_string(),
                error_response: Some(ErrorResponse {
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
        assert_eq!(m.times_called(), 1);
    }

    #[test]
    fn test_post_error_response() {
        let mock_server = MockServer::start();
        let url = mock_server.url("");

        let m = Mock::new()
            .expect_method(Method::POST)
            .expect_path("/foo")
            .return_status(400)
            .return_body(
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
            )
            .create_on(&mock_server);

        let cl = OrthancClient::new(&url, Some("foo"), Some("bar"));
        let resp = cl.post("foo", serde_json::json!("bar"));

        assert_eq!(
            resp.unwrap_err(),
            OrthancError {
                details: "400".to_string(),
                error_response: Some(ErrorResponse {
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
        assert_eq!(m.times_called(), 1);
    }

    #[test]
    fn test_post_bytes_error_response() {
        let mock_server = MockServer::start();
        let url = mock_server.url("");

        let m = Mock::new()
            .expect_method(Method::POST)
            .expect_path("/foo")
            .return_status(400)
            .return_body(
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
            )
            .create_on(&mock_server);

        let cl = OrthancClient::new(&url, Some("foo"), Some("bar"));
        let resp = cl.post_bytes("foo", &[13, 42, 17]);

        assert_eq!(
            resp.unwrap_err(),
            OrthancError {
                details: "400".to_string(),
                error_response: Some(ErrorResponse {
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
        assert_eq!(m.times_called(), 1);
    }

    #[test]
    fn test_delete_error_response() {
        let mock_server = MockServer::start();
        let url = mock_server.url("");

        let m = Mock::new()
            .expect_method(Method::DELETE)
            .expect_path("/foo")
            .return_status(400)
            .return_body(
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
            )
            .create_on(&mock_server);

        let cl = OrthancClient::new(&url, Some("foo"), Some("bar"));
        let resp = cl.delete("foo");

        assert_eq!(
            resp.unwrap_err(),
            OrthancError {
                details: "400".to_string(),
                error_response: Some(ErrorResponse {
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
        assert_eq!(m.times_called(), 1);
    }

    #[test]
    fn test_error_response_no_body() {
        let mock_server = MockServer::start();
        let url = mock_server.url("");

        let m = Mock::new()
            .expect_method(Method::GET)
            .expect_path("/foo")
            .return_status(404)
            .create_on(&mock_server);

        let cl = OrthancClient::new(&url, Some("foo"), Some("bar"));
        let resp = cl.get("foo");

        assert!(resp.is_err());
        assert_eq!(
            resp.unwrap_err(),
            OrthancError {
                details: "404".to_string(),
                error_response: None,
            },
        );
        assert_eq!(m.times_called(), 1);
    }

    #[test]
    fn test_list_modalities() {
        let mock_server = MockServer::start();
        let url = mock_server.url("");

        let m = Mock::new()
            .expect_method(Method::GET)
            .expect_path("/modalities")
            .return_status(200)
            .return_header("Content-Type", "application/json")
            .return_body(r#"["foo", "bar", "baz"]"#)
            .create_on(&mock_server);

        let cl = OrthancClient::new(&url, None, None);
        let modalities = cl.list_modalities().unwrap();

        assert_eq!(modalities, ["foo", "bar", "baz"]);
        assert_eq!(m.times_called(), 1);
    }

    #[test]
    fn test_list_patients() {
        let mock_server = MockServer::start();
        let url = mock_server.url("");

        let m = Mock::new()
            .expect_method(Method::GET)
            .expect_path("/patients")
            .return_status(200)
            .return_header("Content-Type", "application/json")
            .return_body(r#"["foo", "bar", "baz"]"#)
            .create_on(&mock_server);

        let cl = OrthancClient::new(&url, None, None);
        let patient_ids = cl.list_patients().unwrap();

        assert_eq!(patient_ids, ["foo", "bar", "baz"]);
        assert_eq!(m.times_called(), 1);
    }

    #[test]
    fn test_list_studies() {
        let mock_server = MockServer::start();
        let url = mock_server.url("");

        let m = Mock::new()
            .expect_method(Method::GET)
            .expect_path("/studies")
            .return_status(200)
            .return_header("Content-Type", "application/json")
            .return_body(r#"["foo", "bar", "baz"]"#)
            .create_on(&mock_server);

        let cl = OrthancClient::new(&url, None, None);
        let patient_ids = cl.list_studies().unwrap();

        assert_eq!(patient_ids, ["foo", "bar", "baz"]);
        assert_eq!(m.times_called(), 1);
    }

    #[test]
    fn test_list_series() {
        let mock_server = MockServer::start();
        let url = mock_server.url("");

        let m = Mock::new()
            .expect_method(Method::GET)
            .expect_path("/series")
            .return_status(200)
            .return_header("Content-Type", "application/json")
            .return_body(r#"["foo", "bar", "baz"]"#)
            .create_on(&mock_server);

        let cl = OrthancClient::new(&url, None, None);
        let patient_ids = cl.list_series().unwrap();

        assert_eq!(patient_ids, ["foo", "bar", "baz"]);
        assert_eq!(m.times_called(), 1);
    }

    #[test]
    fn test_list_instances() {
        let mock_server = MockServer::start();
        let url = mock_server.url("");

        let m = Mock::new()
            .expect_method(Method::GET)
            .expect_path("/instances")
            .return_status(200)
            .return_header("Content-Type", "application/json")
            .return_body(r#"["foo", "bar", "baz"]"#)
            .create_on(&mock_server);

        let cl = OrthancClient::new(&url, None, None);
        let patient_ids = cl.list_instances().unwrap();

        assert_eq!(patient_ids, ["foo", "bar", "baz"]);
        assert_eq!(m.times_called(), 1);
    }

    #[test]
    fn test_list_modalities_expanded() {
        let mock_server = MockServer::start();
        let url = mock_server.url("");

        let m = Mock::new()
            .expect_method(Method::GET)
            .expect_path("/modalities")
            .expect_query_param_exists("expand")
            .return_status(200)
            .return_header("Content-Type", "application/json")
            .return_body(
                r#"
                    {
                        "foo": {
                            "AET": "FOO",
                            "AllowEcho": true,
                            "AllowFind": true,
                            "AllowGet": true,
                            "AllowMove": true,
                            "AllowStore": true,
                            "AllowNAction": false,
                            "AllowEventReport": false,
                            "AllowTranscoding": false,
                            "Host": "localhost",
                            "Manufacturer": "Generic",
                            "Port": 11114
                        },
                        "bar": {
                            "AET": "BAR",
                            "AllowEcho": true,
                            "AllowFind": true,
                            "AllowGet": true,
                            "AllowMove": true,
                            "AllowStore": true,
                            "AllowNAction": false,
                            "AllowEventReport": false,
                            "AllowTranscoding": false,
                            "Host": "remotehost",
                            "Manufacturer": "Generic",
                            "Port": 11113
                        }
                    }
            "#,
            )
            .create_on(&mock_server);

        let cl = OrthancClient::new(&url, None, None);
        let modalities = cl.list_modalities_expanded().unwrap();

        assert_eq!(
            modalities,
            hashmap! {
                "foo".to_string() => Modality {
                    aet: "FOO".to_string(),
                    host: "localhost".to_string(),
                    port: 11114,
                    manufacturer: "Generic".to_string(),
                    allow_echo: true,
                    allow_find: true,
                    allow_get: true,
                    allow_move: true,
                    allow_store: true,
                    allow_n_action: false,
                    allow_event_report: false,
                    allow_transcoding: false,
                },
                "bar".to_string() => Modality {
                    aet: "BAR".to_string(),
                    host: "remotehost".to_string(),
                    port: 11113,
                    manufacturer: "Generic".to_string(),
                    allow_echo: true,
                    allow_find: true,
                    allow_get: true,
                    allow_move: true,
                    allow_store: true,
                    allow_n_action: false,
                    allow_event_report: false,
                    allow_transcoding: false,
                }
            },
        );
        assert_eq!(m.times_called(), 1);
    }

    #[test]
    fn test_list_patients_expanded() {
        let mock_server = MockServer::start();
        let url = mock_server.url("");

        let m = Mock::new()
            .expect_method(Method::GET)
            .expect_path("/patients")
            .expect_query_param_exists("expand")
            .return_status(200)
            .return_header("Content-Type", "application/json")
            .return_body(
                r#"
                    [
                        {
                            "ID": "f88cbd3f-a00dfc59-9ca1ac2d-7ce9851a-40e5b493",
                            "IsStable": true,
                            "LastUpdate": "20200101T154617",
                            "MainDicomTags": {
                                "OtherPatientIDs": "",
                                "PatientBirthDate": "19670101",
                                "PatientID": "123456789",
                                "PatientName": "Rick Sanchez",
                                "PatientSex": "M"
                            },
                            "Studies": [
                                "e8cafcbe-caf08c39-6e205f15-18554bb8-b3f9ef04"
                            ],
                            "Type": "Patient"
                        },
                        {
                            "ID": "7e43f8d3-e50280e6-470079e9-02241af1-d286bdbe",
                            "IsStable": true,
                            "LastUpdate": "20200826T174531",
                            "MainDicomTags": {
                                "OtherPatientIDs": "",
                                "PatientBirthDate": "19440101",
                                "PatientID": "987654321",
                                "PatientName": "Morty Smith"
                            },
                            "Studies": [
                                "63bf5d42-b5382159-01971752-e0ceea3d-399bbca5"
                            ],
                            "Type": "Patient"
                        }
                    ]
               "#,
            )
            .create_on(&mock_server);

        let cl = OrthancClient::new(&url, None, None);
        let patients = cl.list_patients_expanded().unwrap();

        assert_eq!(
            patients,
            [
                Patient {
                    id: "f88cbd3f-a00dfc59-9ca1ac2d-7ce9851a-40e5b493".to_string(),
                    is_stable: true,
                    last_update: NaiveDate::from_ymd(2020, 1, 1).and_hms(15, 46, 17),
                    main_dicom_tags: hashmap! {
                        "OtherPatientIDs".to_string() => "".to_string(),
                        "PatientBirthDate".to_string() => "19670101".to_string(),
                        "PatientID".to_string() => "123456789".to_string(),
                        "PatientName".to_string() => "Rick Sanchez".to_string(),
                        "PatientSex".to_string() => "M".to_string()
                    },
                    studies: ["e8cafcbe-caf08c39-6e205f15-18554bb8-b3f9ef04".to_string()]
                        .to_vec(),
                    entity_type: EntityType::Patient,
                    anonymized_from: None
                },
                Patient {
                    id: "7e43f8d3-e50280e6-470079e9-02241af1-d286bdbe".to_string(),
                    is_stable: true,
                    last_update: NaiveDate::from_ymd(2020, 8, 26).and_hms(17, 45, 31),
                    main_dicom_tags: hashmap! {
                        "OtherPatientIDs".to_string() => "".to_string(),
                        "PatientBirthDate".to_string() => "19440101".to_string(),
                        "PatientID".to_string() => "987654321".to_string(),
                        "PatientName".to_string() => "Morty Smith".to_string(),
                    },
                    studies: ["63bf5d42-b5382159-01971752-e0ceea3d-399bbca5".to_string()]
                        .to_vec(),
                    entity_type: EntityType::Patient,
                    anonymized_from: None
                },
            ]
        );
        assert_eq!(m.times_called(), 1);
    }

    #[test]
    fn test_list_studies_expanded() {
        let mock_server = MockServer::start();
        let url = mock_server.url("");

        let m = Mock::new()
            .expect_method(Method::GET)
            .expect_path("/studies")
            .expect_query_param_exists("expand")
            .return_status(200)
            .return_header("Content-Type", "application/json")
            .return_body(
                r#"
                    [
                        {
                            "ID": "63bf5d42-b5382159-01971752-e0ceea3d-399bbca5",
                            "IsStable": true,
                            "LastUpdate": "20200830T191109",
                            "MainDicomTags": {
                                "AccessionNumber": "foobar",
                                "StudyDate": "20110101",
                                "StudyDescription": "Brain",
                                "StudyID": "1742",
                                "StudyInstanceUID": "1.2.3.4.5.6789",
                                "StudyTime": "084707"
                            },
                            "ParentPatient": "7e43f8d3-e50280e6-470079e9-02241af1-d286bdbe",
                            "PatientMainDicomTags": {
                                "PatientBirthDate": "19440101",
                                "PatientID": "c137",
                                "PatientName": "Rick Sanchez",
                                "PatientSex": "M"
                            },
                            "Series": [
                                "cd00fffc-db25be29-0c6da430-c56796a5-ba06933c",
                                "2ab7dbe7-f1a18a78-86145443-18a8ff93-0b65f2b2"
                            ],
                            "Type": "Study"
                        },
                        {
                            "ID": "e8cafcbe-caf08c39-6e205f15-18554bb8-b3f9ef04",
                            "IsStable": true,
                            "LastUpdate": "20200901T185211",
                            "MainDicomTags": {
                                "AccessionNumber": "bazqux",
                                "StudyDate": "20120101",
                                "StudyDescription": "Knee",
                                "StudyID": "1010100",
                                "StudyInstanceUID": "1.2.3.4.5.67810",
                                "StudyTime": "130431"
                            },
                            "ParentPatient": "f88cbd3f-a00dfc59-9ca1ac2d-7ce9851a-40e5b493",
                            "PatientMainDicomTags": {
                                "PatientBirthDate": "19670101",
                                "PatientID": "4217",
                                "PatientName": "Summer Smith",
                                "PatientSex": "F"
                            },
                            "Series": [
                                "222bbd7e-4dfbc5a8-ea58f933-f1747134-0810c7c8",
                                "54f8778a-75ba559c-db7c7c1a-c1056140-ef74d487"
                            ],
                            "Type": "Study"
                        }
                    ]
               "#,
            )
            .create_on(&mock_server);

        let cl = OrthancClient::new(&url, None, None);
        let studies = cl.list_studies_expanded().unwrap();

        assert_eq!(
            studies,
            [
                Study {
                    id: "63bf5d42-b5382159-01971752-e0ceea3d-399bbca5".to_string(),
                    is_stable: true,
                    last_update: NaiveDate::from_ymd(2020, 8, 30).and_hms(19, 11, 09),
                    main_dicom_tags: hashmap! {
                        "AccessionNumber".to_string() => "foobar".to_string(),
                        "StudyDate".to_string() => "20110101".to_string(),
                        "StudyDescription".to_string() => "Brain".to_string(),
                        "StudyID".to_string() => "1742".to_string(),
                        "StudyInstanceUID".to_string() => "1.2.3.4.5.6789".to_string(),
                        "StudyTime".to_string() => "084707".to_string()
                    },
                    parent_patient: "7e43f8d3-e50280e6-470079e9-02241af1-d286bdbe"
                        .to_string(),
                    patient_main_dicom_tags: hashmap! {
                        "PatientBirthDate".to_string() => "19440101".to_string(),
                        "PatientID".to_string() => "c137".to_string(),
                        "PatientName".to_string() => "Rick Sanchez".to_string(),
                        "PatientSex".to_string() => "M".to_string(),
                    },
                    series: [
                        "cd00fffc-db25be29-0c6da430-c56796a5-ba06933c".to_string(),
                        "2ab7dbe7-f1a18a78-86145443-18a8ff93-0b65f2b2".to_string()
                    ]
                    .to_vec(),
                    entity_type: EntityType::Study,
                    anonymized_from: None
                },
                Study {
                    id: "e8cafcbe-caf08c39-6e205f15-18554bb8-b3f9ef04".to_string(),
                    is_stable: true,
                    last_update: NaiveDate::from_ymd(2020, 9, 1).and_hms(18, 52, 11),
                    main_dicom_tags: hashmap! {
                        "AccessionNumber".to_string() => "bazqux".to_string(),
                        "StudyDate".to_string() => "20120101".to_string(),
                        "StudyDescription".to_string() => "Knee".to_string(),
                        "StudyID".to_string() => "1010100".to_string(),
                        "StudyInstanceUID".to_string() => "1.2.3.4.5.67810".to_string(),
                        "StudyTime".to_string() => "130431".to_string()
                    },
                    parent_patient: "f88cbd3f-a00dfc59-9ca1ac2d-7ce9851a-40e5b493"
                        .to_string(),
                    patient_main_dicom_tags: hashmap! {
                        "PatientBirthDate".to_string() => "19670101".to_string(),
                        "PatientID".to_string() => "4217".to_string(),
                        "PatientName".to_string() => "Summer Smith".to_string(),
                        "PatientSex".to_string() => "F".to_string(),
                    },
                    series: [
                        "222bbd7e-4dfbc5a8-ea58f933-f1747134-0810c7c8".to_string(),
                        "54f8778a-75ba559c-db7c7c1a-c1056140-ef74d487".to_string()
                    ]
                    .to_vec(),
                    entity_type: EntityType::Study,
                    anonymized_from: None
                },
            ]
        );
        assert_eq!(m.times_called(), 1);
    }

    #[test]
    fn test_list_seies_expanded() {
        let mock_server = MockServer::start();
        let url = mock_server.url("");

        let m = Mock::new()
            .expect_method(Method::GET)
            .expect_path("/series")
            .expect_query_param_exists("expand")
            .return_status(200)
            .return_header("Content-Type", "application/json")
            .return_body(
                r#"
                    [
                        {
                            "ExpectedNumberOfInstances": 17,
                            "ID": "cd00fffc-db25be29-0c6da430-c56796a5-ba06933c",
                            "Instances": [
                                "556530b5-de7c487b-110b9d0e-12cfdbb9-f06b546e",
                                "c46605db-836489fa-cb55fbbc-13c8a913-b0bad6ac",
                                "9b63498d-cae4f25e-f52206b2-cbb4dc0e-dc55c788"
                            ],
                            "IsStable": true,
                            "LastUpdate": "20200830T191109",
                            "MainDicomTags": {
                                "BodyPartExamined": "ABDOMEN",
                                "Modality": "MR",
                                "ProtocolName": "TCP",
                                "SeriesDate": "20110101",
                                "SeriesInstanceUID": "1.2.3.4.5.6789",
                                "SeriesNumber": "1101",
                                "SeriesTime": "091313.93"
                            },
                            "ParentStudy": "63bf5d42-b5382159-01971752-e0ceea3d-399bbca5",
                            "Status": "Unknown",
                            "Type": "Series"
                        },
                        {
                            "ExpectedNumberOfInstances": null,
                            "ID": "2ab7dbe7-f1a18a78-86145443-18a8ff93-0b65f2b2",
                            "Instances": [
                                "a17e85ca-380bb2dc-d29ea4b7-6e73c10a-ca6ba458",
                                "1c81e7e8-30642777-ffc2ca41-c7536670-7ad68124"
                            ],
                            "IsStable": true,
                            "LastUpdate": "20200830T191109",
                            "MainDicomTags": {
                                "BodyPartExamined": "HEAD",
                                "Modality": "CT",
                                "ProtocolName": "UDP",
                                "SeriesDate": "20110101",
                                "SeriesInstanceUID": "1.2.3.4.5.67810",
                                "SeriesNumber": "1102",
                                "SeriesTime": "091313.93"
                            },
                            "ParentStudy": "63bf5d42-b5382159-01971752-e0ceea3d-399bbca5",
                            "Status": "Unknown",
                            "Type": "Series"
                        }
                    ]
               "#,
            )
            .create_on(&mock_server);

        let cl = OrthancClient::new(&url, None, None);
        let series = cl.list_series_expanded().unwrap();

        assert_eq!(
            series,
            [
                Series {
                    id: "cd00fffc-db25be29-0c6da430-c56796a5-ba06933c".to_string(),
                    status: "Unknown".to_string(),
                    is_stable: true,
                    last_update: NaiveDate::from_ymd(2020, 8, 30).and_hms(19, 11, 09),
                    main_dicom_tags: hashmap! {
                        "BodyPartExamined".to_string() => "ABDOMEN".to_string(),
                        "Modality".to_string() => "MR".to_string(),
                        "ProtocolName".to_string() => "TCP".to_string(),
                        "SeriesDate".to_string() => "20110101".to_string(),
                        "SeriesInstanceUID".to_string() => "1.2.3.4.5.6789".to_string(),
                        "SeriesNumber".to_string() => "1101".to_string(),
                        "SeriesTime".to_string() => "091313.93".to_string(),

                    },
                    parent_study: "63bf5d42-b5382159-01971752-e0ceea3d-399bbca5"
                        .to_string(),
                    expected_number_of_instances: Some(17),
                    instances: [
                        "556530b5-de7c487b-110b9d0e-12cfdbb9-f06b546e".to_string(),
                        "c46605db-836489fa-cb55fbbc-13c8a913-b0bad6ac".to_string(),
                        "9b63498d-cae4f25e-f52206b2-cbb4dc0e-dc55c788".to_string(),
                    ]
                    .to_vec(),
                    entity_type: EntityType::Series,
                    anonymized_from: None
                },
                Series {
                    id: "2ab7dbe7-f1a18a78-86145443-18a8ff93-0b65f2b2".to_string(),
                    status: "Unknown".to_string(),
                    is_stable: true,
                    last_update: NaiveDate::from_ymd(2020, 8, 30).and_hms(19, 11, 09),
                    main_dicom_tags: hashmap! {
                        "BodyPartExamined".to_string() => "HEAD".to_string(),
                        "Modality".to_string() => "CT".to_string(),
                        "ProtocolName".to_string() => "UDP".to_string(),
                        "SeriesDate".to_string() => "20110101".to_string(),
                        "SeriesInstanceUID".to_string() => "1.2.3.4.5.67810".to_string(),
                        "SeriesNumber".to_string() => "1102".to_string(),
                        "SeriesTime".to_string() => "091313.93".to_string(),

                    },
                    parent_study: "63bf5d42-b5382159-01971752-e0ceea3d-399bbca5"
                        .to_string(),
                    expected_number_of_instances: None,
                    instances: [
                        "a17e85ca-380bb2dc-d29ea4b7-6e73c10a-ca6ba458".to_string(),
                        "1c81e7e8-30642777-ffc2ca41-c7536670-7ad68124".to_string(),
                    ]
                    .to_vec(),
                    entity_type: EntityType::Series,
                    anonymized_from: None
                },
            ]
        );
        assert_eq!(m.times_called(), 1);
    }

    #[test]
    fn test_list_instances_expanded() {
        let mock_server = MockServer::start();
        let url = mock_server.url("");

        let m = Mock::new()
            .expect_method(Method::GET)
            .expect_path("/instances")
            .expect_query_param_exists("expand")
            .return_status(200)
            .return_header("Content-Type", "application/json")
            .return_body(
                r#"
                    [
                        {
                            "FileSize": 139402,
                            "FileUuid": "d8c5eff3-986c-4fe4-b06e-7e52b2a4238e",
                            "ID": "29fa4d9d-51a69d1d-70e2b29a-fd824316-50850d0c",
                            "IndexInSeries": 13,
                            "MainDicomTags": {
                                "ImageOrientationPatient": "1\\0\\0\\0\\1\\0",
                                "ImagePositionPatient": "-17\\42\\13",
                                "InstanceCreationDate": "20130326",
                                "InstanceCreationTime": "135901",
                                "InstanceNumber": "13",
                                "SOPInstanceUID": "1.2.3.4.5.6789"
                            },
                            "ModifiedFrom": "22c54cb6-28302a69-3ff454a3-676b98f4-b84cd80a",
                            "ParentSeries": "82081568-b6f8f4e6-ced76876-6504da25-ed0dfe03",
                            "Type": "Instance"
                        },
                        {
                            "FileSize": 381642,
                            "FileUuid": "86bbad65-2c98-4cb0-bf77-0ef0243410a4",
                            "ID": "286a251e-46571bd6-0e14ab9a-1baadddc-d0146ea0",
                            "IndexInSeries": 75,
                            "MainDicomTags": {
                                "ImageOrientationPatient": "-1\\0\\0\\0\\1\\0",
                                "ImagePositionPatient": "-17\\42\\14",
                                "InstanceCreationDate": "20130326",
                                "InstanceCreationTime": "135830",
                                "InstanceNumber": "75",
                                "SOPInstanceUID": "1.2.3.4.5.67810"
                            },
                            "ParentSeries": "a240e0d7-538699a0-7464bb4b-a906f72a-fa3a32c7",
                            "Type": "Instance"
                        }
                    ]
               "#,
            )
            .create_on(&mock_server);

        let cl = OrthancClient::new(&url, None, None);
        let instances = cl.list_instances_expanded().unwrap();

        assert_eq!(
            instances,
            [
                Instance {
                    id: "29fa4d9d-51a69d1d-70e2b29a-fd824316-50850d0c".to_string(),
                    main_dicom_tags: hashmap! {
                        "ImageOrientationPatient".to_string() => "1\\0\\0\\0\\1\\0".to_string(),
                        "ImagePositionPatient".to_string() => "-17\\42\\13".to_string(),
                        "InstanceCreationDate".to_string() => "20130326".to_string(),
                        "InstanceCreationTime".to_string() => "135901".to_string(),
                        "InstanceNumber".to_string() => "13".to_string(),
                        "SOPInstanceUID".to_string() => "1.2.3.4.5.6789".to_string(),
                    },
                    parent_series: "82081568-b6f8f4e6-ced76876-6504da25-ed0dfe03"
                        .to_string(),
                    index_in_series: 13,
                    file_uuid: "d8c5eff3-986c-4fe4-b06e-7e52b2a4238e".to_string(),
                    file_size: 139402,
                    modified_from: Some(
                        "22c54cb6-28302a69-3ff454a3-676b98f4-b84cd80a".to_string()
                    ),
                    entity_type: EntityType::Instance,
                    anonymized_from: None
                },
                Instance {
                    id: "286a251e-46571bd6-0e14ab9a-1baadddc-d0146ea0".to_string(),
                    main_dicom_tags: hashmap! {
                        "ImageOrientationPatient".to_string() => "-1\\0\\0\\0\\1\\0".to_string(),
                        "ImagePositionPatient".to_string() => "-17\\42\\14".to_string(),
                        "InstanceCreationDate".to_string() => "20130326".to_string(),
                        "InstanceCreationTime".to_string() => "135830".to_string(),
                        "InstanceNumber".to_string() => "75".to_string(),
                        "SOPInstanceUID".to_string() => "1.2.3.4.5.67810".to_string(),
                    },
                    parent_series: "a240e0d7-538699a0-7464bb4b-a906f72a-fa3a32c7"
                        .to_string(),
                    index_in_series: 75,
                    file_uuid: "86bbad65-2c98-4cb0-bf77-0ef0243410a4".to_string(),
                    file_size: 381642,
                    modified_from: None,
                    entity_type: EntityType::Instance,
                    anonymized_from: None
                },
            ]
        );
        assert_eq!(m.times_called(), 1);
    }

    #[test]
    fn test_get_patient() {
        let mock_server = MockServer::start();
        let url = mock_server.url("");

        let m = Mock::new()
            .expect_method(Method::GET)
            .expect_path("/patients/foo")
            .return_status(200)
            .return_header("Content-Type", "application/json")
            .return_body(
                r#"
                    {
                        "ID": "f88cbd3f-a00dfc59-9ca1ac2d-7ce9851a-40e5b493",
                        "IsStable": true,
                        "LastUpdate": "20200101T154617",
                        "MainDicomTags": {
                            "OtherPatientIDs": "",
                            "PatientBirthDate": "19670101",
                            "PatientID": "123456789",
                            "PatientName": "Rick Sanchez",
                            "PatientSex": "M"
                        },
                        "Studies": [
                            "e8cafcbe-caf08c39-6e205f15-18554bb8-b3f9ef04"
                        ],
                        "Type": "Patient"
                    }
                "#,
            )
            .create_on(&mock_server);

        let cl = OrthancClient::new(&url, None, None);
        let patient = cl.get_patient("foo").unwrap();

        assert_eq!(
            patient,
            Patient {
                id: "f88cbd3f-a00dfc59-9ca1ac2d-7ce9851a-40e5b493".to_string(),
                is_stable: true,
                last_update: NaiveDate::from_ymd(2020, 1, 1).and_hms(15, 46, 17),
                main_dicom_tags: hashmap! {
                    "OtherPatientIDs".to_string() => "".to_string(),
                    "PatientBirthDate".to_string() => "19670101".to_string(),
                    "PatientID".to_string() => "123456789".to_string(),
                    "PatientName".to_string() => "Rick Sanchez".to_string(),
                    "PatientSex".to_string() => "M".to_string()
                },
                studies: ["e8cafcbe-caf08c39-6e205f15-18554bb8-b3f9ef04".to_string()]
                    .to_vec(),
                entity_type: EntityType::Patient,
                anonymized_from: None
            },
        );
        assert_eq!(m.times_called(), 1);
    }

    #[test]
    fn test_get_study() {
        let mock_server = MockServer::start();
        let url = mock_server.url("");

        let m = Mock::new()
            .expect_method(Method::GET)
            .expect_path("/studies/foo")
            .return_status(200)
            .return_header("Content-Type", "application/json")
            .return_body(
                r#"
                    {
                        "ID": "63bf5d42-b5382159-01971752-e0ceea3d-399bbca5",
                        "IsStable": true,
                        "LastUpdate": "20200830T191109",
                        "MainDicomTags": {
                            "AccessionNumber": "foobar",
                            "StudyDate": "20110101",
                            "StudyDescription": "Brain",
                            "StudyID": "1742",
                            "StudyInstanceUID": "1.2.3.4.5.6789",
                            "StudyTime": "084707"
                        },
                        "ParentPatient": "7e43f8d3-e50280e6-470079e9-02241af1-d286bdbe",
                        "PatientMainDicomTags": {
                            "PatientBirthDate": "19440101",
                            "PatientID": "c137",
                            "PatientName": "Rick Sanchez",
                            "PatientSex": "M"
                        },
                        "Series": [
                            "cd00fffc-db25be29-0c6da430-c56796a5-ba06933c",
                            "2ab7dbe7-f1a18a78-86145443-18a8ff93-0b65f2b2"
                        ],
                        "Type": "Study"
                    }
                "#,
            )
            .create_on(&mock_server);

        let cl = OrthancClient::new(&url, None, None);
        let study = cl.get_study("foo").unwrap();

        assert_eq!(
            study,
            Study {
                id: "63bf5d42-b5382159-01971752-e0ceea3d-399bbca5".to_string(),
                is_stable: true,
                last_update: NaiveDate::from_ymd(2020, 8, 30).and_hms(19, 11, 09),
                main_dicom_tags: hashmap! {
                    "AccessionNumber".to_string() => "foobar".to_string(),
                    "StudyDate".to_string() => "20110101".to_string(),
                    "StudyDescription".to_string() => "Brain".to_string(),
                    "StudyID".to_string() => "1742".to_string(),
                    "StudyInstanceUID".to_string() => "1.2.3.4.5.6789".to_string(),
                    "StudyTime".to_string() => "084707".to_string()
                },
                parent_patient: "7e43f8d3-e50280e6-470079e9-02241af1-d286bdbe".to_string(),
                patient_main_dicom_tags: hashmap! {
                    "PatientBirthDate".to_string() => "19440101".to_string(),
                    "PatientID".to_string() => "c137".to_string(),
                    "PatientName".to_string() => "Rick Sanchez".to_string(),
                    "PatientSex".to_string() => "M".to_string(),
                },
                series: [
                    "cd00fffc-db25be29-0c6da430-c56796a5-ba06933c".to_string(),
                    "2ab7dbe7-f1a18a78-86145443-18a8ff93-0b65f2b2".to_string()
                ]
                .to_vec(),
                entity_type: EntityType::Study,
                anonymized_from: None
            },
        );
        assert_eq!(m.times_called(), 1);
    }

    #[test]
    fn test_get_instance() {
        let mock_server = MockServer::start();
        let url = mock_server.url("");

        let m = Mock::new()
            .expect_method(Method::GET)
            .expect_path("/instances/foo")
            .return_status(200)
            .return_header("Content-Type", "application/json")
            .return_body(
                r#"
                    {
                        "FileSize": 139402,
                        "FileUuid": "d8c5eff3-986c-4fe4-b06e-7e52b2a4238e",
                        "ID": "29fa4d9d-51a69d1d-70e2b29a-fd824316-50850d0c",
                        "IndexInSeries": 13,
                        "MainDicomTags": {
                            "ImageOrientationPatient": "1\\0\\0\\0\\1\\0",
                            "ImagePositionPatient": "-17\\42\\13",
                            "InstanceCreationDate": "20130326",
                            "InstanceCreationTime": "135901",
                            "InstanceNumber": "13",
                            "SOPInstanceUID": "1.2.3.4.5.6789"
                        },
                        "ModifiedFrom": "22c54cb6-28302a69-3ff454a3-676b98f4-b84cd80a",
                        "ParentSeries": "82081568-b6f8f4e6-ced76876-6504da25-ed0dfe03",
                        "Type": "Instance"
                    }
                "#,
            )
            .create_on(&mock_server);

        let cl = OrthancClient::new(&url, None, None);
        let instance = cl.get_instance("foo").unwrap();

        assert_eq!(
            instance,
            Instance {
                id: "29fa4d9d-51a69d1d-70e2b29a-fd824316-50850d0c".to_string(),
                main_dicom_tags: hashmap! {
                    "ImageOrientationPatient".to_string() => "1\\0\\0\\0\\1\\0".to_string(),
                    "ImagePositionPatient".to_string() => "-17\\42\\13".to_string(),
                    "InstanceCreationDate".to_string() => "20130326".to_string(),
                    "InstanceCreationTime".to_string() => "135901".to_string(),
                    "InstanceNumber".to_string() => "13".to_string(),
                    "SOPInstanceUID".to_string() => "1.2.3.4.5.6789".to_string(),
                },
                parent_series: "82081568-b6f8f4e6-ced76876-6504da25-ed0dfe03".to_string(),
                index_in_series: 13,
                file_uuid: "d8c5eff3-986c-4fe4-b06e-7e52b2a4238e".to_string(),
                file_size: 139402,
                modified_from: Some(
                    "22c54cb6-28302a69-3ff454a3-676b98f4-b84cd80a".to_string()
                ),
                entity_type: EntityType::Instance,
                anonymized_from: None
            }
        );
        assert_eq!(m.times_called(), 1);
    }

    #[test]
    fn test_get_series() {
        let mock_server = MockServer::start();
        let url = mock_server.url("");

        let m = Mock::new()
            .expect_method(Method::GET)
            .expect_path("/series/foo")
            .return_status(200)
            .return_header("Content-Type", "application/json")
            .return_body(
                r#"
                    {
                        "ExpectedNumberOfInstances": 17,
                        "ID": "cd00fffc-db25be29-0c6da430-c56796a5-ba06933c",
                        "Instances": [
                            "556530b5-de7c487b-110b9d0e-12cfdbb9-f06b546e",
                            "c46605db-836489fa-cb55fbbc-13c8a913-b0bad6ac",
                            "9b63498d-cae4f25e-f52206b2-cbb4dc0e-dc55c788"
                        ],
                        "IsStable": true,
                        "LastUpdate": "20200830T191109",
                        "MainDicomTags": {
                            "BodyPartExamined": "ABDOMEN",
                            "Modality": "MR",
                            "ProtocolName": "TCP",
                            "SeriesDate": "20110101",
                            "SeriesInstanceUID": "1.2.3.4.5.6789",
                            "SeriesNumber": "1101",
                            "SeriesTime": "091313.93"
                        },
                        "ParentStudy": "63bf5d42-b5382159-01971752-e0ceea3d-399bbca5",
                        "Status": "Unknown",
                        "Type": "Series"
                    }
                "#,
            )
            .create_on(&mock_server);

        let cl = OrthancClient::new(&url, None, None);
        let series = cl.get_series("foo").unwrap();

        assert_eq!(
            series,
            Series {
                id: "cd00fffc-db25be29-0c6da430-c56796a5-ba06933c".to_string(),
                status: "Unknown".to_string(),
                is_stable: true,
                last_update: NaiveDate::from_ymd(2020, 8, 30).and_hms(19, 11, 09),
                main_dicom_tags: hashmap! {
                    "BodyPartExamined".to_string() => "ABDOMEN".to_string(),
                    "Modality".to_string() => "MR".to_string(),
                    "ProtocolName".to_string() => "TCP".to_string(),
                    "SeriesDate".to_string() => "20110101".to_string(),
                    "SeriesInstanceUID".to_string() => "1.2.3.4.5.6789".to_string(),
                    "SeriesNumber".to_string() => "1101".to_string(),
                    "SeriesTime".to_string() => "091313.93".to_string(),

                },
                parent_study: "63bf5d42-b5382159-01971752-e0ceea3d-399bbca5".to_string(),
                expected_number_of_instances: Some(17),
                instances: [
                    "556530b5-de7c487b-110b9d0e-12cfdbb9-f06b546e".to_string(),
                    "c46605db-836489fa-cb55fbbc-13c8a913-b0bad6ac".to_string(),
                    "9b63498d-cae4f25e-f52206b2-cbb4dc0e-dc55c788".to_string(),
                ]
                .to_vec(),
                entity_type: EntityType::Series,
                anonymized_from: None
            },
        );
        assert_eq!(m.times_called(), 1);
    }

    #[test]
    fn test_get_instance_tags() {
        let mock_server = MockServer::start();
        let url = mock_server.url("");

        let body = r#"
            {
                "AccessionNumber": "foobar",
                "AcquisitionDate": "20110101",
                "AcquisitionDuration": "219",
                "ReferencedImageSequence": [
                    {
                        "ReferencedSOPClassUID": "1.2.3.4.5.6789",
                        "ReferencedSOPInstanceUID": "1.2.3.4.5.67810"
                    }
                ]
            }
        "#;
        let m = Mock::new()
            .expect_method(Method::GET)
            .expect_path("/instances/foo/simplified-tags")
            .return_status(200)
            .return_header("Content-Type", "application/json")
            .return_body(body)
            .create_on(&mock_server);

        let cl = OrthancClient::new(&url, None, None);
        let resp = cl.get_instance_tags("foo").unwrap();

        let expected_resp: Value = serde_json::from_str(body).unwrap();
        assert_eq!(resp, expected_resp);
        assert_eq!(m.times_called(), 1);
    }

    #[test]
    fn test_get_instance_tags_expanded() {
        let mock_server = MockServer::start();
        let url = mock_server.url("");

        let body = r#"
            {
                "0002,0003": {
                    "Name": "MediaStorageSOPInstanceUID",
                    "Type": "String",
                    "Value": "1.2.3.4567"
                },
                "0008,0005": {
                    "Name": "SpecificCharacterSet",
                    "Type": "String",
                    "Value": "ISO_IR 100"
                },
                "0008,1110": {
                    "Name": "ReferencedStudySequence",
                    "Type": "Sequence",
                    "Value": [
                        {
                            "0008,1150": {
                                "Name": "ReferencedSOPClassUID",
                                "Type": "String",
                                "Value": "1.2.3.4.5.6789"
                            },
                            "0008,1155": {
                                "Name": "ReferencedSOPInstanceUID",
                                "Type": "String",
                                "Value": "1.2.3.4.5.67810"
                            }
                        }
                    ]
                }
            }
        "#;
        let m = Mock::new()
            .expect_method(Method::GET)
            .expect_path("/instances/foo/tags")
            .return_status(200)
            .return_header("Content-Type", "application/json")
            .return_body(body)
            .create_on(&mock_server);

        let cl = OrthancClient::new(&url, None, None);
        let resp = cl.get_instance_tags_expanded("foo").unwrap();

        let expected_resp: Value = serde_json::from_str(body).unwrap();
        assert_eq!(resp, expected_resp);
        assert_eq!(m.times_called(), 1);
    }

    #[test]
    fn test_get_patient_dicom() {
        let mock_server = MockServer::start();
        let url = mock_server.url("");

        let m = Mock::new()
            .expect_method(Method::GET)
            .expect_path("/patients/foo/archive")
            .return_status(200)
            .return_header("Content-Type", "application/json")
            .return_body("foobar")
            .create_on(&mock_server);

        let cl = OrthancClient::new(&url, None, None);
        let resp = cl.get_patient_dicom("foo").unwrap();

        assert_eq!(resp, "foobar".as_bytes());
        assert_eq!(m.times_called(), 1);
    }

    #[test]
    fn test_get_study_dicom() {
        let mock_server = MockServer::start();
        let url = mock_server.url("");

        let m = Mock::new()
            .expect_method(Method::GET)
            .expect_path("/studies/foo/archive")
            .return_status(200)
            .return_header("Content-Type", "application/json")
            .return_body("foobar")
            .create_on(&mock_server);

        let cl = OrthancClient::new(&url, None, None);
        let resp = cl.get_study_dicom("foo").unwrap();

        assert_eq!(resp, "foobar".as_bytes());
        assert_eq!(m.times_called(), 1);
    }

    #[test]
    fn test_get_series_dicom() {
        let mock_server = MockServer::start();
        let url = mock_server.url("");

        let m = Mock::new()
            .expect_method(Method::GET)
            .expect_path("/series/foo/archive")
            .return_status(200)
            .return_header("Content-Type", "application/json")
            .return_body("foobar")
            .create_on(&mock_server);

        let cl = OrthancClient::new(&url, None, None);
        let resp = cl.get_series_dicom("foo").unwrap();

        assert_eq!(resp, "foobar".as_bytes());
        assert_eq!(m.times_called(), 1);
    }

    #[test]
    fn test_get_instance_dicom() {
        let mock_server = MockServer::start();
        let url = mock_server.url("");

        let m = Mock::new()
            .expect_method(Method::GET)
            .expect_path("/instances/foo/file")
            .return_status(200)
            .return_header("Content-Type", "application/json")
            .return_body("foobar")
            .create_on(&mock_server);

        let cl = OrthancClient::new(&url, None, None);
        let resp = cl.get_instance_dicom("foo").unwrap();

        assert_eq!(resp, "foobar".as_bytes());
        assert_eq!(m.times_called(), 1);
    }

    #[test]
    fn test_store() {
        let mock_server = MockServer::start();
        let url = mock_server.url("");

        let m = Mock::new()
            .expect_method(Method::POST)
            .expect_path("/modalities/them/store")
            //.expect_body(r#"["bar", "baz", "qux"]"#)
            .return_status(200)
            .return_header("Content-Type", "application/json")
            .return_body(
                r#"
                    {
                       "Description" : "REST API",
                       "FailedInstancesCount" : 17,
                       "InstancesCount" : 42,
                       "LocalAet" : "US",
                       "ParentResources" : [ "bar", "baz", "qux" ],
                       "RemoteAet" : "THEM"
                    }
                "#,
            )
            .create_on(&mock_server);

        let cl = OrthancClient::new(&url, None, None);
        let resp = cl.store("them", &["bar", "baz", "qux"]).unwrap();

        assert_eq!(
            resp,
            StoreResponse {
                description: "REST API".to_string(),
                local_aet: "US".to_string(),
                remote_aet: "THEM".to_string(),
                parent_resources: vec![
                    "bar".to_string(),
                    "baz".to_string(),
                    "qux".to_string()
                ],
                instances_count: 42,
                failed_instances_count: 17
            }
        );
        assert_eq!(m.times_called(), 1);
    }

    #[test]
    fn test_modify() {
        let mock_server = MockServer::start();
        let url = mock_server.url("");

        let m = Mock::new()
            .expect_method(Method::POST)
            .expect_path("/studies/foo/modify")
            .expect_json_body(&Modification {
                replace: Some(hashmap! {"Tag1".to_string() => "value1".to_string()}),
                remove: Some(vec!["Tag2".to_string()]),
                force: None,
            })
            .return_status(200)
            .return_body(
                r#"
                    {
                        "ID": "86a3054b-32bb888a-e5f42e28-4b2e82d2-b1d7e14c",
                        "Path": "/studies/86a3054b-32bb888a-e5f42e28-4b2e82d2-b1d7e14c",
                        "PatientID": "86a3054b-32bb888a-e5f42e28-4b2e82d2-b1d7e14c",
                        "Type": "Study"
                    }
                "#,
            )
            .create_on(&mock_server);

        let cl = OrthancClient::new(&url, None, None);
        let resp = cl
            .modify(
                "studies",
                "foo",
                Some(hashmap! {"Tag1".to_string() => "value1".to_string()}),
                Some(vec!["Tag2".to_string()]),
                None,
            )
            .unwrap();

        assert_eq!(
            resp,
            ModifyResponse {
                id: "86a3054b-32bb888a-e5f42e28-4b2e82d2-b1d7e14c".to_string(),
                patient_id: "86a3054b-32bb888a-e5f42e28-4b2e82d2-b1d7e14c".to_string(),
                path: "/studies/86a3054b-32bb888a-e5f42e28-4b2e82d2-b1d7e14c".to_string(),
                entity_type: EntityType::Study
            }
        );
        assert_eq!(m.times_called(), 1);
    }

    #[test]
    fn test_anonymize() {
        let mock_server = MockServer::start();
        let url = mock_server.url("");

        let m = Mock::new()
            .expect_method(Method::POST)
            .expect_path("/studies/foo/anonymize")
            .expect_json_body(&Anonymization {
                replace: Some(hashmap! {"Tag1".to_string() => "value1".to_string()}),
                keep: Some(vec!["Tag2".to_string(), "Tag3".to_string()]),
                keep_private_tags: None,
                dicom_version: None,
            })
            .return_status(200)
            .return_body(
                r#"
                    {
                        "ID": "86a3054b-32bb888a-e5f42e28-4b2e82d2-b1d7e14c",
                        "Path": "/studies/86a3054b-32bb888a-e5f42e28-4b2e82d2-b1d7e14c",
                        "PatientID": "86a3054b-32bb888a-e5f42e28-4b2e82d2-b1d7e14c",
                        "Type": "Study"
                    }
                "#,
            )
            .create_on(&mock_server);

        let cl = OrthancClient::new(&url, None, None);
        let resp = cl
            .anonymize(
                "studies",
                "foo",
                Some(hashmap! {"Tag1".to_string() => "value1".to_string()}),
                Some(vec!["Tag2".to_string(), "Tag3".to_string()]),
                None,
                None,
            )
            .unwrap();

        assert_eq!(
            resp,
            ModifyResponse {
                id: "86a3054b-32bb888a-e5f42e28-4b2e82d2-b1d7e14c".to_string(),
                patient_id: "86a3054b-32bb888a-e5f42e28-4b2e82d2-b1d7e14c".to_string(),
                path: "/studies/86a3054b-32bb888a-e5f42e28-4b2e82d2-b1d7e14c".to_string(),
                entity_type: EntityType::Study,
            }
        );
        assert_eq!(m.times_called(), 1);
    }

    #[test]
    fn test_modify_patient() {
        let mock_server = MockServer::start();
        let url = mock_server.url("");

        let m = Mock::new()
            .expect_method(Method::POST)
            .expect_path("/patients/foo/modify")
            .expect_json_body(&Modification {
                replace: Some(hashmap! {"Tag1".to_string() => "value1".to_string()}),
                remove: Some(vec!["Tag2".to_string()]),
                force: Some(true),
            })
            .return_status(200)
            .return_body(
                r#"
                    {
                        "ID": "86a3054b-32bb888a-e5f42e28-4b2e82d2-b1d7e14c",
                        "Path": "/patients/86a3054b-32bb888a-e5f42e28-4b2e82d2-b1d7e14c",
                        "PatientID": "86a3054b-32bb888a-e5f42e28-4b2e82d2-b1d7e14c",
                        "Type": "Patient"
                    }
                "#,
            )
            .create_on(&mock_server);

        let cl = OrthancClient::new(&url, None, None);
        let resp = cl
            .modify_patient(
                "foo",
                Some(hashmap! {"Tag1".to_string() => "value1".to_string()}),
                Some(vec!["Tag2".to_string()]),
            )
            .unwrap();

        assert_eq!(
            resp,
            ModifyResponse {
                id: "86a3054b-32bb888a-e5f42e28-4b2e82d2-b1d7e14c".to_string(),
                patient_id: "86a3054b-32bb888a-e5f42e28-4b2e82d2-b1d7e14c".to_string(),
                path: "/patients/86a3054b-32bb888a-e5f42e28-4b2e82d2-b1d7e14c".to_string(),
                entity_type: EntityType::Patient,
            }
        );
        assert_eq!(m.times_called(), 1);
    }

    #[test]
    fn test_modify_study() {
        let mock_server = MockServer::start();
        let url = mock_server.url("");

        let m = Mock::new()
            .expect_method(Method::POST)
            .expect_path("/studies/foo/modify")
            .expect_json_body(&Modification {
                replace: Some(hashmap! {"Tag1".to_string() => "value1".to_string()}),
                remove: Some(vec!["Tag2".to_string()]),
                force: None,
            })
            .return_status(200)
            .return_body(
                r#"
                    {
                        "ID": "86a3054b-32bb888a-e5f42e28-4b2e82d2-b1d7e14c",
                        "Path": "/studies/86a3054b-32bb888a-e5f42e28-4b2e82d2-b1d7e14c",
                        "PatientID": "86a3054b-32bb888a-e5f42e28-4b2e82d2-b1d7e14c",
                        "Type": "Study"
                    }
                "#,
            )
            .create_on(&mock_server);

        let cl = OrthancClient::new(&url, None, None);
        let resp = cl
            .modify_study(
                "foo",
                Some(hashmap! {"Tag1".to_string() => "value1".to_string()}),
                Some(vec!["Tag2".to_string()]),
            )
            .unwrap();

        assert_eq!(
            resp,
            ModifyResponse {
                id: "86a3054b-32bb888a-e5f42e28-4b2e82d2-b1d7e14c".to_string(),
                patient_id: "86a3054b-32bb888a-e5f42e28-4b2e82d2-b1d7e14c".to_string(),
                path: "/studies/86a3054b-32bb888a-e5f42e28-4b2e82d2-b1d7e14c".to_string(),
                entity_type: EntityType::Study,
            }
        );
        assert_eq!(m.times_called(), 1);
    }

    #[test]
    fn test_modify_series() {
        let mock_server = MockServer::start();
        let url = mock_server.url("");

        let m = Mock::new()
            .expect_method(Method::POST)
            .expect_path("/series/foo/modify")
            .expect_json_body(&Modification {
                replace: Some(hashmap! {"Tag1".to_string() => "value1".to_string()}),
                remove: Some(vec!["Tag2".to_string()]),
                force: None,
            })
            .return_status(200)
            .return_body(
                r#"
                    {
                        "ID": "86a3054b-32bb888a-e5f42e28-4b2e82d2-b1d7e14c",
                        "Path": "/series/86a3054b-32bb888a-e5f42e28-4b2e82d2-b1d7e14c",
                        "PatientID": "86a3054b-32bb888a-e5f42e28-4b2e82d2-b1d7e14c",
                        "Type": "Series"
                    }
                "#,
            )
            .create_on(&mock_server);

        let cl = OrthancClient::new(&url, None, None);
        let resp = cl
            .modify_series(
                "foo",
                Some(hashmap! {"Tag1".to_string() => "value1".to_string()}),
                Some(vec!["Tag2".to_string()]),
            )
            .unwrap();

        assert_eq!(
            resp,
            ModifyResponse {
                id: "86a3054b-32bb888a-e5f42e28-4b2e82d2-b1d7e14c".to_string(),
                patient_id: "86a3054b-32bb888a-e5f42e28-4b2e82d2-b1d7e14c".to_string(),
                path: "/series/86a3054b-32bb888a-e5f42e28-4b2e82d2-b1d7e14c".to_string(),
                entity_type: EntityType::Series,
            }
        );
        assert_eq!(m.times_called(), 1);
    }

    #[test]
    fn test_modify_instance() {
        let mock_server = MockServer::start();
        let url = mock_server.url("");

        let m = Mock::new()
            .expect_method(Method::POST)
            .expect_path("/instances/foo/modify")
            .expect_json_body(&Modification {
                replace: Some(hashmap! {"Tag1".to_string() => "value1".to_string()}),
                remove: Some(vec!["Tag2".to_string()]),
                force: None,
            })
            .return_status(200)
            .return_body("foobar")
            .create_on(&mock_server);

        let cl = OrthancClient::new(&url, None, None);
        let resp = cl
            .modify_instance(
                "foo",
                Some(hashmap! {"Tag1".to_string() => "value1".to_string()}),
                Some(vec!["Tag2".to_string()]),
            )
            .unwrap();

        assert_eq!(resp, "foobar".as_bytes());
        assert_eq!(m.times_called(), 1);
    }

    #[test]
    fn test_anonymize_patient() {
        let mock_server = MockServer::start();
        let url = mock_server.url("");

        let m = Mock::new()
            .expect_method(Method::POST)
            .expect_path("/patients/foo/anonymize")
            .expect_json_body(&Anonymization {
                replace: Some(hashmap! {"Tag1".to_string() => "value1".to_string()}),
                keep: Some(vec!["Tag2".to_string(), "Tag3".to_string()]),
                keep_private_tags: None,
                dicom_version: None,
            })
            .return_status(200)
            .return_body(
                r#"
                    {
                        "ID": "86a3054b-32bb888a-e5f42e28-4b2e82d2-b1d7e14c",
                        "Path": "/patients/86a3054b-32bb888a-e5f42e28-4b2e82d2-b1d7e14c",
                        "PatientID": "86a3054b-32bb888a-e5f42e28-4b2e82d2-b1d7e14c",
                        "Type": "Patient"
                    }
                "#,
            )
            .create_on(&mock_server);

        let cl = OrthancClient::new(&url, None, None);
        let resp = cl
            .anonymize_patient(
                "foo",
                Some(hashmap! {"Tag1".to_string() => "value1".to_string()}),
                Some(vec!["Tag2".to_string(), "Tag3".to_string()]),
                None,
                None,
            )
            .unwrap();

        assert_eq!(
            resp,
            ModifyResponse {
                id: "86a3054b-32bb888a-e5f42e28-4b2e82d2-b1d7e14c".to_string(),
                patient_id: "86a3054b-32bb888a-e5f42e28-4b2e82d2-b1d7e14c".to_string(),
                path: "/patients/86a3054b-32bb888a-e5f42e28-4b2e82d2-b1d7e14c".to_string(),
                entity_type: EntityType::Patient,
            }
        );
        assert_eq!(m.times_called(), 1);
    }

    #[test]
    fn test_anonymize_study() {
        let mock_server = MockServer::start();
        let url = mock_server.url("");

        let m = Mock::new()
            .expect_method(Method::POST)
            .expect_path("/studies/foo/anonymize")
            .expect_json_body(&Anonymization {
                replace: Some(hashmap! {"Tag1".to_string() => "value1".to_string()}),
                keep: Some(vec!["Tag2".to_string(), "Tag3".to_string()]),
                keep_private_tags: Some(true),
                dicom_version: None,
            })
            .return_status(200)
            .return_body(
                r#"
                    {
                        "ID": "86a3054b-32bb888a-e5f42e28-4b2e82d2-b1d7e14c",
                        "Path": "/studies/86a3054b-32bb888a-e5f42e28-4b2e82d2-b1d7e14c",
                        "PatientID": "86a3054b-32bb888a-e5f42e28-4b2e82d2-b1d7e14c",
                        "Type": "Study"
                    }
                "#,
            )
            .create_on(&mock_server);

        let cl = OrthancClient::new(&url, None, None);
        let resp = cl
            .anonymize_study(
                "foo",
                Some(hashmap! {"Tag1".to_string() => "value1".to_string()}),
                Some(vec!["Tag2".to_string(), "Tag3".to_string()]),
                Some(true),
                None,
            )
            .unwrap();

        assert_eq!(
            resp,
            ModifyResponse {
                id: "86a3054b-32bb888a-e5f42e28-4b2e82d2-b1d7e14c".to_string(),
                patient_id: "86a3054b-32bb888a-e5f42e28-4b2e82d2-b1d7e14c".to_string(),
                path: "/studies/86a3054b-32bb888a-e5f42e28-4b2e82d2-b1d7e14c".to_string(),
                entity_type: EntityType::Study,
            }
        );
        assert_eq!(m.times_called(), 1);
    }

    #[test]
    fn test_anonymize_series() {
        let mock_server = MockServer::start();
        let url = mock_server.url("");

        let m = Mock::new()
            .expect_method(Method::POST)
            .expect_path("/series/foo/anonymize")
            .expect_json_body(&Anonymization {
                replace: Some(hashmap! {"Tag1".to_string() => "value1".to_string()}),
                keep: Some(vec!["Tag2".to_string(), "Tag3".to_string()]),
                keep_private_tags: Some(false),
                dicom_version: None,
            })
            .return_status(200)
            .return_body(
                r#"
                    {
                        "ID": "86a3054b-32bb888a-e5f42e28-4b2e82d2-b1d7e14c",
                        "Path": "/series/86a3054b-32bb888a-e5f42e28-4b2e82d2-b1d7e14c",
                        "PatientID": "86a3054b-32bb888a-e5f42e28-4b2e82d2-b1d7e14c",
                        "Type": "Series"
                    }
                "#,
            )
            .create_on(&mock_server);

        let cl = OrthancClient::new(&url, None, None);
        let resp = cl
            .anonymize_series(
                "foo",
                Some(hashmap! {"Tag1".to_string() => "value1".to_string()}),
                Some(vec!["Tag2".to_string(), "Tag3".to_string()]),
                Some(false),
                None,
            )
            .unwrap();

        assert_eq!(
            resp,
            ModifyResponse {
                id: "86a3054b-32bb888a-e5f42e28-4b2e82d2-b1d7e14c".to_string(),
                patient_id: "86a3054b-32bb888a-e5f42e28-4b2e82d2-b1d7e14c".to_string(),
                path: "/series/86a3054b-32bb888a-e5f42e28-4b2e82d2-b1d7e14c".to_string(),
                entity_type: EntityType::Series,
            }
        );
        assert_eq!(m.times_called(), 1);
    }

    #[test]
    fn test_anonymize_instance() {
        let mock_server = MockServer::start();
        let url = mock_server.url("");

        let m = Mock::new()
            .expect_method(Method::POST)
            .expect_path("/instances/foo/anonymize")
            .expect_json_body(&Anonymization {
                replace: Some(hashmap! {"Tag1".to_string() => "value1".to_string()}),
                keep: Some(vec!["Tag2".to_string(), "Tag3".to_string()]),
                keep_private_tags: None,
                dicom_version: None,
            })
            .return_status(200)
            .return_body("foobar")
            .create_on(&mock_server);

        let cl = OrthancClient::new(&url, None, None);
        let resp = cl
            .anonymize_instance(
                "foo",
                Some(hashmap! {"Tag1".to_string() => "value1".to_string()}),
                Some(vec!["Tag2".to_string(), "Tag3".to_string()]),
                None,
                None,
            )
            .unwrap();

        assert_eq!(resp, "foobar".as_bytes());
        assert_eq!(m.times_called(), 1);
    }

    #[test]
    fn test_delete_patient() {
        let mock_server = MockServer::start();
        let url = mock_server.url("");

        let m = Mock::new()
            .expect_method(Method::DELETE)
            .expect_path("/patients/foo")
            .return_status(200)
            .return_body(r#"{"RemainingAncestor": null}"#)
            .create_on(&mock_server);

        let cl = OrthancClient::new(&url, None, None);
        let resp = cl.delete_patient("foo").unwrap();

        assert_eq!(
            resp,
            RemainingAncestorResponse {
                remaining_ancestor: None
            }
        );
        assert_eq!(m.times_called(), 1);
    }

    #[test]
    fn test_delete_study() {
        let mock_server = MockServer::start();
        let url = mock_server.url("");

        let m = Mock::new()
            .expect_method(Method::DELETE)
            .expect_path("/studies/foo")
            .return_status(200)
            .return_body(
                r#"
                    {
                        "RemainingAncestor": {
                            "ID": "bar",
                            "Path": "/patients/bar",
                            "Type": "Patient"
                        }
                    }
                "#,
            )
            .create_on(&mock_server);

        let cl = OrthancClient::new(&url, None, None);
        let resp = cl.delete_study("foo").unwrap();

        assert_eq!(
            resp,
            RemainingAncestorResponse {
                remaining_ancestor: Some(RemainingAncestor {
                    id: "bar".to_string(),
                    path: "/patients/bar".to_string(),
                    entity_type: EntityType::Patient,
                })
            }
        );
        assert_eq!(m.times_called(), 1);
    }

    #[test]
    fn test_delete_series() {
        let mock_server = MockServer::start();
        let url = mock_server.url("");

        let m = Mock::new()
            .expect_method(Method::DELETE)
            .expect_path("/series/foo")
            .return_status(200)
            .return_body(
                r#"
                    {
                        "RemainingAncestor": {
                            "ID": "bar",
                            "Path": "/studies/bar",
                            "Type": "Study"
                        }
                    }
                "#,
            )
            .create_on(&mock_server);

        let cl = OrthancClient::new(&url, None, None);
        let resp = cl.delete_series("foo").unwrap();

        assert_eq!(
            resp,
            RemainingAncestorResponse {
                remaining_ancestor: Some(RemainingAncestor {
                    id: "bar".to_string(),
                    path: "/studies/bar".to_string(),
                    entity_type: EntityType::Study,
                })
            }
        );
        assert_eq!(m.times_called(), 1);
    }

    #[test]
    fn test_delete_instance() {
        let mock_server = MockServer::start();
        let url = mock_server.url("");

        let m = Mock::new()
            .expect_method(Method::DELETE)
            .expect_path("/instances/foo")
            .return_status(200)
            .return_body(
                r#"
                    {
                        "RemainingAncestor": {
                            "ID": "bar",
                            "Path": "/series/bar",
                            "Type": "Series"
                        }
                    }
                "#,
            )
            .create_on(&mock_server);

        let cl = OrthancClient::new(&url, None, None);
        let resp = cl.delete_instance("foo").unwrap();

        assert_eq!(
            resp,
            RemainingAncestorResponse {
                remaining_ancestor: Some(RemainingAncestor {
                    id: "bar".to_string(),
                    path: "/series/bar".to_string(),
                    entity_type: EntityType::Series,
                })
            }
        );
        assert_eq!(m.times_called(), 1);
    }

    #[test]
    fn test_echo() {
        let mock_server = MockServer::start();
        let url = mock_server.url("");

        let m = Mock::new()
            .expect_method(Method::POST)
            .expect_path("/modalities/foo/echo")
            .return_status(200)
            .return_body("{}")
            .create_on(&mock_server);

        let cl = OrthancClient::new(&url, None, None);
        let resp = cl.echo("foo", None).unwrap();

        assert_eq!(resp, ());
        assert_eq!(m.times_called(), 1);
    }

    #[test]
    fn test_echo_with_timeout() {
        let mock_server = MockServer::start();
        let url = mock_server.url("");

        let m = Mock::new()
            .expect_method(Method::POST)
            .expect_path("/modalities/foo/echo")
            .expect_json_body(&hashmap! {"Timeout" => 42})
            .return_status(200)
            .return_body("{}")
            .create_on(&mock_server);

        let cl = OrthancClient::new(&url, None, None);
        let resp = cl.echo("foo", Some(42)).unwrap();

        assert_eq!(resp, ());
        assert_eq!(m.times_called(), 1);
    }

    #[test]
    fn test_echo_failed() {
        let mock_server = MockServer::start();
        let url = mock_server.url("");

        let m = Mock::new()
            .expect_method(Method::POST)
            .expect_path("/modalities/foo/echo")
            .return_status(500)
            .create_on(&mock_server);

        let cl = OrthancClient::new(&url, None, None);
        let resp = cl.echo("foo", None);

        assert_eq!(
            resp.unwrap_err(),
            OrthancError {
                details: "500".to_string(),
                error_response: None
            }
        );
        assert_eq!(m.times_called(), 1);
    }

    #[test]
    fn test_upload_dicom() {
        let mock_server = MockServer::start();
        let url = mock_server.url("");

        let m = Mock::new()
            .expect_method(Method::POST)
            .expect_path("/instances")
            .expect_body("quux")
            .return_status(200)
            .return_body(
                r#"
                    {
                        "ID": "foo",
                        "ParentPatient": "bar",
                        "ParentSeries": "baz",
                        "ParentStudy": "qux",
                        "Path": "/instances/foo",
                        "Status": "Success"
                    }
                "#,
            )
            .create_on(&mock_server);

        let cl = OrthancClient::new(&url, None, None);
        let resp = cl.upload_dicom("quux".as_bytes()).unwrap();

        assert_eq!(
            resp,
            UploadStatusResponse {
                id: "foo".to_string(),
                status: "Success".to_string(),
                path: "/instances/foo".to_string(),
                parent_patient: "bar".to_string(),
                parent_study: "qux".to_string(),
                parent_series: "baz".to_string(),
            }
        );
        assert_eq!(m.times_called(), 1);
    }

    #[test]
    fn test_check_http_error_ok() {
        let res = check_http_error(reqwest::StatusCode::PERMANENT_REDIRECT, "foo");
        assert!(res.is_ok());
    }

    #[test]
    fn test_check_http_error_error() {
        let res = check_http_error(
            reqwest::StatusCode::BAD_REQUEST,
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
        );
        assert_eq!(
            res.unwrap_err(),
            OrthancError {
                details: "400".to_string(),
                error_response: Some(ErrorResponse {
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
        let res = check_http_error(reqwest::StatusCode::UNAUTHORIZED, "");
        assert_eq!(
            res.unwrap_err(),
            OrthancError {
                details: "401".to_string(),
                error_response: None
            },
        );
    }

    // TODO: Firgure out how to handle this
    #[test]
    fn test_check_http_error_error_random_body() {
        let res = check_http_error(reqwest::StatusCode::GATEWAY_TIMEOUT, "foo bar baz");
        assert_eq!(
            res.unwrap_err(),
            OrthancError {
                details: "expected ident at line 1 column 2".to_string(),
                error_response: None
            },
        );
    }
}
