#![warn(missing_debug_implementations)]
#![deny(broken_intra_doc_links)]

//! **orthanc-rs** is a client for the [REST API](https://book.orthanc-server.com/users/rest.html)
//! of [Orthanc](https://book.orthanc-server.com/users/rest.html), an open-source, lightweight
//! DICOM server.
//!
//! To use the crate, add the dependency to your `Cargo.toml`:
//!
//! ```ini
//! [dependencies]
//! orthanc = "0.6.1"
//! ```
//!
//! ## Usage
//!
//! Create an API client instance:
//!
//! ```rust
//! use orthanc::Client;
//! let client = Client::new("http://localhost:8042");
//! ```
//!
//! If authentication is enabled on the Orthanc instance:
//!
//! ```rust
//! client.auth("username", "password");
//! ```
//!
//! List patients:
//!
//! ```rust
//! client.patients();
//! ```
//!
//! Or in an expanded format:
//!
//! ```rust
//! client.patients_expanded();
//! ```
//!
//! Get all DICOM tags of an instance:
//!
//! ```rust
//! let instance_id = "0b62ebce-8ab7b938-e5ca1b05-04802ab3-42ee4307";
//! let tags = client.instance_tags(instance_id);
//! println!("{}", tags["PatientID"]);
//! ```
//!
//! Download a study:
//!
//! ```rust
//! let study_id = "9357491d-427a6c94-4080b6c8-1997f4aa-af658240";
//! let mut file = fs::File::create("/tmp/study.zip").unwrap();
//! client.study_dicom(study_id, &mut file).unwrap();
//! ```
//!
//! Even though the operation is not very efficient, Orthanc allows uploading DICOM files over
//! REST API:
//!
//! ```rust
//! let data = fs::read("/tmp/instance.dcm").unwrap();
//! client.upload(&data).unwrap();
//! ```

pub use client::Client;
pub use error::{ApiError, Error};
use std::result;

pub mod client;
pub mod entity;
pub mod error;
pub mod models;
mod utils;

type Result<T> = result::Result<T, Error>;
