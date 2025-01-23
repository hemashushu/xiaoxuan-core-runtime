// Copyright (c) 2025 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE, LICENSE.additional and CONTRIBUTING.

use std::{collections::HashMap, path::Path, sync::Mutex};

use anc_context::{
    external_function_table::ExternalFunctionTable, process_context::ProcessContext,
    process_property::ProcessProperty, process_resource::ProcessResource,
};
use anc_image::ImageError;
use anc_isa::ModuleDependencyType;
use anc_processor::{multithread_process::start_program, GenericError};

use crate::{
    builder::build_application_by_dependencies, common::RuntimeProperty, entry::RuntimeConfig,
    RuntimeError,
};

pub struct MappedFileProcessResource {
    process_property: ProcessProperty,
    external_function_table: Mutex<ExternalFunctionTable>,
}

impl MappedFileProcessResource {
    pub fn new(process_property: ProcessProperty) -> Self {
        Self {
            process_property,
            external_function_table: Mutex::new(ExternalFunctionTable::default()),
        }
    }
}

impl ProcessResource for MappedFileProcessResource {
    fn create_process_context(&self) -> Result<ProcessContext, ImageError> {
        let process_context = ProcessContext {
            config: todo!(),
            module_images: todo!(),
            external_function_table: todo!(),
        };

        todo!()
    }
}

pub fn launch_application(
    application_path: &str,

    // the entry point names:
    // - empty string for the default entry point (in file "main.anca").
    // - submodule name for the executable units in the "app" folder.
    // - submodule or function name ("test_*") for the unit test (in folder "tests").
    entry_point_name: &str,

    // program arguments
    arguments: Vec<String>,

    // environment variables
    environments: HashMap<String, String>,
) -> Result<u32, GenericError> {
    let process_property = ProcessProperty {
        application_path: application_path.to_owned(),
        is_script: false,
        arguments,
        environments,
    };
    let resource = MappedFileProcessResource::new(process_property);
    let process_context = resource.create_process_context()?;
    start_program(&process_context, entry_point_name, vec![])
}

pub fn launch_single_file_application() -> Result<u32, GenericError> {
    todo!()
}

// Load application image and dependent module images.
fn load_shared_application(name: &str) {
    // todo
}

fn load_builtin_application(name: &str) {
    // todo
}

fn load_remote_application(location: &str) {
    //
}

fn load_local_application(
    module_path: &Path,
    runtime_property: &RuntimeProperty,
    runtime_config: &RuntimeConfig,
) -> Result<(), RuntimeError> {
    let (common_entry, index_entry, application_image_file_full_path) =
        build_application_by_dependencies(
            module_path,
            ModuleDependencyType::Local,
            runtime_property,
            runtime_config,
        )?;

    let dependent_modules =  &index_entry.dependent_module_entries;

    Ok(())
}
