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
        get_app_path, get_asset_assembly_path, get_asset_ir_path, get_asset_object_path,
        get_dependencies, get_file_timestamp, get_hash_asset_path, get_mata_file_path,
        get_mata_file_path_by_full_name, get_module_config_file_path, get_object_file_path,
        get_output_hash_path, get_output_path, get_shared_module_file_path, get_src_path,
        get_tests_path, list_assembly_files, list_object_files, load_file_meta, load_module_config,
        FileMeta, PathWithTimestamp,
    },
    RuntimeError, FILE_EXTENSION_OBJECT, MODULE_CONFIG_FILE_NAME,
};

struct PendingItem {
    // the path of source file (*.anc, *.ancr, and *.anca)
    source_path_buf: PathBuf,
    meta_file_path: PathBuf,
    canonical_name: String,
    submodule_name_path: String,

    // the timestamp of source file (*.anc, *.ancr, and *.anca),
    // it is NOT timestamp of generated file (*.ancr and *.anca in the folder "asset").
    timestamp_opt: Option<u64>,
}

/// Used to get the relative path, canonical name, and submodule name path
/// of the source file.
///
/// e.g.
///
/// - source: "/home/yang/projects/helloworld/src/network/http/get.anca"
/// - prefix: "/home/yang/projects/helloworld/src"
/// - relative path: "network/http/get.anca"
/// - name path: "network/http/get"
/// - canonical name: "network-http-get"
/// - submodule name path: "network::http::get"
struct ScanStartItem {
    source_path: PathBuf,
    prefix_path: PathBuf,
}

/// Compile the specified module and generate the module image file.
/// The last modification time of source files is checked and no
/// module image is generated if all source files remain unchanged.
pub fn build_module(
    module_path: &Path,
    include_unit_tests: bool,
    force_rebuild: bool,
) -> Result<Option<ImageCommonEntry>, RuntimeError> {
    // module config
    let module_config_file_path = get_module_config_file_path(module_path);
    let module_config = load_module_config(&module_config_file_path)?;
    let (import_module_entries, external_library_entries) = get_dependencies(&module_config);

    // output folders
    let output_path = get_output_path(module_path);
    let hash_path = get_output_hash_path(&output_path, None);

    // asset folders
    let asset_path = get_hash_asset_path(&hash_path);
    let ir_path = get_asset_ir_path(&asset_path);
    let assembly_path = get_asset_assembly_path(&asset_path);
    let object_path = get_asset_object_path(&asset_path);

    // check module configuration file.
    // always re-compile/assemble when configuration changed
    let module_config_meta_file_path = get_mata_file_path(&asset_path, MODULE_CONFIG_FILE_NAME);

    let (is_module_config_changed, module_config_timestamp_opt) = {
        let current_timestamp_opt = get_file_timestamp(&module_config_file_path)?;
        let module_config_meta_opt = load_file_meta(&module_config_meta_file_path)?;

        let is_module_config_changed = if let Some(file_meta) = module_config_meta_opt {
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

        (is_module_config_changed, current_timestamp_opt)
    };

    let is_rebuild = force_rebuild || is_module_config_changed;

    // the building process
    //
    //      translate   compile        assemble
    // source0 ----> ir0 ----> assembly0 ----> object0--\  link
    // source1 ----> ir1 ----> assembly1 ----> object1--|-------> module
    // source2 ----> ir2 ----> assembly2 ----> object2--/
    //
    // the target of "pending source" file will be appended to the "pending ir",
    // as well as the target of "pending ir" file will be appended to the "pending assembly".
    let mut pending_source_items: Vec<PendingItem> = vec![];
    let mut ir_files: Vec<PathBuf> = vec![];

    let mut pending_ir_items: Vec<PendingItem> = vec![];
    let mut assembly_files: Vec<PathBuf> = vec![];

    let mut pending_assemble_items: Vec<PendingItem> = vec![];
    let mut object_files: Vec<PathBuf> = vec![];

    // check source files
    // todo

    // check ir files
    // todo

    // scan the assembly files
    let mut assembly_scan_start_items: Vec<ScanStartItem> = vec![];

    {
        let src_path = get_src_path(module_path);
        assembly_scan_start_items.push(ScanStartItem {
            source_path: src_path.clone(),
            prefix_path: src_path.clone(),
        });

        let app_path = get_app_path(module_path);
        assembly_scan_start_items.push(ScanStartItem {
            source_path: app_path,
            prefix_path: module_path.to_path_buf(),
        });

        if include_unit_tests {
            let tests_path = get_tests_path(module_path);
            assembly_scan_start_items.push(ScanStartItem {
                source_path: tests_path,
                prefix_path: module_path.to_path_buf(),
            });
        }
    }

    for assembly_scan_start_item in assembly_scan_start_items {
        let source_file_path_and_timestamps =
            list_assembly_files(&assembly_scan_start_item.source_path)?;

        for PathWithTimestamp {
            file_path,
            timestamp: current_timestamp_opt,
        } in source_file_path_and_timestamps
        {
            // gets the relative path, canonical name, and submodule name path
            // of the source file.
            //
            // e.g.
            //
            // - source: "/home/yang/projects/helloworld/src/network/http/get.anca"
            // - prefix: "/home/yang/projects/helloworld/src"
            // - relative path: "network/http/get.anca"
            // - name path: "network/http/get"
            // - canonical name: "network-http-get"
            // - submodule name path: "network::http::get"
            let relative_path = file_path
                .strip_prefix(&assembly_scan_start_item.prefix_path)
                .unwrap();
            let name_path = relative_path.with_extension("");
            let name_parts = name_path
                .components()
                .map(|comp| comp.as_os_str().to_str().unwrap())
                .collect::<Vec<_>>();
            let canonical_name = name_parts.join("-");
            let submodule_name_path = name_parts.join(NAME_PATH_SEPARATOR);

            // check for the existence of object files and meta.
            let object_file_path = get_object_file_path(&object_path, &canonical_name);
            let assembly_meta_file_path = get_mata_file_path_by_full_name(&object_file_path);
            let assembly_meta_opt = load_file_meta(&assembly_meta_file_path)?;

            let is_assembly_file_changed = if let Some(file_meta) = assembly_meta_opt {
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

            let is_object_file_exists = object_file_path.exists();

            if is_rebuild // re-assemble when configuration changed
                || !is_object_file_exists // re-assemble when object file does not exist
                || is_assembly_file_changed
            {
                pending_assemble_items.push(PendingItem {
                    source_path_buf: file_path,
                    meta_file_path: assembly_meta_file_path,
                    canonical_name,
                    submodule_name_path,
                    timestamp_opt: current_timestamp_opt,
                });
            } else {
                object_files.push(object_file_path);
            }
        }
    }

    // re-assemble
    if !pending_assemble_items.is_empty() {
        std::fs::create_dir_all(&object_path)
            .map_err(|e| RuntimeError::Message(format!("{}", e)))?;

        // assemble
        for pending_assemble_item in &pending_assemble_items {
            println!(
                "!! assemble: {}",
                pending_assemble_item.source_path_buf.to_str().unwrap()
            );

            // the top-most submodule: "lib.{anc,ancr,anca}" and "main.{anc,ancr,anca}"
            let submodule_full_name = if pending_assemble_item.submodule_name_path == "lib"
                || pending_assemble_item.submodule_name_path == "main"
            {
                module_config.name.clone()
            } else {
                format!(
                    "{}::{}",
                    module_config.name, pending_assemble_item.submodule_name_path
                )
            };

            // assemble
            let image_common_entry = assemble(
                &import_module_entries,
                &external_library_entries,
                &submodule_full_name,
                &pending_assemble_item.source_path_buf,
            )?;

            let object_file_path =
                get_object_file_path(&object_path, &pending_assemble_item.canonical_name);
            save_object_file(&image_common_entry, &object_file_path)?;

            println!(
                ">> write object file: {}",
                object_file_path.to_str().unwrap()
            );

            object_files.push(object_file_path);

            println!(
                "^^ update assembly meta: {}",
                pending_assemble_item.meta_file_path.to_str().unwrap()
            );

            save_object_file_meta(
                pending_assemble_item.timestamp_opt,
                &pending_assemble_item.meta_file_path,
            )?;
        }
    }

    // link
    let module_entry_opt = if pending_assemble_items.is_empty() {
        None
    } else {
        let mut object_binaries = vec![];
        let mut image_common_entries = vec![];

        for object_file in object_files {
            let image_binary =
                std::fs::read(&object_file).map_err(|e| RuntimeError::Message(format!("{}", e)))?;
            object_binaries.push(image_binary);
        }

        for object_binary in &object_binaries {
            let image_common_entry = read_object_file(object_binary)
                .map_err(|e| RuntimeError::Message(format!("{}", e)))?;
            image_common_entries.push(image_common_entry);
        }

        let shared_module_file_path = get_shared_module_file_path(&hash_path, &module_config.name);

        println!(
            ">> write module binary: {}",
            shared_module_file_path.to_str().unwrap()
        );

        let module_entry = link(&module_config.name, &image_common_entries)?;
        save_shared_module_file(&module_entry, &shared_module_file_path)?;

        Some(module_entry)
    };

    // update config file meta
    if is_module_config_changed {
        println!(
            "^^ update module config meta: {}",
            module_config_meta_file_path.to_str().unwrap()
        );

        std::fs::create_dir_all(&asset_path)
            .map_err(|e| RuntimeError::Message(format!("{}", e)))?;
        save_module_config_file_meta(module_config_timestamp_opt, &module_config_meta_file_path)?;
    }

    Ok(module_entry_opt)
}

fn save_module_config_file_meta(
    timestamp_opt: Option<u64>,
    module_config_file_meta_full_path: &Path,
) -> Result<(), RuntimeError> {
    let file_meta = FileMeta {
        timestamp: timestamp_opt,
        dependencies: vec![],
    };

    let mut meta_file = File::create(module_config_file_meta_full_path)
        .map_err(|e| RuntimeError::Message(format!("{}", e)))?;

    ason::to_writer(&file_meta, &mut meta_file).map_err(|e| RuntimeError::Message(format!("{}", e)))
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
    let mut file =
        File::create(object_file_full_path).map_err(|e| RuntimeError::Message(format!("{}", e)))?;

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
    let mut file = File::create(shared_module_file_full_path)
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
    let mut file = File::create(application_image_file_full_path)
        .map_err(|e| RuntimeError::Message(format!("{}", e)))?;

    write_image_file(image_common_entry, image_index_entry, &mut file)
        .map_err(|e| RuntimeError::Message(format!("{}", e)))
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::build_module;

    fn get_resources_path_buf() -> PathBuf {
        // returns the project's root folder
        let mut pwd = std::env::current_dir().unwrap();
        // append subfolders
        pwd.push("tests");
        pwd.push("resources");
        pwd
    }

    #[test]
    fn test_build_module() {
        // single_module_app
        {
            let mut moudle_path_buf = get_resources_path_buf();
            moudle_path_buf.push("single_module_app");

            // force rebuild
            let result0 = build_module(&moudle_path_buf, false, true);
            assert!(result0.is_ok());
            println!("{:#?}", result0);

            // unchanged
            let result1 = build_module(&moudle_path_buf, false, false);
            assert!(result1.is_ok());
            println!("{:#?}", result1);
        }

        // single_module_app_with_executable_units
        {
            let mut moudle_path_buf = get_resources_path_buf();
            moudle_path_buf.push("single_module_app_with_executable_units");

            // force rebuild
            let result0 = build_module(&moudle_path_buf, false, true);
            assert!(result0.is_ok());
            println!("{:#?}", result0);

            // unchanged
            let result1 = build_module(&moudle_path_buf, false, false);
            assert!(result1.is_ok());
            println!("{:#?}", result1);
        }

        // single_module_with_unit_tests
        {
            let mut moudle_path_buf = get_resources_path_buf();
            moudle_path_buf.push("single_module_with_unit_tests");

            // no unit tests
            let result0 = build_module(&moudle_path_buf, false, true);
            assert!(result0.is_ok());
            println!("{:#?}", result0);

            // includes unit tests
            let result1 = build_module(&moudle_path_buf, true, true);
            assert!(result1.is_ok());
            println!("{:#?}", result1);
        }
    }
}
