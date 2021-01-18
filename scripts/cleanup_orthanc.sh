#!/usr/bin/env bash

set -e

if ! hash jq 2>/dev/null; then
    echo "jq not found, cannot continue"
    exit 1
fi

curl_command="curl"
if [ -n $ORC_ORTHANC_USERNAME ] && [ -n $ORC_ORTHANC_PASSWORD ]; then
    curl_command="$curl_command --user $ORC_ORTHANC_USERNAME:$ORC_ORTHANC_PASSWORD"
fi

delete() {
    address=$1
    entities=$2
    list_curl_command="$curl_command $address/$entities"
    list=($($list_curl_command | jq -c '.[]' | tr -d '"'))

    for e in "${list[@]}"; do
        delete_curl_command="$list_curl_command/$e -X DELETE"
        echo $delete_curl_command
        $delete_curl_command
    done
}

delete $ORC_ORTHANC_ADDRESS patients
delete $ORC_ORTHANC_ADDRESS modalities
delete $ORC_ORTHANC_ADDRESS peers
delete $ORC_ORTHANC_PEER_ADDRESS patients
