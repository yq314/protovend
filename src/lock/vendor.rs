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

use super::Import;
use crate::Result;
use crate::{check, git};
use failure::format_err;
use lazy_static::lazy_static;
use regex::Regex;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

pub const PROTOS_OUTPUT_DIRECTORY: &str = "third_party/protovend";

lazy_static! {
    static ref PROTO_IMPORTS_RE: Regex = Regex::new(r#"import "([\w\\/\\.]+)";"#).unwrap();
}

pub(super) fn vendor_import(import: &Import) -> Result<()> {
    log::info!(
        "Fetching proto files {} branch from git repo. Current: {}",
        import.branch,
        import.url
    );
    let repo = git::get_repo(&import.url, &import.branch, &import.commit)?;
    let clone_location = repo.workdir().unwrap(); //Can unwrap safely as repository is not bare

    for proto_path in &import.proto_paths {
        let src_dir = &clone_location.join(&import.proto_dir);
        log::info!(
            "calling check with {} and {}",
            clone_location.display(),
            import.url
        );
        check::run_checks(clone_location, &import.proto_dir, proto_path)?;
        let result = find_and_copy_protos(
            src_dir,
            proto_path,
            &import.filename_regex,
            import.resolve_dependency,
        );
        match result {
            Ok(res) => res,
            Err(err) => log::error!("{}", err),
        };
    }
    Ok(())
}

pub(super) fn prepare_output_directory() -> Result<()> {
    let protos_output_dir = Path::new(PROTOS_OUTPUT_DIRECTORY);
    if protos_output_dir.exists() {
        fs::remove_dir_all(protos_output_dir)?;
    }

    fs::create_dir_all(protos_output_dir)?;

    Ok(())
}

fn create_dest_folder_path(repo: &str) -> Result<PathBuf> {
    Ok(env::current_dir()?
        .join(Path::new(PROTOS_OUTPUT_DIRECTORY))
        .join(Path::new(repo)))
}

fn find_and_copy_protos(
    src_dir: &Path,
    proto_path: &str,
    filename_regex: &str,
    resolve_dependency: bool,
) -> Result<()> {
    let src_folder = &src_dir.join(Path::new(proto_path));
    if !src_folder.exists() {
        return Err(format_err!(
            "Cannot find expected directory {}",
            src_folder.display()
        ));
    }

    let re = Regex::new(filename_regex).unwrap();
    for entry in WalkDir::new(src_folder) {
        let entry = entry?;
        if entry.metadata()?.is_file()
            && entry.file_name().to_string_lossy().ends_with(".proto")
            && re.is_match(entry.path().file_stem().unwrap().to_str().unwrap())
        {
            copy_protos(src_dir, proto_path, entry.path(), resolve_dependency)?;
        }
    }

    Ok(())
}

fn copy_protos(
    src_dir: &Path,
    proto_path: &str,
    proto_file_path: &Path,
    resolve_dependency: bool,
) -> Result<()> {
    let dest_folder = create_dest_folder_path(proto_path)?;
    let relative_path = proto_file_path.strip_prefix(src_dir.join(&proto_path))?;
    let dest_file = dest_folder.join(relative_path);
    fs::create_dir_all(dest_file.parent().unwrap())?;
    fs::copy(proto_file_path, &dest_file)?;
    log::debug!(
        "Copied {} to {}",
        proto_file_path.display(),
        dest_file.display()
    );

    if resolve_dependency {
        let file_content = fs::read_to_string(proto_file_path)?;
        for cap in PROTO_IMPORTS_RE.captures_iter(file_content.as_str()) {
            let import_path = Path::new(&src_dir).join(&cap[1]);
            if import_path.exists() {
                log::debug!("Found an imported dependency {}", &cap[1]);
                let import_proto_path = Path::new(&cap[1]).parent().unwrap();
                copy_protos(
                    src_dir,
                    import_proto_path.to_str().unwrap(),
                    &import_path,
                    resolve_dependency,
                )?;
            } else {
                log::debug!(
                    "Imported {} is not in the same repository",
                    import_path.display()
                );
            }
        }
    }

    Ok(())
}
