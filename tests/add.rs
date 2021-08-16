/*
 * Copyright 2020 Skyscanner Limited.
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 * http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
*/

use common::command;
use tempfile;

mod common;

#[cfg(test)]
#[path = "../tests_utils/mod.rs"]
mod tests_utils;

#[test]
fn test_add_no_init() {
    let dir = tempfile::tempdir().unwrap();
    assert!(!dir.path().join(".protovend.yml").exists());

    let status = command(&dir)
        .arg("add")
        .arg("https://github.com/Skyscanner/protovend-test-protos.git")
        .status()
        .unwrap();

    assert!(!status.success());

    assert!(!dir.path().join(".protovend.yml").exists());
}

#[test]
fn test_add_with_init() {
    let dir = tempfile::tempdir().unwrap();
    let status = command(&dir).arg("init").status().unwrap();

    assert!(status.success());

    assert!(dir.path().join(".protovend.yml").exists());

    let status = command(&dir)
        .arg("add")
        .arg("https://github.com/Skyscanner/protovend-test-protos.git")
        .status()
        .unwrap();

    assert!(status.success());

    let expected_contents = String::from(
        "---\
         \nmin_protovend_version: 4.2.0\
         \nvendor:\
         \n  - url: \"https://github.com/Skyscanner/protovend-test-protos.git\"\
         \n    branch: master\
         \n    proto_dir: proto\
         \n    proto_paths:\
         \n      - skyscanner/protovendtestprotos\
         \n    filename_regex: \".*\"\
         \n    resolve_dependency: false\n",
    );

    tests_utils::fs::assert_file_contents_eq(
        expected_contents,
        dir.path().join(".protovend.yml").as_path(),
    );
}

#[test]
fn test_add_two() {
    let dir = tempfile::tempdir().unwrap();
    let status = command(&dir).arg("init").status().unwrap();

    assert!(status.success());

    assert!(dir.path().join(".protovend.yml").exists());

    let status = command(&dir)
        .arg("add")
        .arg("https://github.com/Skyscanner/protovend-test-protos.git")
        .status()
        .unwrap();

    assert!(status.success());

    let status = command(&dir)
        .arg("add")
        .arg("git@github.com:Skyscanner/protovend-test-protos-fake.git")
        .arg("-d=src/proto")
        .arg("-p=path/to")
        .arg("-f=^(a|b)c$")
        .status()
        .unwrap();

    assert!(status.success());

    let expected_contents = String::from(
        "---\
         \nmin_protovend_version: 4.2.0\
         \nvendor:\
         \n  - url: \"git@github.com:Skyscanner/protovend-test-protos-fake.git\"\
         \n    branch: master\
         \n    proto_dir: src/proto\
         \n    proto_paths:\
         \n      - path/to\
         \n    filename_regex: ^(a|b)c$\
         \n    resolve_dependency: false\
         \n  - url: \"https://github.com/Skyscanner/protovend-test-protos.git\"\
         \n    branch: master\
         \n    proto_dir: proto\
         \n    proto_paths:\
         \n      - skyscanner/protovendtestprotos\
         \n    filename_regex: \".*\"\
         \n    resolve_dependency: false\n",
    );

    tests_utils::fs::assert_file_contents_eq(
        expected_contents,
        dir.path().join(".protovend.yml").as_path(),
    );
}

#[test]
fn test_add_with_different_proto_paths() {
    let dir = tempfile::tempdir().unwrap();
    let status = command(&dir).arg("init").status().unwrap();

    assert!(status.success());

    assert!(dir.path().join(".protovend.yml").exists());

    let status = command(&dir)
        .arg("add")
        .arg("git@github.com:Skyscanner/protovend-test-protos2.git")
        .arg("-p=path1/to")
        .status()
        .unwrap();

    assert!(status.success());

    let status = command(&dir)
        .arg("add")
        .arg("git@github.com:Skyscanner/protovend-test-protos2.git")
        .arg("-p=path2/to")
        .status()
        .unwrap();

    assert!(status.success());

    let expected_contents = String::from(
        "---\
         \nmin_protovend_version: 4.2.0\
         \nvendor:\
         \n  - url: \"git@github.com:Skyscanner/protovend-test-protos2.git\"\
         \n    branch: master\
         \n    proto_dir: proto\
         \n    proto_paths:\
         \n      - path1/to\
         \n      - path2/to\
         \n    filename_regex: \".*\"\
         \n    resolve_dependency: false\n",
    );

    tests_utils::fs::assert_file_contents_eq(
        expected_contents,
        dir.path().join(".protovend.yml").as_path(),
    );
}
