use chrono::NaiveDate;
use httpmock::Method::GET;
use httpmock::{Mock, MockServer};
use maplit::hashmap;
use orthanc_client::{OrthancClient, Patient};

#[test]
fn test_list_patients() {
    let mock_server = MockServer::start();
    let url = format!("http://{}:{}", &mock_server.host(), &mock_server.port());

    let m = Mock::new()
        .expect_method(GET)
        .expect_path("/patients")
        .return_status(200)
        .return_header("Content-Type", "application/json")
        .return_body(r#"["foo", "bar", "baz"]"#)
        .create_on(&mock_server);

    let cl = OrthancClient::new(Some(&url), None, None);
    let patient_ids = cl.list_patients().unwrap();

    assert_eq!(patient_ids, ["foo", "bar", "baz"]);
    assert_eq!(m.times_called(), 1);
}

#[test]
fn test_list_patients_expanded() {
    let mock_server = MockServer::start();
    let url = format!("http://{}:{}", &mock_server.host(), &mock_server.port());

    let m = Mock::new()
        .expect_method(GET)
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

    let cl = OrthancClient::new(Some(&url), None, None);
    let patients = cl.list_patients_expanded().unwrap();
    println!("{:#?}", patients);

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
