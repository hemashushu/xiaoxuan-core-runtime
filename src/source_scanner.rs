// Copyright (c) 2025 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE, LICENSE.additional and CONTRIBUTING.

use std::{
    collections::VecDeque,
    path::{Path, PathBuf},
    time::SystemTime,
};

use crate::{RuntimeError, FILE_EXTENSION_ASSEMBLY, FILE_EXTENSION_OBJECT};

pub struct PathAndTimestamp {
    pub file_path: PathBuf,

    /// Some file system (e.g. FAT32) does not support timestamp.
    pub timestamp: Option<u64>,
}

/// Some file system (e.g. FAT32) does not support timestamp.
pub fn get_file_timestamp(file_path: &Path) -> Result</* timestamp */ Option<u64>, RuntimeError> {
    let metadata = file_path
        .metadata()
        .map_err(|e| RuntimeError::Message(format!("{}", e)))?;
    let last_modified = metadata.modified().ok();
    let timestamp =
        last_modified.map(|t| t.duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs());
    Ok(timestamp)
}

pub fn list_source_files(_scan_start_path: &Path) -> Result<Vec<PathAndTimestamp>, RuntimeError> {
    todo!()
}

pub fn list_ir_files(_scan_start_path: &Path) -> Result<Vec<PathAndTimestamp>, RuntimeError> {
    todo!()
}

/// List all assembly files under the specified folder and its subfolders.
///
/// The possible 'start_path' are:
/// - './src'
/// - './app'
/// - './tests'
///
/// Do NOT scan the resource in folder './output/{hash}/asset', because
/// it is possible that some source files have been deleted, but the
/// corresponding resources are still in that folder.
///
/// Return an empty array if the specified folder does not exist.
pub fn list_assembly_files(scan_start_path: &Path) -> Result<Vec<PathAndTimestamp>, RuntimeError> {
    let mut assembly_files = vec![];

    if !scan_start_path.exists() {
        return Ok(assembly_files);
    }

    let mut subfolders = VecDeque::new();

    let start_path_buf = PathBuf::from(scan_start_path);
    subfolders.push_back(start_path_buf);

    while !subfolders.is_empty() {
        let current_path_buf = subfolders.pop_front().unwrap();
        let current_dir = std::fs::read_dir(current_path_buf)
            .map_err(|e| RuntimeError::Message(format!("{}", e)))?;

        for file in current_dir {
            let entry = file.unwrap();
            let metadata = entry
                .metadata()
                .map_err(|e| RuntimeError::Message(format!("{}", e)))?;

            let last_modified = metadata.modified().ok();
            let timestamp =
                last_modified.map(|t| t.duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs());
            let path_buf = entry.path();

            if metadata.is_dir() {
                subfolders.push_back(path_buf);
            } else if matches!(
                path_buf.extension().map(|e| e.to_str().unwrap()),
                Some(FILE_EXTENSION_ASSEMBLY)
            ) {
                assembly_files.push(PathAndTimestamp {
                    file_path: path_buf,
                    timestamp,
                });
            }
        }
    }

    Ok(assembly_files)
}

pub fn list_object_files(object_file_directory: &Path) -> Result<Vec<PathBuf>, RuntimeError> {
    let mut object_files = vec![];

    let path_buf = PathBuf::from(object_file_directory);
    let dir = std::fs::read_dir(path_buf).map_err(|e| RuntimeError::Message(format!("{}", e)))?;

    for file in dir {
        let entry = file.unwrap();
        let path_buf = entry.path();
        if matches!(
            path_buf.extension().map(|e| e.to_str().unwrap()),
            Some(FILE_EXTENSION_OBJECT)
        ) {
            object_files.push(path_buf);
        }
    }

    Ok(object_files)
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::locations::get_module_folder_src_path;

    use super::list_assembly_files;

    fn get_resources_path() -> PathBuf {
        // `std::env::current_dir()` returns the current Rust project's root folder
        let mut pwd = std::env::current_dir().unwrap();
        pwd.push("tests");
        pwd.push("resources");
        pwd
    }

    #[test]
    fn test_list_assembly_files() {
        let mut moudle_path_buf = get_resources_path();
        moudle_path_buf.push("single_module_app");

        let src_path_buf = get_module_folder_src_path(&moudle_path_buf);
        let src_path = src_path_buf.as_path();
        let assembly_files = list_assembly_files(src_path).unwrap();

        let name_paths = assembly_files
            .iter()
            .map(|item| item.file_path.strip_prefix(src_path).unwrap())
            .collect::<Vec<_>>();

        let names = name_paths
            .iter()
            .map(|item| item.to_str().unwrap())
            .collect::<Vec<_>>();

        // the order of list item is variable
        assert!(names.iter().any(|n| *n == "math.anca"));
        assert!(names.iter().any(|n| *n == "base.anca"));
        assert!(names.iter().any(|n| *n == "lib.anca"));
        assert!(names.iter().any(|n| *n == "main.anca"));
        assert!(names.iter().any(|n| *n == "base/primitive.anca"));
    }
}
