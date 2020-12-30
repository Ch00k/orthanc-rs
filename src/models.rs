use crate::entity::EntityKind;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// System
#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub struct System {
    pub name: String,
    pub version: String,
    pub api_version: u8,
    pub database_version: u8,
    pub database_backend_plugin: Option<String>,
    pub dicom_aet: String,
    pub dicom_port: u16,
    pub http_port: u16,
    pub is_http_server_secure: bool,
    pub plugins_enabled: bool,
    pub storage_area_plugin: Option<String>,
}

/// Modality
#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub struct Modality {
    #[serde(rename = "AET")]
    pub aet: String,
    pub host: String,
    pub port: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub manufacturer: Option<String>,
    #[serde(rename = "AllowEcho")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allow_c_echo: Option<bool>,
    #[serde(rename = "AllowFind")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allow_c_find: Option<bool>,
    #[serde(rename = "AllowGet")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allow_c_get: Option<bool>,
    #[serde(rename = "AllowMove")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allow_c_move: Option<bool>,
    #[serde(rename = "AllowStore")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allow_c_store: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allow_n_action: Option<bool>,
    #[serde(rename = "AllowEventReport")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allow_n_event_report: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allow_transcoding: Option<bool>,
}

/// Anonymization request body
#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
pub struct Anonymization {
    #[serde(rename(serialize = "Replace"))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub replace: Option<HashMap<String, String>>,
    #[serde(rename(serialize = "Keep"))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub keep: Option<Vec<String>>,
    #[serde(rename(serialize = "KeepPrivateTags"))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub keep_private_tags: Option<bool>,
    #[serde(rename(serialize = "DicomVersion"))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dicom_version: Option<String>,
    #[serde(rename(serialize = "Force"))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub force: Option<bool>,
}

/// Modification request body
#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
pub struct Modification {
    #[serde(rename(serialize = "Replace"))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub replace: Option<HashMap<String, String>>,
    #[serde(rename(serialize = "Remove"))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remove: Option<Vec<String>>,
    #[serde(rename(serialize = "Force"))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub force: Option<bool>,
}

/// Ancestor of an entity
///
/// Returned as response body in DELETE responses to indicate the remaining ancestor of the deleted
/// entity.
///
/// For example, an ancestor of a deleted [`Instance`] is a [`Series`], an ancestor of a deleted [`Study`] is a
/// [`Patient`]. [`Patient`] does not have an ancestor.
#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub struct Ancestor {
    #[serde(rename = "ID")]
    pub id: String,
    pub path: String,
    #[serde(rename = "Type")]
    pub entity: EntityKind,
}

/// Remaining ancestor response
///
/// Returned as response body in DELETE responses. See [`Ancestor`] for details.
#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub struct RemainingAncestor {
    pub remaining_ancestor: Option<Ancestor>,
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub struct Search {
    pub level: EntityKind,
    pub query: HashMap<String, String>,
    pub expand: Option<bool>,
}

/// Result of a DICOM upload request
#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub struct UploadResult {
    #[serde(rename = "ID")]
    pub id: String,
    pub status: String,
    pub path: String,
    pub parent_patient: String,
    pub parent_study: String,
    pub parent_series: String,
}

/// Result of a C-STORE request (sending entities to a modality)
#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub struct StoreResult {
    pub description: String,
    pub local_aet: String,
    pub remote_aet: String,
    pub parent_resources: Vec<String>,
    pub instances_count: u64,
    pub failed_instances_count: u64,
}

/// Result of a modification or anonymization request
#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub struct ModificationResult {
    #[serde(rename = "ID")]
    pub id: String,
    #[serde(rename = "PatientID")]
    pub patient_id: String,
    pub path: String,
    #[serde(rename = "Type")]
    pub entity: EntityKind,
}

#[cfg(test)]
mod tests {
    use super::*;
    use maplit::hashmap;

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
}
