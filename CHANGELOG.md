# Changelog

## [0.5.2](https://github.com/Ch00k/orthanc-rs/compare/0.5.1...0.5.2) - 2020-12-06

**Fixes and improvements**

* Implemented getting system info through `/system` endpoint
  [#42](https://github.com/Ch00k/orthanc-rs/pull/42)

## [0.5.1](https://github.com/Ch00k/orthanc-rs/compare/0.5.0...0.5.1) - 2020-11-16

**Fixes and improvements**

* Implemented `Debug` for `Client` type [#36](https://github.com/Ch00k/orthanc-rs/pull/36)

## [0.5.0](https://github.com/Ch00k/orthanc-rs/compare/0.4.0...0.5.0) - 2020-11-15

**Fixes and improvements**

* Added methods to create, modify and delete modalities
  [#34](https://github.com/Ch00k/orthanc-rs/pull/34)


## [0.4.0](https://github.com/Ch00k/orthanc-rs/compare/0.3.0...0.4.0) - 2020-11-06

**Fixes and improvements**

* Added `force` field to `Anonymization` struct to support anonymization of protected
  DICOM tags [#30](https://github.com/Ch00k/orthanc-rs/pull/30)

## [0.3.0](https://github.com/Ch00k/orthanc-rs/compare/0.2.1...0.3.0) - 2020-11-01

**Fixes and improvements**

* Fixed an issue with unexpectedly absent `IndexInSeries` field of instance JSON
  [#26](https://github.com/Ch00k/orthanc-rs/pull/26)
* Increased client timeout to 600 seconds
  [#27](https://github.com/Ch00k/orthanc-rs/pull/27)
* Switched license from WTFPL to Unlicense
  [#28](https://github.com/Ch00k/orthanc-rs/pull/28)
