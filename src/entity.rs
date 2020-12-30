use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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
        match self.parent_kind() {
            Some(k) => Some(format!("{:?}", k)),
            None => None,
        }
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
    #[serde(with = "datetime_format")]
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
    #[serde(with = "datetime_format")]
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
    #[serde(with = "datetime_format")]
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

mod datetime_format {
    use chrono::NaiveDateTime;
    use serde::{self, Deserialize, Deserializer, Serializer};

    const FORMAT: &str = "%Y%m%dT%H%M%S";

    pub fn serialize<S>(date: &NaiveDateTime, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = format!("{}", date.format(FORMAT));
        serializer.serialize_str(&s)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<NaiveDateTime, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        NaiveDateTime::parse_from_str(&s, FORMAT).map_err(serde::de::Error::custom)
    }
}
