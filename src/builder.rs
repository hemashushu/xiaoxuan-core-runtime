// Copyright (c) 2025 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE, LICENSE.additional and CONTRIBUTING.

use std::{
    fs::File,
    path::{Path, PathBuf},
};

use anc_assembler::assembler::assemble_module_node;
use anc_image::{
    entry::{ExternalLibraryEntry, ImageCommonEntry, ImageIndexEntry, ImportModuleEntry},
    entry_reader::read_object_file,
    entry_writer::{write_image_file, write_object_file},
};
use anc_linker::{
    indexer::{build_indices, sort_modules},
    linker::link_modules,
};
use anc_parser_asm::{parser::parse_from_str, NAME_PATH_SEPARATOR};

use crate::{
    common::{
        get_dependencies, get_file_mata_path, get_file_mata_path_by_full_name, get_file_meta,
        get_file_timestamp, get_module_config_file, get_output_asset_path, get_output_object_path,
        get_output_path, get_output_shared_module_file, get_src_path, list_assembly_files,
        list_object_files, load_module_config, FileMeta, PathWithTimestamp,
    },
    RuntimeError, MODULE_CONFIG_FILE_NAME,
};

/// Compile the specified module and generate the module image file.
/// The last modification time of source files is checked and no
/// module image is generated if all source files remain unchanged.
pub fn rebuild_module(module_path: &Path) -> Result<PathBuf, RuntimeError> {
    let output_path = get_output_path(module_path, None);

    let module_config_file_path = get_module_config_file(module_path);
    let module_config = load_module_config(&module_config_file_path)?;
    let shared_module_file_full_path =
        get_output_shared_module_file(&output_path, &module_config.name);
    let (import_module_entries, external_library_entries) = get_dependencies(&module_config);

    // check module configuration file.
    // always re-compile/assemble when configuration changed
    let config_changed = {
        let current_timestamp_opt = get_file_timestamp(&module_config_file_path)?;

        let output_asset_path = get_output_asset_path(&output_path);
        let meta_file_path = get_file_mata_path(&output_asset_path, MODULE_CONFIG_FILE_NAME);
        let file_meta_opt = get_file_meta(&meta_file_path)?;

        if let Some(file_meta) = file_meta_opt {
            if let Some(last_timestamp) = file_meta.timestamp {
                if let Some(current_timestamp) = current_timestamp_opt {
                    current_timestamp > last_timestamp
                } else {
                    true
                }
            } else {
                true
            }
        } else {
            true
        }
    };

    // check assembly files
    let mut pending_assemble_files = vec![];
    {
        let output_object_path = get_output_object_path(&output_path);
        let src_path = get_src_path(module_path);
        let src_file_path_and_timestamps = list_assembly_files(&src_path)?;

        for PathWithTimestamp {
            path_buf: file_path,
            timestamp: current_timestamp_opt,
        } in src_file_path_and_timestamps
        {
            // gets the relative path of source file,
            // and converts it into a file name, e.g.
            // - source: "/home/yang/projects/helloworld/src/network/http/get.anca"
            // - relative path: "network/http/get.anca"
            // - canonical name: "network-http-get.anca"
            let relative_path = file_path.strip_prefix(&src_path).unwrap();
            let name_parts = relative_path
                .components()
                .map(|comp| comp.as_os_str().to_str().unwrap())
                .collect::<Vec<_>>();
            let canonical_name = name_parts.join("-");
            let submodule_relative_name_path = name_parts.join(NAME_PATH_SEPARATOR);
            let meta_file_path = get_file_mata_path(&output_object_path, &canonical_name);
            let file_meta_opt = get_file_meta(&meta_file_path)?;

            let changed = config_changed // always re-assemble when configuration changed
                || if let Some(file_meta) = file_meta_opt {
                    if let Some(last_timestamp) = file_meta.timestamp {
                        if let Some(current_timestamp) = current_timestamp_opt {
                            current_timestamp > last_timestamp
                        } else {
                            true
                        }
                    } else {
                        true
                    }
                } else {
                    true
                };

            if changed {
                pending_assemble_files.push((
                    file_path,
                    meta_file_path,
                    submodule_relative_name_path,
                    current_timestamp_opt,
                ));
            }
        }
    }

    // re-assemble
    if !pending_assemble_files.is_empty() {
        let output_object_path = get_output_object_path(&output_path);
        std::fs::create_dir_all(&output_object_path)
            .map_err(|e| RuntimeError::Message(format!("{}", e)))?;

        // assemble
        for (src_path_buf, dst_path_buf, submodule_relative_name_path, timestamp_opt) in
            pending_assemble_files
        {
            let submodule_full_name = if submodule_relative_name_path == "lib.anca"
                || submodule_relative_name_path == "main.anca"
            {
                module_config.name.clone()
            } else {
                format!("{}::{submodule_relative_name_path}", &module_config.name)
            };

            let image_common_entry = assemble(
                &import_module_entries,
                &external_library_entries,
                &submodule_full_name,
                &src_path_buf,
            )?;

            save_object_file(&image_common_entry, &dst_path_buf)?;
            save_object_file_meta(
                timestamp_opt,
                &get_file_mata_path_by_full_name(&dst_path_buf),
            )?;
        }

        // link
        let object_files = list_object_files(&output_object_path)?;

        let mut image_binaries = vec![];
        let mut image_common_entries = vec![];

        for object_file in object_files {
            let image_binary =
                std::fs::read(&object_file).map_err(|e| RuntimeError::Message(format!("{}", e)))?;
            image_binaries.push(image_binary);
        }

        for image_binary in &image_binaries {
            let image_common_entry = read_object_file(image_binary)
                .map_err(|e| RuntimeError::Message(format!("{}", e)))?;
            image_common_entries.push(image_common_entry);
        }

        let linked_image_common_entry = link(&module_config.name, &image_common_entries)?;

        save_shared_module_file(&linked_image_common_entry, &shared_module_file_full_path)?;
    }

    Ok(shared_module_file_full_path)
}

fn assemble(
    import_module_entries: &[ImportModuleEntry],
    external_library_entries: &[ExternalLibraryEntry],
    submodule_full_name: &str,
    assembly_file_path: &Path,
) -> Result<ImageCommonEntry, RuntimeError> {
    let source_code = std::fs::read_to_string(assembly_file_path)
        .map_err(|e| RuntimeError::Message(format!("{}", e)))?;

    let module_node = parse_from_str(&source_code)
        .map_err(|e| RuntimeError::Message(e.with_source(&source_code)))?;

    assemble_module_node(
        &module_node,
        submodule_full_name,
        import_module_entries,
        external_library_entries,
    )
    .map_err(|e| RuntimeError::Message(format!("{}", e)))
}

fn save_object_file(
    image_common_entry: &ImageCommonEntry,
    object_file_full_path: &Path,
) -> Result<(), RuntimeError> {
    let mut file = File::create_new(object_file_full_path)
        .map_err(|e| RuntimeError::Message(format!("{}", e)))?;

    write_object_file(image_common_entry, false, &mut file)
        .map_err(|e| RuntimeError::Message(format!("{}", e)))
}

fn save_object_file_meta(
    timestamp_opt: Option<u64>,
    object_file_meta_full_path: &Path,
) -> Result<(), RuntimeError> {
    let file_meta = FileMeta {
        timestamp: timestamp_opt,
        dependencies: vec![],
    };

    let mut meta_file = File::create(object_file_meta_full_path)
        .map_err(|e| RuntimeError::Message(format!("{}", e)))?;

    ason::to_writer(&file_meta, &mut meta_file).map_err(|e| RuntimeError::Message(format!("{}", e)))
}

fn link(
    target_module_name: &str,
    submodule_entries: &[ImageCommonEntry],
) -> Result<ImageCommonEntry, RuntimeError> {
    link_modules(target_module_name, true, submodule_entries)
        .map_err(|e| RuntimeError::Message(format!("{}", e)))
}

fn save_shared_module_file(
    image_common_entry: &ImageCommonEntry,
    shared_module_file_full_path: &Path,
) -> Result<(), RuntimeError> {
    let mut file = File::create_new(shared_module_file_full_path)
        .map_err(|e| RuntimeError::Message(format!("{}", e)))?;

    write_object_file(image_common_entry, true, &mut file)
        .map_err(|e| RuntimeError::Message(format!("{}", e)))
}

/**
 * image_common_entries: Unsorted image common entries.
 */
fn generate(
    image_common_entries: &mut [ImageCommonEntry],
) -> Result<ImageIndexEntry, RuntimeError> {
    let module_entries = sort_modules(image_common_entries);
    build_indices(&image_common_entries, &module_entries)
        .map_err(|e| RuntimeError::Message(format!("{}", e)))
}

fn save_application_image_file(
    image_common_entry: &ImageCommonEntry,
    image_index_entry: &ImageIndexEntry,
    application_image_file_full_path: &Path,
) -> Result<(), RuntimeError> {
    let mut file = File::create_new(application_image_file_full_path)
        .map_err(|e| RuntimeError::Message(format!("{}", e)))?;

    write_image_file(image_common_entry, image_index_entry, &mut file)
        .map_err(|e| RuntimeError::Message(format!("{}", e)))
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::rebuild_module;

    fn get_resources_path_buf() -> PathBuf {
        // returns the project's root folder
        let mut pwd = std::env::current_dir().unwrap();
        // append subfolders
        pwd.push("tests");
        pwd.push("resources");
        pwd
    }

    #[test]
    fn test_rebuild_module() {
        let mut moudle_path_buf = get_resources_path_buf();
        moudle_path_buf.push("single-module-app");

        let a = rebuild_module(&moudle_path_buf).unwrap();
        println!("{:?}", a);
    }
}
