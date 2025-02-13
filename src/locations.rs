// Copyright (c) 2025 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE, LICENSE.additional and CONTRIBUTING.

use std::path::{Path, PathBuf};

use anc_image::{
    entry::{DynamicLinkModuleEntry, ModuleLocation},
    format_dependency_hash, DependencyHash,
};

use crate::{
    entry::RuntimeProperty, DIRECTORY_NAME_APP, DIRECTORY_NAME_ASSEMBLY, DIRECTORY_NAME_ASSET,
    DIRECTORY_NAME_IR, DIRECTORY_NAME_OBJECT, DIRECTORY_NAME_OUTPUT, DIRECTORY_NAME_SRC,
    DIRECTORY_NAME_TESTS, DIRECTORY_NAME_VERSION_REMOTE, FILE_EXTENSION_ASSEMBLY,
    FILE_EXTENSION_IMAGE, FILE_EXTENSION_IR, FILE_EXTENSION_META, FILE_EXTENSION_MODULE,
    FILE_EXTENSION_OBJECT, FILE_NAME_MODULE_CONFIG,
};

pub fn get_shared_module_image_file_path_by_dynamic_link_module_entry(
    dynamic_link_module_entry: &DynamicLinkModuleEntry,
    runtime_property: &RuntimeProperty,
) -> PathBuf {
    let module_name = &dynamic_link_module_entry.name;
    let module_location = dynamic_link_module_entry.module_location.as_ref();

    match module_location {
        ModuleLocation::Local(location_local) => {
            // {module_path}/output/{hash}/{name}.ancm
            let mut path_buf = PathBuf::from(&location_local.module_path);
            path_buf.push(DIRECTORY_NAME_OUTPUT);
            path_buf.push(&location_local.hash);
            get_shared_module_image_file_path_by_hash_path(&path_buf, module_name)
        }
        ModuleLocation::Remote(location_remote) => {
            let mut path_buf = runtime_property.get_modules_directory();
            path_buf.push(module_name);
            path_buf.push(DIRECTORY_NAME_VERSION_REMOTE);
            path_buf.push(DIRECTORY_NAME_OUTPUT);
            path_buf.push(&location_remote.hash);
            get_shared_module_image_file_path_by_hash_path(&path_buf, module_name)
        }
        ModuleLocation::Share(location_share) => {
            let mut path_buf = runtime_property.get_modules_directory();
            path_buf.push(module_name);
            path_buf.push(&location_share.version);
            path_buf.push(DIRECTORY_NAME_OUTPUT);
            path_buf.push(&location_share.hash);
            get_shared_module_image_file_path_by_hash_path(&path_buf, module_name)
        }
        ModuleLocation::Runtime => {
            let mut path_buf = runtime_property.get_builtin_modules_directory();
            path_buf.push(module_name);
            path_buf.push(DIRECTORY_NAME_OUTPUT);

            let hash_path_buf = get_output_folder_hash_path(&path_buf, None);
            get_shared_module_image_file_path_by_hash_path(&hash_path_buf, module_name)
        }
        ModuleLocation::Embed => unreachable!(),
    }
}

/// Returns `{hash_path}/{name}.ancm`
pub fn get_shared_module_image_file_path_by_hash_path(
    hash_path: &Path,
    module_name: &str,
) -> PathBuf {
    let mut path_buf = PathBuf::from(hash_path);
    path_buf.push(module_name);
    path_buf.set_extension(FILE_EXTENSION_MODULE);
    path_buf
}

/// Returns `{module_folder}/output/name.anci`
pub fn get_application_module_image_file_path_by_output_path(
    output_path: &Path,
    module_name: &str,
) -> PathBuf {
    let mut path_buf = PathBuf::from(output_path);
    path_buf.push(module_name);
    path_buf.set_extension(FILE_EXTENSION_IMAGE);
    path_buf
}

/// Returns `{module_folder}/output/name.anci`
pub fn get_application_module_image_file_path(module_path: &Path, module_name: &str) -> PathBuf {
    let mut path_buf = get_module_folder_output_path(module_path);
    path_buf.push(module_name);
    path_buf.set_extension(FILE_EXTENSION_IMAGE);
    path_buf
}

/// Returns `{module_folder}/module.anc.ason`
pub fn get_module_config_file_path(module_path: &Path) -> PathBuf {
    let mut path_buf = PathBuf::from(module_path);
    path_buf.push(FILE_NAME_MODULE_CONFIG);
    path_buf
}

/// `{module_folder}/src`
pub fn get_module_folder_src_path(module_path: &Path) -> PathBuf {
    let mut path_buf = PathBuf::from(module_path);
    path_buf.push(DIRECTORY_NAME_SRC);
    path_buf
}

/// `{module_folder}/app`
pub fn get_module_folder_app_path(module_path: &Path) -> PathBuf {
    let mut path_buf = PathBuf::from(module_path);
    path_buf.push(DIRECTORY_NAME_APP);
    path_buf
}

/// `{module_folder}/tests`
pub fn get_module_folder_tests_path(module_path: &Path) -> PathBuf {
    let mut path_buf = PathBuf::from(module_path);
    path_buf.push(DIRECTORY_NAME_TESTS);
    path_buf
}

/// `{module_folder}/output`
pub fn get_module_folder_output_path(module_path: &Path) -> PathBuf {
    let mut path_buf = PathBuf::from(module_path);
    path_buf.push(DIRECTORY_NAME_OUTPUT);
    path_buf
}

/// - `{module_folder}/output/{hash}`
/// - `{module_folder}/output`
pub fn get_output_folder_hash_path(
    output_path: &Path,
    hash_opt: Option<&DependencyHash>,
) -> PathBuf {
    if let Some(hash) = hash_opt {
        let hash_string = format_dependency_hash(hash);
        let mut path_buf = PathBuf::from(output_path);
        path_buf.push(hash_string);
        path_buf
    } else {
        output_path.to_path_buf()
    }
}

/// `{module_folder}/output/{hash}/asset`
pub fn get_hash_folder_asset_path(hash_path: &Path) -> PathBuf {
    let mut path_buf = PathBuf::from(hash_path);
    path_buf.push(DIRECTORY_NAME_ASSET);
    path_buf
}

/// `{module_folder}/output/{hash}/asset/ir`
pub fn get_asset_folder_ir_path(asset_path: &Path) -> PathBuf {
    let mut path_buf = PathBuf::from(asset_path);
    path_buf.push(DIRECTORY_NAME_IR);
    path_buf
}

/// `{module_folder}/output/{hash}/asset/assembly`
pub fn get_asset_folder_assembly_path(asset_path: &Path) -> PathBuf {
    let mut path_buf = PathBuf::from(asset_path);
    path_buf.push(DIRECTORY_NAME_ASSEMBLY);
    path_buf
}

/// `{module_folder}/output/{hash}/asset/object`
pub fn get_asset_folder_object_path(asset_path: &Path) -> PathBuf {
    let mut path_buf = PathBuf::from(asset_path);
    path_buf.push(DIRECTORY_NAME_OBJECT);
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

/// Replace the extension name of the specified file to ".meta.ason",
/// e.g.
/// "something.anco" -> "something.meta.ason"
pub fn get_mata_file_path(directory: &Path, file_name: &str) -> PathBuf {
    let mut meta_file_path_buf = PathBuf::from(directory);
    meta_file_path_buf.push(file_name);
    meta_file_path_buf.set_extension(FILE_EXTENSION_META);
    meta_file_path_buf
}

/// Replace the extension name of file to ".meta.ason",
/// e.g.
/// "something.anco" -> "something.meta.ason"
pub fn get_mata_file_path_by_full_name(full_path: &Path) -> PathBuf {
    let mut meta_file_path_buf = PathBuf::from(full_path);
    meta_file_path_buf.set_extension(FILE_EXTENSION_META);
    meta_file_path_buf
}
