use crate::entity::*;
use crate::error::Error;
use crate::models::*;
use crate::Result;
use bytes::Bytes;
use reqwest;
use serde_json::Value;
use std::collections::HashMap;
use std::io::prelude::*;
use std::time;

/// Client type
///
/// The client itself is fairly simple. There are only 3 fields that the end-user should care
/// about: `server` (the address of the Orthanc server, an HTTP(S) URL), `username` and `password`.
///
/// Creating a new client instance:
///
/// ```
/// let client = Client::new("http://localhost:8042");
/// ```
///
/// Authentication (setting `username`/`password`) can be done by calling the `auth` method:
///
/// ```
/// client.auth("username", "password");
/// ```
///
/// Or combined:
///
/// ```
/// let client = Client::new("http://localhost:8042").auth("username", "password");
/// ```
#[derive(Debug)]
pub struct Client {
    server: String,
    username: Option<String>,
    password: Option<String>,
    client: reqwest::blocking::Client,
}

impl Client {
    /// Creates a new client instance
    ///
    /// ```
    /// let client = Client::new("http://localhost:8042");
    /// ```
    pub fn new(server: impl Into<String>) -> Client {
        let client = reqwest::blocking::ClientBuilder::new()
            .timeout(time::Duration::from_secs(600))
            .build()
            // TODO: Should we be catching the error here?
            .unwrap();
        Client {
            server: server.into(),
            username: None,
            password: None,
            client,
        }
    }

    /// Adds authentication to the client instance
    ///
    /// ```
    /// let client = Client::new("http://localhost:8042").auth("username", "password");
    /// ```
    pub fn auth(
        mut self,
        username: impl Into<String>,
        password: impl Into<String>,
    ) -> Client {
        self.username = Some(username.into());
        self.password = Some(password.into());
        self
    }

    fn add_auth(
        &self,
        request: reqwest::blocking::RequestBuilder,
    ) -> reqwest::blocking::RequestBuilder {
        match (&self.username, &self.password) {
            (Some(u), Some(p)) => request.basic_auth(u, Some(p)),
            _ => request,
        }
    }

    fn get(&self, path: &str) -> Result<Bytes> {
        let url = format!("{}/{}", self.server, &path);
        let mut request = self.client.get(&url);
        request = self.add_auth(request);
        let resp = request.send()?;
        let status = resp.status();
        let body = resp.bytes()?;
        check_http_error(status, body)
    }

    fn get_stream<W: Write>(&self, path: &str, writer: &mut W) -> Result<()> {
        let url = format!("{}/{}", self.server, &path);
        let mut request = self.client.get(&url);
        request = self.add_auth(request);
        let mut resp = request.send()?;
        let status = resp.status();

        // TODO: Simplify this
        if status >= reqwest::StatusCode::BAD_REQUEST {
            let message = format!("API error: {}", status);
            let body = resp.bytes()?;
            if body.is_empty() {
                return Err(Error::new(&message, None));
            };
            return Err(Error::new(&message, serde_json::from_slice(&body)?));
        }
        resp.copy_to(writer)?;
        Ok(())
    }

    fn post(&self, path: &str, data: Value) -> Result<Bytes> {
        let url = format!("{}/{}", self.server, path);
        let mut request = self.client.post(&url).json(&data);
        request = self.add_auth(request);
        let resp = request.send()?;
        let status = resp.status();
        let body = resp.bytes()?;
        check_http_error(status, body)
    }

    fn post_receive_stream<W: Write>(
        &self,
        path: &str,
        data: Value,
        writer: &mut W,
    ) -> Result<()> {
        let url = format!("{}/{}", self.server, path);
        let mut request = self.client.post(&url).json(&data);
        request = self.add_auth(request);
        let mut resp = request.send()?;
        let status = resp.status();

        // TODO: Simplify this
        if status >= reqwest::StatusCode::BAD_REQUEST {
            let message = format!("API error: {}", status);
            let body = resp.bytes()?;
            if body.is_empty() {
                return Err(Error::new(&message, None));
            };
            return Err(Error::new(&message, serde_json::from_slice(&body)?));
        }
        resp.copy_to(writer)?;
        Ok(())
    }

    fn post_bytes(&self, path: &str, data: &[u8]) -> Result<Bytes> {
        let url = format!("{}/{}", self.server, path);
        // TODO: .to_vec() here is probably not a good idea?
        let mut request = self.client.post(&url).body(data.to_vec());
        request = self.add_auth(request);
        let resp = request.send()?;
        let status = resp.status();
        let body = resp.bytes()?;
        check_http_error(status, body)
    }

    fn put(&self, path: &str, data: Value) -> Result<Bytes> {
        let url = format!("{}/{}", self.server, path);
        let mut request = self.client.put(&url).json(&data);
        request = self.add_auth(request);
        let resp = request.send()?;
        let status = resp.status();
        let body = resp.bytes()?;
        check_http_error(status, body)
    }

    fn delete(&self, path: &str) -> Result<Bytes> {
        let url = format!("{}/{}", self.server, &path);
        let mut request = self.client.delete(&url);
        request = self.add_auth(request);
        let resp = request.send()?;
        let status = resp.status();
        let body = resp.bytes()?;
        check_http_error(status, body)
    }

    fn list(&self, entity: &str) -> Result<Vec<String>> {
        let resp = self.get(entity)?;
        let json: Vec<String> = serde_json::from_slice(&resp)?;
        Ok(json)
    }

    /// System information
    pub fn system(&self) -> Result<System> {
        let resp = self.get("system")?;
        let json: System = serde_json::from_slice(&resp)?;
        Ok(json)
    }

    /// List modalities
    pub fn modalities(&self) -> Result<Vec<String>> {
        self.list("modalities")
    }

    /// List patients
    pub fn patients(&self) -> Result<Vec<String>> {
        self.list("patients")
    }

    /// List studies
    pub fn studies(&self) -> Result<Vec<String>> {
        self.list("studies")
    }

    /// List series
    pub fn series_list(&self) -> Result<Vec<String>> {
        self.list("series")
    }

    /// List instances
    pub fn instances(&self) -> Result<Vec<String>> {
        self.list("instances")
    }

    /// List all modalities in an expanded format
    pub fn modalities_expanded(&self) -> Result<HashMap<String, Modality>> {
        let resp = self.get("modalities?expand")?;
        let json: HashMap<String, Modality> = serde_json::from_slice(&resp)?;
        Ok(json)
    }

    /// List all patients in an expanded format
    pub fn patients_expanded(&self) -> Result<Vec<Patient>> {
        let resp = self.get("patients?expand")?;
        let json: Vec<Patient> = serde_json::from_slice(&resp)?;
        Ok(json)
    }

    /// List all studies in an expanded format
    pub fn studies_expanded(&self) -> Result<Vec<Study>> {
        let resp = self.get("studies?expand")?;
        let json: Vec<Study> = serde_json::from_slice(&resp)?;
        Ok(json)
    }

    /// List all series in an expanded format
    pub fn series_expanded(&self) -> Result<Vec<Series>> {
        let resp = self.get("series?expand")?;
        let json: Vec<Series> = serde_json::from_slice(&resp)?;
        Ok(json)
    }

    /// List all instances in an expanded format
    pub fn instances_expanded(&self) -> Result<Vec<Instance>> {
        let resp = self.get("instances?expand")?;
        let json: Vec<Instance> = serde_json::from_slice(&resp)?;
        Ok(json)
    }

    /// Get a patient by its ID
    pub fn patient(&self, id: &str) -> Result<Patient> {
        let resp = self.get(&format!("patients/{}", id))?;
        let json: Patient = serde_json::from_slice(&resp)?;
        Ok(json)
    }

    /// Get a study by its ID
    pub fn study(&self, id: &str) -> Result<Study> {
        let resp = self.get(&format!("studies/{}", id))?;
        let json: Study = serde_json::from_slice(&resp)?;
        Ok(json)
    }

    /// Get a series by its ID
    pub fn series(&self, id: &str) -> Result<Series> {
        let resp = self.get(&format!("series/{}", id))?;
        let json: Series = serde_json::from_slice(&resp)?;
        Ok(json)
    }

    /// Get an instance by its ID
    pub fn instance(&self, id: &str) -> Result<Instance> {
        let resp = self.get(&format!("instances/{}", id))?;
        let json: Instance = serde_json::from_slice(&resp)?;
        Ok(json)
    }

    /// Get all DICOM tags of an instance in a simplified format
    ///
    /// See related Orthanc documentation
    /// [section](https://book.orthanc-server.com/users/rest.html#accessing-the-dicom-fields-of-an-instance-as-a-json-file)
    /// for details
    pub fn instance_tags(&self, id: &str) -> Result<Value> {
        let resp = self.get(&format!("instances/{}/simplified-tags", id))?;
        let json: Value = serde_json::from_slice(&resp)?;
        Ok(json)
    }

    /// Get all DICOM tags of an instance in an expanded format
    ///
    /// See related Orthanc documentation
    /// [section](https://book.orthanc-server.com/users/rest.html#accessing-the-dicom-fields-of-an-instance-as-a-json-file)
    /// for details
    pub fn instance_tags_expanded(&self, id: &str) -> Result<Value> {
        let resp = self.get(&format!("instances/{}/tags", id))?;
        let json: Value = serde_json::from_slice(&resp)?;
        Ok(json)
    }

    /// Get all DICOM tags' codings of an instance
    ///
    /// Returns a [`Vec`]<[`String`]> of the following format: `["0008-0018", "0040-0260", "0040-0254"]`
    pub fn instance_content(&self, id: &str) -> Result<Vec<String>> {
        let resp = self.get(&format!("instances/{}/content", id))?;
        let json = serde_json::from_slice(&resp)?;
        Ok(json)
    }

    /// Get the value of a specific DICOM tag of an instance
    ///
    /// `tag` is the DICOM tag coding, e.g. `0008-0018`
    pub fn instance_tag(&self, id: &str, tag: &str) -> Result<String> {
        let resp = self.get(&format!("instances/{}/content/{}", id, tag))?;
        Ok(String::from_utf8_lossy(&resp).trim().to_string())
    }

    /// Download a patient as a collection of DICOM files
    ///
    /// Accepts a mutable reference to an object, that implements a [`Write`] trait, and mutates the
    /// object, writing the data into it in a streaming fashion.
    ///
    /// Streamed data is a ZIP archive
    ///
    /// Example:
    ///
    /// ```
    /// let mut file = fs::File::create("/tmp/patient.zip").unwrap();
    /// client().patient_dicom("3693b9d5-8b0e2a80-2cf45dda-d19e7c22-8749103c", &mut file).unwrap();
    /// ```
    pub fn patient_dicom<W: Write>(&self, id: &str, writer: &mut W) -> Result<()> {
        let path = format!("patients/{}/archive", id);
        self.get_stream(&path, writer)
    }

    /// Download a study as a collection of DICOM files
    ///
    /// Accepts a mutable reference to an object, that implements a [`Write`] trait, and mutates the
    /// object, writing the data into it in a streaming fashion.
    ///
    /// Streamed data is a ZIP archive
    ///
    /// Example:
    ///
    /// ```
    /// let mut file = fs::File::create("/tmp/study.zip").unwrap();
    /// client().study_dicom("3693b9d5-8b0e2a80-2cf45dda-d19e7c22-8749103c", &mut file).unwrap();
    /// ```
    pub fn study_dicom<W: Write>(&self, id: &str, writer: &mut W) -> Result<()> {
        let path = format!("studies/{}/archive", id);
        self.get_stream(&path, writer)?;
        Ok(())
    }

    /// Download a series as a collection of DICOM files
    ///
    /// Accepts a mutable reference to an object, that implements a [`Write`] trait, and mutates the
    /// object, writing the data into it in a streaming fashion.
    ///
    /// Streamed data is a ZIP archive
    ///
    /// Example:
    ///
    /// ```
    /// let mut file = fs::File::create("/tmp/series.zip").unwrap();
    /// client().series_dicom("3693b9d5-8b0e2a80-2cf45dda-d19e7c22-8749103c", &mut file).unwrap();
    /// ```
    pub fn series_dicom<W: Write>(&self, id: &str, writer: &mut W) -> Result<()> {
        let path = format!("series/{}/archive", id);
        self.get_stream(&path, writer)
    }

    /// Download an instance as a DICOM file
    ///
    /// Accepts a mutable reference to an object, that implements a [`Write`] trait, and mutates the
    /// object, writing the data into it in a streaming fashion.
    ///
    /// Example:
    ///
    /// ```
    /// let mut file = fs::File::create("/tmp/instance.dcm").unwrap();
    /// client().instance_dicom("3693b9d5-8b0e2a80-2cf45dda-d19e7c22-8749103c", &mut file).unwrap();
    /// ```
    pub fn instance_dicom<W: Write>(&self, id: &str, writer: &mut W) -> Result<()> {
        let path = format!("instances/{}/file", id);
        self.get_stream(&path, writer)
    }

    /// Delete a patient
    pub fn delete_patient(&self, id: &str) -> Result<RemainingAncestor> {
        let resp = self.delete(&format!("patients/{}", id))?;
        let json: RemainingAncestor = serde_json::from_slice(&resp)?;
        Ok(json)
    }

    /// Delete a study
    pub fn delete_study(&self, id: &str) -> Result<RemainingAncestor> {
        let resp = self.delete(&format!("studies/{}", id))?;
        let json: RemainingAncestor = serde_json::from_slice(&resp)?;
        Ok(json)
    }

    /// Delete a series
    pub fn delete_series(&self, id: &str) -> Result<RemainingAncestor> {
        let resp = self.delete(&format!("series/{}", id))?;
        let json: RemainingAncestor = serde_json::from_slice(&resp)?;
        Ok(json)
    }

    /// Delete an instance
    pub fn delete_instance(&self, id: &str) -> Result<RemainingAncestor> {
        let resp = self.delete(&format!("instances/{}", id))?;
        let json: RemainingAncestor = serde_json::from_slice(&resp)?;
        Ok(json)
    }

    /// Send a C-ECHO request to a remote modality
    ///
    /// If no error is returned, the request was successful
    pub fn echo(&self, modality: &str, timeout: Option<u32>) -> Result<()> {
        let mut data = HashMap::new();
        if let Some(to) = timeout {
            data.insert("Timeout", to);
        }
        self.post(
            &format!("modalities/{}/echo", modality),
            serde_json::json!(data),
        )
        .map(|_| ())
    }

    /// Send a C-STORE request to a remote modality
    ///
    /// `ids` is a slice of entity IDs to send. An ID can signify either of [`Patient`], [`Study`],
    /// [`Series`] or [`Instance`]
    pub fn store(&self, modality: &str, ids: &[&str]) -> Result<StoreResult> {
        let resp = self.post(
            &format!("modalities/{}/store", modality),
            serde_json::json!(ids),
        )?;
        let json: StoreResult = serde_json::from_slice(&resp)?;
        Ok(json)
    }

    fn anonymize(
        &self,
        entity: &str,
        id: &str,
        anonymization: Option<Anonymization>,
    ) -> Result<ModificationResult> {
        let data = match anonymization {
            Some(a) => a,
            // TODO: Just pass an empty object?
            None => Anonymization {
                replace: None,
                keep: None,
                keep_private_tags: None,
                dicom_version: None,
                force: None,
            },
        };
        let resp = self.post(
            &format!("{}/{}/anonymize", entity, id),
            serde_json::to_value(data)?,
        )?;
        let json: ModificationResult = serde_json::from_slice(&resp)?;
        Ok(json)
    }

    fn modify(
        &self,
        entity: &str,
        id: &str,
        modification: Modification,
    ) -> Result<ModificationResult> {
        let resp = self.post(
            &format!("{}/{}/modify", entity, id),
            serde_json::to_value(modification)?,
        )?;
        let json: ModificationResult = serde_json::from_slice(&resp)?;
        Ok(json)
    }

    /// Anonymize a patient
    pub fn anonymize_patient(
        &self,
        id: &str,
        anonymization: Option<Anonymization>,
    ) -> Result<ModificationResult> {
        self.anonymize("patients", id, anonymization)
    }

    /// Anonymize a study
    pub fn anonymize_study(
        &self,
        id: &str,
        anonymization: Option<Anonymization>,
    ) -> Result<ModificationResult> {
        self.anonymize("studies", id, anonymization)
    }

    /// Anonymize a series
    pub fn anonymize_series(
        &self,
        id: &str,
        anonymization: Option<Anonymization>,
    ) -> Result<ModificationResult> {
        self.anonymize("series", id, anonymization)
    }

    /// Anonymize an instance
    ///
    /// Accepts a mutable reference to an object, that implements a [`Write`] trait, and mutates the
    /// object, writing the data into it in a streaming fashion.
    ///
    /// Example:
    ///
    /// ```
    /// let mut file = fs::File::create("/tmp/anonymized_instance.dcm").unwrap();
    /// client().anonymize_instance("3693b9d5-8b0e2a80-2cf45dda-d19e7c22-8749103c", None, &mut file).unwrap();
    /// ```
    pub fn anonymize_instance<W: Write>(
        &self,
        id: &str,
        anonymization: Option<Anonymization>,
        writer: &mut W,
    ) -> Result<()> {
        let data = match anonymization {
            Some(a) => a,
            // TODO: Just pass an empty object?
            None => Anonymization {
                replace: None,
                keep: None,
                keep_private_tags: None,
                dicom_version: None,
                force: None,
            },
        };
        self.post_receive_stream(
            &format!("instances/{}/anonymize", id),
            serde_json::to_value(data)?,
            writer,
        )?;
        Ok(())
    }

    /// Modify a patient
    pub fn modify_patient(
        &self,
        id: &str,
        modification: Modification,
    ) -> Result<ModificationResult> {
        self.modify("patients", id, modification)
    }

    /// Modify a study
    pub fn modify_study(
        &self,
        id: &str,
        modification: Modification,
    ) -> Result<ModificationResult> {
        self.modify("studies", id, modification)
    }

    /// Modify a series
    pub fn modify_series(
        &self,
        id: &str,
        modification: Modification,
    ) -> Result<ModificationResult> {
        self.modify("series", id, modification)
    }

    /// Modify an instance
    ///
    /// Accepts a mutable reference to an object, that implements a [`Write`] trait, and mutates the
    /// object, writing the data into it in a streaming fashion.
    ///
    /// Example:
    ///
    /// ```
    /// let mut file = fs::File::create("/tmp/modified_instance.dcm").unwrap();
    /// let modification = Modification {
    ///     replace: None,
    ///     remove: vec!["PatientName"],
    ///     force: false,
    /// };
    /// client().modify_instance("3693b9d5-8b0e2a80-2cf45dda-d19e7c22-8749103c", modification, &mut file).unwrap();
    /// ```
    pub fn modify_instance<W: Write>(
        &self,
        id: &str,
        modification: Modification,
        writer: &mut W,
    ) -> Result<()> {
        self.post_receive_stream(
            &format!("instances/{}/modify", id),
            serde_json::to_value(modification)?,
            writer,
        )?;
        Ok(())
    }

    /// Upload a DICOM file to Orthanc
    ///
    /// ```
    /// let data = fs::read("/tmp/instance.dcm").unwrap();
    /// let client = Client::new("http://localhost:8042");
    /// client.upload(&data).unwrap();
    /// ```
    pub fn upload(&self, data: &[u8]) -> Result<UploadResult> {
        let resp = self.post_bytes("instances", data)?;
        let json: UploadResult = serde_json::from_slice(&resp)?;
        Ok(json)
    }

    // TODO: The following two methods are exactly the same
    /// Create a modality
    pub fn create_modality(&self, name: &str, modality: Modality) -> Result<()> {
        self.put(
            &format!("modalities/{}", name),
            serde_json::to_value(modality)?,
        )
        .map(|_| ())
    }

    /// Modify a modality
    pub fn modify_modality(&self, name: &str, modality: Modality) -> Result<()> {
        self.put(
            &format!("modalities/{}", name),
            serde_json::to_value(modality)?,
        )
        .map(|_| ())
    }

    /// Delete a modality
    pub fn delete_modality(&self, name: &str) -> Result<()> {
        self.delete(&format!("modalities/{}", name)).map(|_| ())
    }

    /// Search for Entities in Orthanc
    pub fn search<T: Entity>(&self, query: HashMap<String, String>) -> Result<Vec<T>> {
        let kind = T::kind();
        let search = Search {
            level: kind,
            query,
            expand: Some(true),
        };
        let resp = self.post("tools/find", serde_json::to_value(search)?)?;
        let json: Vec<T> = serde_json::from_slice(&resp)?;
        Ok(json)
    }
}

fn check_http_error(status: reqwest::StatusCode, body: Bytes) -> Result<Bytes> {
    if status >= reqwest::StatusCode::BAD_REQUEST {
        let message = format!("API error: {}", status);
        if body.is_empty() {
            return Err(Error::new(&message, None));
        };
        return Err(Error::new(&message, serde_json::from_slice(&body)?));
    }
    Ok(body)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::ApiError;
    use chrono::NaiveDate;
    use httpmock::{Method, Mock, MockServer};
    use maplit::hashmap;
    use serde_json::Value;
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
        let mock_server = MockServer::start();
        let url = mock_server.url("");

        Mock::new()
            .expect_method(Method::GET)
            .expect_path("/patients")
            .return_status(200)
            .return_header("Content-Type", "application/json")
            .return_body("foo")
            .create_on(&mock_server);

        let cl = Client::new(url);
        let resp = cl.patients();
        assert_eq!(
            resp.unwrap_err(),
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

    #[test]
    fn test_default_fields() {
        let cl = Client::new("http://localhost:8042");
        assert_eq!(cl.server, "http://localhost:8042".to_string());
        assert_eq!(cl.username, None);
        assert_eq!(cl.password, None);
    }

    #[test]
    fn test_auth() {
        let cl = Client::new("http://localhost:8042").auth("foo", "bar");
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

        let cl = Client::new(url).auth("foo", "bar");
        let resp = cl.get("foo").unwrap();

        assert_eq!(resp, "bar");
        assert_eq!(m.times_called(), 1);
    }

    #[test]
    fn test_get_stream() {
        let mock_server = MockServer::start();
        let url = mock_server.url("");

        let m = Mock::new()
            .expect_method(Method::GET)
            .expect_path("/foo")
            .expect_header("Authorization", "Basic Zm9vOmJhcg==")
            .return_status(200)
            .return_body("bar")
            .create_on(&mock_server);

        let cl = Client::new(url).auth("foo", "bar");
        let mut writer: Vec<u8> = vec![];
        cl.get_stream("foo", &mut writer).unwrap();

        assert_eq!(&writer, &b"bar");
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

        let cl = Client::new(url).auth("foo", "bar");
        let resp = cl.post("foo", serde_json::json!("bar")).unwrap();

        assert_eq!(resp, "baz");
        assert_eq!(m.times_called(), 1);
    }

    #[test]
    fn test_post_receive_stream() {
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

        let cl = Client::new(url).auth("foo", "bar");
        let mut writer: Vec<u8> = vec![];
        cl.post_receive_stream("foo", serde_json::json!("bar"), &mut writer)
            .unwrap();

        assert_eq!(&writer, &b"baz");
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

        let cl = Client::new(url).auth("foo", "bar");
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

        let cl = Client::new(url).auth("foo", "bar");
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

        let cl = Client::new(url);
        let resp = cl.get("foo");

        assert_eq!(
            resp.unwrap_err(),
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
        assert_eq!(m.times_called(), 1);
    }

    #[test]
    fn test_get_stream_error_response() {
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

        let cl = Client::new(url);
        let mut writer: Vec<u8> = vec![];
        let resp = cl.get_stream("foo", &mut writer);

        assert_eq!(
            resp.unwrap_err(),
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

        let cl = Client::new(url);
        let resp = cl.post("foo", serde_json::json!("bar"));

        assert_eq!(
            resp.unwrap_err(),
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
        assert_eq!(m.times_called(), 1);
    }

    #[test]
    fn test_post_receive_stream_error_response() {
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

        let cl = Client::new(url);
        let mut writer: Vec<u8> = vec![];
        let resp = cl.post_receive_stream("foo", serde_json::json!("bar"), &mut writer);

        assert_eq!(
            resp.unwrap_err(),
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

        let cl = Client::new(url);
        let resp = cl.post_bytes("foo", &[13, 42, 17]);

        assert_eq!(
            resp.unwrap_err(),
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

        let cl = Client::new(url);
        let resp = cl.delete("foo");

        assert_eq!(
            resp.unwrap_err(),
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

        let cl = Client::new(url);
        let resp = cl.get("foo");

        assert!(resp.is_err());
        assert_eq!(
            resp.unwrap_err(),
            Error {
                message: "API error: 404 Not Found".to_string(),
                details: None,
            },
        );
        assert_eq!(m.times_called(), 1);
    }

    #[test]
    fn test_get_system_info() {
        let mock_server = MockServer::start();
        let url = mock_server.url("");

        let m = Mock::new()
            .expect_method(Method::GET)
            .expect_path("/system")
            .return_status(200)
            .return_header("Content-Type", "application/json")
            .return_body(
                r#"
                    {
                        "ApiVersion": 8,
                        "DatabaseBackendPlugin": null,
                        "DatabaseVersion": 6,
                        "DicomAet": "ORTHANC",
                        "DicomPort": 4242,
                        "HttpPort": 8042,
                        "IsHttpServerSecure": true,
                        "Name": "Orthanc",
                        "PluginsEnabled": true,
                        "StorageAreaPlugin": null,
                        "Version": "1.8.0"
                    }
                "#,
            )
            .create_on(&mock_server);

        let cl = Client::new(url);
        let system = cl.system().unwrap();

        assert_eq!(
            system,
            System {
                name: "Orthanc".to_string(),
                version: "1.8.0".to_string(),
                api_version: 8,
                database_version: 6,
                database_backend_plugin: None,
                dicom_aet: "ORTHANC".to_string(),
                dicom_port: 4242,
                http_port: 8042,
                is_http_server_secure: true,
                plugins_enabled: true,
                storage_area_plugin: None,
            }
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

        let cl = Client::new(url);
        let modalities = cl.modalities().unwrap();

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

        let cl = Client::new(url);
        let patient_ids = cl.patients().unwrap();

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

        let cl = Client::new(url);
        let patient_ids = cl.studies().unwrap();

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

        let cl = Client::new(url);
        let patient_ids = cl.series_list().unwrap();

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

        let cl = Client::new(url);
        let patient_ids = cl.instances().unwrap();

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

        let cl = Client::new(url);
        let modalities = cl.modalities_expanded().unwrap();

        assert_eq!(
            modalities,
            hashmap! {
                "foo".to_string() => Modality {
                    aet: "FOO".to_string(),
                    host: "localhost".to_string(),
                    port: 11114,
                    manufacturer: Some("Generic".to_string()),
                    allow_c_echo: Some(true),
                    allow_c_find: Some(true),
                    allow_c_get: Some(true),
                    allow_c_move: Some(true),
                    allow_c_store: Some(true),
                    allow_n_action: Some(false),
                    allow_n_event_report: Some(false),
                    allow_transcoding: Some(false),
                },
                "bar".to_string() => Modality {
                    aet: "BAR".to_string(),
                    host: "remotehost".to_string(),
                    port: 11113,
                    manufacturer: Some("Generic".to_string()),
                    allow_c_echo: Some(true),
                    allow_c_find: Some(true),
                    allow_c_get: Some(true),
                    allow_c_move: Some(true),
                    allow_c_store: Some(true),
                    allow_n_action: Some(false),
                    allow_n_event_report: Some(false),
                    allow_transcoding: Some(false),
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

        let cl = Client::new(url);
        let patients = cl.patients_expanded().unwrap();

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
                    entity: EntityKind::Patient,
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
                    entity: EntityKind::Patient,
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

        let cl = Client::new(url);
        let studies = cl.studies_expanded().unwrap();

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
                    entity: EntityKind::Study,
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
                    entity: EntityKind::Study,
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

        let cl = Client::new(url);
        let series = cl.series_expanded().unwrap();

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
                    entity: EntityKind::Series,
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
                    entity: EntityKind::Series,
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

        let cl = Client::new(url);
        let instances = cl.instances_expanded().unwrap();

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
                    index_in_series: Some(13),
                    file_uuid: "d8c5eff3-986c-4fe4-b06e-7e52b2a4238e".to_string(),
                    file_size: 139402,
                    modified_from: Some(
                        "22c54cb6-28302a69-3ff454a3-676b98f4-b84cd80a".to_string()
                    ),
                    entity: EntityKind::Instance,
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
                    index_in_series: None,
                    file_uuid: "86bbad65-2c98-4cb0-bf77-0ef0243410a4".to_string(),
                    file_size: 381642,
                    modified_from: None,
                    entity: EntityKind::Instance,
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

        let cl = Client::new(url);
        let patient = cl.patient("foo").unwrap();

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
                entity: EntityKind::Patient,
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

        let cl = Client::new(url);
        let study = cl.study("foo").unwrap();

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
                entity: EntityKind::Study,
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

        let cl = Client::new(url);
        let instance = cl.instance("foo").unwrap();

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
                index_in_series: Some(13),
                file_uuid: "d8c5eff3-986c-4fe4-b06e-7e52b2a4238e".to_string(),
                file_size: 139402,
                modified_from: Some(
                    "22c54cb6-28302a69-3ff454a3-676b98f4-b84cd80a".to_string()
                ),
                entity: EntityKind::Instance,
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

        let cl = Client::new(url);
        let series = cl.series("foo").unwrap();

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
                entity: EntityKind::Series,
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

        let cl = Client::new(url);
        let resp = cl.instance_tags("foo").unwrap();

        let expected_resp: Value = serde_json::from_str(body).unwrap();
        assert_eq!(resp, expected_resp);
        assert_eq!(m.times_called(), 1);
    }

    #[test]
    fn test_instance_content() {
        let mock_server = MockServer::start();
        let url = mock_server.url("");
        let body = r#"
            [
                "0040-0253",
                "0040-0254",
                "0040-0260"

            ]
        "#;
        let m = Mock::new()
            .expect_method(Method::GET)
            .expect_path("/instances/foo/content")
            .return_status(200)
            .return_header("Content-Type", "application/json")
            .return_body(body)
            .create_on(&mock_server);

        let cl = Client::new(url);
        let resp = cl.instance_content("foo").unwrap();

        assert_eq!(resp, ["0040-0253", "0040-0254", "0040-0260"]);
        assert_eq!(m.times_called(), 1);
    }

    #[test]
    fn test_instance_tag() {
        let mock_server = MockServer::start();
        let url = mock_server.url("");
        let m = Mock::new()
            .expect_method(Method::GET)
            .expect_path("/instances/foo/content/bar")
            .return_status(200)
            .return_header("Content-Type", "application/json")
            .return_body("FOOBAR")
            .create_on(&mock_server);

        let cl = Client::new(url);
        let resp = cl.instance_tag("foo", "bar").unwrap();

        assert_eq!(resp, "FOOBAR");
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

        let cl = Client::new(url);
        let resp = cl.instance_tags_expanded("foo").unwrap();

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

        let cl = Client::new(url);
        let mut writer: Vec<u8> = vec![];
        cl.patient_dicom("foo", &mut writer).unwrap();

        assert_eq!(&writer, &b"foobar");
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

        let cl = Client::new(url);
        let mut writer: Vec<u8> = vec![];
        cl.study_dicom("foo", &mut writer).unwrap();

        assert_eq!(&writer, &b"foobar");
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

        let cl = Client::new(url);
        let mut writer: Vec<u8> = vec![];
        cl.series_dicom("foo", &mut writer).unwrap();

        assert_eq!(&writer, &b"foobar");
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

        let cl = Client::new(url);
        let mut writer: Vec<u8> = vec![];
        cl.instance_dicom("foo", &mut writer).unwrap();

        assert_eq!(&writer, &b"foobar");
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

        let cl = Client::new(url);
        let resp = cl.store("them", &["bar", "baz", "qux"]).unwrap();

        assert_eq!(
            resp,
            StoreResult {
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

        let cl = Client::new(url);
        let resp = cl
            .modify(
                "studies",
                "foo",
                Modification {
                    replace: Some(hashmap! {"Tag1".to_string() => "value1".to_string()}),
                    remove: Some(vec!["Tag2".to_string()]),
                    force: None,
                },
            )
            .unwrap();

        assert_eq!(
            resp,
            ModificationResult {
                id: "86a3054b-32bb888a-e5f42e28-4b2e82d2-b1d7e14c".to_string(),
                patient_id: "86a3054b-32bb888a-e5f42e28-4b2e82d2-b1d7e14c".to_string(),
                path: "/studies/86a3054b-32bb888a-e5f42e28-4b2e82d2-b1d7e14c".to_string(),
                entity: EntityKind::Study
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

        let cl = Client::new(url);
        let resp = cl
            .anonymize(
                "studies",
                "foo",
                Some(Anonymization {
                    replace: Some(hashmap! {"Tag1".to_string() => "value1".to_string()}),
                    keep: Some(vec!["Tag2".to_string(), "Tag3".to_string()]),
                    keep_private_tags: None,
                    dicom_version: None,
                    force: None,
                }),
            )
            .unwrap();

        assert_eq!(
            resp,
            ModificationResult {
                id: "86a3054b-32bb888a-e5f42e28-4b2e82d2-b1d7e14c".to_string(),
                patient_id: "86a3054b-32bb888a-e5f42e28-4b2e82d2-b1d7e14c".to_string(),
                path: "/studies/86a3054b-32bb888a-e5f42e28-4b2e82d2-b1d7e14c".to_string(),
                entity: EntityKind::Study,
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

        let cl = Client::new(url);
        let resp = cl
            .modify_patient(
                "foo",
                Modification {
                    replace: Some(hashmap! {"Tag1".to_string() => "value1".to_string()}),
                    remove: Some(vec!["Tag2".to_string()]),
                    force: Some(true),
                },
            )
            .unwrap();

        assert_eq!(
            resp,
            ModificationResult {
                id: "86a3054b-32bb888a-e5f42e28-4b2e82d2-b1d7e14c".to_string(),
                patient_id: "86a3054b-32bb888a-e5f42e28-4b2e82d2-b1d7e14c".to_string(),
                path: "/patients/86a3054b-32bb888a-e5f42e28-4b2e82d2-b1d7e14c".to_string(),
                entity: EntityKind::Patient,
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

        let cl = Client::new(url);
        let resp = cl
            .modify_study(
                "foo",
                Modification {
                    replace: Some(hashmap! {"Tag1".to_string() => "value1".to_string()}),
                    remove: Some(vec!["Tag2".to_string()]),
                    force: None,
                },
            )
            .unwrap();

        assert_eq!(
            resp,
            ModificationResult {
                id: "86a3054b-32bb888a-e5f42e28-4b2e82d2-b1d7e14c".to_string(),
                patient_id: "86a3054b-32bb888a-e5f42e28-4b2e82d2-b1d7e14c".to_string(),
                path: "/studies/86a3054b-32bb888a-e5f42e28-4b2e82d2-b1d7e14c".to_string(),
                entity: EntityKind::Study,
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

        let cl = Client::new(url);
        let resp = cl
            .modify_series(
                "foo",
                Modification {
                    replace: Some(hashmap! {"Tag1".to_string() => "value1".to_string()}),
                    remove: Some(vec!["Tag2".to_string()]),
                    force: None,
                },
            )
            .unwrap();

        assert_eq!(
            resp,
            ModificationResult {
                id: "86a3054b-32bb888a-e5f42e28-4b2e82d2-b1d7e14c".to_string(),
                patient_id: "86a3054b-32bb888a-e5f42e28-4b2e82d2-b1d7e14c".to_string(),
                path: "/series/86a3054b-32bb888a-e5f42e28-4b2e82d2-b1d7e14c".to_string(),
                entity: EntityKind::Series,
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

        let cl = Client::new(url);
        let mut writer: Vec<u8> = vec![];

        cl.modify_instance(
            "foo",
            Modification {
                replace: Some(hashmap! {"Tag1".to_string() => "value1".to_string()}),
                remove: Some(vec!["Tag2".to_string()]),
                force: None,
            },
            &mut writer,
        )
        .unwrap();

        assert_eq!(&writer, &b"foobar");
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
                force: None,
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

        let cl = Client::new(url);
        let resp = cl
            .anonymize_patient(
                "foo",
                Some(Anonymization {
                    replace: Some(hashmap! {"Tag1".to_string() => "value1".to_string()}),
                    keep: Some(vec!["Tag2".to_string(), "Tag3".to_string()]),
                    keep_private_tags: None,
                    dicom_version: None,
                    force: None,
                }),
            )
            .unwrap();

        assert_eq!(
            resp,
            ModificationResult {
                id: "86a3054b-32bb888a-e5f42e28-4b2e82d2-b1d7e14c".to_string(),
                patient_id: "86a3054b-32bb888a-e5f42e28-4b2e82d2-b1d7e14c".to_string(),
                path: "/patients/86a3054b-32bb888a-e5f42e28-4b2e82d2-b1d7e14c".to_string(),
                entity: EntityKind::Patient,
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

        let cl = Client::new(url);
        let resp = cl
            .anonymize_study(
                "foo",
                Some(Anonymization {
                    replace: Some(hashmap! {"Tag1".to_string() => "value1".to_string()}),
                    keep: Some(vec!["Tag2".to_string(), "Tag3".to_string()]),
                    keep_private_tags: Some(true),
                    dicom_version: None,
                    force: None,
                }),
            )
            .unwrap();

        assert_eq!(
            resp,
            ModificationResult {
                id: "86a3054b-32bb888a-e5f42e28-4b2e82d2-b1d7e14c".to_string(),
                patient_id: "86a3054b-32bb888a-e5f42e28-4b2e82d2-b1d7e14c".to_string(),
                path: "/studies/86a3054b-32bb888a-e5f42e28-4b2e82d2-b1d7e14c".to_string(),
                entity: EntityKind::Study,
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

        let cl = Client::new(url);
        let resp = cl
            .anonymize_series(
                "foo",
                Some(Anonymization {
                    replace: Some(hashmap! {"Tag1".to_string() => "value1".to_string()}),
                    keep: Some(vec!["Tag2".to_string(), "Tag3".to_string()]),
                    keep_private_tags: Some(false),
                    dicom_version: None,
                    force: None,
                }),
            )
            .unwrap();

        assert_eq!(
            resp,
            ModificationResult {
                id: "86a3054b-32bb888a-e5f42e28-4b2e82d2-b1d7e14c".to_string(),
                patient_id: "86a3054b-32bb888a-e5f42e28-4b2e82d2-b1d7e14c".to_string(),
                path: "/series/86a3054b-32bb888a-e5f42e28-4b2e82d2-b1d7e14c".to_string(),
                entity: EntityKind::Series,
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
                force: None,
            })
            .return_status(200)
            .return_body("foobar")
            .create_on(&mock_server);

        let cl = Client::new(url);
        let mut writer: Vec<u8> = vec![];
        cl.anonymize_instance(
            "foo",
            Some(Anonymization {
                replace: Some(hashmap! {"Tag1".to_string() => "value1".to_string()}),
                keep: Some(vec!["Tag2".to_string(), "Tag3".to_string()]),
                keep_private_tags: None,
                dicom_version: None,
                force: None,
            }),
            &mut writer,
        )
        .unwrap();

        assert_eq!(&writer, &b"foobar");
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

        let cl = Client::new(url);
        let resp = cl.delete_patient("foo").unwrap();

        assert_eq!(
            resp,
            RemainingAncestor {
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

        let cl = Client::new(url);
        let resp = cl.delete_study("foo").unwrap();

        assert_eq!(
            resp,
            RemainingAncestor {
                remaining_ancestor: Some(Ancestor {
                    id: "bar".to_string(),
                    path: "/patients/bar".to_string(),
                    entity: EntityKind::Patient,
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

        let cl = Client::new(url);
        let resp = cl.delete_series("foo").unwrap();

        assert_eq!(
            resp,
            RemainingAncestor {
                remaining_ancestor: Some(Ancestor {
                    id: "bar".to_string(),
                    path: "/studies/bar".to_string(),
                    entity: EntityKind::Study,
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

        let cl = Client::new(url);
        let resp = cl.delete_instance("foo").unwrap();

        assert_eq!(
            resp,
            RemainingAncestor {
                remaining_ancestor: Some(Ancestor {
                    id: "bar".to_string(),
                    path: "/series/bar".to_string(),
                    entity: EntityKind::Series,
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

        let cl = Client::new(url);
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

        let cl = Client::new(url);
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

        let cl = Client::new(url);
        let resp = cl.echo("foo", None);

        assert_eq!(
            resp.unwrap_err(),
            Error {
                message: "API error: 500 Internal Server Error".to_string(),
                details: None
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

        let cl = Client::new(url);
        let resp = cl.upload("quux".as_bytes()).unwrap();

        assert_eq!(
            resp,
            UploadResult {
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

    // The following 2 tests are exactly the same except one calls `create_modality`,
    // the other one calls `modify_modality`.
    #[test]
    fn test_create_modality() {
        let mock_server = MockServer::start();
        let url = mock_server.url("");

        let m = Mock::new()
            .expect_method(Method::PUT)
            .expect_path("/modalities/bazqux")
            .expect_json_body(&Modality {
                aet: "foobar".to_string(),
                host: "localhost".to_string(),
                port: 11113,
                manufacturer: None,
                allow_c_echo: None,
                allow_c_find: None,
                allow_c_get: None,
                allow_c_move: None,
                allow_c_store: None,
                allow_n_action: None,
                allow_n_event_report: None,
                allow_transcoding: None,
            })
            .return_status(200)
            .return_body("")
            .create_on(&mock_server);

        let cl = Client::new(url);
        let modality = Modality {
            aet: "foobar".to_string(),
            host: "localhost".to_string(),
            port: 11113,
            manufacturer: None,
            allow_c_echo: None,
            allow_c_find: None,
            allow_c_get: None,
            allow_c_move: None,
            allow_c_store: None,
            allow_n_action: None,
            allow_n_event_report: None,
            allow_transcoding: None,
        };
        let resp = cl.create_modality("bazqux", modality).unwrap();

        assert_eq!(resp, ());
        assert_eq!(m.times_called(), 1);
    }

    #[test]
    fn test_modify_modality() {
        let mock_server = MockServer::start();
        let url = mock_server.url("");

        let m = Mock::new()
            .expect_method(Method::PUT)
            .expect_path("/modalities/bazqux")
            .expect_json_body(&Modality {
                aet: "foobar".to_string(),
                host: "localhost".to_string(),
                port: 11113,
                manufacturer: None,
                allow_c_echo: None,
                allow_c_find: None,
                allow_c_get: None,
                allow_c_move: None,
                allow_c_store: None,
                allow_n_action: None,
                allow_n_event_report: None,
                allow_transcoding: None,
            })
            .return_status(200)
            .return_body("")
            .create_on(&mock_server);

        let cl = Client::new(url);
        let modality = Modality {
            aet: "foobar".to_string(),
            host: "localhost".to_string(),
            port: 11113,
            manufacturer: None,
            allow_c_echo: None,
            allow_c_find: None,
            allow_c_get: None,
            allow_c_move: None,
            allow_c_store: None,
            allow_n_action: None,
            allow_n_event_report: None,
            allow_transcoding: None,
        };
        let resp = cl.modify_modality("bazqux", modality).unwrap();

        assert_eq!(resp, ());
        assert_eq!(m.times_called(), 1);
    }

    #[test]
    fn test_delete_modality() {
        let mock_server = MockServer::start();
        let url = mock_server.url("");

        let m = Mock::new()
            .expect_method(Method::DELETE)
            .expect_path("/modalities/bazqux")
            .return_status(200)
            .return_body("")
            .create_on(&mock_server);

        let cl = Client::new(url);
        let resp = cl.delete_modality("bazqux").unwrap();

        assert_eq!(resp, ());
        assert_eq!(m.times_called(), 1);
    }

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

    #[test]
    fn test_get_dicom_tag_value_patient() {
        let patient = Patient {
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
            entity: EntityKind::Patient,
            anonymized_from: None,
        };
        assert_eq!(patient.main_dicom_tag("PatientID"), Some("123456789"));
        assert_eq!(patient.main_dicom_tag("FooBar"), None);
    }

    #[test]
    fn test_get_dicom_tag_value_study() {
        let study = Study {
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
                "2ab7dbe7-f1a18a78-86145443-18a8ff93-0b65f2b2".to_string(),
            ]
            .to_vec(),
            entity: EntityKind::Study,
            anonymized_from: None,
        };
        assert_eq!(study.main_dicom_tag("StudyID"), Some("1742"));
        assert_eq!(study.main_dicom_tag("PatientID"), Some("c137"));
        assert_eq!(study.main_dicom_tag("FooBar"), None);
    }

    #[test]
    fn test_get_dicom_tag_value_series() {
        let series = Series {
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
            entity: EntityKind::Series,
            anonymized_from: None,
        };
        assert_eq!(series.main_dicom_tag("SeriesNumber"), Some("1101"));
        assert_eq!(series.main_dicom_tag("FooBar"), None);
    }

    #[test]
    fn test_get_dicom_tag_value_instance() {
        let instance = Instance {
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
            index_in_series: Some(13),
            file_uuid: "d8c5eff3-986c-4fe4-b06e-7e52b2a4238e".to_string(),
            file_size: 139402,
            modified_from: Some("22c54cb6-28302a69-3ff454a3-676b98f4-b84cd80a".to_string()),
            entity: EntityKind::Instance,
            anonymized_from: None,
        };
        assert_eq!(instance.main_dicom_tag("InstanceNumber"), Some("13"));
        assert_eq!(instance.main_dicom_tag("FooBar"), None);
    }

    #[test]
    fn test_search_patient_level() {
        let mock_server = MockServer::start();
        let url = mock_server.url("");

        let m = Mock::new()
            .expect_method(Method::POST)
            .expect_path("/tools/find")
            .expect_json_body(&Search {
                level: EntityKind::Patient,
                query: hashmap! {"PatientID".to_string() => "foobar".to_string()},
                expand: Some(true),
            })
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

        let cl = Client::new(url);
        let patients: Vec<Patient> = cl
            .search(hashmap! {"PatientID".to_string() => "foobar".to_string()})
            .unwrap();

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
                    entity: EntityKind::Patient,
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
                    entity: EntityKind::Patient,
                    anonymized_from: None
                },
            ]
        );
        assert_eq!(m.times_called(), 1);
    }

    #[test]
    fn test_search_study_level() {
        let mock_server = MockServer::start();
        let url = mock_server.url("");

        let m = Mock::new()
            .expect_method(Method::POST)
            .expect_path("/tools/find")
            .expect_json_body(&Search {
                level: EntityKind::Study,
                query: hashmap! {"StudyID".to_string() => "foobar".to_string()},
                expand: Some(true),
            })
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

        let cl = Client::new(url);
        let studies: Vec<Study> = cl
            .search(hashmap! {"StudyID".to_string() => "foobar".to_string()})
            .unwrap();

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
                    entity: EntityKind::Study,
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
                    entity: EntityKind::Study,
                    anonymized_from: None
                },
            ]
        );
        assert_eq!(m.times_called(), 1);
    }

    #[test]
    fn test_search_series_level() {
        let mock_server = MockServer::start();
        let url = mock_server.url("");

        let m = Mock::new()
            .expect_method(Method::POST)
            .expect_path("/tools/find")
            .expect_json_body(&Search {
                level: EntityKind::Series,
                query: hashmap! {"SeriesID".to_string() => "foobar".to_string()},
                expand: Some(true),
            })
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

        let cl = Client::new(url);
        let series: Vec<Series> = cl
            .search(hashmap! {"SeriesID".to_string() => "foobar".to_string()})
            .unwrap();

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
                    entity: EntityKind::Series,
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
                    entity: EntityKind::Series,
                    anonymized_from: None
                },
            ]
        );
        assert_eq!(m.times_called(), 1);
    }

    #[test]
    fn test_search_instance_level() {
        let mock_server = MockServer::start();
        let url = mock_server.url("");

        let m = Mock::new()
            .expect_method(Method::POST)
            .expect_path("/tools/find")
            .expect_json_body(&Search {
                level: EntityKind::Instance,
                query: hashmap! {"InstanceID".to_string() => "foobar".to_string()},
                expand: Some(true),
            })
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

        let cl = Client::new(url);
        let instances: Vec<Instance> = cl
            .search(hashmap! {"InstanceID".to_string() => "foobar".to_string()})
            .unwrap();

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
                    index_in_series: Some(13),
                    file_uuid: "d8c5eff3-986c-4fe4-b06e-7e52b2a4238e".to_string(),
                    file_size: 139402,
                    modified_from: Some(
                        "22c54cb6-28302a69-3ff454a3-676b98f4-b84cd80a".to_string()
                    ),
                    entity: EntityKind::Instance,
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
                    index_in_series: None,
                    file_uuid: "86bbad65-2c98-4cb0-bf77-0ef0243410a4".to_string(),
                    file_size: 381642,
                    modified_from: None,
                    entity: EntityKind::Instance,
                    anonymized_from: None
                },
            ]
        );
        assert_eq!(m.times_called(), 1);
    }

    #[test]
    fn test_search_error() {
        let mock_server = MockServer::start();
        let url = mock_server.url("");

        let m = Mock::new()
            .expect_method(Method::POST)
            .expect_path("/tools/find")
            .expect_json_body(&Search {
                level: EntityKind::Patient,
                query: hashmap! {"PatientID".to_string() => "foobar".to_string()},
                expand: Some(true),
            })
            .return_status(500)
            .return_header("Content-Type", "application/json")
            .return_body(
                r#"
                    {
                       "HttpError" : "Internal Server Error",
                       "HttpStatus" : 500,
                       "Message" : "Unknown DICOM tag",
                       "Method" : "POST",
                       "OrthancError" : "Unknown DICOM tag",
                       "OrthancStatus" : 27,
                       "Uri" : "/tools/find"
                    }
               "#,
            )
            .create_on(&mock_server);

        let cl = Client::new(url);
        let res: Result<Vec<Patient>> =
            cl.search(hashmap! {"PatientID".to_string() => "foobar".to_string()});

        assert_eq!(
            res.unwrap_err(),
            Error {
                message: "API error: 500 Internal Server Error".to_string(),
                details: Some(ApiError {
                    method: "POST".to_string(),
                    uri: "/tools/find".to_string(),
                    message: "Unknown DICOM tag".to_string(),
                    details: None,
                    http_status: 500,
                    http_error: "Internal Server Error".to_string(),
                    orthanc_status: 27,
                    orthanc_error: "Unknown DICOM tag".to_string(),
                },),
            },
        );
        assert_eq!(m.times_called(), 1);
    }

    #[test]
    fn test_modification_deserialize() {
        let json = r#"
            {
                "replace": {
                    "Foo": "42",
                    "Bar": "17"
                },
                "remove": ["Baz", "Qux"],
                "force": true
            }
        "#;
        let m1: Modification = serde_json::from_str(json).unwrap();
        assert_eq!(
            m1,
            Modification {
                replace: Some(
                    hashmap! {"Foo".to_string() => "42".to_string(), "Bar".to_string() => "17".to_string()}
                ),
                remove: Some(vec!["Baz".to_string(), "Qux".to_string()]),
                force: Some(true)
            }
        );

        let m2: Modification = serde_json::from_str("{}").unwrap();
        assert_eq!(
            m2,
            Modification {
                replace: None,
                remove: None,
                force: None
            }
        );
    }

    #[test]
    fn test_anonymization_deserialize() {
        let json = r#"
            {
                "replace": {
                    "Foo": "42",
                    "Bar": "17"
                },
                "keep": ["Baz", "Qux"],
                "keep_private_tags": true,
                "dicom_version": "42.17",
                "force": true
            }
        "#;
        let a1: Anonymization = serde_json::from_str(json).unwrap();
        assert_eq!(
            a1,
            Anonymization {
                replace: Some(
                    hashmap! {"Foo".to_string() => "42".to_string(), "Bar".to_string() => "17".to_string()}
                ),
                keep: Some(vec!["Baz".to_string(), "Qux".to_string()]),
                keep_private_tags: Some(true),
                dicom_version: Some("42.17".to_string()),
                force: Some(true)
            }
        );
        let a2: Anonymization = serde_json::from_str("{}").unwrap();
        assert_eq!(
            a2,
            Anonymization {
                replace: None,
                keep: None,
                keep_private_tags: None,
                dicom_version: None,
                force: None
            }
        );
    }

    #[test]
    fn test_entity_trait_patient() {
        assert_eq!(Patient::kind(), EntityKind::Patient);

        let patient = Patient {
            id: "f88cbd3f-a00dfc59-9ca1ac2d-7ce9851a-40e5b493".to_string(),
            is_stable: true,
            last_update: NaiveDate::from_ymd(2020, 1, 1).and_hms(15, 46, 17),
            main_dicom_tags: hashmap! {
                "PatientName".to_string() => "Rick Sanchez".to_string(),
            },
            studies: ["e8cafcbe-caf08c39-6e205f15-18554bb8-b3f9ef04".to_string()].to_vec(),
            entity: EntityKind::Patient,
            anonymized_from: None,
        };

        assert_eq!(patient.id(), "f88cbd3f-a00dfc59-9ca1ac2d-7ce9851a-40e5b493");
        assert_eq!(patient.parent_id(), None);
        assert_eq!(patient.main_dicom_tag("PatientName"), Some("Rick Sanchez"));
        assert_eq!(
            patient.children(),
            ["e8cafcbe-caf08c39-6e205f15-18554bb8-b3f9ef04".to_string()]
        );
        assert_eq!(patient.children_len(), 1);
        assert_eq!(patient.index(), None);
        assert_eq!(patient.size(), 0);
        assert_eq!(patient.parent_kind(), None);
        assert_eq!(patient.parent_kind_name(), None);
        assert_eq!(patient.children_kind_name(), Some("Studies"));
    }

    #[test]
    fn test_entity_trait_study() {
        assert_eq!(Study::kind(), EntityKind::Study);

        let study = Study {
            id: "63bf5d42-b5382159-01971752-e0ceea3d-399bbca5".to_string(),
            is_stable: true,
            last_update: NaiveDate::from_ymd(2020, 8, 30).and_hms(19, 11, 09),
            main_dicom_tags: hashmap! {
                "AccessionNumber".to_string() => "foobar".to_string(),
            },
            parent_patient: "7e43f8d3-e50280e6-470079e9-02241af1-d286bdbe".to_string(),
            patient_main_dicom_tags: hashmap! {
                "PatientName".to_string() => "Rick Sanchez".to_string(),
            },
            series: [
                "cd00fffc-db25be29-0c6da430-c56796a5-ba06933c".to_string(),
                "2ab7dbe7-f1a18a78-86145443-18a8ff93-0b65f2b2".to_string(),
            ]
            .to_vec(),
            entity: EntityKind::Study,
            anonymized_from: None,
        };

        assert_eq!(study.id(), "63bf5d42-b5382159-01971752-e0ceea3d-399bbca5");
        assert_eq!(
            study.parent_id(),
            Some("7e43f8d3-e50280e6-470079e9-02241af1-d286bdbe")
        );
        assert_eq!(study.main_dicom_tag("AccessionNumber"), Some("foobar"));
        assert_eq!(study.main_dicom_tag("PatientName"), Some("Rick Sanchez"));
        assert_eq!(
            study.children(),
            [
                "cd00fffc-db25be29-0c6da430-c56796a5-ba06933c".to_string(),
                "2ab7dbe7-f1a18a78-86145443-18a8ff93-0b65f2b2".to_string()
            ]
        );
        assert_eq!(study.children_len(), 2);
        assert_eq!(study.index(), None);
        assert_eq!(study.size(), 0);
        assert_eq!(study.parent_kind(), Some(EntityKind::Patient));
        assert_eq!(study.parent_kind_name(), Some("Patient".to_string()));
        assert_eq!(study.children_kind_name(), Some("Series"));
    }

    #[test]
    fn test_entity_trait_series() {
        assert_eq!(Series::kind(), EntityKind::Series);

        let series = Series {
            id: "cd00fffc-db25be29-0c6da430-c56796a5-ba06933c".to_string(),
            status: "Unknown".to_string(),
            is_stable: true,
            last_update: NaiveDate::from_ymd(2020, 8, 30).and_hms(19, 11, 09),
            main_dicom_tags: hashmap! {
                "BodyPartExamined".to_string() => "ABDOMEN".to_string(),
            },
            parent_study: "63bf5d42-b5382159-01971752-e0ceea3d-399bbca5".to_string(),
            expected_number_of_instances: Some(17),
            instances: [
                "556530b5-de7c487b-110b9d0e-12cfdbb9-f06b546e".to_string(),
                "c46605db-836489fa-cb55fbbc-13c8a913-b0bad6ac".to_string(),
            ]
            .to_vec(),
            entity: EntityKind::Series,
            anonymized_from: None,
        };

        assert_eq!(series.id(), "cd00fffc-db25be29-0c6da430-c56796a5-ba06933c");
        assert_eq!(
            series.parent_id(),
            Some("63bf5d42-b5382159-01971752-e0ceea3d-399bbca5")
        );
        assert_eq!(series.main_dicom_tag("BodyPartExamined"), Some("ABDOMEN"));
        assert_eq!(
            series.children(),
            [
                "556530b5-de7c487b-110b9d0e-12cfdbb9-f06b546e".to_string(),
                "c46605db-836489fa-cb55fbbc-13c8a913-b0bad6ac".to_string()
            ]
        );
        assert_eq!(series.children_len(), 2);
        assert_eq!(series.index(), None);
        assert_eq!(series.size(), 0);
        assert_eq!(series.parent_kind(), Some(EntityKind::Study));
        assert_eq!(series.parent_kind_name(), Some("Study".to_string()));
        assert_eq!(series.children_kind_name(), Some("Instances"));
    }

    #[test]
    fn test_entity_trait_instance() {
        assert_eq!(Instance::kind(), EntityKind::Instance);

        let instance = Instance {
            id: "29fa4d9d-51a69d1d-70e2b29a-fd824316-50850d0c".to_string(),
            main_dicom_tags: hashmap! {
                "SOPInstanceUID".to_string() => "1.2.3.4.5.6789".to_string(),
            },
            parent_series: "82081568-b6f8f4e6-ced76876-6504da25-ed0dfe03".to_string(),
            index_in_series: Some(13),
            file_uuid: "d8c5eff3-986c-4fe4-b06e-7e52b2a4238e".to_string(),
            file_size: 139402,
            modified_from: Some("22c54cb6-28302a69-3ff454a3-676b98f4-b84cd80a".to_string()),
            entity: EntityKind::Instance,
            anonymized_from: None,
        };

        assert_eq!(
            instance.id(),
            "29fa4d9d-51a69d1d-70e2b29a-fd824316-50850d0c"
        );
        assert_eq!(
            instance.parent_id(),
            Some("82081568-b6f8f4e6-ced76876-6504da25-ed0dfe03")
        );
        assert_eq!(
            instance.main_dicom_tag("SOPInstanceUID"),
            Some("1.2.3.4.5.6789")
        );
        let ch: &[String] = &[];
        assert_eq!(instance.children(), ch);
        assert_eq!(instance.children_len(), 0);
        assert_eq!(instance.index(), Some(13));
        assert_eq!(instance.size(), 139402);
        assert_eq!(instance.parent_kind(), Some(EntityKind::Series));
        assert_eq!(instance.parent_kind_name(), Some("Series".to_string()));
        assert_eq!(instance.children_kind_name(), None);
    }
}