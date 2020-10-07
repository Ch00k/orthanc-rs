use orthanc_client::*;
use serde_json;
use serde_json::{from_slice, json, Value};
use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use std::process::Command;

fn address() -> String {
    env::var("ORC_ORTHANC_ADDRESS").unwrap()
}

fn username() -> String {
    env::var("ORC_ORTHANC_USERNAME").unwrap()
}

fn password() -> String {
    env::var("ORC_ORTHANC_PASSWORD").unwrap()
}

fn datafiles_path() -> String {
    env::var("ORC_DATAFILES_PATH").unwrap()
}

fn client() -> OrthancClient {
    OrthancClient::new(&address(), Some(&username()), Some(&password()))
}

fn first_patient() -> String {
    client().list_patients().unwrap().remove(0)
}

fn first_study() -> String {
    client().list_studies().unwrap().remove(0)
}

fn first_series() -> String {
    client().list_series().unwrap().remove(0)
}

fn first_instance() -> String {
    client().list_instances().unwrap().remove(0)
}

fn get_instance_by_sop_instance_uid(sop_instance_uid: &str) -> Option<Instance> {
    let instances = client().list_instances_expanded().unwrap();
    for i in instances {
        if i.main_dicom_tags["SOPInstanceUID"] == sop_instance_uid {
            return Some(i);
        }
    }
    return None;
}

fn run_curl(url: &str) -> Vec<u8> {
    Command::new("curl")
        .arg("--user")
        .arg(format!("{}:{}", username(), password()))
        .arg(url)
        .output()
        .unwrap()
        .stdout
}

fn expected_response(path: &str) -> Value {
    from_slice(&run_curl(&format!("{}/{}", address(), path))).unwrap()
}

#[test]
fn test_list_modalities() {
    assert_eq!(
        json!(client().list_modalities().unwrap()),
        expected_response("modalities")
    );
}

#[test]
fn test_list_modalities_expanded() {
    assert_eq!(
        json!(client().list_modalities_expanded().unwrap()),
        expected_response("modalities?expand")
    );
}

#[test]
fn test_list_patients() {
    assert_eq!(
        json!(client().list_patients().unwrap()),
        expected_response("patients")
    );
}

#[test]
fn test_list_patients_expanded() {
    assert_eq!(
        json!(client().list_patients_expanded().unwrap()),
        expected_response("patients?expand")
    );
}

#[test]
fn test_list_studies() {
    assert_eq!(
        json!(client().list_studies().unwrap()),
        expected_response("studies")
    );
}

#[test]
fn test_list_studies_expanded() {
    assert_eq!(
        json!(client().list_studies_expanded().unwrap()),
        expected_response("studies?expand")
    );
}

#[test]
fn test_list_series() {
    assert_eq!(
        json!(client().list_series().unwrap()),
        expected_response("series")
    );
}

#[test]
fn test_list_series_expanded() {
    assert_eq!(
        json!(client().list_series_expanded().unwrap()),
        expected_response("series?expand")
    );
}

#[test]
fn test_list_instances() {
    assert_eq!(
        json!(client().list_instances().unwrap()),
        expected_response("instances")
    );
}

#[test]
fn test_list_instances_expanded() {
    assert_eq!(
        json!(client().list_instances_expanded().unwrap()),
        expected_response("instances?expand")
    );
}

#[test]
fn test_get_patient() {
    let patient = first_patient();
    assert_eq!(
        json!(client().get_patient(&patient).unwrap()),
        expected_response(&format!("patients/{}", patient))
    );
}

#[test]
fn test_get_study() {
    let study = first_study();
    assert_eq!(
        json!(client().get_study(&study).unwrap()),
        expected_response(&format!("studies/{}", study))
    );
}

#[test]
fn test_get_series() {
    let series = first_series();
    assert_eq!(
        json!(client().get_series(&series).unwrap()),
        expected_response(&format!("series/{}", series))
    );
}

#[test]
fn test_get_instance() {
    let instance = first_instance();
    assert_eq!(
        json!(client().get_instance(&instance).unwrap()),
        expected_response(&format!("instances/{}", instance))
    );
}

#[test]
fn test_delete() {
    let instance =
        get_instance_by_sop_instance_uid("1.3.46.670589.11.1.5.0.7080.2012100313435153441")
            .unwrap();
    let series = client().get_series(&instance.parent_series).unwrap();
    let study = client().get_study(&series.parent_study).unwrap();
    let patient = client().get_patient(&study.parent_patient).unwrap();

    // delete instance
    let resp = client().delete_instance(&instance.id).unwrap();
    assert_eq!(
        resp,
        RemainingAncestorResponse {
            remaining_ancestor: Some(RemainingAncestor {
                id: instance.parent_series,
                path: format!("/series/{}", series.id),
                entity_type: EntityType::Series,
            })
        }
    );
    let resp = client().get_instance(&instance.id);
    assert_eq!(
        resp.unwrap_err(),
        OrthancError {
            details: "404".to_string(),
            error_response: None,
        },
    );

    // delete series
    let resp = client().delete_series(&series.id).unwrap();
    assert_eq!(
        resp,
        RemainingAncestorResponse {
            remaining_ancestor: Some(RemainingAncestor {
                id: series.parent_study,
                path: format!("/studies/{}", study.id),
                entity_type: EntityType::Study,
            })
        }
    );
    let resp = client().get_series(&series.id);
    assert_eq!(
        resp.unwrap_err(),
        OrthancError {
            details: "404".to_string(),
            error_response: None,
        },
    );

    // delete study
    let resp = client().delete_study(&study.id).unwrap();
    assert_eq!(
        resp,
        RemainingAncestorResponse {
            remaining_ancestor: Some(RemainingAncestor {
                id: study.parent_patient,
                path: format!("/patients/{}", patient.id),
                entity_type: EntityType::Patient,
            })
        }
    );
    let resp = client().get_study(&study.id);
    assert_eq!(
        resp.unwrap_err(),
        OrthancError {
            details: "404".to_string(),
            error_response: None,
        },
    );

    // delete patient
    let resp = client().delete_patient(&patient.id).unwrap();
    assert_eq!(
        resp,
        RemainingAncestorResponse {
            remaining_ancestor: None
        }
    );
    let resp = client().get_patient(&patient.id);
    assert_eq!(
        resp.unwrap_err(),
        OrthancError {
            details: "404".to_string(),
            error_response: None,
        },
    );
}

#[test]
fn test_upload_dicom() {
    let mut f = File::open(Path::new(&datafiles_path()).join("upload")).unwrap();
    let mut data = Vec::new();
    f.read_to_end(&mut data).unwrap();

    let resp = client().upload_dicom(&data).unwrap();
    assert_eq!(resp.status, "Success");

    let resp = client().upload_dicom(&data).unwrap();
    assert_eq!(resp.status, "AlreadyStored");
}
