use maplit::hashmap;
use orthanc::*;
use regex::Regex;
use serde_json;
use serde_json::{from_slice, json, Value};
use std::env;
use std::fs;
use std::fs::File;
use std::io::prelude::*;
use std::process::Command;
use zip;

const SOP_INSTANCE_UID: &str = "1.3.46.670589.11.1.5.0.3724.2011072815265975004";
const SOP_INSTANCE_UID_DELETE: &str = "1.3.46.670589.11.1.5.0.7080.2012100313435153441";
const SERIES_INSTANCE_UID: &str = "1.3.46.670589.11.1.5.0.3724.2011072815265926000";
const STUDY_INSTANCE_UID: &str = "1.3.46.670589.11.1.5.0.6560.2011072814060507000";
const PATIENT_ID: &str = "patient_2";

const UPLOAD_INSTANCE_FILE_PATH: &str = "upload";

const DCMDUMP_LINE_PATTERN: &str = r"\s*\(\d{4},\d{4}\)\s+[A-Z]{2}\s+([\[\(].*[\]\)])\s+.*";
const DEIDENTIFICATION_TAG_PATTERN: &str =
    r"\[Orthanc\s\d+.\d+.\d+\s-\sPS\s3.15-2017c\sTable\sE.1-1\sBasic\sProfile\]";

fn client() -> Client {
    Client::new(env::var("ORC_ORTHANC_ADDRESS").unwrap()).auth(
        env::var("ORC_ORTHANC_USERNAME").unwrap(),
        env::var("ORC_ORTHANC_PASSWORD").unwrap(),
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

fn run_curl(url: &str) -> Vec<u8> {
    Command::new("curl")
        .arg("--user")
        .arg(format!(
            "{}:{}",
            env::var("ORC_ORTHANC_USERNAME").unwrap(),
            env::var("ORC_ORTHANC_PASSWORD").unwrap()
        ))
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

fn assert_tag_has_value(path: &str, tag_id: &str, value: &str) {
    let dcmdump_out = run_dcmdump(path, tag_id);
    let groups = dcmdump_line_pattern().captures(&dcmdump_out).unwrap();
    assert_eq!(groups.get(1).unwrap().as_str(), format!("[{}]", value));
}

fn assert_tag_value_contains(path: &str, tag_id: &str, substring: &str) {
    let dcmdump_out = run_dcmdump(path, tag_id);
    let groups = dcmdump_line_pattern().captures(&dcmdump_out).unwrap();
    assert!(groups
        .get(1)
        .unwrap()
        .as_str()
        .to_string()
        .contains(substring));
}

fn assert_tag_value_matches(path: &str, tag_id: &str, pattern: &str) {
    let re = Regex::new(pattern).unwrap();
    let dcmdump_out = run_dcmdump(path, tag_id);
    let groups = dcmdump_line_pattern().captures(&dcmdump_out).unwrap();
    let tag_value = groups.get(1).unwrap().as_str();
    assert!(re.is_match(tag_value));
}

fn assert_tag_is_empty(path: &str, tag_id: &str) {
    let dcmdump_out = run_dcmdump(path, tag_id);
    let groups = dcmdump_line_pattern().captures(&dcmdump_out).unwrap();
    assert_eq!(groups.get(1).unwrap().as_str(), "(no value available)");
}

fn assert_tag_is_absent(path: &str, tag_id: &str) {
    let dcmdump_out = run_dcmdump(path, tag_id);
    assert_eq!(dcmdump_out, "");
}

fn expected_response(path: &str) -> Value {
    from_slice(&run_curl(&format!(
        "{}/{}",
        env::var("ORC_ORTHANC_ADDRESS").unwrap(),
        path
    )))
    .unwrap()
}

#[test]
fn test_list_modalities() {
    assert_eq!(
        json!(client().modalities().unwrap()),
        expected_response("modalities")
    );
}

#[test]
fn test_list_modalities_expanded() {
    assert_eq!(
        json!(client().modalities_expanded().unwrap()),
        expected_response("modalities?expand")
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
fn test_get_patient_dicom() {
    let patient = find_patient_by_patient_id(PATIENT_ID).unwrap();
    let resp = client().patient_dicom(&patient.id).unwrap();

    let reader = std::io::Cursor::new(resp);
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
    let resp = client().study_dicom(&study.id).unwrap();

    let reader = std::io::Cursor::new(resp);
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
    let resp = client().series_dicom(&series.id).unwrap();

    let reader = std::io::Cursor::new(resp);
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
    let resp = client().instance_dicom(&instance.id).unwrap();

    let path = "/tmp/instance_dicom";
    let mut file = File::create(path).unwrap();
    file.write_all(&resp).unwrap();
    assert_tag_has_value(path, "0008,0018", SOP_INSTANCE_UID);
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
                entity: Entity::Series,
            })
        }
    );
    let resp = client().instance(&instance.id);
    assert_eq!(
        resp.unwrap_err(),
        Error {
            details: "404".to_string(),
            api_error: None,
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
                entity: Entity::Study,
            })
        }
    );
    let resp = client().series(&series.id);
    assert_eq!(
        resp.unwrap_err(),
        Error {
            details: "404".to_string(),
            api_error: None,
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
                entity: Entity::Patient,
            })
        }
    );
    let resp = client().study(&study.id);
    assert_eq!(
        resp.unwrap_err(),
        Error {
            details: "404".to_string(),
            api_error: None,
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
            details: "404".to_string(),
            api_error: None,
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
    let resp = client()
        .modify_instance(&instance.id, modification)
        .unwrap();

    let path = "/tmp/modified_instance";
    let mut file = File::create(path).unwrap();
    file.write_all(&resp).unwrap();

    assert_tag_has_value(path, "0008,0005", "ISO_IR 13");
    assert_tag_has_value(path, "0008,1070", "Summer Smith");
    assert_tag_is_absent(path, "0008,0031");
    assert_tag_is_absent(path, "0008,0032");
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
    assert_ne!(tags["PatientID"], "42");
    assert_ne!(tags["PatientName"], "Summer Smith");
    assert!(tags.contains_key("PatientSex"));

    let replace = hashmap! {
        "PatientID".to_string() => "42".to_string(),
        "PatientName".to_string() => "Summer Smith".to_string()
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

    assert_eq!(modified_tags["PatientID"], "42");
    assert_eq!(modified_tags["PatientName"], "Summer Smith");
    assert!(!modified_tags.contains_key("PatientSex"));
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
    };
    let resp = client()
        .anonymize_instance(&instance.id, Some(anonymization))
        .unwrap();

    let path = "/tmp/anonymized_instance";
    let mut file = File::create(path).unwrap();
    file.write_all(&resp).unwrap();

    assert_tag_has_value(path, "0008,0005", "ISO_IR 13");
    assert_tag_has_value(path, "0008,1070", "Summer Smith");
    assert_tag_has_value(path, "0008,0050", "REMOVED");
    assert_tag_has_value(path, "0008,1030", "Study 1");
    assert_tag_value_contains(path, "0010,0010", "Anonymized");

    // When anonymization is customized, Orthanc does not add the 0012,0063 tag. A bug?
    //assert_tag_value_matches(path, "0012,0063", DEIDENTIFICATION_TAG_PATTERN);
}

#[test]
fn test_anonymize_instance_empty_body() {
    let instance = find_instance_by_sop_instance_uid(SOP_INSTANCE_UID).unwrap();
    let resp = client().anonymize_instance(&instance.id, None).unwrap();

    let path = "/tmp/anonymized_instance";
    let mut file = File::create(path).unwrap();
    file.write_all(&resp).unwrap();

    assert_tag_is_empty(path, "0008,0050");
    assert_tag_is_absent(path, "0008,1030");
    assert_tag_value_contains(path, "0010,0010", "Anonymized");
    assert_tag_value_matches(path, "0012,0063", DEIDENTIFICATION_TAG_PATTERN);
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
fn test_upload_dicom() {
    let data = fs::read(format!(
        "{}/{}",
        env::var("ORC_DATAFILES_PATH").unwrap(),
        UPLOAD_INSTANCE_FILE_PATH
    ))
    .unwrap();

    let resp = client().upload(&data).unwrap();
    assert_eq!(resp.status, "Success");

    let resp = client().upload(&data).unwrap();
    assert_eq!(resp.status, "AlreadyStored");
}
