#!/usr/bin/env bash

# ftp://medical.nema.org/medical/dicom/DataSets/WG16/Philips/
curl -i https://minuteware.net/orc/test_data.tar.bz2 > /tmp/test_data.tar.bz2
tar xvjf /tmp/test_data.tar.bz2

curl_command="curl -i -X POST -H 'Expect:'"

if [ -n $ORC_ORTHANC_USERNAME ] && [ -n $ORC_ORTHANC_PASSWORD ]; then
    curl_command="$curl_command --user $ORC_ORTHANC_USERNAME:$ORC_ORTHANC_PASSWORD $ORC_ORTHANC_ADDRESS/instances"
fi

for f in $(find /tmp/test_data -type f); do
    cmd="$curl_command --data-binary @$f"
    $cmd
done

cargo test --test integration -- --show-output
