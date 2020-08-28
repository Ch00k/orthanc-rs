use bytes::Bytes;
use chrono::NaiveDateTime;
use reqwest::blocking::{Client, RequestBuilder, Response};
use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;
use std::collections::HashMap;
use std::fmt;

#[derive(Debug)]
pub struct OrthancError {
    details: String,
}

impl fmt::Display for OrthancError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.details)
    }
}

impl OrthancError {
    pub fn new(msg: &str) -> OrthancError {
        OrthancError {
            details: msg.to_string(),
        }
    }
}

impl From<reqwest::Error> for OrthancError {
    fn from(e: reqwest::Error) -> Self {
        OrthancError::new(&e.to_string())
    }
}

impl From<serde_json::error::Error> for OrthancError {
    fn from(e: serde_json::error::Error) -> Self {
        OrthancError::new(&e.to_string())
    }
}

#[derive(Deserialize, Debug, Eq, PartialEq)]
pub struct Modality {
    #[serde(rename(deserialize = "AET"))]
    pub aet: String,

    #[serde(rename(deserialize = "Host"))]
    pub host: String,

    #[serde(rename(deserialize = "Port"))]
    pub port: u32,

    #[serde(rename(deserialize = "Manufacturer"))]
    pub manufacturer: String,

    #[serde(rename(deserialize = "AllowEcho"))]
    pub allow_echo: bool,

    #[serde(rename(deserialize = "AllowFind"))]
    pub allow_find: bool,

    #[serde(rename(deserialize = "AllowGet"))]
    pub allow_get: bool,

    #[serde(rename(deserialize = "AllowMove"))]
    pub allow_move: bool,

    #[serde(rename(deserialize = "AllowStore"))]
    pub allow_store: bool,

    #[serde(rename(deserialize = "AllowNAction"))]
    pub allow_n_action: bool,

    #[serde(rename(deserialize = "AllowEventReport"))]
    pub allow_event_report: bool,

    #[serde(rename(deserialize = "AllowTranscoding"))]
    pub allow_transcoding: bool,
}

#[derive(Deserialize, Debug, Eq, PartialEq)]
pub struct Patient {
    #[serde(rename(deserialize = "ID"))]
    pub id: String,

    #[serde(rename(deserialize = "IsStable"))]
    pub is_stable: bool,

    #[serde(with = "datetime_format", rename(deserialize = "LastUpdate"))]
    pub last_update: NaiveDateTime,

    #[serde(rename(deserialize = "MainDicomTags"))]
    pub main_dicom_tags: HashMap<String, String>,

    #[serde(rename(deserialize = "Studies"))]
    pub studies: Vec<String>,
}

#[derive(Deserialize, Debug, Eq, PartialEq)]
pub struct Study {
    #[serde(rename(deserialize = "ID"))]
    pub id: String,

    #[serde(rename(deserialize = "IsStable"))]
    pub is_stable: bool,

    #[serde(with = "datetime_format", rename(deserialize = "LastUpdate"))]
    pub last_update: NaiveDateTime,

    #[serde(rename(deserialize = "MainDicomTags"))]
    pub main_dicom_tags: HashMap<String, String>,

    #[serde(rename(deserialize = "ParentPatient"))]
    pub patient_id: String,

    #[serde(rename(deserialize = "PatientMainDicomTags"))]
    pub patient_main_dicom_tags: HashMap<String, String>,

    #[serde(rename(deserialize = "Series"))]
    pub series: Vec<String>,
}

#[derive(Deserialize, Debug, Eq, PartialEq)]
pub struct Series {
    #[serde(rename(deserialize = "ID"))]
    pub id: String,

    #[serde(rename(deserialize = "Status"))]
    pub status: String,

    #[serde(rename(deserialize = "IsStable"))]
    pub is_stable: bool,

    #[serde(with = "datetime_format", rename(deserialize = "LastUpdate"))]
    pub last_update: NaiveDateTime,

    #[serde(rename(deserialize = "MainDicomTags"))]
    pub main_dicom_tags: HashMap<String, String>,

    #[serde(rename(deserialize = "ParentStudy"))]
    pub study_id: String,

    #[serde(rename(deserialize = "ExpectedNumberOfInstances"))]
    pub num_instances: Option<u32>,

    #[serde(rename(deserialize = "Instances"))]
    pub instances: Vec<String>,
}

#[derive(Deserialize, Debug, Eq, PartialEq)]
pub struct Instance {
    #[serde(rename(deserialize = "ID"))]
    pub id: String,

    #[serde(with = "datetime_format", rename(deserialize = "LastUpdate"))]
    pub last_update: NaiveDateTime,

    #[serde(rename(deserialize = "MainDicomTags"))]
    pub main_dicom_tags: HashMap<String, String>,

    #[serde(rename(deserialize = "ParentSeries"))]
    pub series_id: String,

    #[serde(rename(deserialize = "IndexInSeries"))]
    pub index_in_series: u32,

    #[serde(rename(deserialize = "FileUuid"))]
    pub file_uuid: String,

    #[serde(rename(deserialize = "FileSize"))]
    pub file_size: u32,
}

#[derive(Deserialize, Debug, Eq, PartialEq)]
pub struct UploadStatusResponse {
    #[serde(rename(deserialize = "ID"))]
    pub id: String,

    #[serde(rename(deserialize = "Status"))]
    pub status: String,

    #[serde(rename(deserialize = "Path"))]
    pub path: String,

    #[serde(rename(deserialize = "ParentPatient"))]
    parent_patient: String,

    #[serde(rename(deserialize = "ParentStudy"))]
    parent_study: String,

    #[serde(rename(deserialize = "ParentSeries"))]
    parent_series: String,
}

#[derive(Deserialize, Debug, Eq, PartialEq)]
pub struct RemainingAncestorResponse {
    #[serde(rename(deserialize = "ID"))]
    pub id: String,

    #[serde(rename(deserialize = "Path"))]
    pub path: String,

    #[serde(rename(deserialize = "Type"))]
    pub entity_type: String,
}

#[derive(Deserialize, Debug, Eq, PartialEq)]
pub struct ErrorResponse {
    #[serde(rename(deserialize = "Method"))]
    method: String,

    #[serde(rename(deserialize = "Uri"))]
    uri: String,

    #[serde(rename(deserialize = "Message"))]
    message: String,

    #[serde(rename(deserialize = "Details"))]
    details: String,

    #[serde(rename(deserialize = "HttpStatus"))]
    http_status: String,

    #[serde(rename(deserialize = "HttpError"))]
    http_error: String,

    #[serde(rename(deserialize = "OrthancStatus"))]
    orthanc_status: String,

    #[serde(rename(deserialize = "OrthancError"))]
    orthanc_error: String,
}

#[derive(Serialize, Debug, Eq, PartialEq)]
struct Modifications {
    #[serde(rename = "Remove")]
    #[serde(skip_serializing_if = "Option::is_none")]
    remove: Option<HashMap<String, String>>,

    #[serde(rename = "Replace")]
    #[serde(skip_serializing_if = "Option::is_none")]
    replace: Option<HashMap<String, String>>,

    #[serde(rename = "Force")]
    #[serde(skip_serializing_if = "Option::is_none")]
    force: Option<bool>,
}

pub struct OrthancClient<'a> {
    server_address: &'a str,
    username: Option<&'a str>,
    password: Option<&'a str>,
    client: Client,
}

impl<'a> OrthancClient<'a> {
    pub fn new(
        server_address: &'a str,
        username: Option<&'a str>,
        password: Option<&'a str>,
    ) -> OrthancClient<'a> {
        OrthancClient {
            server_address,
            username,
            password,
            client: Client::new(),
        }
    }

    fn add_auth(&self, request: RequestBuilder) -> RequestBuilder {
        match (&self.username, &self.password) {
            (Some(u), Some(p)) => request.basic_auth(u, Some(p)),
            _ => request,
        }
    }

    fn get(&self, path: &str) -> Result<Response, OrthancError> {
        let url = format!("{}/{}", self.server_address, &path);
        let mut request = self.client.get(&url);
        request = self.add_auth(request);
        let resp = request.send()?;

        if let Err(err) = check_http_error(&resp) {
            return Err(err);
        }
        Ok(resp)
    }

    fn get_bytes(&self, path: &str) -> Result<Bytes, OrthancError> {
        let url = format!("{}/{}", self.server_address, &path);
        let mut request = self.client.get(&url);
        request = self.add_auth(request);
        let resp = request.send()?;

        if let Err(err) = check_http_error(&resp) {
            return Err(err);
        }

        let bytes = resp.bytes()?;
        Ok(bytes)
    }

    fn post(&self, path: &str, data: Value) -> Result<Value, OrthancError> {
        let url = format!("{}/{}", self.server_address, path);
        let mut request = self.client.post(&url).json(&data);
        request = self.add_auth(request);
        let resp = request.send()?;

        if let Err(err) = check_http_error(&resp) {
            return Err(err);
        }

        let json = resp.json::<Value>()?;
        Ok(json)
    }

    fn post_bytes(&self, path: &str, data: &[u8]) -> Result<Response, OrthancError> {
        let url = format!("{}/{}", self.server_address, path);
        // TODO: .to_vec() here is probably not a good idea
        let mut request = self.client.post(&url).body(data.to_vec());
        request = self.add_auth(request);
        let resp = request.send()?;

        if let Err(err) = check_http_error(&resp) {
            return Err(err);
        }
        Ok(resp)
    }

    fn delete(&self, path: &str) -> Result<(), OrthancError> {
        let url = format!("{}/{}", self.server_address, &path);
        let mut request = self.client.delete(&url);
        request = self.add_auth(request);
        let resp = request.send()?;

        if let Err(err) = check_http_error(&resp) {
            return Err(err);
        }
        Ok(())
    }

    fn list(&self, entity: &str) -> Result<Vec<String>, OrthancError> {
        let resp = self.get(entity)?;
        let json = resp.json::<Vec<String>>()?;
        Ok(json)
    }

    pub fn list_modalities(&self) -> Result<Vec<String>, OrthancError> {
        self.list("modalities")
    }

    pub fn list_patients(&self) -> Result<Vec<String>, OrthancError> {
        self.list("patients")
    }

    pub fn list_studies(&self) -> Result<Vec<String>, OrthancError> {
        self.list("studies")
    }

    pub fn list_series(&self) -> Result<Vec<String>, OrthancError> {
        self.list("series")
    }

    pub fn list_instances(&self) -> Result<Vec<String>, OrthancError> {
        self.list("instances")
    }

    pub fn list_modalities_expanded(&self) -> Result<HashMap<String, Modality>, OrthancError> {
        let resp = self.get("modalities?expand")?;
        let json = resp.json::<HashMap<String, Modality>>()?;
        Ok(json)
    }

    pub fn list_patients_expanded(&self) -> Result<Vec<Patient>, OrthancError> {
        let resp = self.get("patients?expand")?;
        let json = resp.json::<Vec<Patient>>()?;
        Ok(json)
    }

    pub fn list_studies_expanded(&self) -> Result<Vec<Study>, OrthancError> {
        let resp = self.get("studies?expand")?;
        let json = resp.json::<Vec<Study>>()?;
        Ok(json)
    }

    pub fn list_series_expanded(&self) -> Result<Vec<Series>, OrthancError> {
        let resp = self.get("series?expand")?;
        let json = resp.json::<Vec<Series>>()?;
        Ok(json)
    }

    pub fn list_instances_expanded(&self) -> Result<Vec<Instance>, OrthancError> {
        let resp = self.get("instances?expand")?;
        let json = resp.json::<Vec<Instance>>()?;
        Ok(json)
    }

    pub fn get_patient(&self, id: &str) -> Result<Patient, OrthancError> {
        let path = format!("/patients/{}", id);
        let resp = self.get(&path)?;
        let json = resp.json::<Patient>()?;
        Ok(json)
    }

    pub fn get_study(&self, id: &str) -> Result<Study, OrthancError> {
        let path = format!("/studies/{}", id);
        let resp = self.get(&path)?;
        let json = resp.json::<Study>()?;
        Ok(json)
    }

    pub fn get_series(&self, id: &str) -> Result<Series, OrthancError> {
        let path = format!("/series/{}", id);
        let resp = self.get(&path)?;
        let json = resp.json::<Series>()?;
        Ok(json)
    }

    pub fn get_instance(&self, id: &str) -> Result<Instance, OrthancError> {
        let path = format!("/instances/{}", id);
        let resp = self.get(&path)?;
        let json = resp.json::<Instance>()?;
        Ok(json)
    }

    pub fn get_instance_tags(&self, id: &str) -> Result<Value, OrthancError> {
        let path = format!("/instances/{}/simplified-tags", id);
        let resp = self.get(&path)?;
        let json = resp.json::<Value>()?;
        Ok(json)
    }

    pub fn get_instance_tags_expanded(&self, id: &str) -> Result<Value, OrthancError> {
        let path = format!("/instances/{}/tags", id);
        let resp = self.get(&path)?;
        let json = resp.json::<Value>()?;
        Ok(json)
    }

    pub fn get_instance_content(&self, id: &str) -> Result<Vec<String>, OrthancError> {
        let path = format!("/instances/{}/content", id);
        let resp = self.get(&path)?;
        let json = resp.json::<Vec<String>>()?;
        Ok(json)
    }

    pub fn get_instance_tag(&self, instance_id: &str, tag_id: &str) -> Result<Value, OrthancError> {
        let path = format!("/instances/{}/content/{}", instance_id, tag_id);
        let resp = self.get(&path)?;
        let json = resp.json::<Value>()?;
        Ok(json)
    }

    pub fn get_instance_dicom(&self, id: &str) -> Result<Bytes, OrthancError> {
        let path = format!("/instances/{}/file", id);
        self.get_bytes(&path)
    }

    pub fn get_study_dicom(&self, id: &str) -> Result<Bytes, OrthancError> {
        let path = format!("/studies/{}/archive", id);
        self.get_bytes(&path)
    }

    pub fn delete_patient(&self, id: &str) -> Result<(), OrthancError> {
        let path = format!("/patients/{}", id);
        self.delete(&path)
    }

    pub fn delete_study(&self, id: &str) -> Result<(), OrthancError> {
        let path = format!("/studies/{}", id);
        println!("{:?}", path);
        self.delete(&path)
    }

    pub fn delete_series(&self, id: &str) -> Result<(), OrthancError> {
        let path = format!("/series/{}", id);
        self.delete(&path)
    }

    pub fn delete_instance(&self, id: &str) -> Result<(), OrthancError> {
        let path = format!("/instances/{}", id);
        self.delete(&path)
    }

    pub fn echo(&self, modality: &str, timeout: Option<u32>) -> Result<Value, OrthancError> {
        let path = format!("/modalities/{}/echo", modality);
        let mut data = HashMap::new();
        // TODO: This does not seem idiomatic
        if timeout != None {
            data.insert("Timeout", timeout);
        }
        self.post(&path, serde_json::json!(data))
    }

    // TODO: Implement async send (https://book.orthanc-server.com/users/advanced-rest.html#jobs)
    pub fn store(&self, modality: &str, id: &str) -> Result<Value, OrthancError> {
        let path = format!("/modalities/{}/store", modality);
        self.post(&path, serde_json::json!(id))
    }

    fn modify(
        &self,
        entity: &str,
        id: &str,
        replace: Option<HashMap<String, String>>,
        remove: Option<HashMap<String, String>>,
        force: Option<bool>,
    ) -> Result<Value, OrthancError> {
        let path = format!("/{}/{}/modify", entity, id);
        let data = Modifications {
            remove,
            replace,
            force,
        };
        self.post(&path, serde_json::to_value(data)?)
    }

    pub fn modify_patient(
        &self,
        id: &str,
        replace: Option<HashMap<String, String>>,
        remove: Option<HashMap<String, String>>,
    ) -> Result<Value, OrthancError> {
        self.modify("patients", id, replace, remove, Some(true))
    }

    pub fn modify_study(
        &self,
        id: &str,
        replace: Option<HashMap<String, String>>,
        remove: Option<HashMap<String, String>>,
    ) -> Result<Value, OrthancError> {
        self.modify("studies", id, replace, remove, None)
    }

    pub fn upload_dicom(&self, data: &[u8]) -> Result<UploadStatusResponse, OrthancError> {
        let path = "/instances";
        let resp = self.post_bytes(path, data)?;
        let json = resp.json::<UploadStatusResponse>()?;
        Ok(json)
    }
}

fn check_http_error(resp: &Response) -> Result<(), OrthancError> {
    if resp.status() >= reqwest::StatusCode::BAD_REQUEST {
        //eprintln!("{:?}", resp.text());
        return Err(OrthancError::new(resp.status().as_str()));
    }
    Ok(())
}

mod datetime_format {
    use chrono::NaiveDateTime;
    use serde::{self, Deserialize, Deserializer};

    const FORMAT: &'static str = "%Y%m%dT%H%M%S";

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
    fn test_default_fields() {
        let cl = OrthancClient::new("http://localhost:8042", None, None);
        assert_eq!(cl.server_address, "http://localhost:8042");
        assert_eq!(cl.username, None);
        assert_eq!(cl.password, None);
    }

    #[test]
    fn test_auth() {
        let cl = OrthancClient::new("http://localhost:8042", Some("foo"), Some("bar"));
        assert_eq!(cl.username, Some("foo"));
        assert_eq!(cl.password, Some("bar"));
    }

    #[test]
    fn test_get() {
        let mock_server = MockServer::start();
        let url = format!("http://{}:{}", &mock_server.host(), &mock_server.port());

        let m = Mock::new()
            .expect_method(Method::GET)
            .expect_path("/foo")
            .expect_header("Authorization", "Basic Zm9vOmJhcg==")
            .return_status(200)
            .return_body("bar")
            .create_on(&mock_server);

        let cl = OrthancClient::new(&url, Some("foo"), Some("bar"));
        let resp = cl.get("foo").unwrap();

        assert_eq!(resp.text().unwrap(), "bar");
        assert_eq!(m.times_called(), 1);
    }

    #[test]
    fn test_get_bytes() {
        let mock_server = MockServer::start();
        let url = format!("http://{}:{}", &mock_server.host(), &mock_server.port());

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
        let url = format!("http://{}:{}", &mock_server.host(), &mock_server.port());

        let m = Mock::new()
            .expect_method(Method::POST)
            .expect_path("/foo")
            .expect_body("\"bar\"")
            .expect_header("Authorization", "Basic Zm9vOmJhcg==")
            .return_header("Content-Type", "application/json")
            .return_status(200)
            .return_body("\"baz\"")
            .create_on(&mock_server);

        let cl = OrthancClient::new(&url, Some("foo"), Some("bar"));
        let resp = cl.post("foo", serde_json::json!("bar")).unwrap();

        assert_eq!(resp, "baz");
        assert_eq!(m.times_called(), 1);
    }

    #[test]
    fn test_post_bytes() {
        let mock_server = MockServer::start();
        let url = format!("http://{}:{}", &mock_server.host(), &mock_server.port());

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

        assert_eq!(resp.text().unwrap(), "baz");
        assert_eq!(m.times_called(), 1);
    }

    #[test]
    fn test_delete() {
        let mock_server = MockServer::start();
        let url = format!("http://{}:{}", &mock_server.host(), &mock_server.port());

        let m = Mock::new()
            .expect_method(Method::DELETE)
            .expect_path("/foo")
            .expect_header("Authorization", "Basic Zm9vOmJhcg==")
            .return_status(200)
            .create_on(&mock_server);

        let cl = OrthancClient::new(&url, Some("foo"), Some("bar"));
        let resp = cl.delete("foo").unwrap();

        assert_eq!(resp, ());
        assert_eq!(m.times_called(), 1);
    }

    #[test]
    fn test_list_modalities() {
        let mock_server = MockServer::start();
        let url = format!("http://{}:{}", &mock_server.host(), &mock_server.port());

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
    fn test_list_modalities_expanded() {
        let mock_server = MockServer::start();
        let url = format!("http://{}:{}", &mock_server.host(), &mock_server.port());

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
    fn test_list_patients() {
        let mock_server = MockServer::start();
        let url = format!("http://{}:{}", &mock_server.host(), &mock_server.port());

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
    fn test_list_patients_expanded() {
        let mock_server = MockServer::start();
        let url = format!("http://{}:{}", &mock_server.host(), &mock_server.port());

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
                    studies: ["e8cafcbe-caf08c39-6e205f15-18554bb8-b3f9ef04".to_string()].to_vec(),
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
                    studies: ["63bf5d42-b5382159-01971752-e0ceea3d-399bbca5".to_string()].to_vec(),
                },
            ]
        );
        assert_eq!(m.times_called(), 1);
    }
}
