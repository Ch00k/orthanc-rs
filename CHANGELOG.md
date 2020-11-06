# Changelog

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
