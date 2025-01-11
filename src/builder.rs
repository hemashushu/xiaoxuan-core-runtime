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
    entry::{ExternalLibraryEntry, ImageCommonEntry, ImportModuleEntry},
    entry_writer::write_object_file,
    DependencyHash,
};
use anc_linker::linker::link_modules;
use anc_parser_asm::parser::parse_from_str;

use crate::{RuntimeError, DIRECTORY_NAME_OBJECT, DIRECTORY_NAME_OUTPUT};

pub fn get_output_path(module_path: &str, hash_opt: Option<&DependencyHash>) -> PathBuf {
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

pub fn get_objects_path(output_path: &Path) -> PathBuf {
    let mut path_buf = PathBuf::from(output_path);
    path_buf.push(DIRECTORY_NAME_OBJECT);
    path_buf
}

pub fn assemble(
    import_module_entries: &[ImportModuleEntry],
    external_library_entries: &[ExternalLibraryEntry],
    submodule_full_name: &str,
    assembly_file_path: &str,
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

pub fn save_object_file(
    image_common_entry: &ImageCommonEntry,
    object_file_full_path: &Path,
) -> Result<(), RuntimeError> {
    let mut file = File::create_new(object_file_full_path)
        .map_err(|e| RuntimeError::Message(format!("{}", e)))?;

    write_object_file(image_common_entry, false, &mut file)
        .map_err(|e| RuntimeError::Message(format!("{}", e)))
}

pub fn link(
    target_module_name: &str,
    submodule_entries: &[ImageCommonEntry],
) -> Result<ImageCommonEntry, RuntimeError> {
    link_modules(target_module_name, true, submodule_entries)
        .map_err(|e| RuntimeError::Message(format!("{}", e)))
}

pub fn save_shared_module_file(
    image_common_entry: &ImageCommonEntry,
    shared_module_file_full_path: &Path,
) -> Result<(), RuntimeError> {
    let mut file = File::create_new(shared_module_file_full_path)
        .map_err(|e| RuntimeError::Message(format!("{}", e)))?;

    write_object_file(image_common_entry, true, &mut file)
        .map_err(|e| RuntimeError::Message(format!("{}", e)))
}

#[cfg(test)]
mod tests {}
