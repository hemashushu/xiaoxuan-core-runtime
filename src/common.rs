// Copyright (c) 2025 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE, LICENSE.additional and CONTRIBUTING.

use std::{
    collections::VecDeque,
    fs::File,
    io::ErrorKind,
    path::{Path, PathBuf},
    time::SystemTime,
};

use anc_image::{
    entry::{ExternalLibraryEntry, ImportModuleEntry},
    DependencyHash,
};
use serde::{Deserialize, Serialize};

use crate::{
    entry::ModuleConfig, RuntimeError, DIRECTORY_NAME_ASSEMBLY, DIRECTORY_NAME_ASSET,
    DIRECTORY_NAME_IR, DIRECTORY_NAME_OBJECT, DIRECTORY_NAME_OUTPUT, FILE_EXTENSION_ASSEMBLY,
    FILE_EXTENSION_META, FILE_EXTENSION_MODULE, FILE_EXTENSION_OBJECT, MODULE_CONFIG_FILE_NAME,
    MODULE_DIRECTORY_NAME_APP, MODULE_DIRECTORY_NAME_SRC, MODULE_DIRECTORY_NAME_TESTS,
};

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct FileMeta {
    // The last modified time of source file
    pub timestamp: Option<u64>,

    /// the default value is []
    #[serde(default)]
    pub dependencies: Vec<String>,
}

pub struct PathWithTimestamp {
    pub path_buf: PathBuf,

    /// Some file system (e.g. FAT32) does not support timestamp.
    pub timestamp: Option<u64>,
}

pub fn load_module_config(module_config_file_path: &Path) -> Result<ModuleConfig, RuntimeError> {
    let config_file =
        File::open(module_config_file_path).map_err(|e| RuntimeError::Message(format!("{}", e)))?;

    ason::from_reader(config_file).map_err(|e| {
        RuntimeError::Message(match std::fs::read_to_string(module_config_file_path) {
            Ok(source_code) => e.with_source(&source_code),
            Err(e) => format!("{}", e),
        })
    })
}

pub fn get_dependencies(
    module_config: &ModuleConfig,
) -> (Vec<ImportModuleEntry>, Vec<ExternalLibraryEntry>) {
    let import_module_entries = module_config
        .modules
        .iter()
        .map(|(name, module_dependency)| {
            ImportModuleEntry::new(name.to_owned(), Box::new(module_dependency.to_owned()))
        })
        .collect::<Vec<_>>();

    let external_library_entries = module_config
        .libraries
        .iter()
        .map(|(name, external_library_dependency)| {
            ExternalLibraryEntry::new(
                name.to_owned(),
                Box::new(external_library_dependency.to_owned()),
            )
        })
        .collect::<Vec<_>>();

    (import_module_entries, external_library_entries)
}

pub fn get_file_mata_path(target_directory: &Path, file_name: &str) -> PathBuf {
    let mut meta_file_path_buf = PathBuf::from(target_directory);
    meta_file_path_buf.push(file_name);
    meta_file_path_buf.set_extension(FILE_EXTENSION_META);
    meta_file_path_buf
}

pub fn get_file_mata_path_by_full_name(full_path: &Path) -> PathBuf {
    let mut meta_file_path_buf = PathBuf::from(full_path);
    meta_file_path_buf.set_extension(FILE_EXTENSION_META);
    meta_file_path_buf
}

pub fn get_file_meta(meta_file_path: &Path) -> Result<Option<FileMeta>, RuntimeError> {
    let config_file = match File::open(meta_file_path) {
        Ok(f) => f,
        Err(e) if e.kind() == ErrorKind::NotFound => {
            return Ok(None);
        }
        Err(e) => {
            return Err(RuntimeError::Message(format!("{}", e)));
        }
    };

    ason::from_reader(config_file)
        .map_err(|e| {
            RuntimeError::Message(match std::fs::read_to_string(meta_file_path) {
                Ok(source_code) => e.with_source(&source_code),
                Err(e) => format!("{}", e),
            })
        })
        .map(|o| Some(o))
}

pub fn get_module_config_file(module_path: &Path) -> PathBuf {
    let mut path_buf = PathBuf::from(module_path);
    path_buf.push(MODULE_CONFIG_FILE_NAME);
    path_buf
}

pub fn get_src_path(module_path: &Path) -> PathBuf {
    let mut path_buf = PathBuf::from(module_path);
    path_buf.push(MODULE_DIRECTORY_NAME_SRC);
    path_buf
}

pub fn get_app_path(module_path: &Path) -> PathBuf {
    let mut path_buf = PathBuf::from(module_path);
    path_buf.push(MODULE_DIRECTORY_NAME_APP);
    path_buf
}

pub fn get_tests_path(module_path: &Path) -> PathBuf {
    let mut path_buf = PathBuf::from(module_path);
    path_buf.push(MODULE_DIRECTORY_NAME_TESTS);
    path_buf
}

pub fn get_output_path(module_path: &Path, hash_opt: Option<&DependencyHash>) -> PathBuf {
    let mut path_buf = PathBuf::from(module_path);
    path_buf.push(DIRECTORY_NAME_OUTPUT);

    // application type modules have no hash directory.
    if let Some(hash) = hash_opt {
        let hash_string = hash
            .iter()
            .map(|value| format!("{:02x}", value))
            .collect::<Vec<String>>()
            .join("");

        path_buf.push(hash_string);
    }

    path_buf
}

pub fn get_output_asset_path(output_path: &Path) -> PathBuf {
    let mut path_buf = PathBuf::from(output_path);
    path_buf.push(DIRECTORY_NAME_ASSET);
    path_buf
}

pub fn get_output_ir_path(output_path: &Path) -> PathBuf {
    let mut path_buf = PathBuf::from(output_path);
    path_buf.push(DIRECTORY_NAME_IR);
    path_buf
}

pub fn get_output_assembly_path(output_path: &Path) -> PathBuf {
    let mut path_buf = PathBuf::from(output_path);
    path_buf.push(DIRECTORY_NAME_ASSEMBLY);
    path_buf
}

pub fn get_output_object_path(output_path: &Path) -> PathBuf {
    let mut path_buf = PathBuf::from(output_path);
    path_buf.push(DIRECTORY_NAME_OBJECT);
    path_buf
}

pub fn get_output_shared_module_file(output_path: &Path, module_name: &str) -> PathBuf {
    let mut path_buf = PathBuf::from(output_path);
    path_buf.push(module_name);
    path_buf.set_extension(FILE_EXTENSION_MODULE);
    path_buf
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

/**
 * The possible 'start_path' are:
 * - './src'
 * - './app'
 * - './tests'
 * - './output/assembly'
 */
pub fn list_assembly_files(start_path: &Path) -> Result<Vec<PathWithTimestamp>, RuntimeError> {
    let mut assembly_files = vec![];
    let mut subfolders = VecDeque::new();

    let start_path_buf = PathBuf::from(start_path);
    subfolders.push_front(start_path_buf);

    while subfolders.len() > 0 {
        let current_path_buf = subfolders.pop_back().unwrap();
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
                subfolders.push_front(path_buf);
            } else {
                if matches!(
                    path_buf.extension().map(|e| e.to_str().unwrap()),
                    Some(FILE_EXTENSION_ASSEMBLY)
                ) {
                    assembly_files.push(PathWithTimestamp {
                        path_buf,
                        timestamp,
                    });
                }
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

    use crate::common::get_src_path;

    use super::list_assembly_files;

    fn get_resources_path_buf() -> PathBuf {
        // returns the project's root folder
        let mut pwd = std::env::current_dir().unwrap();
        // append subfolders
        pwd.push("tests");
        pwd.push("resources");
        pwd
    }

    #[test]
    fn test_list_assembly_files() {
        let mut moudle_path_buf = get_resources_path_buf();
        moudle_path_buf.push("single_module_app");

        let src_path_buf = get_src_path(&moudle_path_buf);
        let src_path = src_path_buf.as_path();
        let assembly_files = list_assembly_files(src_path).unwrap();

        let name_paths = assembly_files
            .iter()
            .map(|item| item.path_buf.strip_prefix(src_path).unwrap())
            .collect::<Vec<_>>();

        let names = name_paths
            .iter()
            .map(|item| item.to_str().unwrap())
            .collect::<Vec<_>>();

        // the order of list item is variable
        assert!(names.iter().find(|n| **n == "math.anca").is_some());
        assert!(names.iter().find(|n| **n == "base.anca").is_some());
        assert!(names.iter().find(|n| **n == "lib.anca").is_some());
        assert!(names.iter().find(|n| **n == "main.anca").is_some());
        assert!(names
            .iter()
            .find(|n| **n == "base/primitive.anca")
            .is_some());
    }
}
