use orthanc_client::*;
use serde_json;
use std::env;
use std::process::Command;

fn run_curl(url: &str) -> Vec<u8> {
    Command::new("curl").arg(url).output().unwrap().stdout
}

fn server_address() -> String {
    env::var("ORC_ORTHANC_ADDRESS").unwrap()
}

#[test]
fn test_list_patients() {
    let address = &server_address();

    let expected_patients_json = run_curl(&format!("{}/patients", &server_address()));
    let expected_patients: Vec<String> =
        serde_json::from_slice(&expected_patients_json).unwrap();

    let cl = OrthancClient::new(address, None, None);
    let patients = cl.list_patients().unwrap();

    assert_eq!(patients, expected_patients);
}

#[test]
fn test_list_patients_expanded() {
    let address = &server_address();

    let expected_patients_json =
        run_curl(&format!("{}/patients?expand", &server_address()));
    let expected_patients: Vec<Patient> =
        serde_json::from_slice(&expected_patients_json).unwrap();

    let cl = OrthancClient::new(address, None, None);
    let patients = cl.list_patients_expanded().unwrap();

    assert_eq!(patients, expected_patients);
}
