use maplit::hashmap;
use orthanc_client::*;
use regex::Regex;
use serde_json;
use serde_json::{from_slice, json, Value};
use std::env;
use std::fs;
use std::fs::File;
use std::io::prelude::*;
use std::process::Command;

const SOP_INSTANCE_UID: &str = "1.3.46.670589.11.1.5.0.3724.2011072815265975004";
const SOP_INSTANCE_UID_DELETE: &str = "1.3.46.670589.11.1.5.0.7080.2012100313435153441";
//const SERIES_INSTANCE_UID: &str =
//    "1.3.46.670589.11.436918600.1211176830.3842984857.54226705";
//const STUDY_INSTANCE_UID: &str = "1.3.46.670589.11.1.5.0.7116.2012100313043060185";
//const PATIENT_ID: &str = "patient_2";

const UPLOAD_INSTANCE_FILE_PATH: &str = "upload";

const DCMDUMP_LINE_PATTERN: &str = r"\s*\(\d{4},\d{4}\)\s+[A-Z]{2}\s+\[(.*)\]\s+.*";

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

fn dcmdump_line_pattern() -> Regex {
    Regex::new(DCMDUMP_LINE_PATTERN).unwrap()
}

fn run_dcmdump(path: &str, tag_id: &str) -> String {
    let output = Command::new("dcmdump")
        .arg("--search")
        .arg(tag_id)
        .arg(path)
        .output()
        .unwrap()
        .stdout;
    String::from_utf8(output).unwrap()
}

fn assert_tag_value(path: &str, tag_id: &str, value: &str) {
    let dcmdump_out = run_dcmdump(path, tag_id);
    let groups = dcmdump_line_pattern().captures(&dcmdump_out).unwrap();
    assert_eq!(groups.get(1).unwrap().as_str(), value);
}

fn assert_tag_absent(path: &str, tag_id: &str) {
    let dcmdump_out = run_dcmdump(path, tag_id);
    assert_eq!(dcmdump_out, "");
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
    let instance = get_instance_by_sop_instance_uid(SOP_INSTANCE_UID_DELETE).unwrap();
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
fn test_modify_instance() {
    let instance = get_instance_by_sop_instance_uid(SOP_INSTANCE_UID).unwrap();

    let replace = hashmap! {
        "SpecificCharacterSet".to_string() => "ISO_IR 13".to_string(),
        "OperatorsName".to_string() => "Summer Smith".to_string()
    };
    let remove = vec!["SeriesTime".to_string(), "AcquisitionTime".to_string()];
    let resp = client()
        .modify_instance(&instance.id, Some(replace), Some(remove))
        .unwrap();

    let path = "/tmp/modified_instance";
    let mut file = File::create("/tmp/modified_instance").unwrap();
    file.write_all(&resp).unwrap();

    assert_tag_value(path, "0008,0005", "ISO_IR 13");
    assert_tag_value(path, "0008,1070", "Summer Smith");
    assert_tag_absent(path, "0008,0031");
    assert_tag_absent(path, "0008,0032");
}

#[test]
fn test_upload_dicom() {
    let data = fs::read(format!(
        "{}/{}",
        datafiles_path(),
        UPLOAD_INSTANCE_FILE_PATH
    ))
    .unwrap();

    let resp = client().upload_dicom(&data).unwrap();
    assert_eq!(resp.status, "Success");

    let resp = client().upload_dicom(&data).unwrap();
    assert_eq!(resp.status, "AlreadyStored");
}
