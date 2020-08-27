use bytes::Bytes;
use chrono::NaiveDateTime;
use reqwest;
use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;
use std::collections::HashMap;
use std::env;
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

#[derive(Deserialize, Debug)]
pub struct Modality {
    #[serde(rename(deserialize = "AET"))]
    aet: String,

    #[serde(rename(deserialize = "Host"))]
    host: String,

    #[serde(rename(deserialize = "Port"))]
    port: u32,

    #[serde(rename(deserialize = "Manufacturer"))]
    manufacturer: String,

    #[serde(rename(deserialize = "AllowEcho"))]
    allow_echo: bool,

    #[serde(rename(deserialize = "AllowFind"))]
    allow_find: bool,

    #[serde(rename(deserialize = "AllowGet"))]
    allow_get: bool,

    #[serde(rename(deserialize = "AllowMove"))]
    allow_move: bool,

    #[serde(rename(deserialize = "AllowStore"))]
    allow_store: bool,

    #[serde(rename(deserialize = "AllowNAction"))]
    allow_n_action: bool,

    #[serde(rename(deserialize = "AllowEventReport"))]
    allow_event_report: bool,

    #[serde(rename(deserialize = "AllowTranscoding"))]
    allow_transcoding: bool,
}

#[derive(Deserialize, Debug, Eq, PartialEq)]
pub struct Patient {
    #[serde(rename(deserialize = "ID"))]
    pub id: String,

    #[serde(rename(deserialize = "IsStable"))]
    pub is_stable: bool,

    #[serde(with = "orthanc_datetime_format", rename(deserialize = "LastUpdate"))]
    pub last_update: NaiveDateTime,

    #[serde(rename(deserialize = "MainDicomTags"))]
    pub main_dicom_tags: HashMap<String, String>,

    #[serde(rename(deserialize = "Studies"))]
    pub studies: Vec<String>,
}

#[derive(Deserialize, Debug)]
pub struct Study {
    #[serde(rename(deserialize = "ID"))]
    id: String,

    #[serde(rename(deserialize = "IsStable"))]
    is_stable: bool,

    #[serde(with = "orthanc_datetime_format", rename(deserialize = "LastUpdate"))]
    last_update: NaiveDateTime,

    #[serde(rename(deserialize = "MainDicomTags"))]
    main_dicom_tags: HashMap<String, String>,

    #[serde(rename(deserialize = "ParentPatient"))]
    patient_id: String,

    #[serde(rename(deserialize = "PatientMainDicomTags"))]
    patient_main_dicom_tags: HashMap<String, String>,

    #[serde(rename(deserialize = "Series"))]
    series: Vec<String>,
}

#[derive(Deserialize, Debug)]
pub struct Series {
    #[serde(rename(deserialize = "ID"))]
    id: String,

    #[serde(rename(deserialize = "Status"))]
    status: String,

    #[serde(rename(deserialize = "IsStable"))]
    is_stable: bool,

    #[serde(with = "orthanc_datetime_format", rename(deserialize = "LastUpdate"))]
    last_update: NaiveDateTime,

    #[serde(rename(deserialize = "MainDicomTags"))]
    main_dicom_tags: HashMap<String, String>,

    #[serde(rename(deserialize = "ParentStudy"))]
    study_id: String,

    #[serde(rename(deserialize = "ExpectedNumberOfInstances"))]
    num_instances: Option<u32>,

    #[serde(rename(deserialize = "Instances"))]
    instances: Vec<String>,
}

#[derive(Deserialize, Debug)]
pub struct Instance {
    #[serde(rename(deserialize = "ID"))]
    id: String,

    #[serde(with = "orthanc_datetime_format", rename(deserialize = "LastUpdate"))]
    last_update: NaiveDateTime,

    #[serde(rename(deserialize = "MainDicomTags"))]
    main_dicom_tags: HashMap<String, String>,

    #[serde(rename(deserialize = "ParentSeries"))]
    series_id: String,

    #[serde(rename(deserialize = "IndexInSeries"))]
    index_in_series: u32,

    #[serde(rename(deserialize = "FileUuid"))]
    file_uuid: String,

    #[serde(rename(deserialize = "FileSize"))]
    file_size: u32,
}

#[derive(Deserialize, Debug)]
pub struct StatusResponse {
    #[serde(rename(deserialize = "ID"))]
    id: String,

    #[serde(rename(deserialize = "Path"))]
    path: String,

    #[serde(rename(deserialize = "Status"))]
    status: String,
}

pub struct OrthancClient {
    base_url: String,
    username: Option<String>,
    password: Option<String>,
    client: reqwest::blocking::Client,
}

impl OrthancClient {
    pub fn new(
        base_url: Option<&str>,
        username: Option<&str>,
        password: Option<&str>,
    ) -> OrthancClient {
        let base_url = match base_url {
            Some(b) => b.into(),
            None => env::var("ORC_ORTHANC_URL").unwrap_or("http://localhost:8042".into()),
        };

        let username = match username {
            Some(u) => Some(u.into()),
            None => env::var("ORC_ORTHANC_USERNAME").ok(),
        };

        let password = match password {
            Some(p) => Some(p.into()),
            None => env::var("ORC_ORTHANC_PASSWORD").ok(),
        };

        OrthancClient {
            base_url,
            username,
            password,
            client: reqwest::blocking::Client::new(),
        }
    }

    fn get(&self, path: &str) -> Result<reqwest::blocking::Response, OrthancError> {
        let url = format!("{}/{}", self.base_url, &path);
        let request = self.client.get(&url);

        let request = match (&self.username, &self.password) {
            (Some(username), Some(password)) => request.basic_auth(username, Some(password)),
            _ => request,
        };

        let resp = request.send()?;

        if let Err(err) = check_http_error(&resp) {
            // TODO: I don't understand why and how this works
            return Err(err);
        }
        Ok(resp)
    }

    fn get_bytes(&self, path: &str) -> Result<Bytes, OrthancError> {
        let url = format!("{}/{}", self.base_url, &path);
        let request = self.client.get(&url);

        let request = match (&self.username, &self.password) {
            (Some(username), Some(password)) => request.basic_auth(username, Some(password)),
            _ => request,
        };

        let resp = request.send()?;

        if let Err(err) = check_http_error(&resp) {
            // TODO: I don't understand why and how this works
            return Err(err);
        }

        let bytes = resp.bytes()?;
        Ok(bytes)
    }

    fn post(&self, path: &str, data: Value) -> Result<Value, OrthancError> {
        let url = format!("{}/{}", self.base_url, path);
        let request = self.client.post(&url).json(&data);

        let request = match (&self.username, &self.password) {
            (Some(username), Some(password)) => request.basic_auth(username, Some(password)),
            _ => request,
        };

        let resp = request.send()?;

        if let Err(err) = check_http_error(&resp) {
            // TODO: I don't understand why and how this works
            return Err(err);
        }

        let json = resp.json::<Value>()?;
        Ok(json)
    }

    fn post_bytes(&self, path: &str, data: Vec<u8>) -> Result<StatusResponse, OrthancError> {
        let url = format!("{}/{}", self.base_url, path);
        let request = self.client.post(&url).body(data);

        let request = match (&self.username, &self.password) {
            (Some(username), Some(password)) => request.basic_auth(username, Some(password)),
            _ => request,
        };

        let resp = request.send()?;

        if let Err(err) = check_http_error(&resp) {
            // TODO: I don't understand why and how this works
            return Err(err);
        }

        let json = resp.json::<StatusResponse>()?;
        Ok(json)
    }

    fn delete(&self, path: &str) -> Result<(), OrthancError> {
        let url = format!("{}/{}", self.base_url, &path);
        let request = self.client.delete(&url);

        let request = match (&self.username, &self.password) {
            (Some(username), Some(password)) => request.basic_auth(username, Some(password)),
            _ => request,
        };

        let resp = request.send()?;

        if let Err(err) = check_http_error(&resp) {
            // TODO: I don't understand why and how this works
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

    pub fn list_modalities_expanded(&self) -> Result<Vec<Modality>, OrthancError> {
        let resp = self.get("modalities?expand")?;
        let json = resp.json::<Vec<Modality>>()?;
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

    pub fn upload_dicom(&self, data: Vec<u8>) -> Result<StatusResponse, OrthancError> {
        let path = "/instances";
        self.post_bytes(path, data)
    }
}

#[derive(Serialize, Debug)]
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

fn check_http_error(resp: &reqwest::blocking::Response) -> Result<(), OrthancError> {
    if resp.status() >= reqwest::StatusCode::BAD_REQUEST {
        //eprintln!("{:?}", resp.text());
        return Err(OrthancError::new(resp.status().as_str()));
    }
    Ok(())
}

mod orthanc_datetime_format {
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
