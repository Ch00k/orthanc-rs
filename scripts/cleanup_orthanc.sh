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

ADDRESS_HOST=http://localhost
PORTS=( $ORC_MAIN_PORT $ORC_PEER_PORT $ORC_MODALITY_ONE_PORT $ORC_MODALITY_TWO_PORT )

for port in "${PORTS[@]}"
do
    delete $ADDRESS_HOST:$port patients
    delete $ADDRESS_HOST:$port modalities
    delete $ADDRESS_HOST:$port peers
done
