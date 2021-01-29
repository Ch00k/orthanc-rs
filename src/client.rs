use crate::entity::*;
use crate::error::Error;
use crate::models::*;
use crate::utils::check_http_error;
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

    fn get_stream<W: Write>(&self, path: &str, mut writer: W) -> Result<()> {
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
        resp.copy_to(&mut writer)?;
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
        mut writer: W,
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
        resp.copy_to(&mut writer)?;
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

    /// List peers
    pub fn peers(&self) -> Result<Vec<String>> {
        self.list("peers")
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

    /// List all peers in an expanded format
    pub fn peers_expanded(&self) -> Result<HashMap<String, Peer>> {
        let resp = self.get("peers?expand")?;
        let json: HashMap<String, Peer> = serde_json::from_slice(&resp)?;
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
    pub fn patient_dicom<W: Write>(&self, id: &str, writer: W) -> Result<()> {
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
    pub fn study_dicom<W: Write>(&self, id: &str, writer: W) -> Result<()> {
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
    pub fn series_dicom<W: Write>(&self, id: &str, writer: W) -> Result<()> {
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
    pub fn instance_dicom<W: Write>(&self, id: &str, writer: W) -> Result<()> {
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
        writer: W,
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
        writer: W,
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

    /// Send a C-STORE DICOM request to a remote modality
    ///
    /// `ids` is a slice of entity IDs to send. An ID can signify either of [`Patient`], [`Study`],
    /// [`Series`] or [`Instance`]
    pub fn store(
        &self,
        modality: &str,
        ids: &[&str],
    ) -> Result<StoreResult> {
        let resp = self.post(
            &format!("modalities/{}/store", modality),
            serde_json::json!(ids),
        )?;
        let json: StoreResult = serde_json::from_slice(&resp)?;
        Ok(json)
    }

    // TODO: The following two methods are exactly the same
    /// Create a peer
    pub fn create_peer(&self, name: &str, peer: Peer) -> Result<()> {
        self.put(&format!("peers/{}", name), serde_json::to_value(peer)?)
            .map(|_| ())
    }

    /// Modify a peer
    pub fn modify_peer(&self, name: &str, peer: Peer) -> Result<()> {
        self.put(&format!("peers/{}", name), serde_json::to_value(peer)?)
            .map(|_| ())
    }

    /// Delete a peer
    pub fn delete_peer(&self, name: &str) -> Result<()> {
        self.delete(&format!("peers/{}", name)).map(|_| ())
    }

    /// Send entities to a peer
    ///
    /// `ids` is a slice of entity IDs to send. An ID can signify either of [`Patient`], [`Study`],
    /// [`Series`] or [`Instance`]
    pub fn peer_store(&self, peer: &str, ids: &[&str]) -> Result<PeerStoreResult> {
        let resp = self.post(&format!("peers/{}/store", peer), serde_json::json!(ids))?;
        let json: PeerStoreResult = serde_json::from_slice(&resp)?;
        Ok(json)
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::ApiError;
    use httpmock::{Method, Mock, MockServer};
    use maplit::hashmap;

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
    fn test_get_stream_error_response_empty_body() {
        let mock_server = MockServer::start();
        let url = mock_server.url("");

        let m = Mock::new()
            .expect_method(Method::GET)
            .expect_path("/foo")
            .return_status(400)
            .create_on(&mock_server);

        let cl = Client::new(url);
        let mut writer: Vec<u8> = vec![];
        let resp = cl.get_stream("foo", &mut writer);

        assert_eq!(
            resp.unwrap_err(),
            Error {
                message: "API error: 400 Bad Request".to_string(),
                details: None,
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
    fn test_post_receive_stream_error_response_empty_body() {
        let mock_server = MockServer::start();
        let url = mock_server.url("");

        let m = Mock::new()
            .expect_method(Method::POST)
            .expect_path("/foo")
            .return_status(400)
            .create_on(&mock_server);

        let cl = Client::new(url);
        let mut writer: Vec<u8> = vec![];
        let resp = cl.post_receive_stream("foo", serde_json::json!("bar"), &mut writer);

        assert_eq!(
            resp.unwrap_err(),
            Error {
                message: "API error: 400 Bad Request".to_string(),
                details: None,
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
}
