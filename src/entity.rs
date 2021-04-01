use crate::utils::serde_datetime;
use crate::Error;
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::convert::TryFrom;
use std::str;

/// Orthanc entity kinds (types).
///
/// Orthanc operates with 4 entity kinds, which correspond to the ones, available in DICOM.
/// In descending hierarchical order: Patient, Study, Series, Instance
#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
pub enum EntityKind {
    Patient,
    Study,
    Series,
    Instance,
}

impl TryFrom<bytes::Bytes> for EntityKind {
    type Error = Error;

    fn try_from(value: bytes::Bytes) -> Result<EntityKind, Error> {
        let v = value.to_vec();
        let s = str::from_utf8(&v)?;
        match s {
            "Patient" => Ok(EntityKind::Patient),
            "Study" => Ok(EntityKind::Study),
            "Series" => Ok(EntityKind::Series),
            "Instance" => Ok(EntityKind::Instance),
            _ => Err(Error::new(&format!("Unknown entity kind: {}", s), None)),
        }
    }
}

/// A trait, that implements common methods for all entity kinds
pub trait Entity: serde::de::DeserializeOwned {
    /// Entity kind
    fn kind() -> EntityKind;

    /// The ID of the entity
    fn id(&self) -> &str;

    /// The ID of the entity's parent ([`Patient`] for [`Study`], [`Study`] for [`Series`],
    /// [`Series`] for [`Instance`]. [`None`] if the Entity does not have a parent (e.g. is a
    /// [`Patient`])
    fn parent_id(&self) -> Option<&str> {
        None
    }

    /// Get the value of a DICOM tag from `main_dicom_tags`
    fn main_dicom_tag(&self, tag: &str) -> Option<&str>;

    /// The list of ID of the entity's children (studies for [`Patient`], series for [`Study`],
    /// instances for [`Series`])
    fn children(&self) -> &[String] {
        &[]
    }

    /// Number of children that the entity has
    fn children_len(&self) -> usize {
        0
    }

    /// Index of the instance in the series in case the entity is an [`Instance`], [`None`]
    /// otherwise
    fn index(&self) -> Option<u32> {
        None
    }

    /// Size of the instance file in case the entity is an [`Instance`], [`None`] otherwise
    fn size(&self) -> u64 {
        0
    }

    /// The kind of the entity's parent entity. [`None`] if the entity does not have a parent (e.g.
    /// is a `Patient`)
    fn parent_kind(&self) -> Option<EntityKind> {
        None
    }

    /// The name of the entity's parent entity. [`None`] if the entity does not have a parent (e.g.
    /// is a `Patient`)
    fn parent_kind_name(&self) -> Option<String> {
        self.parent_kind().map(|k| format!("{:?}", k))
    }

    /// Then name of the entity's child entity, pluralized (e.g. "Studies", "Series", "Instances").
    /// [`None`] if the entity does not have children (e.g. is an [`Instance`])
    fn children_kind_name(&self) -> Option<&str> {
        None
    }
}

/// Patient
#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub struct Patient {
    #[serde(rename = "ID")]
    pub id: String,
    pub is_stable: bool,
    #[serde(with = "serde_datetime")]
    pub last_update: NaiveDateTime,
    pub main_dicom_tags: HashMap<String, String>,
    pub studies: Vec<String>,
    #[serde(rename = "Type")]
    pub entity: EntityKind,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub anonymized_from: Option<String>,
}

impl Entity for Patient {
    /// Returns the [`EntityKind::Patient`] variant
    fn kind() -> EntityKind {
        EntityKind::Patient
    }

    /// The ID of the patient
    fn id(&self) -> &str {
        &self.id
    }

    /// Returns "studies"
    fn children_kind_name(&self) -> Option<&str> {
        Some("Studies")
    }

    /// Get the value of a DICOM tag from `main_dicom_tags`
    fn main_dicom_tag(&self, tag: &str) -> Option<&str> {
        self.main_dicom_tags.get(tag).map(AsRef::as_ref)
    }

    /// Returns the list of IDs of all studies that belong to this patient
    fn children(&self) -> &[String] {
        &self.studies
    }

    /// Number of studies that belong to this patient
    fn children_len(&self) -> usize {
        self.children().len()
    }
}

/// Study
#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub struct Study {
    #[serde(rename = "ID")]
    pub id: String,
    pub is_stable: bool,
    #[serde(with = "serde_datetime")]
    pub last_update: NaiveDateTime,
    pub main_dicom_tags: HashMap<String, String>,
    pub parent_patient: String,
    pub patient_main_dicom_tags: HashMap<String, String>,
    pub series: Vec<String>,
    #[serde(rename = "Type")]
    pub entity: EntityKind,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub anonymized_from: Option<String>,
}

impl Entity for Study {
    /// Returns the [`EntityKind::Study`] variant
    fn kind() -> EntityKind {
        EntityKind::Study
    }
    /// The ID of the study
    fn id(&self) -> &str {
        &self.id
    }

    /// The ID of the patient, that is the parent of this study
    fn parent_id(&self) -> Option<&str> {
        Some(&self.parent_patient)
    }

    /// Returns "series"
    fn children_kind_name(&self) -> Option<&str> {
        Some("Series")
    }

    /// Get the value of a DICOM tag from `main_dicom_tags`, or if the tag is absent there, from
    /// `patient_main_dicom_tags`.
    fn main_dicom_tag(&self, tag: &str) -> Option<&str> {
        match self.main_dicom_tags.get(tag).map(AsRef::as_ref) {
            Some(v) => Some(v),
            None => self.patient_main_dicom_tags.get(tag).map(AsRef::as_ref),
        }
    }

    /// Returns the list of IDs of all series that belong to this study
    fn children(&self) -> &[String] {
        &self.series
    }

    /// Number of series that belong to this study
    fn children_len(&self) -> usize {
        self.children().len()
    }

    /// Returns [`EntityKind::Patient`]
    fn parent_kind(&self) -> Option<EntityKind> {
        Some(EntityKind::Patient)
    }
}

/// Series
#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub struct Series {
    #[serde(rename = "ID")]
    pub id: String,
    pub status: String,
    pub is_stable: bool,
    #[serde(with = "serde_datetime")]
    pub last_update: NaiveDateTime,
    pub main_dicom_tags: HashMap<String, String>,
    pub parent_study: String,
    pub expected_number_of_instances: Option<u32>,
    pub instances: Vec<String>,
    #[serde(rename = "Type")]
    pub entity: EntityKind,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub anonymized_from: Option<String>,
}

impl Entity for Series {
    fn kind() -> EntityKind {
        EntityKind::Series
    }
    /// The ID of the series
    fn id(&self) -> &str {
        &self.id
    }

    /// The ID of the study, that is the parent of this series
    fn parent_id(&self) -> Option<&str> {
        Some(&self.parent_study)
    }

    /// Returns "instances"
    fn children_kind_name(&self) -> Option<&str> {
        Some("Instances")
    }

    /// Get the value of a DICOM tag from `main_dicom_tags`
    fn main_dicom_tag(&self, tag: &str) -> Option<&str> {
        self.main_dicom_tags.get(tag).map(AsRef::as_ref)
    }

    /// Returns the list of IDs of all instances that belong to this series
    fn children(&self) -> &[String] {
        &self.instances
    }

    /// Number of instances that belong to this series
    fn children_len(&self) -> usize {
        self.children().len()
    }

    /// Returns [`EntityKind::Study`]
    fn parent_kind(&self) -> Option<EntityKind> {
        Some(EntityKind::Study)
    }
}

/// Instance
#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub struct Instance {
    #[serde(rename = "ID")]
    pub id: String,
    pub main_dicom_tags: HashMap<String, String>,
    pub parent_series: String,
    pub index_in_series: Option<u32>,
    pub file_uuid: String,
    pub file_size: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modified_from: Option<String>,
    #[serde(rename = "Type")]
    pub entity: EntityKind,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub anonymized_from: Option<String>,
}

impl Entity for Instance {
    /// Returns the [`EntityKind::Instance`] variant
    fn kind() -> EntityKind {
        EntityKind::Instance
    }
    /// The ID of the instance
    fn id(&self) -> &str {
        &self.id
    }

    /// The ID of the series, that is the parent of this instance
    fn parent_id(&self) -> Option<&str> {
        Some(&self.parent_series)
    }

    /// Get the value of a DICOM tag from `main_dicom_tags`
    fn main_dicom_tag(&self, tag: &str) -> Option<&str> {
        self.main_dicom_tags.get(tag).map(AsRef::as_ref)
    }

    /// Returns [`EntityKind::Series`]
    fn parent_kind(&self) -> Option<EntityKind> {
        Some(EntityKind::Series)
    }

    /// Index of the instance in the series
    fn index(&self) -> Option<u32> {
        self.index_in_series
    }

    /// Size of the instance file
    fn size(&self) -> u64 {
        self.file_size
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;
    use maplit::hashmap;

    #[test]
    fn test_entity_kind_try_from_bytes() {
        assert_eq!(
            EntityKind::try_from(bytes::Bytes::from_static(b"Patient")).unwrap(),
            EntityKind::Patient
        );
        assert_eq!(
            EntityKind::try_from(bytes::Bytes::from_static(b"Study")).unwrap(),
            EntityKind::Study
        );
        assert_eq!(
            EntityKind::try_from(bytes::Bytes::from_static(b"Series")).unwrap(),
            EntityKind::Series
        );
        assert_eq!(
            EntityKind::try_from(bytes::Bytes::from_static(b"Instance")).unwrap(),
            EntityKind::Instance
        );
        assert_eq!(
            EntityKind::try_from(bytes::Bytes::from_static(b"Foobar")).unwrap_err(),
            Error::new("Unknown entity kind: Foobar", None)
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
