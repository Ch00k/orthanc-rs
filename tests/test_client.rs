use chrono::NaiveDate;
use httpmock::{Method, Mock, MockServer};
use maplit::hashmap;
use orthanc::entity::*;
use orthanc::models::*;
use orthanc::{ApiError, Client, Error};
use serde_json::Value;

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
fn test_list_peers() {
    let mock_server = MockServer::start();
    let url = mock_server.url("");

    let m = Mock::new()
        .expect_method(Method::GET)
        .expect_path("/peers")
        .return_status(200)
        .return_header("Content-Type", "application/json")
        .return_body(r#"["foo", "bar", "baz"]"#)
        .create_on(&mock_server);

    let cl = Client::new(url);
    let peers = cl.peers().unwrap();

    assert_eq!(peers, ["foo", "bar", "baz"]);
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
fn test_list_peers_expanded() {
    let mock_server = MockServer::start();
    let url = mock_server.url("");

    let m = Mock::new()
        .expect_method(Method::GET)
        .expect_path("/peers")
        .expect_query_param_exists("expand")
        .return_status(200)
        .return_header("Content-Type", "application/json")
        .return_body(
            r#"
                {
                    "foobar": {
                        "HttpHeaders": [
                            "Bar",
                            "Foo"
                        ],
                        "Password": null,
                        "Pkcs11": false,
                        "Url": "http://orthanc_peer:8029/",
                        "Username": "orthanc"
                    },
                    "garble": {
                        "HttpHeaders": [],
                        "Pkcs11": false,
                        "Url": "http://orthanc_peer:8092/",
                        "CertificateFile": "foo",
                        "CertificateKeyFile": "bar",
                        "CertificateKeyPassword": null
                    }
                }
            "#,
        )
        .create_on(&mock_server);

    let cl = Client::new(url);
    let peers = cl.peers_expanded().unwrap();

    assert_eq!(
        peers,
        hashmap! {
            "foobar".to_string() => Peer {
                url: "http://orthanc_peer:8029/".to_string(),
                username: Some("orthanc".to_string()),
                password: None, // empty for security reasons
                http_headers: None,
                certificate_file: None,
                certificate_key_file: None,
                certificate_key_password: None,
            },
            "garble".to_string() => Peer {
                url: "http://orthanc_peer:8092/".to_string(),
                username: None,
                password: None,
                http_headers: None,
                certificate_file: Some("foo".to_string()),
                certificate_key_file: Some("bar".to_string()),
                certificate_key_password: None,
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
                parent_patient: "f88cbd3f-a00dfc59-9ca1ac2d-7ce9851a-40e5b493".to_string(),
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
                parent_study: "63bf5d42-b5382159-01971752-e0ceea3d-399bbca5".to_string(),
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
                parent_series: "82081568-b6f8f4e6-ced76876-6504da25-ed0dfe03".to_string(),
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
                parent_series: "a240e0d7-538699a0-7464bb4b-a906f72a-fa3a32c7".to_string(),
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
            studies: ["e8cafcbe-caf08c39-6e205f15-18554bb8-b3f9ef04".to_string()].to_vec(),
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
            modified_from: Some("22c54cb6-28302a69-3ff454a3-676b98f4-b84cd80a".to_string()),
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
fn test_modality_store() {
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
    assert_eq!(
        cl.modality_store("them", &["bar", "baz", "qux"]).unwrap(),
        ModalityStoreResult {
            description: "REST API".to_string(),
            local_aet: "US".to_string(),
            remote_aet: "THEM".to_string(),
            parent_resources: vec!["bar".to_string(), "baz".to_string(), "qux".to_string()],
            instances_count: 42,
            failed_instances_count: 17
        }
    );

    assert_eq!(
        cl.store("them", &["bar", "baz", "qux"]).unwrap(),
        StoreResult {
            description: "REST API".to_string(),
            local_aet: "US".to_string(),
            remote_aet: "THEM".to_string(),
            parent_resources: vec!["bar".to_string(), "baz".to_string(), "qux".to_string()],
            instances_count: 42,
            failed_instances_count: 17
        }
    );
    assert_eq!(m.times_called(), 2);
}

#[test]
fn test_peer_store() {
    let mock_server = MockServer::start();
    let url = mock_server.url("");

    let m = Mock::new()
        .expect_method(Method::POST)
        .expect_path("/peers/foobar/store")
        //.expect_body(r#"["bar", "baz", "qux"]"#)
        .return_status(200)
        .return_header("Content-Type", "application/json")
        .return_body(
            r#"
                    {
                       "Description" : "REST API",
                       "FailedInstancesCount" : 17,
                       "InstancesCount" : 42,
                       "ParentResources" : [ "bar", "baz", "qux" ],
                       "Peer": [ "foobar" ]
                    }
                "#,
        )
        .create_on(&mock_server);

    let cl = Client::new(url);
    let resp = cl.peer_store("foobar", &["bar", "baz", "qux"]).unwrap();

    assert_eq!(
        resp,
        PeerStoreResult {
            description: "REST API".to_string(),
            peer: vec!["foobar".to_string()],
            parent_resources: vec!["bar".to_string(), "baz".to_string(), "qux".to_string()],
            instances_count: 42,
            failed_instances_count: 17
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
fn test_modality_echo() {
    let mock_server = MockServer::start();
    let url = mock_server.url("");

    let m = Mock::new()
        .expect_method(Method::POST)
        .expect_path("/modalities/foo/echo")
        .return_status(200)
        .return_body("{}")
        .create_on(&mock_server);

    let cl = Client::new(url);
    assert_eq!(cl.modality_echo("foo", None).unwrap(), ());
    assert_eq!(cl.echo("foo", None).unwrap(), ());
    assert_eq!(m.times_called(), 2);
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
    assert_eq!(cl.modality_echo("foo", Some(42)).unwrap(), ());
    assert_eq!(cl.echo("foo", Some(42)).unwrap(), ());
    assert_eq!(m.times_called(), 2);
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
    assert_eq!(
        cl.modality_echo("foo", None).unwrap_err(),
        Error {
            message: "API error: 500 Internal Server Error".to_string(),
            details: None
        }
    );
    assert_eq!(
        cl.echo("foo", None).unwrap_err(),
        Error {
            message: "API error: 500 Internal Server Error".to_string(),
            details: None
        }
    );
    assert_eq!(m.times_called(), 2);
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

// The following 2 tests are exactly the same except one calls `create_peer`,
// the other one calls `modify_peer`.
#[test]
fn test_create_peer() {
    let mock_server = MockServer::start();
    let url = mock_server.url("");

    let m = Mock::new()
        .expect_method(Method::PUT)
        .expect_path("/peers/bazqux")
        .expect_json_body(
            &Peer {
                url: "http://orthanc_peer:8029".to_string(),
                username: Some("orthanc".to_string()),
                password: Some("orthanc".to_string()),
                http_headers: Some(
                    hashmap! {"Foo".to_string() => "foo".to_string(), "Bar".to_string() => "bar".to_string()},
                ),
                certificate_file: None,
                certificate_key_file: None,
                certificate_key_password: None,
        })
        .return_status(200)
        .return_body("")
        .create_on(&mock_server);

    let cl = Client::new(url);
    let peer = Peer {
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
    let resp = cl.create_peer("bazqux", peer).unwrap();

    assert_eq!(resp, ());
    assert_eq!(m.times_called(), 1);
}

#[test]
fn test_modify_peer() {
    let mock_server = MockServer::start();
    let url = mock_server.url("");

    let m = Mock::new()
        .expect_method(Method::PUT)
        .expect_path("/peers/bazqux")
        .expect_json_body(
            &Peer {
                url: "http://orthanc_peer:8029".to_string(),
                username: Some("orthanc".to_string()),
                password: Some("orthanc".to_string()),
                http_headers: Some(
                    hashmap! {"Foo".to_string() => "foo".to_string(), "Bar".to_string() => "bar".to_string()},
                ),
                certificate_file: None,
                certificate_key_file: None,
                certificate_key_password: None,
        })
        .return_status(200)
        .return_body("")
        .create_on(&mock_server);

    let cl = Client::new(url);
    let peer = Peer {
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
    let resp = cl.modify_peer("bazqux", peer).unwrap();

    assert_eq!(resp, ());
    assert_eq!(m.times_called(), 1);
}

#[test]
fn test_delete_peer() {
    let mock_server = MockServer::start();
    let url = mock_server.url("");

    let m = Mock::new()
        .expect_method(Method::DELETE)
        .expect_path("/peers/bazqux")
        .return_status(200)
        .return_body("")
        .create_on(&mock_server);

    let cl = Client::new(url);
    let resp = cl.delete_peer("bazqux").unwrap();

    assert_eq!(resp, ());
    assert_eq!(m.times_called(), 1);
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
                parent_patient: "f88cbd3f-a00dfc59-9ca1ac2d-7ce9851a-40e5b493".to_string(),
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
                parent_study: "63bf5d42-b5382159-01971752-e0ceea3d-399bbca5".to_string(),
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
                parent_series: "82081568-b6f8f4e6-ced76876-6504da25-ed0dfe03".to_string(),
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
                parent_series: "a240e0d7-538699a0-7464bb4b-a906f72a-fa3a32c7".to_string(),
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
    let res: std::result::Result<Vec<Patient>, Error> =
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
fn test_modality_move() {
    let mock_server = MockServer::start();
    let url = mock_server.url("");

    let m = Mock::new()
        .expect_method(Method::POST)
        .expect_path("/modalities/foo/move")
        .expect_json_body(&ModalityMove {
            level: EntityKind::Study,
            target_aet: Some("MODALITY_TWO".to_string()),
            resources: vec![hashmap! {
                "StudyInstanceUID".to_string() => "99.88.77.66.5.4.3.2.1.0".to_string(),
            }],
            timeout: None,
        })
        .return_status(200)
        .create_on(&mock_server);

    let cl = Client::new(url);
    let res = cl
        .modality_move(
            "foo",
            ModalityMove {
                level: EntityKind::Study,
                target_aet: Some("MODALITY_TWO".to_string()),
                resources: vec![hashmap! {
                    "StudyInstanceUID".to_string() => "99.88.77.66.5.4.3.2.1.0".to_string(),
                }],
                timeout: None,
            },
        )
        .unwrap();

    assert_eq!(res, ());
    assert_eq!(m.times_called(), 1);
}

#[test]
fn test_modality_move_error() {
    let mock_server = MockServer::start();
    let url = mock_server.url("");

    let m = Mock::new()
        .expect_method(Method::POST)
        .expect_path("/modalities/foo/move")
        .expect_json_body(&ModalityMove {
            level: EntityKind::Study,
            target_aet: Some("MODALITY_TWO".to_string()),
            resources: vec![hashmap! {
                "StudyInstanceUID".to_string() => "99.88.77.66.5.4.3.2.1.0".to_string(),
            }],
            timeout: None,
        })
        .return_status(500)
        .return_header("Content-Type", "application/json")
        .return_body(
            r#"
                    {
                       "HttpError" : "Internal Server Error",
                       "HttpStatus" : 500,
                       "Message" : "Boom!",
                       "Method" : "POST",
                       "OrthancError" : "Boom!",
                       "OrthancStatus" : 27,
                       "Uri" : "/modalities/foo/move"
                    }
               "#,
        )
        .create_on(&mock_server);

    let cl = Client::new(url);
    let res = cl.modality_move(
        "foo",
        ModalityMove {
            level: EntityKind::Study,
            target_aet: Some("MODALITY_TWO".to_string()),
            resources: vec![hashmap! {
                "StudyInstanceUID".to_string() => "99.88.77.66.5.4.3.2.1.0".to_string(),
            }],
            timeout: None,
        },
    );

    assert_eq!(
        res.unwrap_err(),
        Error {
            message: "API error: 500 Internal Server Error".to_string(),
            details: Some(ApiError {
                method: "POST".to_string(),
                uri: "/modalities/foo/move".to_string(),
                message: "Boom!".to_string(),
                details: None,
                http_status: 500,
                http_error: "Internal Server Error".to_string(),
                orthanc_status: 27,
                orthanc_error: "Boom!".to_string(),
            },),
        },
    );
    assert_eq!(m.times_called(), 1);
}

#[test]
fn test_modality_find() {
    let mock_server = MockServer::start();
    let url = mock_server.url("");

    let m = Mock::new()
        .expect_method(Method::POST)
        .expect_path("/modalities/them/query")
        .expect_json_body(&ModalityFind {
            level: EntityKind::Study,
            query: hashmap! {
                "StudyInstanceUID".to_string() => "1.2.3.4.5.6.999".to_string()
            },
            normalize: None,
        })
        .return_status(200)
        .return_header("Content-Type", "application/json")
        .return_body(
            r#"
                {
                    "ID": "1c315256-3eef-4ef6-aa8a-03947cc53513",
                    "Path": "/queries/1c315256-3eef-4ef6-aa8a-03947cc53513"

                }
            "#,
        )
        .create_on(&mock_server);

    let cl = Client::new(url);
    assert_eq!(
        cl.modality_find(
            "them",
            EntityKind::Study,
            hashmap! {
                "StudyInstanceUID".to_string() => "1.2.3.4.5.6.999".to_string()
            },
            None
        )
        .unwrap(),
        ModalityFindResult {
            id: "1c315256-3eef-4ef6-aa8a-03947cc53513".to_string(),
            path: "/queries/1c315256-3eef-4ef6-aa8a-03947cc53513".to_string()
        }
    );

    assert_eq!(m.times_called(), 1);
}

#[test]
fn test_modality_find_error() {
    let mock_server = MockServer::start();
    let url = mock_server.url("");

    let m = Mock::new()
        .expect_method(Method::POST)
        .expect_path("/modalities/them/query")
        .expect_json_body(&ModalityFind {
            level: EntityKind::Study,
            query: hashmap! {
                "StudyInstanceUID".to_string() => "1.2.3.4.5.6.999".to_string()
            },
            normalize: None,
        })
        .return_status(500)
        .return_header("Content-Type", "application/json")
        .return_body(
            r#"
                {
                   "Details" : "DicomAssociation - C-FIND to AET \"ORTHANC\": Peer aborted Association (or never connected)",
                   "HttpError" : "Internal Server Error",
                   "HttpStatus" : 500,
                   "Message" : "Error in the network protocol",
                   "Method" : "POST",
                   "OrthancError" : "Error in the network protocol",
                   "OrthancStatus" : 9,
                   "Uri" : "/modalities/orthanc_main/query"
                }
            "#,
        )
        .create_on(&mock_server);

    let cl = Client::new(url);
    assert_eq!(
        cl.modality_find(
            "them",
            EntityKind::Study,
            hashmap! {
                "StudyInstanceUID".to_string() => "1.2.3.4.5.6.999".to_string()
            },
            None
        )
        .unwrap_err(),
        Error{
            message: "DicomAssociation - C-FIND to AET \"ORTHANC\": Peer aborted Association (or never connected)".to_string(),
            details: Some(ApiError {
                method: "POST".to_string(),
                uri: "/modalities/them/query".to_string(),
                message: "Error in the network protocol".to_string(),
                details: None,
                http_status: 500,
                http_error: "Internal Server Error".to_string(),
                orthanc_status: 9,
                orthanc_error: "Error in the network protocol".to_string(),

            })
        }
    );

    assert_eq!(m.times_called(), 1);
}
