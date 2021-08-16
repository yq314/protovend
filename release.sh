#!/usr/bin/env bash

if [[ "${GITLAB_TOKEN}" == "" ]]; then
    echo "env var GITLAB_TOKEN is not set"
    exit 1
fi

echo "Creating new release version"
cargo build --release

cd target/release/ || exit
echo "Packaging release"
tar -czvf protovend.tar.gz protovend
echo "Generating sha256 checksum"
shasum -a 256 protovend.tar.gz > sha256.txt

VERSION=$(./protovend -V | awk  '{print $2}')
echo "Uploading version ${VERSION} to gitlab packages"
curl --header "PRIVATE-TOKEN: ${GITLAB_TOKEN}" --upload-file protovend.tar.gz "https://source.golabs.io/api/v4/projects/24005/packages/generic/protovend/${VERSION}/protovend.tar.gz"
curl --header "PRIVATE-TOKEN: ${GITLAB_TOKEN}" --upload-file sha256.txt "https://source.golabs.io/api/v4/projects/24005/packages/generic/protovend/${VERSION}/sha256.txt"
