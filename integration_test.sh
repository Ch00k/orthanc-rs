#!/usr/bin/env bash

set -e

cleanup() {
    curl_command="curl"
    if [ -n $ORC_ORTHANC_USERNAME ] && [ -n $ORC_ORTHANC_PASSWORD ]; then
        curl_command="$curl_command --user $ORC_ORTHANC_USERNAME:$ORC_ORTHANC_PASSWORD"
    fi

    patients_curl_command="$curl_command $ORC_ORTHANC_ADDRESS/patients"
    patients=($($patients_curl_command | jq -c '.[]' | tr -d '"'))

    for patient in "${patients[@]}"; do
        delete_curl_command="$patients_curl_command/$patient -X DELETE"
        echo $delete_curl_command
        $delete_curl_command
    done
}

trap cleanup EXIT

export ORC_DATAFILES_PATH=/tmp/orc_test_data

# ftp://medical.nema.org/medical/dicom/DataSets/WG16/Philips/
curl https://minuteware.net/orc/test_data.tar.bz2 > /tmp/test_data.tar.bz2
mkdir -p $ORC_DATAFILES_PATH
tar xvjf /tmp/test_data.tar.bz2 -C $ORC_DATAFILES_PATH

curl_command="curl -i -X POST -H 'Expect:'"

if [ -n $ORC_ORTHANC_USERNAME ] && [ -n $ORC_ORTHANC_PASSWORD ]; then
    curl_command="$curl_command --user $ORC_ORTHANC_USERNAME:$ORC_ORTHANC_PASSWORD"
fi

curl_command="$curl_command $ORC_ORTHANC_ADDRESS/instances"

for f in $(find $ORC_DATAFILES_PATH/initial -type f); do
    cmd="$curl_command --data-binary @$f"
    $cmd
done

cargo test --test integration
