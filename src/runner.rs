// Copyright (c) 2025 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE, LICENSE.additional and CONTRIBUTING.

use std::{collections::HashMap, sync::Mutex};

use anc_context::{
    external_function_table::ExternalFunctionTable, process_config::ProcessConfig,
    process_context::ProcessContext, process_resource::ProcessResource,
};
use anc_image::ImageError;
use anc_processor::{multithread_process::start_program, GenericError};

pub struct MappedFileProcessResource {
    process_config: ProcessConfig,
    external_function_table: Mutex<ExternalFunctionTable>,
}

impl MappedFileProcessResource {
    pub fn new(process_config: ProcessConfig) -> Self {
        Self {
            process_config,
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

pub fn launch(
    application_path: &str,

    // the entry point names:
    // - empty string for the default entry point (in file "main.anca").
    // - submodule name for the executable units in the "app" folder.
    // - submodule and function name ("test_*") for the unit test (in folder "tests").
    entry_point_name: &str,

    // program arguments
    arguments: Vec<String>,

    // environment variables
    environments: HashMap<String, String>,
) -> Result<u32, GenericError> {
    let process_config = ProcessConfig {
        application_path: application_path.to_owned(),
        is_script: false,
        arguments,
        environments,
    };
    let resource = MappedFileProcessResource::new(process_config);
    let process_context = resource.create_process_context()?;
    start_program(&process_context, entry_point_name, vec![])
}
