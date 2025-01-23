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
    format_dependency_hash, DependencyHash, ZERO_DEPENDENCY_HASH,
};
use anc_isa::ModuleDependency;
use serde::{Deserialize, Serialize};

use crate::{
    entry::ModuleConfig, RuntimeError, DIRECTORY_NAME_ASSEMBLY, DIRECTORY_NAME_ASSET,
    DIRECTORY_NAME_IR, DIRECTORY_NAME_LIBRARIES, DIRECTORY_NAME_MODULES, DIRECTORY_NAME_OBJECT,
    DIRECTORY_NAME_OUTPUT, DIRECTORY_NAME_RUNTIME, FILE_EXTENSION_ASSEMBLY, FILE_EXTENSION_IMAGE,
    FILE_EXTENSION_IR, FILE_EXTENSION_META, FILE_EXTENSION_MODULE, FILE_EXTENSION_OBJECT,
    MODULE_CONFIG_FILE_NAME, MODULE_DIRECTORY_NAME_APP, MODULE_DIRECTORY_NAME_SRC,
    MODULE_DIRECTORY_NAME_TESTS, VERSION_NAME_LOCAL_AND_REMOTE,
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
    pub file_path: PathBuf,

    /// Some file system (e.g. FAT32) does not support timestamp.
    pub timestamp: Option<u64>,
}

pub struct RuntimeProperty {
    /// - User: `~/.local/lib/anc`
    /// - Global: `/usr/local/lib/anc`
    /// - System: `/usr/lib/anc`
    current_anc_root_path: PathBuf,

    /// e.g. "2025"
    current_edition: String,
}

/*
 * path of modules
 */

pub fn get_module_path_by_dependency(
    module_name: &str,
    module_dependency: &ModuleDependency,
    runtime_property: &RuntimeProperty,
) -> PathBuf {
    match module_dependency {
        ModuleDependency::Local(dependency_local) => {
            let path_buf = PathBuf::from(&dependency_local.path);
            path_buf
        }
        ModuleDependency::Remote(_) => {
            // check existance
            let mut path_buf = runtime_property.get_modules_directory();
            path_buf.push(module_name);
            path_buf.push(VERSION_NAME_LOCAL_AND_REMOTE);
            path_buf
        }
        ModuleDependency::Share(dependency_share) => {
            // check existance
            let mut path_buf = runtime_property.get_modules_directory();
            path_buf.push(module_name);
            path_buf.push(&dependency_share.version);
            path_buf
        }
        ModuleDependency::Runtime => {
            let mut path_buf = runtime_property.get_builtin_modules_directory();
            path_buf.push(module_name);
            path_buf
        }
        ModuleDependency::Current => unreachable!(),
    }
}

/*
 * path of module folder
 */

/// `./src`
pub fn get_src_path(module_path: &Path) -> PathBuf {
    let mut path_buf = PathBuf::from(module_path);
    path_buf.push(MODULE_DIRECTORY_NAME_SRC);
    path_buf
}

/// `./app`
pub fn get_app_path(module_path: &Path) -> PathBuf {
    let mut path_buf = PathBuf::from(module_path);
    path_buf.push(MODULE_DIRECTORY_NAME_APP);
    path_buf
}

/// `./tests`
pub fn get_tests_path(module_path: &Path) -> PathBuf {
    let mut path_buf = PathBuf::from(module_path);
    path_buf.push(MODULE_DIRECTORY_NAME_TESTS);
    path_buf
}

/// `./output`
pub fn get_output_path(module_path: &Path) -> PathBuf {
    let mut path_buf = PathBuf::from(module_path);
    path_buf.push(DIRECTORY_NAME_OUTPUT);
    path_buf
}

/// `./output/{hash}`
pub fn get_output_hash_path(output_path: &Path, hash_opt: Option<&DependencyHash>) -> PathBuf {
    let hash = if let Some(hash) = hash_opt {
        hash
    } else {
        &ZERO_DEPENDENCY_HASH
    };

    let hash_string = format_dependency_hash(hash);
    let mut path_buf = PathBuf::from(output_path);
    path_buf.push(hash_string);
    path_buf
}

/// `./output/{hash}/asset`
pub fn get_hash_asset_path(hash_path: &Path) -> PathBuf {
    let mut path_buf = PathBuf::from(hash_path);
    path_buf.push(DIRECTORY_NAME_ASSET);
    path_buf
}

/// `./output/{hash}/asset/ir`
pub fn get_asset_ir_path(asset_path: &Path) -> PathBuf {
    let mut path_buf = PathBuf::from(asset_path);
    path_buf.push(DIRECTORY_NAME_IR);
    path_buf
}

/// `./output/{hash}/asset/assembly`
pub fn get_asset_assembly_path(asset_path: &Path) -> PathBuf {
    let mut path_buf = PathBuf::from(asset_path);
    path_buf.push(DIRECTORY_NAME_ASSEMBLY);
    path_buf
}

/// `./output/{hash}/asset/object`
pub fn get_asset_object_path(asset_path: &Path) -> PathBuf {
    let mut path_buf = PathBuf::from(asset_path);
    path_buf.push(DIRECTORY_NAME_OBJECT);
    path_buf
}

/// `./output/name.anci`
pub fn get_application_image_file_path(output_path: &Path, module_name: &str) -> PathBuf {
    let mut path_buf = PathBuf::from(output_path);
    path_buf.push(module_name);
    path_buf.set_extension(FILE_EXTENSION_IMAGE);
    path_buf
}

/// `./output/{hash}/name.ancm`
pub fn get_shared_module_file_path(hash_path: &Path, module_name: &str) -> PathBuf {
    let mut path_buf = PathBuf::from(hash_path);
    path_buf.push(module_name);
    path_buf.set_extension(FILE_EXTENSION_MODULE);
    path_buf
}

/// `./module.anc.ason`
pub fn get_module_config_file_path(module_path: &Path) -> PathBuf {
    let mut path_buf = PathBuf::from(module_path);
    path_buf.push(MODULE_CONFIG_FILE_NAME);
    path_buf
}

pub fn get_ir_file_path(ir_path: &Path, canonical_name: &str) -> PathBuf {
    let mut path_buf = PathBuf::from(ir_path);
    path_buf.push(canonical_name);
    path_buf.set_extension(FILE_EXTENSION_IR);
    path_buf
}

pub fn get_assembly_file_path(assembly_path: &Path, canonical_name: &str) -> PathBuf {
    let mut path_buf = PathBuf::from(assembly_path);
    path_buf.push(canonical_name);
    path_buf.set_extension(FILE_EXTENSION_ASSEMBLY);
    path_buf
}

pub fn get_object_file_path(object_path: &Path, canonical_name: &str) -> PathBuf {
    let mut path_buf = PathBuf::from(object_path);
    path_buf.push(canonical_name);
    path_buf.set_extension(FILE_EXTENSION_OBJECT);
    path_buf
}

/*
 * path of file
 */

/// Replace the extension name of file to ".meta.ason", e.g.
/// "lib.anco" -> "lib.meta.ason"
pub fn get_mata_file_path(directory: &Path, file_name: &str) -> PathBuf {
    let mut meta_file_path_buf = PathBuf::from(directory);
    meta_file_path_buf.push(file_name);
    meta_file_path_buf.set_extension(FILE_EXTENSION_META);
    meta_file_path_buf
}

/// Replace the extension name of file to ".meta.ason", e.g.
/// "lib.anco" -> "lib.meta.ason"
pub fn get_mata_file_path_by_full_name(full_path: &Path) -> PathBuf {
    let mut meta_file_path_buf = PathBuf::from(full_path);
    meta_file_path_buf.set_extension(FILE_EXTENSION_META);
    meta_file_path_buf
}

/*
 * load or list data
 */

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

pub fn get_dependencies_by_module_config(
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

pub fn load_file_meta(meta_file_path: &Path) -> Result<Option<FileMeta>, RuntimeError> {
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

/// List all assembly files under the specified folder and its subfolders.
///
/// The possible 'start_path' are:
/// - './src'
/// - './app'
/// - './tests'
///
///
/// Do NOT scan the resource in folder './output/{hash}/asset', because
/// it is possible that some source files have been deleted, but the
/// corresponding resources are still in that folder.
///
/// Return an empty array if the specified folder does not exist.
pub fn list_assembly_files(scan_start_path: &Path) -> Result<Vec<PathWithTimestamp>, RuntimeError> {
    let mut assembly_files = vec![];

    if !scan_start_path.exists() {
        return Ok(assembly_files);
    }

    let mut subfolders = VecDeque::new();

    let start_path_buf = PathBuf::from(scan_start_path);
    subfolders.push_back(start_path_buf);

    while subfolders.len() > 0 {
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
            } else {
                if matches!(
                    path_buf.extension().map(|e| e.to_str().unwrap()),
                    Some(FILE_EXTENSION_ASSEMBLY)
                ) {
                    assembly_files.push(PathWithTimestamp {
                        file_path: path_buf,
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

impl RuntimeProperty {
    pub fn new(current_anc_root_path: PathBuf, current_edition: String) -> Self {
        Self {
            current_anc_root_path,
            current_edition,
        }
    }

    /// - User: `~/.local/lib/anc/EDITION/runtime`
    /// - Global: `/usr/local/lib/anc/EDITION/runtime`
    /// - System: `/usr/lib/anc/EDITION/runtime`
    pub fn get_runtime_directory(&self) -> PathBuf {
        let mut path_buf = PathBuf::from(&self.current_anc_root_path);
        path_buf.push(&self.current_edition);
        path_buf.push(DIRECTORY_NAME_RUNTIME);
        path_buf
    }

    /// Default:
    /// - User: `~/.local/lib/anc/EDITION/runtime/modules`
    /// - Global: `/usr/local/lib/anc/EDITION/runtime/modules`
    /// - System: `/usr/lib/anc/EDITION/runtime/modules`
    pub fn get_builtin_modules_directory(&self) -> PathBuf {
        // let mut path_buf = self.get_runtime_directory();
        // path_buf.push(DIRECTORY_NAME_MODULES);
        // path_buf

        // get the path of runtime executable
        // todo
        todo!()
    }

    /// - User: `~/.local/lib/anc/EDITION/runtime/libraries`
    /// - Global: `/usr/local/lib/anc/EDITION/runtime/libraries`
    /// - System: `/usr/lib/anc/EDITION/runtime/libraries`
    pub fn get_builtin_libraries_directory(&self) -> PathBuf {
        let mut path_buf = self.get_runtime_directory();
        path_buf.push(DIRECTORY_NAME_LIBRARIES);
        path_buf
    }

    /// - User: `~/.local/lib/anc/EDITION/modules`
    /// - Global: `/usr/local/lib/anc/EDITION/modules`
    /// - System: `/usr/lib/anc/EDITION/modules`
    pub fn get_modules_directory(&self) -> PathBuf {
        let mut path_buf = PathBuf::from(&self.current_anc_root_path);
        path_buf.push(&self.current_edition);
        path_buf.push(DIRECTORY_NAME_MODULES);
        path_buf
    }

    /// - User: `~/.local/lib/anc/EDITION/libraries`
    /// - Global: `/usr/local/lib/anc/EDITION/libraries`
    /// - System: `/usr/lib/anc/EDITION/libraries`
    pub fn get_libraries_directory(&self) -> PathBuf {
        let mut path_buf = PathBuf::from(&self.current_anc_root_path);
        path_buf.push(&self.current_edition);
        path_buf.push(DIRECTORY_NAME_MODULES);
        path_buf
    }
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
            .map(|item| item.file_path.strip_prefix(src_path).unwrap())
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
