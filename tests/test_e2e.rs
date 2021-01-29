use dicom_object::{open_file, Error as DicomError};
use maplit::hashmap;
use orthanc::entity::*;
use orthanc::error::ApiError;
use orthanc::models::*;
use orthanc::Client;
use orthanc::Error;
use regex::Regex;
use reqwest;
use serde_json;
use serde_json::{from_str, json, Value};
use std::env;
use std::fs;
use std::io::BufReader;
use zip;

const DEFAULT_SERVER_ADDRESS: &str = "http://localhost:8028";
const DEFAULT_USERNAME: &str = "orthanc";
const DEFAULT_PASSWORD: &str = "orthanc";

const DEFAULT_DINO_HOST: &str = "dino"; // docker-compose
const DEFAULT_DINO_PORT: &str = "5252";
const DEFAULT_DINO_AET: &str = "DINO";

const SOP_INSTANCE_UID: &str = "1.3.46.670589.11.1.5.0.3724.2011072815265975004";
const SOP_INSTANCE_UID_DELETE: &str = "1.3.46.670589.11.1.5.0.7080.2012100313435153441";
const SERIES_INSTANCE_UID: &str = "1.3.46.670589.11.1.5.0.3724.2011072815265926000";
const STUDY_INSTANCE_UID: &str = "1.3.46.670589.11.1.5.0.6560.2011072814060507000";
const PATIENT_ID: &str = "patient_2";

const UPLOAD_INSTANCE_FILE_PATH: &str = "upload";

const DEIDENTIFICATION_TAG_PATTERN: &str =
    r"Orthanc\s\d+.\d+.\d+\s-\sPS\s3.15-2017c\sTable\sE.1-1\sBasic\sProfile";

fn client() -> Client {
    Client::new(
        env::var("ORC_ORTHANC_ADDRESS").unwrap_or(DEFAULT_SERVER_ADDRESS.to_string()),
    )
    .auth(
        env::var("ORC_ORTHANC_USERNAME").unwrap_or(DEFAULT_USERNAME.to_string()),
        env::var("ORC_ORTHANC_PASSWORD").unwrap_or(DEFAULT_PASSWORD.to_string()),
    )
}

fn first_patient() -> String {
    client().patients().unwrap().remove(0)
}

fn first_study() -> String {
    client().studies().unwrap().remove(0)
}

fn first_series() -> String {
    client().series_list().unwrap().remove(0)
}

fn first_instance() -> String {
    client().instances().unwrap().remove(0)
}

fn find_instance_by_sop_instance_uid(sop_instance_uid: &str) -> Option<Instance> {
    let instances = client().instances_expanded().unwrap();
    for i in instances {
        if i.main_dicom_tags["SOPInstanceUID"] == sop_instance_uid {
            return Some(i);
        }
    }
    return None;
}

fn find_series_by_series_instance_uid(series_instance_uid: &str) -> Option<Series> {
    let series = client().series_expanded().unwrap();
    for s in series {
        if s.main_dicom_tags["SeriesInstanceUID"] == series_instance_uid {
            return Some(s);
        }
    }
    return None;
}

fn find_study_by_study_instance_uid(study_instance_uid: &str) -> Option<Study> {
    let studies = client().studies_expanded().unwrap();
    for s in studies {
        if s.main_dicom_tags["StudyInstanceUID"] == study_instance_uid {
            return Some(s);
        }
    }
    return None;
}

fn find_patient_by_patient_id(patient_id: &str) -> Option<Patient> {
    let patients = client().patients_expanded().unwrap();
    for p in patients {
        if p.main_dicom_tags["PatientID"] == patient_id {
            return Some(p);
        }
    }
    return None;
}

fn get(url: &str) -> String {
    let client = reqwest::blocking::ClientBuilder::new().build().unwrap();
    client
        .get(url)
        .basic_auth(
            env::var("ORC_ORTHANC_USERNAME").unwrap_or(DEFAULT_USERNAME.to_string()),
            Some(env::var("ORC_ORTHANC_PASSWORD").unwrap_or(DEFAULT_PASSWORD.to_string())),
        )
        .send()
        .unwrap()
        .text()
        .unwrap()
}

// TODO: Figure out how to not use `trim` everywhere (element_by_name apppends trailing whitespace)
fn assert_tag_has_value(path: &str, tag_id: &str, value: &str) {
    let obj = open_file(path).unwrap();
    assert_eq!(
        obj.element_by_name(tag_id)
            .unwrap()
            .to_str()
            .unwrap()
            .trim(),
        value
    );
}

fn assert_tag_value_contains(path: &str, tag_id: &str, substring: &str) {
    let obj = open_file(path).unwrap();
    assert!(obj
        .element_by_name(tag_id)
        .unwrap()
        .to_str()
        .unwrap()
        .trim()
        .contains(substring));
}

fn assert_tag_value_matches(path: &str, tag_id: &str, pattern: &str) {
    let re = Regex::new(pattern).unwrap();
    let obj = open_file(path).unwrap();
    let tag_value = obj.element_by_name(tag_id).unwrap().to_str().unwrap();
    assert!(re.is_match(&tag_value.trim()));
}

fn assert_tag_is_empty(path: &str, tag_id: &str) {
    let obj = open_file(path).unwrap();
    assert_eq!(
        obj.element_by_name(tag_id)
            .unwrap()
            .to_str()
            .unwrap()
            .trim(),
        ""
    );
}

fn assert_tag_is_absent(path: &str, tag_id: &str) {
    let obj = open_file(path).unwrap();
    let res = obj.element_by_name(tag_id).unwrap_err();
    assert!(matches!(res, DicomError::NoSuchDataElementAlias{..}));
}

fn expected_response(path: &str) -> Value {
    from_str(&get(&format!(
        "{}/{}",
        env::var("ORC_ORTHANC_ADDRESS").unwrap_or(DEFAULT_SERVER_ADDRESS.to_string()),
        path
    )))
    .unwrap()
}

#[test]
fn test_no_auth() {
    let client = Client::new(
        env::var("ORC_ORTHANC_ADDRESS").unwrap_or(DEFAULT_SERVER_ADDRESS.to_string()),
    );
    let resp = client.modalities();
    assert_eq!(
        resp.unwrap_err(),
        Error {
            message: "API error: 401 Unauthorized".to_string(),
            details: None
        }
    );
}

#[test]
fn test_wrong_auth() {
    let client = Client::new(
        env::var("ORC_ORTHANC_ADDRESS").unwrap_or(DEFAULT_SERVER_ADDRESS.to_string()),
    )
    .auth("foo", "bar");
    let resp = client.modalities();
    assert_eq!(
        resp.unwrap_err(),
        Error {
            message: "API error: 401 Unauthorized".to_string(),
            details: None
        }
    );
}

#[test]
fn test_get_system_info() {
    assert_eq!(
        json!(client().system().unwrap()),
        expected_response("system")
    );
}

#[test]
fn test_list_patients() {
    assert_eq!(
        json!(client().patients().unwrap()),
        expected_response("patients")
    );
}

#[test]
fn test_list_patients_expanded() {
    assert_eq!(
        json!(client().patients_expanded().unwrap()),
        expected_response("patients?expand")
    );
}

#[test]
fn test_list_studies() {
    assert_eq!(
        json!(client().studies().unwrap()),
        expected_response("studies")
    );
}

#[test]
fn test_list_studies_expanded() {
    assert_eq!(
        json!(client().studies_expanded().unwrap()),
        expected_response("studies?expand")
    );
}

#[test]
fn test_list_series() {
    assert_eq!(
        json!(client().series_list().unwrap()),
        expected_response("series")
    );
}

#[test]
fn test_list_series_expanded() {
    assert_eq!(
        json!(client().series_expanded().unwrap()),
        expected_response("series?expand")
    );
}

#[test]
fn test_list_instances() {
    assert_eq!(
        json!(client().instances().unwrap()),
        expected_response("instances")
    );
}

#[test]
fn test_list_instances_expanded() {
    assert_eq!(
        json!(client().instances_expanded().unwrap()),
        expected_response("instances?expand")
    );
}

#[test]
fn test_get_patient() {
    let patient = first_patient();
    assert_eq!(
        json!(client().patient(&patient).unwrap()),
        expected_response(&format!("patients/{}", patient))
    );
}

#[test]
fn test_get_study() {
    let study = first_study();
    assert_eq!(
        json!(client().study(&study).unwrap()),
        expected_response(&format!("studies/{}", study))
    );
}

#[test]
fn test_get_series() {
    let series = first_series();
    assert_eq!(
        json!(client().series(&series).unwrap()),
        expected_response(&format!("series/{}", series))
    );
}

#[test]
fn test_get_instance() {
    let instance = first_instance();
    assert_eq!(
        json!(client().instance(&instance).unwrap()),
        expected_response(&format!("instances/{}", instance))
    );
}

#[test]
fn test_get_instance_tags() {
    let instance = first_instance();
    assert_eq!(
        json!(client().instance_tags(&instance).unwrap()),
        expected_response(&format!("instances/{}/simplified-tags", instance))
    );
}

#[test]
fn test_get_instance_tags_expanded() {
    let instance = first_instance();
    assert_eq!(
        json!(client().instance_tags_expanded(&instance).unwrap()),
        expected_response(&format!("instances/{}/tags", instance))
    );
}

#[test]
fn test_instance_content() {
    let instance = first_instance();
    assert_eq!(
        json!(client().instance_content(&instance).unwrap()),
        expected_response(&format!("instances/{}/content", instance))
    );
}

#[test]
fn test_instance_tag() {
    let instance = first_instance();
    assert_eq!(
        client().instance_tag(&instance, "0020-0013").unwrap(),
        client().instance(&instance).unwrap().main_dicom_tags["InstanceNumber"]
    );
}

#[test]
fn test_get_patient_dicom() {
    let patient = find_patient_by_patient_id(PATIENT_ID).unwrap();
    let mut file = fs::File::create("/tmp/patient.zip").unwrap();
    client().patient_dicom(&patient.id, &mut file).unwrap();

    let file = fs::File::open("/tmp/patient.zip").unwrap();
    let reader = BufReader::new(file);
    let zip = zip::ZipArchive::new(reader).unwrap();
    let mut files: Vec<&str> = zip.file_names().collect();
    files.sort();

    assert_eq!(
        files,
        vec![
            "patient_2 Patient 2/REMOVED Study 1/MR Series 1/MR000000.dcm",
            "patient_2 Patient 2/REMOVED Study 1/PR/PR000000.dcm",
        ]
    );
}

#[test]
fn test_get_study_dicom() {
    let study = find_study_by_study_instance_uid(STUDY_INSTANCE_UID).unwrap();
    let mut file = fs::File::create("/tmp/study.zip").unwrap();
    client().study_dicom(&study.id, &mut file).unwrap();

    let file = fs::File::open("/tmp/study.zip").unwrap();
    let reader = BufReader::new(file);
    let zip = zip::ZipArchive::new(reader).unwrap();
    let mut files: Vec<&str> = zip.file_names().collect();
    files.sort();

    assert_eq!(
        files,
        vec![
            "patient_2 Patient 2/REMOVED Study 1/MR Series 1/MR000000.dcm",
            "patient_2 Patient 2/REMOVED Study 1/PR/PR000000.dcm",
        ]
    );
}

#[test]
fn test_get_series_dicom() {
    let series = find_series_by_series_instance_uid(SERIES_INSTANCE_UID).unwrap();
    let mut file = fs::File::create("/tmp/series.zip").unwrap();
    client().series_dicom(&series.id, &mut file).unwrap();

    let file = fs::File::open("/tmp/series.zip").unwrap();
    let reader = BufReader::new(file);
    let zip = zip::ZipArchive::new(reader).unwrap();
    let mut files: Vec<&str> = zip.file_names().collect();
    files.sort();

    assert_eq!(
        files,
        vec!["patient_2 Patient 2/REMOVED Study 1/MR Series 1/MR000000.dcm",]
    );
}

#[test]
fn test_get_intance_dicom() {
    let instance = find_instance_by_sop_instance_uid(SOP_INSTANCE_UID).unwrap();
    let mut file = fs::File::create("/tmp/instance_dicom").unwrap();
    client().instance_dicom(&instance.id, &mut file).unwrap();
    // TODO: dicom_object element_by_name returns the value with some trailing characters
    assert_tag_value_contains("/tmp/instance_dicom", "SOPInstanceUID", SOP_INSTANCE_UID);
}

#[test]
fn test_delete() {
    let instance = find_instance_by_sop_instance_uid(SOP_INSTANCE_UID_DELETE).unwrap();
    let series = client().series(&instance.parent_series).unwrap();
    let study = client().study(&series.parent_study).unwrap();
    let patient = client().patient(&study.parent_patient).unwrap();

    // delete instance
    let resp = client().delete_instance(&instance.id).unwrap();
    assert_eq!(
        resp,
        RemainingAncestor {
            remaining_ancestor: Some(Ancestor {
                id: instance.parent_series,
                path: format!("/series/{}", series.id),
                entity: EntityKind::Series,
            })
        }
    );
    let resp = client().instance(&instance.id);
    assert_eq!(
        resp.unwrap_err(),
        Error {
            message: "API error: 404 Not Found".to_string(),
            details: None,
        },
    );

    // delete series
    let resp = client().delete_series(&series.id).unwrap();
    assert_eq!(
        resp,
        RemainingAncestor {
            remaining_ancestor: Some(Ancestor {
                id: series.parent_study,
                path: format!("/studies/{}", study.id),
                entity: EntityKind::Study,
            })
        }
    );
    let resp = client().series(&series.id);
    assert_eq!(
        resp.unwrap_err(),
        Error {
            message: "API error: 404 Not Found".to_string(),
            details: None,
        },
    );

    // delete study
    let resp = client().delete_study(&study.id).unwrap();
    assert_eq!(
        resp,
        RemainingAncestor {
            remaining_ancestor: Some(Ancestor {
                id: study.parent_patient,
                path: format!("/patients/{}", patient.id),
                entity: EntityKind::Patient,
            })
        }
    );
    let resp = client().study(&study.id);
    assert_eq!(
        resp.unwrap_err(),
        Error {
            message: "API error: 404 Not Found".to_string(),
            details: None,
        },
    );

    // delete patient
    let resp = client().delete_patient(&patient.id).unwrap();
    assert_eq!(
        resp,
        RemainingAncestor {
            remaining_ancestor: None
        }
    );
    let resp = client().patient(&patient.id);
    assert_eq!(
        resp.unwrap_err(),
        Error {
            message: "API error: 404 Not Found".to_string(),
            details: None,
        },
    );
}

#[test]
fn test_modify_instance() {
    let instance = find_instance_by_sop_instance_uid(SOP_INSTANCE_UID).unwrap();

    let replace = hashmap! {
        "SpecificCharacterSet".to_string() => "ISO_IR 13".to_string(),
        "OperatorsName".to_string() => "Summer Smith".to_string()
    };
    let remove = vec!["SeriesTime".to_string(), "AcquisitionTime".to_string()];
    let modification = Modification {
        replace: Some(replace),
        remove: Some(remove),
        force: None,
    };
    let path = "/tmp/modified_instance";
    let mut file = fs::File::create(path).unwrap();
    client()
        .modify_instance(&instance.id, modification, &mut file)
        .unwrap();

    assert_tag_has_value(path, "SpecificCharacterSet", "ISO_IR 13");
    assert_tag_has_value(path, "OperatorsName", "Summer Smith");
    assert_tag_is_absent(path, "SeriesTime");
    assert_tag_is_absent(path, "AcquisitionTime");
}

#[test]
fn test_modify_series() {
    let series = find_series_by_series_instance_uid(SERIES_INSTANCE_UID).unwrap();
    let tags = series.main_dicom_tags;
    assert_ne!(tags["BodyPartExamined"], "PINKY");
    assert_ne!(tags["OperatorsName"], "Summer Smith");
    assert!(tags.contains_key("StationName"));
    assert!(tags.contains_key("SeriesDate"));

    let replace = hashmap! {
        "BodyPartExamined".to_string() => "PINKY".to_string(),
        "OperatorsName".to_string() => "Summer Smith".to_string()
    };
    let remove = vec!["StationName".to_string(), "SeriesDate".to_string()];
    let modification = Modification {
        replace: Some(replace),
        remove: Some(remove),
        force: None,
    };
    let resp = client().modify_series(&series.id, modification).unwrap();
    let modified_series = client().series(&resp.id).unwrap();
    let modified_tags = modified_series.main_dicom_tags;

    assert_eq!(modified_tags["BodyPartExamined"], "PINKY");
    assert_eq!(modified_tags["OperatorsName"], "Summer Smith");
    assert!(!modified_tags.contains_key("StationName"));
    assert!(!modified_tags.contains_key("SeriesDate"));
}

#[test]
fn test_modify_study() {
    let study = find_study_by_study_instance_uid(STUDY_INSTANCE_UID).unwrap();
    let tags = study.main_dicom_tags;
    assert_ne!(tags["StudyID"], "foobar");
    assert_ne!(tags["ReferringPhysicianName"], "Summer Smith");
    assert!(tags.contains_key("InstitutionName"));
    assert!(tags.contains_key("StudyTime"));

    let replace = hashmap! {
        "StudyID".to_string() => "foobar".to_string(),
        "ReferringPhysicianName".to_string() => "Summer Smith".to_string()
    };
    let remove = vec!["InstitutionName".to_string(), "StudyTime".to_string()];
    let modification = Modification {
        replace: Some(replace),
        remove: Some(remove),
        force: None,
    };
    let resp = client().modify_study(&study.id, modification).unwrap();
    let modified_study = client().study(&resp.id).unwrap();
    let modified_tags = modified_study.main_dicom_tags;

    assert_eq!(modified_tags["StudyID"], "foobar");
    assert_eq!(modified_tags["ReferringPhysicianName"], "Summer Smith");
    assert!(!modified_tags.contains_key("InstitutionName"));
    assert!(!modified_tags.contains_key("StudyTime"));
}

#[test]
fn test_modify_patient() {
    let patient = find_patient_by_patient_id(PATIENT_ID).unwrap();
    let tags = patient.main_dicom_tags;
    assert_eq!(tags["PatientID"], "patient_2");
    assert_eq!(tags["PatientName"], "Patient 2");
    assert_eq!(tags["PatientBirthDate"], "19790101");
    assert!(tags.contains_key("PatientSex"));

    let replace = hashmap! {
        "PatientID".to_string() => "gazorpazorp".to_string(),
        "PatientName".to_string() => "Summer Smith".to_string(),
        "PatientBirthDate".to_string() => "20330303".to_string(),
    };
    let remove = vec!["PatientSex".to_string()];
    let modification = Modification {
        replace: Some(replace),
        remove: Some(remove),
        force: Some(true),
    };
    let resp = client().modify_patient(&patient.id, modification).unwrap();
    let modified_patient = client().patient(&resp.id).unwrap();
    let modified_tags = modified_patient.main_dicom_tags;

    assert_eq!(modified_tags["PatientID"], "gazorpazorp");
    assert_eq!(modified_tags["PatientName"], "Summer Smith");
    assert_eq!(modified_tags["PatientBirthDate"], "20330303");
    assert!(!modified_tags.contains_key("PatientSex"));
}

#[test]
fn test_modify_patient_keep_patient_id() {
    // This is a feature (or a bug?) ot Orthanc
    // If the PatientID stays the same, the patient is _not_ modified,
    // even if `replace` and `remove` request modifications.

    let patient = find_patient_by_patient_id(PATIENT_ID).unwrap();
    let tags = patient.main_dicom_tags;
    assert_eq!(tags["PatientName"], "Patient 2");
    assert_eq!(tags["PatientBirthDate"], "19790101");
    assert!(tags.contains_key("PatientSex"));

    let replace = hashmap! {
        "PatientID".to_string() => PATIENT_ID.to_string(),
        "PatientName".to_string() => "Summer Smith".to_string(),
        "PatientBirthDate".to_string() => "20330303".to_string(),
    };
    let remove = vec!["PatientSex".to_string()];
    let modification = Modification {
        replace: Some(replace),
        remove: Some(remove),
        force: Some(true),
    };
    let resp = client().modify_patient(&patient.id, modification).unwrap();
    let modified_patient = client().patient(&resp.id).unwrap();
    let modified_tags = modified_patient.main_dicom_tags;

    assert_eq!(modified_tags["PatientID"], PATIENT_ID);
    assert_eq!(modified_tags["PatientName"], "Patient 2");
    assert_eq!(modified_tags["PatientBirthDate"], "19790101");
    assert!(modified_tags.contains_key("PatientSex"));
}

#[test]
fn test_modify_without_patient_id() {
    let patient = find_patient_by_patient_id(PATIENT_ID).unwrap();
    let replace = hashmap! {
        "PatientSex".to_string() => "F".to_string(),
    };
    let modification = Modification {
        replace: Some(replace),
        remove: None,
        force: None,
    };
    let resp = client().modify_patient(&patient.id, modification);

    assert_eq!(
        resp.unwrap_err(),
        Error {
            message: "API error: 400 Bad Request".to_string(),
            details: Some(ApiError {
                method: "POST".to_string(),
                uri: format!("/patients/{}/modify", &patient.id).to_string(),
                message: "Bad request".to_string(),
                details: Some(
                    "When modifying a patient, her PatientID is required to be modified"
                        .to_string()
                ),
                http_status: 400,
                http_error: "Bad Request".to_string(),
                orthanc_status: 8,
                orthanc_error: "Bad request".to_string(),
            },),
        },
    );
}

#[test]
fn test_modify_patient_id_without_force() {
    let patient = find_patient_by_patient_id(PATIENT_ID).unwrap();
    let replace = hashmap! {
        "PatientID".to_string() => "C137".to_string(),
    };
    let modification = Modification {
        replace: Some(replace),
        remove: None,
        force: None,
    };
    let resp = client().modify_patient(&patient.id, modification);

    assert_eq!(
        resp.unwrap_err(),
        Error {
            message: "API error: 400 Bad Request".to_string(),
            details: Some(ApiError {
                method: "POST".to_string(),
                uri: format!("/patients/{}/modify", &patient.id).to_string(),
                message: "Bad request".to_string(),
                details: Some(
                    "Marking tag \"PatientID\" as to be replaced requires the \"Force\" option to be set to true".to_string()
                ),
                http_status: 400,
                http_error: "Bad Request".to_string(),
                orthanc_status: 8,
                orthanc_error: "Bad request".to_string(),
            },),
        },
    );
}

#[test]
fn test_anonymize_instance() {
    let instance = find_instance_by_sop_instance_uid(SOP_INSTANCE_UID).unwrap();

    let replace = hashmap! {
        "SpecificCharacterSet".to_string() => "ISO_IR 13".to_string(),
        "OperatorsName".to_string() => "Summer Smith".to_string()
    };
    let keep = vec![
        "AccessionNumber".to_string(),
        "StudyDescription".to_string(),
    ];
    let anonymization = Anonymization {
        replace: Some(replace),
        keep: Some(keep),
        keep_private_tags: None,
        dicom_version: None,
        force: None,
    };
    let path = "/tmp/anonymized_instance";
    let mut file = fs::File::create(path).unwrap();
    client()
        .anonymize_instance(&instance.id, Some(anonymization), &mut file)
        .unwrap();

    assert_tag_has_value(path, "SpecificCharacterSet", "ISO_IR 13");
    assert_tag_has_value(path, "OperatorsName", "Summer Smith");
    assert_tag_has_value(path, "AccessionNumber", "REMOVED");
    assert_tag_has_value(path, "StudyDescription", "Study 1");
    assert_tag_value_contains(path, "PatientName", "Anonymized");

    // When anonymization is customized, Orthanc does not add the 0012,0063 tag. A bug?
    //assert_tag_value_matches(path, "0012,0063", DEIDENTIFICATION_TAG_PATTERN);
}

#[test]
fn test_anonymize_instance_empty_body() {
    let instance = find_instance_by_sop_instance_uid(SOP_INSTANCE_UID).unwrap();
    let path = "/tmp/anonymized_instance";
    let mut file = fs::File::create(path).unwrap();
    client()
        .anonymize_instance(&instance.id, None, &mut file)
        .unwrap();

    assert_tag_is_empty(path, "AccessionNumber");
    assert_tag_is_absent(path, "StudyDescription");
    assert_tag_value_contains(path, "PatientName", "Anonymized");
    assert_tag_value_matches(path, "DeidentificationMethod", DEIDENTIFICATION_TAG_PATTERN);
}

#[test]
fn test_anonymize_series() {
    let series = find_series_by_series_instance_uid(SERIES_INSTANCE_UID).unwrap();
    let initial_tags = client().instance_tags(&series.instances[0]).unwrap();

    assert_ne!(initial_tags["SpecificCharacterSet"], "ISO_IR 13");
    assert_ne!(initial_tags["OperatorsName"], "Summer Smith");

    let replace = hashmap! {
        "SpecificCharacterSet".to_string() => "ISO_IR 13".to_string(),
        "OperatorsName".to_string() => "Summer Smith".to_string()
    };
    let keep = vec![
        "AccessionNumber".to_string(),
        "SeriesDescription".to_string(),
    ];
    let anonymization = Anonymization {
        replace: Some(replace),
        keep: Some(keep),
        keep_private_tags: None,
        dicom_version: None,
        force: None,
    };
    let resp = client()
        .anonymize_series(&series.id, Some(anonymization))
        .unwrap();

    let modified_series = client().series(&resp.id).unwrap();

    let tags = client()
        .instance_tags(&modified_series.instances[0])
        .unwrap();

    assert_eq!(tags["SpecificCharacterSet"], "ISO_IR 13");
    assert_eq!(tags["OperatorsName"], "Summer Smith");
    assert_eq!(tags["AccessionNumber"], initial_tags["AccessionNumber"]);
    assert_eq!(tags["SeriesDescription"], initial_tags["SeriesDescription"]);
}

#[test]
fn test_anonymize_series_empty_body() {
    let series = find_series_by_series_instance_uid(SERIES_INSTANCE_UID).unwrap();
    let initial_tags = client().instance_tags(&series.instances[0]).unwrap();

    assert_ne!(initial_tags["AccessionNumber"], "");
    assert_ne!(initial_tags["StudyID"], "");

    let resp = client().anonymize_series(&series.id, None).unwrap();

    let modified_series = client().series(&resp.id).unwrap();

    let tags = client()
        .instance_tags(&modified_series.instances[0])
        .unwrap();

    assert_eq!(tags["AccessionNumber"], "");
    assert_eq!(tags["StudyID"], "");
}

#[test]
fn test_anonymize_study() {
    let study = find_study_by_study_instance_uid(STUDY_INSTANCE_UID).unwrap();
    let initial_series = client().series(&study.series[0]).unwrap();
    let initial_tags = client()
        .instance_tags(&initial_series.instances[0])
        .unwrap();

    assert_ne!(initial_tags["SpecificCharacterSet"], "ISO_IR 13");
    assert_ne!(initial_tags["OperatorsName"], "Summer Smith");

    let replace = hashmap! {
        "SpecificCharacterSet".to_string() => "ISO_IR 13".to_string(),
        "OperatorsName".to_string() => "Summer Smith".to_string()
    };
    let keep = vec![
        "AccessionNumber".to_string(),
        "StudyDescription".to_string(),
    ];
    let anonymization = Anonymization {
        replace: Some(replace),
        keep: Some(keep),
        keep_private_tags: None,
        dicom_version: None,
        force: None,
    };
    let resp = client()
        .anonymize_study(&study.id, Some(anonymization))
        .unwrap();

    let modified_study = client().study(&resp.id).unwrap();
    let modified_series = client().series(&modified_study.series[0]).unwrap();
    let tags = client()
        .instance_tags(&modified_series.instances[0])
        .unwrap();

    assert_eq!(tags["SpecificCharacterSet"], "ISO_IR 13");
    assert_eq!(tags["OperatorsName"], "Summer Smith");
    assert_eq!(tags["AccessionNumber"], initial_tags["AccessionNumber"]);
    assert_eq!(tags["StudyDescription"], initial_tags["StudyDescription"]);
}

#[test]
fn test_anonymize_study_empty_body() {
    let study = find_study_by_study_instance_uid(STUDY_INSTANCE_UID).unwrap();
    let initial_series = client().series(&study.series[0]).unwrap();
    let initial_tags = client()
        .instance_tags(&initial_series.instances[0])
        .unwrap();

    assert_ne!(initial_tags["AccessionNumber"], "");
    assert_ne!(initial_tags["StudyID"], "");

    let resp = client().anonymize_study(&study.id, None).unwrap();

    let modified_study = client().study(&resp.id).unwrap();
    let modified_series = client().series(&modified_study.series[0]).unwrap();
    let tags = client()
        .instance_tags(&modified_series.instances[0])
        .unwrap();

    assert_eq!(tags["AccessionNumber"], "");
    assert_eq!(tags["StudyID"], "");
}

#[test]
fn test_anonymize_patient() {
    let patient = find_patient_by_patient_id(PATIENT_ID).unwrap();
    let initial_study = client().study(&patient.studies[0]).unwrap();
    let initial_series = client().series(&initial_study.series[0]).unwrap();
    let initial_tags = client()
        .instance_tags(&initial_series.instances[0])
        .unwrap();

    assert_ne!(initial_tags["SpecificCharacterSet"], "ISO_IR 13");
    assert_ne!(initial_tags["OperatorsName"], "Summer Smith");

    let replace = hashmap! {
        "SpecificCharacterSet".to_string() => "ISO_IR 13".to_string(),
        "OperatorsName".to_string() => "Summer Smith".to_string(),
    };
    let keep = vec![
        "AccessionNumber".to_string(),
        "StudyDescription".to_string(),
    ];
    let anonymization = Anonymization {
        replace: Some(replace),
        keep: Some(keep),
        keep_private_tags: None,
        dicom_version: None,
        force: None,
    };
    let resp = client()
        .anonymize_patient(&patient.id, Some(anonymization))
        .unwrap();

    let modified_patient = client().patient(&resp.id).unwrap();
    let modified_study = client().study(&modified_patient.studies[0]).unwrap();
    let modified_series = client().series(&modified_study.series[0]).unwrap();
    let tags = client()
        .instance_tags(&modified_series.instances[0])
        .unwrap();

    assert_eq!(tags["SpecificCharacterSet"], "ISO_IR 13");
    assert_eq!(tags["OperatorsName"], "Summer Smith");
    assert_eq!(tags["AccessionNumber"], initial_tags["AccessionNumber"]);
    assert_eq!(tags["StudyDescription"], initial_tags["StudyDescription"]);
}

#[test]
fn test_anonymize_patient_empty_body() {
    let patient = find_patient_by_patient_id(PATIENT_ID).unwrap();
    let initial_study = client().study(&patient.studies[0]).unwrap();
    let initial_series = client().series(&initial_study.series[0]).unwrap();
    let initial_tags = client()
        .instance_tags(&initial_series.instances[0])
        .unwrap();

    assert_ne!(initial_tags["AccessionNumber"], "");
    assert_ne!(initial_tags["StudyID"], "");

    let resp = client().anonymize_patient(&patient.id, None).unwrap();

    let modified_patient = client().patient(&resp.id).unwrap();
    let modified_study = client().study(&modified_patient.studies[0]).unwrap();
    let modified_series = client().series(&modified_study.series[0]).unwrap();
    let tags = client()
        .instance_tags(&modified_series.instances[0])
        .unwrap();

    assert_eq!(tags["AccessionNumber"], "");
    assert_eq!(tags["StudyID"], "");
}

#[test]
fn test_anonymize_without_force() {
    let patient = find_patient_by_patient_id(PATIENT_ID).unwrap();
    let replace = hashmap! {
        "PatientID".to_string() => "C137".to_string(),
    };
    let anonymization = Anonymization {
        replace: Some(replace),
        keep: None,
        keep_private_tags: None,
        dicom_version: None,
        force: None,
    };
    let resp = client().anonymize_patient(&patient.id, Some(anonymization));

    assert_eq!(
        resp.unwrap_err(),
        Error {
            message: "API error: 400 Bad Request".to_string(),
            details: Some(ApiError {
                method: "POST".to_string(),
                uri: format!("/patients/{}/anonymize", &patient.id).to_string(),
                message: "Bad request".to_string(),
                details: Some(
                    "Marking tag \"PatientID\" as to be replaced requires the \"Force\" option to be set to true".to_string()
                ),
                http_status: 400,
                http_error: "Bad Request".to_string(),
                orthanc_status: 8,
                orthanc_error: "Bad request".to_string(),
            },),
        },
    );
}

#[test]
fn test_anonymize_with_force() {
    let patient = find_patient_by_patient_id(PATIENT_ID).unwrap();
    let replace = hashmap! {
        "PatientID".to_string() => "C137".to_string(),
    };
    let anonymization = Anonymization {
        replace: Some(replace),
        keep: None,
        keep_private_tags: None,
        dicom_version: None,
        force: Some(true),
    };
    let resp = client()
        .anonymize_patient(&patient.id, Some(anonymization))
        .unwrap();

    let modified_patient = client().patient(&resp.id).unwrap();
    let modified_study = client().study(&modified_patient.studies[0]).unwrap();
    let modified_series = client().series(&modified_study.series[0]).unwrap();
    let tags = client()
        .instance_tags(&modified_series.instances[0])
        .unwrap();

    assert_eq!(tags["PatientID"], "C137");
}

#[test]
fn test_upload_dicom() {
    let data = fs::read(format!(
        "{}/{}",
        env::var("ORC_DATAFILES_PATH").unwrap_or("./data/dicom".to_string()),
        UPLOAD_INSTANCE_FILE_PATH
    ))
    .unwrap();

    let resp = client().upload(&data).unwrap();
    assert_eq!(resp.status, "Success");

    let resp = client().upload(&data).unwrap();
    assert_eq!(resp.status, "AlreadyStored");
}

// These just test the method access
#[test]
fn test_get_dicom_tag_value_patient() {
    assert_eq!(
        client()
            .patient(&first_patient())
            .unwrap()
            .main_dicom_tag("FooBar"),
        None
    );
}

#[test]
fn test_get_dicom_tag_value_study() {
    assert_eq!(
        client()
            .study(&first_study())
            .unwrap()
            .main_dicom_tag("FooBar"),
        None
    );
}

#[test]
fn test_get_dicom_tag_value_series() {
    assert_eq!(
        client()
            .series(&first_series())
            .unwrap()
            .main_dicom_tag("FooBar"),
        None
    );
}

#[test]
fn test_get_dicom_tag_value_instance() {
    assert_eq!(
        client()
            .instance(&first_instance())
            .unwrap()
            .main_dicom_tag("FooBar"),
        None
    );
}

#[test]
fn test_modalities() {
    // Get system info
    let sysinfo = client().system().unwrap();
    let mut allow_transcoding = None;
    if sysinfo.api_version > 6 {
        allow_transcoding = Some(true);
    }

    // Create
    let modality_1 = Modality {
        aet: "foobar".to_string(),
        host: "1.2.3.4".to_string(),
        port: 4217,
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
    assert_eq!(client().create_modality("bazqux", modality_1).unwrap(), ());
    let mut created: bool = false;
    for (m_name, m_config) in client().modalities_expanded().unwrap() {
        if m_name == "bazqux" {
            assert_eq!(
                m_config,
                Modality {
                    aet: "foobar".to_string(),
                    host: "1.2.3.4".to_string(),
                    port: 4217,
                    manufacturer: Some("Generic".to_string()),
                    allow_c_echo: Some(true),
                    allow_c_find: Some(true),
                    allow_c_get: Some(true),
                    allow_c_move: Some(true),
                    allow_c_store: Some(true),
                    allow_n_action: Some(true),
                    allow_n_event_report: Some(true),
                    allow_transcoding: allow_transcoding,
                }
            );
            created = true;
        }
    }
    if !created {
        panic!("Modality not created");
    };

    // Create another one for listing
    let modality_2 = Modality {
        aet: "garble".to_string(),
        host: "9.8.7.6".to_string(),
        port: 1742,
        manufacturer: Some("GE".to_string()),
        allow_c_echo: Some(false),
        allow_c_find: Some(false),
        allow_c_get: Some(false),
        allow_c_move: Some(false),
        allow_c_store: Some(false),
        allow_n_action: Some(false),
        allow_n_event_report: Some(false),
        allow_transcoding,
    };
    assert_eq!(client().create_modality("garble", modality_2).unwrap(), ());

    // List
    assert_eq!(
        json!(client().modalities().unwrap()),
        expected_response("modalities")
    );

    // List expanded
    assert_eq!(
        json!(client().modalities_expanded().unwrap()),
        expected_response("modalities?expand")
    );
    println!("{:#?}", client().modalities_expanded().unwrap());

    // Modify
    let modality = Modality {
        aet: "quuxquuz".to_string(),
        host: "4.3.2.1".to_string(),
        port: 4217,
        manufacturer: Some("GE".to_string()),
        allow_c_echo: Some(false),
        allow_c_find: None,
        allow_c_get: None,
        allow_c_move: Some(false),
        allow_c_store: None,
        allow_n_action: Some(true),
        allow_n_event_report: None,
        allow_transcoding: None,
    };
    assert_eq!(client().modify_modality("bazqux", modality).unwrap(), ());
    let mut modified: bool = false;
    for (m_name, m_config) in client().modalities_expanded().unwrap() {
        if m_name == "bazqux" {
            assert_eq!(
                m_config,
                Modality {
                    aet: "quuxquuz".to_string(),
                    host: "4.3.2.1".to_string(),
                    port: 4217,
                    manufacturer: Some("GE".to_string()),
                    allow_c_echo: Some(false),
                    allow_c_find: Some(true),
                    allow_c_get: Some(true),
                    allow_c_move: Some(false),
                    allow_c_store: Some(true),
                    allow_n_action: Some(true),
                    allow_n_event_report: Some(true),
                    allow_transcoding: allow_transcoding,
                }
            );
            modified = true;
        }
    }
    if !modified {
        panic!("Modality not modified");
    }

    // Delete
    assert_eq!(client().delete_modality("bazqux").unwrap(), ());
    let modalities = client().modalities_expanded().unwrap();
    assert!(!modalities.contains_key("bazqux"));
}

#[test]
fn _test_peers() {
    // Create
    let peer_1 = Peer {
        url: "http://orthanc_peer:8029".to_string(),
        username: Some("orthanc".to_string()),
        password: Some("orthanc".to_string()),
        http_headers: Some(
            hashmap! {"Foo".to_string() => "foo".to_string(), "Bar".to_string() => "bar".to_string()},
        ),
        certificate_file: None,
        certificate_key_file: None,
        certificate_key_password: None,
    };

    assert_eq!(client().create_peer("foobar", peer_1).unwrap(), ());
    let mut created: bool = false;
    for (p_name, p_config) in client().peers_expanded().unwrap() {
        if p_name == "foobar" {
            assert_eq!(
                p_config,
                Peer {
                    url: "http://orthanc_peer:8029/".to_string(),
                    username: Some("orthanc".to_string()),
                    password: None, // empty for security reasons
                    http_headers: None,
                    certificate_file: None,
                    certificate_key_file: None,
                    certificate_key_password: None,
                }
            );
            created = true;
        }
    }
    if !created {
        panic!("Peer not created");
    };

    // Create another one for listing
    let peer_2 = Peer {
        url: "http://orthanc_peer:8092".to_string(),
        username: None,
        password: None,
        http_headers: None,
        certificate_file: None,
        certificate_key_file: None,
        certificate_key_password: None,
    };
    assert_eq!(client().create_peer("garble", peer_2).unwrap(), ());

    // List
    assert_eq!(json!(client().peers().unwrap()), expected_response("peers"));

    // List expanded
    // TODO: Expanded list JSON omits all the `null` fields, while our deserialization does not.
    // Is there a simpler way to do the assertion?
    let list = client().peers_expanded().unwrap();
    assert_eq!(list.len(), 2);
    assert_eq!(
        list.get("foobar").unwrap(),
        &Peer {
            url: "http://orthanc_peer:8029/".to_string(),
            username: Some("orthanc".to_string()),
            password: None, // empty for security reasons
            http_headers: None,
            certificate_file: None,
            certificate_key_file: None,
            certificate_key_password: None,
        },
    );
    assert_eq!(
        list.get("garble").unwrap(),
        &Peer {
            url: "http://orthanc_peer:8092/".to_string(),
            username: None,
            password: None,
            http_headers: None,
            certificate_file: None,
            certificate_key_file: None,
            certificate_key_password: None,
        }
    );

    // Modify
    let peer = Peer {
        url: "http://random_peer:1234".to_string(),
        username: Some("foo".to_string()),
        password: Some("bar".to_string()),
        http_headers: None,
        certificate_file: None,
        certificate_key_file: None,
        certificate_key_password: None,
    };

    assert_eq!(client().modify_peer("foobar", peer).unwrap(), ());
    let mut modified: bool = false;
    for (p_name, p_config) in client().peers_expanded().unwrap() {
        if p_name == "foobar" {
            assert_eq!(
                p_config,
                Peer {
                    url: "http://random_peer:1234/".to_string(),
                    username: Some("foo".to_string()),
                    password: None, // empty for security reasons
                    http_headers: None,
                    certificate_file: None,
                    certificate_key_file: None,
                    certificate_key_password: None,
                }
            );
            modified = true;
        }
    }
    if !modified {
        panic!("Peer not modified");
    }

    // Delete
    assert_eq!(client().delete_peer("foobar").unwrap(), ());
    let peers = client().peers_expanded().unwrap();
    assert!(!peers.contains_key("foobar"));
}

#[test]
fn test_modality_echo() {
    let modality = Modality {
        aet: env::var("DINO_SCP_AET").unwrap_or(DEFAULT_DINO_AET.to_string()),
        host: DEFAULT_DINO_HOST.to_string(),
        port: env::var("DINO_SCP_PORT")
            .unwrap_or(DEFAULT_DINO_PORT.to_string())
            .parse::<i32>()
            .unwrap(),
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
    client().create_modality("dino", modality).unwrap();

    assert_eq!(client().echo("dino", None).unwrap(), ());
}

#[test]
fn test_modality_store() {
    let modality = Modality {
        aet: env::var("DINO_SCP_AET").unwrap_or(DEFAULT_DINO_AET.to_string()),
        host: DEFAULT_DINO_HOST.to_string(),
        port: env::var("DINO_SCP_PORT")
            .unwrap_or(DEFAULT_DINO_PORT.to_string())
            .parse::<i32>()
            .unwrap(),
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
    client().create_modality("dino", modality).unwrap();

    assert_eq!(
        client().store("dino", &[&first_study()]).unwrap(),
        StoreResult {
            description: "REST API".to_string(),
            local_aet: "ORTHANC".to_string(),
            remote_aet: "DINO".to_string(),
            parent_resources: vec!(first_study()),
            instances_count: 2,
            failed_instances_count: 0,
        }
    );
}

#[test]
fn test_peer_store() {
    let peer = Peer {
        url: "http://orthanc_peer:8042".to_string(),
        username: Some("orthanc".to_string()),
        password: Some("orthanc".to_string()),
        http_headers: None,
        certificate_file: None,
        certificate_key_file: None,
        certificate_key_password: None,
    };
    client().create_peer("orthanc_peer", peer).unwrap();

    let peer_client = Client::new("http://localhost:8029").auth("orthanc", "orthanc");
    assert_eq!(peer_client.studies().unwrap().len(), 0);

    client()
        .peer_store("orthanc_peer", &[&first_study()])
        .unwrap();

    assert_eq!(peer_client.studies().unwrap().len(), 1);
}

#[test]
fn test_search_patient_level() {
    let res: Vec<Patient> = client()
        .search(hashmap! {"PatientID".to_string() => PATIENT_ID.to_string()})
        .unwrap();
    assert_eq!(res.len(), 1);
    assert_eq!(res[0].main_dicom_tag("PatientID").unwrap(), PATIENT_ID);
}

#[test]
fn test_search_study_level() {
    let res: Vec<Study> = client()
        .search(hashmap! {"StudyInstanceUID".to_string() => STUDY_INSTANCE_UID.to_string()})
        .unwrap();
    assert_eq!(res.len(), 1);
    assert_eq!(
        res[0].main_dicom_tag("StudyInstanceUID").unwrap(),
        STUDY_INSTANCE_UID
    );
}

#[test]
fn test_search_series_level() {
    let res: Vec<Series> = client()
        .search(
            hashmap! {"SeriesInstanceUID".to_string() => SERIES_INSTANCE_UID.to_string()},
        )
        .unwrap();
    assert_eq!(res.len(), 1);
    assert_eq!(
        res[0].main_dicom_tag("SeriesInstanceUID").unwrap(),
        SERIES_INSTANCE_UID
    );
}

#[test]
fn test_search_instance_level() {
    let res: Vec<Instance> = client()
        .search(hashmap! {"SOPInstanceUID".to_string() => SOP_INSTANCE_UID.to_string()})
        .unwrap();
    assert_eq!(res.len(), 1);
    assert_eq!(
        res[0].main_dicom_tag("SOPInstanceUID").unwrap(),
        SOP_INSTANCE_UID
    );
}

#[test]
fn _test_search_instances_in_patient_level() {
    let res: Vec<Instance> = client()
        .search(hashmap! {"PatientID".to_string() => PATIENT_ID.to_string()})
        .unwrap();
    assert_eq!(res.len(), 2);
}
