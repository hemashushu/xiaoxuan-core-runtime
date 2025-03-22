// Copyright (c) 2025 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE, LICENSE.additional and CONTRIBUTING.

use std::{collections::HashMap, fs::File, io::Write, path::Path, sync::Mutex};

use anc_context::{
    external_function_table::ExternalFunctionTable, process_context::ProcessContext,
    process_property::ProcessProperty, process_resource::ProcessResource,
};
use anc_image::{
    entry::{EntryPointEntry, ImageIndexEntry},
    module_image::ModuleImage,
    ImageError,
};
use anc_isa::ModuleDependencyType;
use anc_linker::DEFAULT_ENTRY_FUNCTION_NAME;
use anc_processor::{multithread_process::start_program, GenericError};
use memmap2::Mmap;

use crate::{
    builder::{build_application_by_dependency_tree, build_application_by_single_file},
    entry::RuntimeProperty,
    locations::get_shared_module_image_file_path_by_dynamic_link_module_entry,
    RuntimeError,
};

pub const EXECUTABLE_UNIT_NAME_SEPARATOR: &str = ":";

pub fn launch_application(
    module_path: &Path,

    // executable_unit_name:
    // - "" (empty string)
    //   executes function 'app_module_name::_start', the default entry point.
    //   internal entry point name is "_start".
    //
    // - ":{submodule_name}"
    //   executes 'app_module_name::app::{submodule_name}::_start', the additional executable units.
    //   internal entry point name is the name of submodule.
    executable_unit_name: &str,
    arguments: Vec<String>,                // program arguments
    environments: HashMap<String, String>, // environment variables
    extra_registries: Vec<String>,
) -> Result<u32, GenericError> {
    let runtime_property = RuntimeProperty::from_runtime_exec_file(&extra_registries)?;

    let runtime_home = &runtime_property.runtime_home;
    if !runtime_home.exists() {
        std::fs::create_dir_all(runtime_home).unwrap();
    }

    let (image_files, _entry_point_entries) =
        load_application(module_path, &runtime_property, false)?;

    // create process

    let process_property = ProcessProperty {
        application_path: module_path.to_path_buf(),
        is_script: false,
        arguments,
        environments,
    };

    let entry_point_name = if executable_unit_name.is_empty() {
        DEFAULT_ENTRY_FUNCTION_NAME.to_owned()
    } else if let Some(name) = executable_unit_name.strip_prefix(EXECUTABLE_UNIT_NAME_SEPARATOR) {
        name.to_owned()
    } else {
        return Err(Box::new(RuntimeError::Message(
            "Incorrect entry point name.".to_owned(),
        )));
    };

    execute_unit(&image_files, &entry_point_name, process_property)
}

pub fn launch_unit_tests(
    module_path: &Path,

    // matching any function with the specified prefix path name, e.g.
    // - "{submodule_name}::" matches functions "app_module_name::tests::{submodule_name}::test_*".
    // - "{submodule_name}::test_abc*" matches functions "app_module_name::tests::{submodule_name}::test_abc*".
    //
    // note that the internal entry point name is "{submodule_name}::test_*".
    unit_test_name_path_prefix: &str,

    arguments: Vec<String>,                // program arguments
    environments: HashMap<String, String>, // environment variables
    extra_registries: Vec<String>,
    logger: &mut dyn Write,
) -> Result<Vec<UnitTestResult>, GenericError> {
    let runtime_property = RuntimeProperty::from_runtime_exec_file(&extra_registries)?;

    let runtime_home = &runtime_property.runtime_home;
    if !runtime_home.exists() {
        std::fs::create_dir_all(runtime_home).unwrap();
    }

    let (image_files, entry_point_entries) =
        load_application(module_path, &runtime_property, true)?;

    let process_property = ProcessProperty {
        application_path: module_path.to_path_buf(),
        is_script: false,
        arguments,
        environments,
    };

    // unit test
    let mut unit_test_results = vec![];

    let count = entry_point_entries
        .iter()
        .filter(|entry_point_entry| {
            entry_point_entry
                .unit_name
                .starts_with(unit_test_name_path_prefix)
        })
        .count();

    writeln!(logger, "running {count} tests")?;

    for entry_point_entry in &entry_point_entries {
        let entry_point_name = &entry_point_entry.unit_name;
        if entry_point_name.starts_with(unit_test_name_path_prefix) {
            write!(logger, "testing {entry_point_name} ... ")?;

            let result = execute_unit(&image_files, entry_point_name, process_property.clone())?;
            let success = result == 0;
            writeln!(logger, "{}", if success { "ok" } else { "FAILED" })?;

            unit_test_results.push(UnitTestResult {
                name: entry_point_name.to_owned(),
                success,
            });
        }
    }

    Ok(unit_test_results)
}

#[derive(Debug, PartialEq)]
pub struct UnitTestResult {
    pub name: String,
    pub success: bool,
}

impl UnitTestResult {
    pub fn new(name: String, success: bool) -> Self {
        Self { name, success }
    }
}

pub fn launch_single_file_application(
    script_file_path: &Path,
    arguments: Vec<String>,                // program arguments
    environments: HashMap<String, String>, // environment variables
    extra_registries: Vec<String>,
) -> Result<u32, GenericError> {
    let runtime_property = RuntimeProperty::from_runtime_exec_file(&extra_registries)?;

    let runtime_home = &runtime_property.runtime_home;
    if !runtime_home.exists() {
        std::fs::create_dir_all(runtime_home).unwrap();
    }

    let (main_image_data, image_files, _entry_point_entries) =
        load_single_file_application(script_file_path, &runtime_property)?;

    // create process

    let process_property = ProcessProperty {
        application_path: script_file_path.to_path_buf(),
        is_script: true,
        arguments,
        environments,
    };

    execute_script(main_image_data, &image_files, process_property)
}

fn execute_unit(
    image_files: &[File],

    // entry point names:
    // - "_start"
    //   executes function 'app_module_name::_start', the default entry point.
    //   public executable unit name is: "" (empty string).
    //
    // - "submodule_name"
    //   executes function 'app_module_name::app::{submodule_name}::_start', the additional executable units.
    //   public executable unit name is: ".{submodule_name}"
    //
    // - "submodule_name::test_*"
    //   executes function 'app_module_name::tests::{submodule_name}::test_*' for unit test.
    //   public executable unit name is: matching any prefix of "submodule_name::test_*"
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

fn execute_script(
    main_image_data: Vec<u8>,
    image_files: &[File],
    process_property: ProcessProperty,
) -> Result<u32, GenericError> {
    let mut mapped_files = vec![];

    for image_file in image_files {
        let mmap = unsafe { Mmap::map(image_file).expect("Failed to map the image file.") };
        mapped_files.push(mmap);
    }

    let resource = ScriptFileProcessResource::new(main_image_data, mapped_files, process_property);
    let process_context = resource.create_process_context()?;
    start_program(&process_context, DEFAULT_ENTRY_FUNCTION_NAME, vec![])
}

fn load_application(
    module_path: &Path,
    runtime_property: &RuntimeProperty,
    include_unit_tests: bool,
) -> Result<(Vec<File>, Vec<EntryPointEntry>), RuntimeError> {
    let (_, index_entry, application_image_file_full_path) = build_application_by_dependency_tree(
        module_path,
        ModuleDependencyType::Local,
        runtime_property,
        include_unit_tests,
    )?;

    let mut image_file_paths = vec![application_image_file_full_path];

    for dynamic_link_module_entry in &index_entry.dynamic_link_module_entries[1..] {
        let image_file_path = get_shared_module_image_file_path_by_dynamic_link_module_entry(
            dynamic_link_module_entry,
            runtime_property,
        );
        image_file_paths.push(image_file_path);
    }

    let mut image_files = vec![];
    for path_buf in &image_file_paths {
        let file = File::open(path_buf).map_err(|e| RuntimeError::Message(e.to_string()))?;
        image_files.push(file);
    }

    let ImageIndexEntry {
        entry_point_entries,
        ..
    } = index_entry;

    Ok((image_files, entry_point_entries))
}

fn load_single_file_application(
    script_file_path: &Path,
    runtime_property: &RuntimeProperty,
) -> Result<(Vec<u8>, Vec<File>, Vec<EntryPointEntry>), RuntimeError> {
    let (_, index_entry, main_image_data) =
        build_application_by_single_file(script_file_path, runtime_property)?;

    let mut image_file_paths = vec![];

    for dynamic_link_module_entry in &index_entry.dynamic_link_module_entries[1..] {
        let image_file_path = get_shared_module_image_file_path_by_dynamic_link_module_entry(
            dynamic_link_module_entry,
            runtime_property,
        );
        image_file_paths.push(image_file_path);
    }

    let mut image_files = vec![];
    for path_buf in &image_file_paths {
        let file = File::open(path_buf).map_err(|e| RuntimeError::Message(e.to_string()))?;
        image_files.push(file);
    }

    let ImageIndexEntry {
        entry_point_entries,
        ..
    } = index_entry;

    Ok((main_image_data, image_files, entry_point_entries))
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

pub struct ScriptFileProcessResource {
    main_image_data: Vec<u8>,
    mapped_files: Vec<Mmap>,
    process_property: ProcessProperty,
    external_function_table: Mutex<ExternalFunctionTable>,
}

impl ScriptFileProcessResource {
    pub fn new(
        main_image_data: Vec<u8>,
        mapped_files: Vec<Mmap>,
        process_property: ProcessProperty,
    ) -> Self {
        Self {
            main_image_data,
            mapped_files,
            process_property,
            external_function_table: Mutex::new(ExternalFunctionTable::default()),
        }
    }
}

impl ProcessResource for ScriptFileProcessResource {
    fn create_process_context(&self) -> Result<ProcessContext, ImageError> {
        let mut module_images = vec![];

        let main_module_image = ModuleImage::read(&self.main_image_data)?;
        module_images.push(main_module_image);

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

    use pretty_assertions::assert_eq;

    use crate::runner::{
        launch_application, launch_single_file_application, launch_unit_tests, UnitTestResult,
    };

    fn get_resources_path_buf() -> PathBuf {
        // `std::env::current_dir()` returns the current Rust project's root folder
        let mut pwd = std::env::current_dir().unwrap();
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
                vec![],
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
                vec![],
            );

            assert_eq!(result0.unwrap(), 2);

            let result1 = launch_application(
                &moudle_path_buf,
                ":foo",
                vec![],
                HashMap::<String, String>::new(),
                vec![],
            );

            assert_eq!(result1.unwrap(), 3);

            let result2 = launch_application(
                &moudle_path_buf,
                ":bar",
                vec![],
                HashMap::<String, String>::new(),
                vec![],
            );

            assert_eq!(result2.unwrap(), 5);
        }

        // multiple_module_app
        {
            let mut moudle_path_buf = get_resources_path_buf();
            moudle_path_buf.push("multiple_module_app");
            moudle_path_buf.push("cli");

            let result0 = launch_application(
                &moudle_path_buf,
                "",
                vec![],
                HashMap::<String, String>::new(),
                vec![],
            );

            assert_eq!(result0.unwrap(), 11);
        }
    }

    #[test]
    fn test_launch_unit_tests() {
        let mut output: Vec<u8> = vec![];
        // let mut output = stdout();

        // single_module_with_unit_tests - without specify testing name
        {
            let mut moudle_path_buf = get_resources_path_buf();
            moudle_path_buf.push("single_module_with_unit_tests");

            let result0 = launch_unit_tests(
                &moudle_path_buf,
                "",
                vec![],
                HashMap::<String, String>::new(),
                vec![],
                &mut output,
            );

            assert_eq!(
                result0.unwrap(),
                vec![
                    UnitTestResult::new("foo::test_two".to_owned(), true),
                    UnitTestResult::new("foo::test_three".to_owned(), true),
                    UnitTestResult::new("bar::test_five".to_owned(), true),
                    UnitTestResult::new("bar::test_five_failed".to_owned(), false),
                ]
            )
        }

        // single_module_with_unit_tests - specify testing name
        {
            let mut moudle_path_buf = get_resources_path_buf();
            moudle_path_buf.push("single_module_with_unit_tests");

            let result0 = launch_unit_tests(
                &moudle_path_buf,
                "foo::",
                vec![],
                HashMap::<String, String>::new(),
                vec![],
                &mut output,
            );

            assert_eq!(
                result0.unwrap(),
                vec![
                    UnitTestResult::new("foo::test_two".to_owned(), true),
                    UnitTestResult::new("foo::test_three".to_owned(), true),
                ]
            )
        }
    }

    #[test]
    fn test_launch_script_application() {
        // no config
        {
            let mut script_file_path_buf = get_resources_path_buf();
            script_file_path_buf.push("single_file_app");
            script_file_path_buf.push("no_conf.anca");

            let result0 = launch_single_file_application(
                &script_file_path_buf,
                vec![],
                HashMap::<String, String>::new(),
                vec![],
            );

            assert_eq!(result0.unwrap(), 13);
        }

        // with config
        {
            let mut script_file_path_buf = get_resources_path_buf();
            script_file_path_buf.push("single_file_app");
            script_file_path_buf.push("with_conf.anca");

            let result0 = launch_single_file_application(
                &script_file_path_buf,
                vec![],
                HashMap::<String, String>::new(),
                vec![],
            );

            assert_eq!(result0.unwrap(), 11);
        }
    }
}
