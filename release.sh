#!/usr/bin/env bash

set -e

echo "Creating new release version"
CC=o64-clang CXX=o64-clang++ cargo build --release --target x86_64-apple-darwin

cd target/x86_64-apple-darwin/release
echo "Packaging release"
tar -czvf protovend.tar.gz protovend
echo "Generating sha256 checksum"
shasum -a 256 protovend.tar.gz > sha256.txt

echo "Uploading version ${CI_COMMIT_TAG} to gitlab packages"
curl --header "JOB-TOKEN: $CI_JOB_TOKEN" --upload-file protovend.tar.gz "${CI_API_V4_URL}/projects/${CI_PROJECT_ID}/packages/generic/${CI_PROJECT_NAME}/${CI_COMMIT_TAG}/protovend.tar.gz"
curl --header "JOB-TOKEN: $CI_JOB_TOKEN" --upload-file sha256.txt "${CI_API_V4_URL}/projects/${CI_PROJECT_ID}/packages/generic/${CI_PROJECT_NAME}/${CI_COMMIT_TAG}/sha256.txt"

echo "✅ done!"