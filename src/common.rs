// Copyright (c) 2025 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE, LICENSE.additional and CONTRIBUTING.

use std::{
    collections::VecDeque,
    fs::File,
    path::{Path, PathBuf},
};

use anc_image::entry::{ExternalLibraryEntry, ImportModuleEntry};

use crate::{entry::ModuleConfig, RuntimeError, FILE_EXTENSION_ASSEMBLY, MODULE_CONFIG_FILE_NAME};

pub fn load_module_config(module_path: &Path) -> Result<ModuleConfig, RuntimeError> {
    let mut path_buf = PathBuf::from(module_path);
    path_buf.push(MODULE_CONFIG_FILE_NAME);

    let config_file =
        File::open(path_buf.clone()).map_err(|e| RuntimeError::Message(format!("{}", e)))?;

    ason::from_reader(config_file).map_err(|e| {
        RuntimeError::Message(match std::fs::read_to_string(path_buf) {
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

/**
 * The possible 'start_path' are:
 * - './src'
 * - './app'
 * - './tests'
 * - './output/assembly'
 */
pub fn list_assembly_files(start_path: &Path) -> Result<Vec<PathBuf>, RuntimeError> {
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
            let file_path = entry.path();
            if metadata.is_dir() {
                subfolders.push_front(file_path);
            } else {
                if matches!(
                    file_path.extension().map(|e| e.to_str().unwrap()),
                    Some(FILE_EXTENSION_ASSEMBLY)
                ) {
                    assembly_files.push(file_path);
                }
            }
        }
    }

    Ok(assembly_files)
}

// pub load_images_by_application(module_path:&str) -> Vec<ModuleImage> {
//
// }

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::MODULE_DIRECTORY_NAME_SRC;

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
        moudle_path_buf.push("single-module-app");

        let mut src_path_buf = PathBuf::from(moudle_path_buf);
        src_path_buf.push(MODULE_DIRECTORY_NAME_SRC);

        let src_path = src_path_buf.as_path();
        let assembly_files = list_assembly_files(src_path).unwrap();

        let name_paths = assembly_files
            .iter()
            .map(|item| item.strip_prefix(src_path).unwrap())
            .collect::<Vec<_>>();

        let names = name_paths
            .iter()
            .map(|item| item.to_str().unwrap())
            .collect::<Vec<_>>();

        // the order of list item is variable
        assert!(names.iter().find(|n| **n == "math.anca").is_some());
        assert!(names.iter().find(|n| **n == "base.anca").is_some());
        assert!(names.iter().find(|n| **n == "lib.anca").is_some());
        assert!(names
            .iter()
            .find(|n| **n == "base/primitive.anca")
            .is_some());
    }
}
