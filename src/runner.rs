// Copyright (c) 2025 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE, LICENSE.additional and CONTRIBUTING.

use std::{collections::HashMap, fs::File, path::Path, sync::Mutex};

use anc_context::{
    external_function_table::ExternalFunctionTable, process_context::ProcessContext,
    process_property::ProcessProperty, process_resource::ProcessResource,
};
use anc_image::{
    entry::{EntryPointEntry, ImageIndexEntry},
    module_image::ModuleImage,
    ImageError,
};
use anc_isa::{ModuleDependencyType, RUNTIME_EDITION_STRING};
use anc_linker::DEFAULT_ENTRY_FUNCTION_NAME;
use anc_processor::{multithread_process::start_program, GenericError};
use memmap2::Mmap;

use crate::{
    builder::build_application_by_dependencies,
    common::{get_module_path_by_dependency, RuntimeProperty},
    entry::RuntimeConfig,
    RuntimeError,
};

pub fn launch_application(
    module_path: &Path,

    // entry points:
    // - 'app_module_name::_start' for the default entry point.
    //   internal entry point name is "_start".
    //   public executable unit name is "" (empty string).
    //
    // - 'app_module_name::app::{submodule_name}::_start' for the executable units.
    //   internal entry point name is the name of submodule.
    //   public executable unit name is ".{submodule_name}"
    //
    // - 'app_module_name::tests::{submodule_name}::test_*' for unit tests.
    //   internal entry point name is "submodule_name::test_*".
    //   public executable unit name is "{submodule_name}" or "{submodule_name}::test_{*}"
    executable_unit_name: &str,

    // program arguments
    arguments: Vec<String>,

    // environment variables
    environments: HashMap<String, String>,
) -> Result<u32, GenericError> {
    let runtime_config = RuntimeConfig::load_and_merge_user_config()?;

    let home_path_buf = std::env::home_dir().unwrap();
    let anc_root_path_buf = home_path_buf.join(".local/lib/anc");
    if !anc_root_path_buf.exists() {
        std::fs::create_dir_all(&anc_root_path_buf).unwrap();
    }

    let runtime_property =
        RuntimeProperty::new(anc_root_path_buf, RUNTIME_EDITION_STRING.to_owned());

    let (image_files, _entry_point_entries) =
        load_local_application(module_path, &runtime_property, &runtime_config)?;

    // create process

    let process_property = ProcessProperty {
        application_path: module_path.to_path_buf(),
        is_script: false,
        arguments,
        environments,
    };

    let entry_point_name = if executable_unit_name.is_empty() {
        DEFAULT_ENTRY_FUNCTION_NAME.to_owned()
    } else if executable_unit_name.starts_with('.') {
        executable_unit_name[1..].to_owned()
    } else {
        return Err(Box::new(RuntimeError::Message(
            "Incorrect entry point name.".to_owned(),
        )));
    };

    execute_unit(&image_files, &entry_point_name, process_property)
}

pub fn launch_single_file_application() -> Result<u32, GenericError> {
    todo!()
}

pub fn launch_unit_tests() {
    todo!()
}

fn execute_unit(
    image_files: &[File],

    // entry points:
    // - 'app_module_name::_start' for the default entry point.
    //   internal entry point name is "_start".
    //   public executable unit name is "" (empty string).
    //
    // - 'app_module_name::app::{submodule_name}::_start' for the executable units.
    //   internal entry point name is the name of submodule.
    //   public executable unit name is ".{submodule_name}"
    //
    // - 'app_module_name::tests::{submodule_name}::test_*' for unit tests.
    //   internal entry point name is "submodule_name::test_*".
    //   public executable unit name is "{submodule_name}" or "{submodule_name}::test_{*}"
    entry_point_name: &str,
    process_property: ProcessProperty,
) -> Result<u32, GenericError> {
    let mut mapped_files = vec![];

    for image_file in image_files {
        let mmap = unsafe { Mmap::map(image_file).expect("Failed to map the image file.") };
        mapped_files.push(mmap);
    }

    let resource = MappedFileProcessResource::new(mapped_files, process_property);
    let process_context = resource.create_process_context()?;
    start_program(&process_context, entry_point_name, vec![])
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
) -> Result<(Vec<File>, Vec<EntryPointEntry>), RuntimeError> {
    let (_, index_entry, application_image_file_full_path) = build_application_by_dependencies(
        module_path,
        ModuleDependencyType::Local,
        runtime_property,
        runtime_config,
    )?;

    let mut image_file_paths = vec![application_image_file_full_path];

    for dependent_module_entry in &index_entry.dependent_module_entries[1..] {
        let module_path = get_module_path_by_dependency(
            &dependent_module_entry.name,
            &dependent_module_entry.value,
            runtime_property,
        );
        image_file_paths.push(module_path);
    }

    let mut image_files = vec![];
    for path_buf in &image_file_paths {
        let file = File::open(&path_buf).map_err(|e| RuntimeError::Message(e.to_string()))?;
        image_files.push(file);
    }

    let ImageIndexEntry {
        entry_point_entries,
        ..
    } = index_entry;

    Ok((image_files, entry_point_entries))
}

pub struct MappedFileProcessResource {
    mapped_files: Vec<Mmap>,
    process_property: ProcessProperty,
    external_function_table: Mutex<ExternalFunctionTable>,
}

impl MappedFileProcessResource {
    pub fn new(mapped_files: Vec<Mmap>, process_property: ProcessProperty) -> Self {
        Self {
            mapped_files,
            process_property,
            external_function_table: Mutex::new(ExternalFunctionTable::default()),
        }
    }
}

impl ProcessResource for MappedFileProcessResource {
    fn create_process_context(&self) -> Result<ProcessContext, ImageError> {
        let mut module_images = vec![];

        for file in &self.mapped_files {
            let module_image = ModuleImage::read(file)?;
            module_images.push(module_image);
        }

        let process_context = ProcessContext {
            process_property: &self.process_property,
            module_images,
            external_function_table: &self.external_function_table,
        };

        Ok(process_context)
    }
}

#[cfg(test)]
mod tests {
    use std::{collections::HashMap, path::PathBuf};

    use crate::runner::launch_application;

    fn get_resources_path_buf() -> PathBuf {
        // returns the project's root folder
        let mut pwd = std::env::current_dir().unwrap();
        // append subfolders
        pwd.push("tests");
        pwd.push("resources");
        pwd
    }

    #[test]
    fn test_launch_application() {
        // single_module_app
        {
            let mut moudle_path_buf = get_resources_path_buf();
            moudle_path_buf.push("single_module_app");

            let result0 = launch_application(
                &moudle_path_buf,
                "",
                vec![],
                HashMap::<String, String>::new(),
            );

            assert_eq!(result0.unwrap(), 11);
        }

        // single_module_app_with_executable_units
        {
            let mut moudle_path_buf = get_resources_path_buf();
            moudle_path_buf.push("single_module_app_with_executable_units");

            let result0 = launch_application(
                &moudle_path_buf,
                "",
                vec![],
                HashMap::<String, String>::new(),
            );

            assert_eq!(result0.unwrap(), 2);

            let result1 = launch_application(
                &moudle_path_buf,
                ".foo",
                vec![],
                HashMap::<String, String>::new(),
            );

            assert_eq!(result1.unwrap(), 3);

            let result2 = launch_application(
                &moudle_path_buf,
                ".bar",
                vec![],
                HashMap::<String, String>::new(),
            );

            assert_eq!(result2.unwrap(), 5);
        }
    }
}
