[![crate](https://img.shields.io/crates/v/orthanc.svg)](https://crates.io/crates/orthanc)
[![doc](https://docs.rs/orthanc/badge.svg)](https://docs.rs/orthanc)
[![test](https://github.com/Ch00k/orthanc-rs/workflows/tests/badge.svg)](https://github.com/Ch00k/orthanc-rs/actions)
[![codecov](https://codecov.io/gh/Ch00k/orthanc-rs/branch/master/graphs/badge.svg)](https://codecov.io/github/Ch00k/orthanc-rs)
[![license](https://img.shields.io/crates/l/orthanc.svg)](./UNLICENSE)

# orthanc-rs

**orthanc-rs** is a client for the [REST API](https://book.orthanc-server.com/users/rest.html)
of [Orthanc](https://book.orthanc-server.com/users/rest.html), an open-source, lightweight
DICOM server.

To use the crate, add the dependency to your `Cargo.toml`:

```ini
[dependencies]
orthanc = "0.4.0"
```

## Usage

Create an API client instance:

```rust
use orthanc::Client;
let client = Client::new("http://localhost:8042".to_string());
```

If authentication is enabled on the Orthanc instance:

```rust
client.auth("username".to_string(), "password".to_string());
```

List patients:

```rust
client.patients();
```

Or in an expanded format:

```rust
client.patients_expanded();
```

Get all DICOM tags of an instance:

```rust
let instance_id = "0b62ebce-8ab7b938-e5ca1b05-04802ab3-42ee4307";
let tags = client.instance_tags(instance_id);
println!("{}", tags["PatientID"]);
```

Download a study:

```rust
let study_id = "9357491d-427a6c94-4080b6c8-1997f4aa-af658240";
let mut file = fs::File::create("/tmp/study.zip").unwrap();
client.study_dicom(study_id, &mut file).unwrap();
```

Even though the operation is not very efficient, Orthanc allows uploading DICOM files over
REST API:

```rust
let data = fs::read("/tmp/instance.dcm").unwrap();
client.upload(&data).unwrap();
```

See `tests` directory for more usage examples.

## TODO

* Instance images (`/instances/<id>/{preview,image-uint8,image-uint16}`)
* Split/merge studies (`/studies/<id>/{split,merge}`)
* C-MOVE (`/modalities/<id>/move`)
* C-FIND (`/modalities/<id>/query`)
* Peers API (`/peers`)
* Tools API (`/tools`)
* Log API (`/changes`, `/exports`)
* Asynchronous requests (`/jobs`)
