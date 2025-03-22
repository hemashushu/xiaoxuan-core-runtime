// Copyright (c) 2025 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE, LICENSE.additional and CONTRIBUTING.

use std::{
    collections::{HashMap, VecDeque},
    fs::File,
    path::{Path, PathBuf},
};

use anc_assembler::assembler::assemble_module_node;
use anc_image::{
    entry::{
        DynamicLinkModuleEntry, ExternalLibraryEntry, ImageCommonEntry, ImageIndexEntry,
        ImportModuleEntry, ModuleLocation, ModuleLocationLocal, ModuleLocationRemote,
        ModuleLocationShare,
    },
    entry_reader::read_object_file,
    entry_writer::{write_image_file, write_object_file},
    format_dependency_hash, DependencyHash, DEPENDENCY_HASH_ZERO,
};
use anc_isa::{
    EffectiveVersion, ModuleDependency, ModuleDependencyType, VersionCompatibility,
    RUNTIME_EDITION_STRING,
};
use anc_linker::{
    dynamic_linker::{dynamic_link, sort_modules_by_dependent_deepth},
    static_linker::static_link,
};
use anc_parser_asm::{parser::parse_from_str, NAME_PATH_SEPARATOR};
use resolve_path::PathResolveExt;

use crate::{
    entry::{FileMeta, ModuleConfig, RuntimeProperty},
    fetcher::{
        checkout_module, fetch_module, get_shared_module_remote_location,
        RemoteRepositoryResourceLocation,
    },
    locations::{
        get_application_module_image_file_path_by_output_path, get_asset_folder_assembly_path,
        get_asset_folder_ir_path, get_asset_folder_object_path, get_hash_folder_asset_path,
        get_mata_file_path, get_mata_file_path_by_full_name, get_module_config_file_path,
        get_module_folder_app_path, get_module_folder_output_path, get_module_folder_src_path,
        get_module_folder_tests_path, get_object_file_path, get_output_folder_hash_path,
        get_shared_module_image_file_path_by_hash_path,
    },
    peekableiter::PeekableIter,
    source_scanner::{get_file_timestamp, list_assembly_files, PathAndTimestamp},
    RuntimeError, DIRECTORY_NAME_VERSION_REMOTE, FILE_NAME_MODULE_CONFIG,
};

pub const INLINE_CONFIG_MARK: &str = "@config";

/// Compile the specified module and generate the module image file.
/// The last modification time of source files is checked and no
/// module image is generated if all source files remain unchanged.
///
/// This procedure does not generate the application sections.
pub fn build_module(
    module_path: &Path,
    dependency_hash: &DependencyHash,
    include_unit_tests: bool,
) -> Result<Option<ImageCommonEntry>, RuntimeError> {
    // module config
    let module_config_file_path = get_module_config_file_path(module_path);
    let module_config = ModuleConfig::load(&module_config_file_path)?;

    // seal module can not be recompiled
    if module_config.seal {
        return Err(RuntimeError::Message(format!(
            "The module \"{}\"is sealed so it cannot be recompiled.",
            module_path.to_str().unwrap()
        )));
    };

    let (import_module_entries, external_library_entries) =
        module_config.get_dependencies_by_module_config();

    // output folders
    let output_path = get_module_folder_output_path(module_path);
    let hash_path = get_output_folder_hash_path(&output_path, Some(dependency_hash));

    // asset folders
    let asset_path = get_hash_folder_asset_path(&hash_path);
    let _ir_path = get_asset_folder_ir_path(&asset_path);
    let _assembly_path = get_asset_folder_assembly_path(&asset_path);
    let object_path = get_asset_folder_object_path(&asset_path);

    // check module configuration file meta.
    // always re-compile/assemble when configuration changed
    let module_config_meta_file_path = get_mata_file_path(&asset_path, FILE_NAME_MODULE_CONFIG);

    let (is_module_config_changed, module_config_timestamp_opt) = {
        let current_timestamp_opt = get_file_timestamp(&module_config_file_path)?;
        let module_config_meta_opt = FileMeta::load(&module_config_meta_file_path)?;

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

    // source file includes:
    // - *.anc
    // - *.ancr
    // - *.anca
    struct SourceBuildPendingItem {
        // the path of source file (*.anc, *.ancr, and *.anca)
        source_path_buf: PathBuf,
        meta_file_path: PathBuf,

        // `canonical_name` is the relative path name with replacing '/' with '-',
        // e.g.
        // - source: "/home/yang/projects/helloworld/src/network/http/get.anca"
        // - prefix: "/home/yang/projects/helloworld/src"
        // - relative path: "network/http/get.anca"
        // - name path: "network/http/get"
        // - canonical name: "network-http-get"
        // - submodule name path: "network::http::get"
        canonical_name: String,

        // `canonical_name` is the `canonical_name` with replacing '-' with '::',
        // e.g.
        // - canonical name: "network-http-get"
        // - submodule name path: "network::http::get"
        submodule_name_path: String,

        // the timestamp of source file (*.anc, *.ancr, and *.anca),
        // note that it is NOT the timestamp of
        // generated file (i.e., it is not the file in the folder "asset").
        timestamp_opt: Option<u64>,
    }

    // the building process
    //
    //      translate   compile        assemble
    //           v         v               v
    // source0 ----> ir0 ----> assembly0 ----> object0--\  link
    // source1 ----> ir1 ----> assembly1 ----> object1--|-------> module
    // source2 ----> ir2 ----> assembly2 ----> object2--/
    //
    // the target of "pending source" file will be appended to the "pending ir",
    // as well as the target of "pending ir" file will be appended to the "pending assembly" in further.
    let mut _pending_source_items: Vec<SourceBuildPendingItem> = vec![];
    let mut _ir_files: Vec<PathBuf> = vec![];

    let mut _pending_ir_items: Vec<SourceBuildPendingItem> = vec![];
    let mut _assembly_files: Vec<PathBuf> = vec![];

    let mut pending_assemble_items: Vec<SourceBuildPendingItem> = vec![];
    let mut object_files: Vec<PathBuf> = vec![];

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

    // process the source files
    // todo

    // process the ir files
    // todo

    // process the assembly files
    let has_reassembled = {
        // add scanning directories
        let mut assembly_scan_start_items: Vec<ScanStartItem> = vec![];

        let src_path = get_module_folder_src_path(module_path);
        assembly_scan_start_items.push(ScanStartItem {
            source_path: src_path.clone(),
            prefix_path: src_path.clone(),
        });

        let app_path = get_module_folder_app_path(module_path);
        assembly_scan_start_items.push(ScanStartItem {
            source_path: app_path,
            prefix_path: module_path.to_path_buf(),
        });

        if include_unit_tests {
            let tests_path = get_module_folder_tests_path(module_path);
            assembly_scan_start_items.push(ScanStartItem {
                source_path: tests_path,
                prefix_path: module_path.to_path_buf(),
            });
        }

        // scan assembly files
        for assembly_scan_start_item in assembly_scan_start_items {
            let source_file_path_and_timestamps =
                list_assembly_files(&assembly_scan_start_item.source_path)?;

            for PathAndTimestamp {
                file_path: current_file_path,
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
                let relative_path = current_file_path
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
                let assembly_meta_opt = FileMeta::load(&assembly_meta_file_path)?;

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

                if is_module_config_changed // re-assemble when configuration changed
                || !is_object_file_exists // re-assemble when object file does not exist
                || is_assembly_file_changed
                // re-assemble when assembly file changed
                {
                    pending_assemble_items.push(SourceBuildPendingItem {
                        source_path_buf: current_file_path,
                        meta_file_path: assembly_meta_file_path,
                        canonical_name,
                        submodule_name_path,
                        timestamp_opt: current_timestamp_opt,
                    });
                } else {
                    // append the existing object file
                    object_files.push(object_file_path);
                }
            }
        }

        // re-assemble assembly files
        if !pending_assemble_items.is_empty() {
            std::fs::create_dir_all(&object_path)
                .map_err(|e| RuntimeError::Message(format!("{}", e)))?;

            // assemble
            for pending_assemble_item in &pending_assemble_items {
                println!(
                    "!! assemble: {}",
                    pending_assemble_item.source_path_buf.to_str().unwrap()
                );

                // rename the name path of the top most submodule:
                // - "lib.{anc,ancr,anca}"
                // - "main.{anc,ancr,anca}"
                // to emtpy string.
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
                let image_common_entry = assemble_by_file(
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

                // append generated object file
                object_files.push(object_file_path);

                println!(
                    "^^ update assembly meta: {}",
                    pending_assemble_item.meta_file_path.to_str().unwrap()
                );

                save_object_meta(
                    pending_assemble_item.timestamp_opt,
                    &pending_assemble_item.meta_file_path,
                )?;
            }

            true
        } else {
            false
        }
    };

    // link
    let shared_module_file_path =
        get_shared_module_image_file_path_by_hash_path(&hash_path, &module_config.name);
    let is_shared_module_file_exist = shared_module_file_path.exists();

    let module_entry_opt = if !has_reassembled && is_shared_module_file_exist {
        // no any building is needed, reuse the existing module image file
        None
    } else {
        let mut object_binaries = vec![];
        let mut image_common_entries = vec![];

        for object_file in object_files {
            let object_binary =
                std::fs::read(&object_file).map_err(|e| RuntimeError::Message(format!("{}", e)))?;
            object_binaries.push(object_binary);
        }

        for object_binary in &object_binaries {
            let image_common_entry = read_object_file(object_binary)
                .map_err(|e| RuntimeError::Message(format!("{}", e)))?;
            image_common_entries.push(image_common_entry);
        }

        println!(
            ">> write module binary: {}",
            shared_module_file_path.to_str().unwrap()
        );

        let module_name = &module_config.name;
        let module_version = EffectiveVersion::from_str(&module_config.version);
        let module_entry = link(module_name, &module_version, &image_common_entries)?;
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
        save_module_config_meta(module_config_timestamp_opt, &module_config_meta_file_path)?;
    }

    Ok(module_entry_opt)
}

/// Recompile if the module image does not exist and it is not sealing.
/// Cache checking can be bypasswd with the parameter "check_modification".
/// Note that only the local module needs to be checked for changes
/// each time it is run.
pub fn load_or_build_module(
    module_path: &Path,
    dependency_hash_opt: Option<&DependencyHash>,
    include_unit_tests: bool,
    check_modification: bool,
) -> Result<ImageCommonEntry, RuntimeError> {
    // module config
    let module_config_file_path = get_module_config_file_path(module_path);
    let module_config = ModuleConfig::load(&module_config_file_path)?;

    // output folders
    let output_path = get_module_folder_output_path(module_path);
    let hash_path = get_output_folder_hash_path(&output_path, dependency_hash_opt);

    let shared_module_file_path =
        get_shared_module_image_file_path_by_hash_path(&hash_path, &module_config.name);
    let is_shared_module_file_exist = shared_module_file_path.exists();

    let load_module = |module_file: &Path| -> Result<ImageCommonEntry, RuntimeError> {
        let module_binary =
            std::fs::read(module_file).map_err(|e| RuntimeError::Message(format!("{}", e)))?;
        read_object_file(&module_binary).map_err(|e| RuntimeError::Message(format!("{}", e)))
    };

    if is_shared_module_file_exist && (module_config.seal || !check_modification) {
        load_module(&shared_module_file_path)
    } else {
        match build_module(
            module_path,
            dependency_hash_opt.unwrap(),
            include_unit_tests,
        ) {
            Ok(module_opt) => match module_opt {
                // rebuild
                Some(module) => Ok(module),
                // source is not changed
                None => load_module(&shared_module_file_path),
            },
            Err(e) => Err(e),
        }
    }
}

pub fn load_or_build_module_by_import_module_entry(
    parent_module_path: &Path,
    import_module_entry: &ImportModuleEntry,
    runtime_property: &RuntimeProperty,
) -> Result<(PathBuf, Option<DependencyHash>, ImageCommonEntry), RuntimeError> {
    let ImportModuleEntry {
        name: module_name,
        module_dependency,
    } = import_module_entry;

    let (module_path, hash_opt, check_modification) = match module_dependency.as_ref() {
        ModuleDependency::Local(dependency_local) => {
            let module_path = dependency_local
                .path
                .try_resolve_in(parent_module_path)
                .map_err(|e| RuntimeError::Message(format!("{}", e)))?
                .to_path_buf();

            let hash = DEPENDENCY_HASH_ZERO; // todo:: calculate the hash
            (module_path, Some(hash), true)
        }
        ModuleDependency::Remote(dependency_remote) => {
            let mut path_buf = runtime_property.get_modules_directory();
            path_buf.push(module_name);
            path_buf.push(DIRECTORY_NAME_VERSION_REMOTE);

            let remote_repository_resource_location = RemoteRepositoryResourceLocation::new(
                &dependency_remote.url,
                &dependency_remote.reversion,
            );

            // check existance of module
            // todo

            // if does not exist...
            // download and checkout
            let repositories_path = &runtime_property.get_repositories_directory();
            let repository_path =
                fetch_module(&remote_repository_resource_location.url, &repositories_path)?;
            let module_path = checkout_module(
                &repository_path,
                &remote_repository_resource_location.revision,
            )?;

            let hash = DEPENDENCY_HASH_ZERO; // todo:: calculate the hash
            (module_path, Some(hash), false)
        }
        ModuleDependency::Share(dependency_share) => {
            let mut path_buf = runtime_property.get_modules_directory();
            path_buf.push(module_name);
            path_buf.push(&dependency_share.version);

            // check existance of module
            // todo

            // if does not exist...
            // get remote location
            let remote_repository_resource_location_result = get_shared_module_remote_location(
                &runtime_property.registries,
                module_name,
                &EffectiveVersion::from_str(&dependency_share.version),
            );

            let remote_repository_resource_location =
                match remote_repository_resource_location_result {
                Ok(r) => r,
                Err(_e) /* if ... */ => {
                    // update module index if the cache does not exist.
                    // get remote location again
                    todo!()
                }
                // Err(e) => {return Err(e);}
            };

            // download and checkout
            let repositories_path = &runtime_property.get_repositories_directory();
            let repository_path =
                fetch_module(&remote_repository_resource_location.url, &repositories_path)?;
            let module_path = checkout_module(
                &repository_path,
                &remote_repository_resource_location.revision,
            )?;

            let hash = DEPENDENCY_HASH_ZERO; // todo:: calculate the hash
            (module_path, Some(hash), false)
        }
        ModuleDependency::Runtime => {
            let mut path_buf = runtime_property.get_builtin_modules_directory();
            path_buf.push(module_name);
            (path_buf, None, false)
        }
        ModuleDependency::Module => unreachable!(),
    };

    let image_common_entry =
        load_or_build_module(&module_path, hash_opt.as_ref(), false, check_modification)?;

    let mp = module_path.canonicalize().unwrap();

    Ok((mp, hash_opt, image_common_entry))
}

pub fn build_application_by_dependency_tree(
    module_path: &Path,
    module_dependency_type: ModuleDependencyType,
    runtime_property: &RuntimeProperty,
    include_unit_tests: bool,
) -> Result<(ImageCommonEntry, ImageIndexEntry, PathBuf), RuntimeError> {
    // todo:: calculate the hash
    // todo:: using the default properties and compilation environment varialbes
    let main_hash = DEPENDENCY_HASH_ZERO;
    let main_module = load_or_build_module(
        module_path,
        Some(&main_hash),
        include_unit_tests,
        module_dependency_type == ModuleDependencyType::Local,
    )?;

    let module_name = main_module.name.clone();

    // build all dependent modules
    let (mut image_common_entries, mut dynamic_link_module_entries) =
        build_all_dependent_modules_by_dependency_tree(
            &module_name,
            module_path,
            &main_module,
            module_dependency_type,
            runtime_property,
        )?;

    // build index
    // append main module to all common module entries
    image_common_entries.insert(0, main_module);
    dynamic_link_module_entries.insert(
        0,
        DynamicLinkModuleEntry {
            name: module_name.to_owned(),
            module_location: Box::new(ModuleLocation::Embed),
        },
    );
    let index_entry = index(&mut image_common_entries, &dynamic_link_module_entries)?;
    let common_entry = image_common_entries.remove(0);

    // save image file
    let output_path = get_module_folder_output_path(module_path);
    let application_image_file_full_path =
        get_application_module_image_file_path_by_output_path(&output_path, &module_name);

    save_application_image_file(
        &common_entry,
        &index_entry,
        &application_image_file_full_path,
    )?;

    Ok((common_entry, index_entry, application_image_file_full_path))
}

pub fn build_application_by_dependency_list(
    _module_path: &Path,
    _module_dependency_type: ModuleDependencyType,
    _runtime_property: &RuntimeProperty,
) -> Result<(), RuntimeError> {
    todo!()
}

pub fn build_application_by_single_file(
    script_file_path: &Path,
    runtime_property: &RuntimeProperty,
) -> Result<(ImageCommonEntry, ImageIndexEntry, Vec<u8>), RuntimeError> {
    let source_code = std::fs::read_to_string(script_file_path)
        .map_err(|e| RuntimeError::Message(format!("{}", e)))?;

    let module_config_from_file_opt =
        load_inline_config_from_single_file_application_source(&source_code)?;
    let module_config = if let Some(module_config_from_file) = module_config_from_file_opt {
        module_config_from_file
    } else {
        let file_base_name = script_file_path.file_stem().unwrap().to_str().unwrap();

        ModuleConfig {
            name: file_base_name.to_owned(),
            version: "1.0.0".to_owned(),
            edition: RUNTIME_EDITION_STRING.to_owned(),
            properties: HashMap::new(),
            modules: HashMap::new(), // todo:: add std module
            libraries: HashMap::new(),
            seal: false,
        }
    };

    let (import_module_entries, external_library_entries) =
        module_config.get_dependencies_by_module_config();

    let module_path = script_file_path.parent().unwrap();
    let module_name = module_config.name.clone();
    let main_module = assemble(
        &import_module_entries,
        &external_library_entries,
        &module_name,
        &source_code,
    )?;

    // build all dependent modules
    let (mut image_common_entries, mut dynamic_link_module_entries) =
        build_all_dependent_modules_by_dependency_tree(
            &module_name,
            module_path,
            &main_module,
            ModuleDependencyType::Local,
            runtime_property,
        )?;

    // build index
    // append main module to all common module entries
    image_common_entries.insert(0, main_module);
    dynamic_link_module_entries.insert(
        0,
        DynamicLinkModuleEntry {
            name: module_name.to_owned(),
            module_location: Box::new(ModuleLocation::Embed),
        },
    );
    let index_entry = index(&mut image_common_entries, &dynamic_link_module_entries)?;
    let common_entry = image_common_entries.remove(0);

    // save
    let mut buffer: Vec<u8> = vec![];

    write_image_file(&common_entry, &index_entry, &mut buffer)
        .map_err(|e| RuntimeError::Message(format!("{}", e)))?;

    Ok((common_entry, index_entry, buffer))
}

/// If a module is referenced multiple times in the dependency tree with
/// different parameters, there is a risk that the program may not run correctly
/// because the application will only select one of the dependent parameters.
fn build_all_dependent_modules_by_dependency_tree(
    module_name: &str,
    module_path: &Path,
    main_module: &ImageCommonEntry,
    module_dependency_type: ModuleDependencyType,
    runtime_property: &RuntimeProperty,
) -> Result<(Vec<ImageCommonEntry>, Vec<DynamicLinkModuleEntry>), RuntimeError> {
    let mut loaded_module_items: Vec<DependencyBuildCompleteItem> = vec![]; // all loaded modules
    let mut pending_import_module_items = VecDeque::<DependencyBuildPendingItem>::new();

    // append dependencies of the first module
    add_import_module_entries_to_build_pending_items_with_rules_check(
        module_name,
        module_path,
        module_dependency_type,
        &main_module.import_module_entries,
        &mut pending_import_module_items,
    )?;

    // build each dependency
    while !pending_import_module_items.is_empty() {
        let module_build_pending_item = pending_import_module_items.pop_front().unwrap();
        let (new_module_path, new_hash_opt, new_module_entry) =
            load_or_build_module_by_import_module_entry(
                &module_build_pending_item.parent_module_path_buf,
                &module_build_pending_item.import_module_entry,
                runtime_property,
            )?;

        let new_module_denpendency = module_build_pending_item
            .import_module_entry
            .module_dependency
            .as_ref()
            .to_owned();
        let new_module_dependency_type = get_module_dependency_type(&new_module_denpendency);

        // append dependencies of the current module
        add_import_module_entries_to_build_pending_items_with_rules_check(
            &new_module_entry.name,
            &new_module_path,
            new_module_dependency_type,
            &new_module_entry.import_module_entries,
            &mut pending_import_module_items,
        )?;

        // append loaded module to `loaded_module_items`.
        loaded_module_items.push(DependencyBuildCompleteItem {
            module_dependency: new_module_denpendency,
            module_path: new_module_path,
            hash_opt: new_hash_opt,
            image_common_entry: new_module_entry,
        });
    }

    // remove duplicated modules
    let mut dedup_module_items: Vec<DependencyBuildCompleteItem> = vec![];
    for loaded_item in loaded_module_items {
        let loaded_import_module_name = &loaded_item.image_common_entry.name;

        let pos_dedup_opt = dedup_module_items.iter().position(|dedup_item| {
            &dedup_item.image_common_entry.name == loaded_import_module_name
        });

        if let Some(pos_dedup) = pos_dedup_opt {
            let dedup_item = &dedup_module_items[pos_dedup];

            if dedup_item.module_dependency == loaded_item.module_dependency {
                // module identical
                continue;
            } else {
                match &loaded_item.module_dependency {
                    ModuleDependency::Local(_) => {
                        if matches!(dedup_item.module_dependency, ModuleDependency::Local(_)) {
                            if dedup_item.module_path != loaded_item.module_path {
                                return Err(RuntimeError::Message(format!(
                                    "Dependency module \"{}\" source conflict.",
                                    loaded_import_module_name
                                )));
                            } else {
                                // - these two modules are considered identical because they have
                                //   the same final path.
                                // - in the dependencies, the local modules use relative paths, so
                                //   they are assumed to be different when compared directly.
                                // - ignore parameters
                                continue;
                            }
                        } else {
                            return Err(RuntimeError::Message(format!(
                                "Dependency module \"{}\" has different type.",
                                loaded_import_module_name
                            )));
                        }
                    }
                    ModuleDependency::Remote(_) => {
                        if matches!(dedup_item.module_dependency, ModuleDependency::Remote(_)) {
                            return Err(RuntimeError::Message(format!(
                                "Dependency module \"{}\" source conflict.",
                                loaded_import_module_name
                            )));
                        } else {
                            return Err(RuntimeError::Message(format!(
                                "Dependency module \"{}\" has different type.",
                                loaded_import_module_name
                            )));
                        }
                    }
                    ModuleDependency::Share(share_loaded) => {
                        if let ModuleDependency::Share(share_dedup) = &dedup_item.module_dependency
                        {
                            // compare version
                            match EffectiveVersion::from_str(&share_loaded.version)
                                .compatible(&EffectiveVersion::from_str(&share_dedup.version))
                            {
                                VersionCompatibility::Equals | VersionCompatibility::LessThan => {
                                    // keep the existing loaded module (`share_dedup`) because
                                    // the existing item is newer than (or equals to) the new loaded one (`share_loaded`).
                                    continue;
                                }
                                VersionCompatibility::GreaterThan => {
                                    // replace the existing loaded module (`share_dedup`) with the new one because
                                    // the existing item is older than the new loaded one (`share_loaded`).
                                    dedup_module_items.remove(pos_dedup);
                                    dedup_module_items.push(loaded_item);
                                }
                                VersionCompatibility::Conflict => {
                                    // major versions are different
                                    return Err(RuntimeError::Message(format!(
                                        "Dependency module \"{}\" has conflict versions.",
                                        loaded_import_module_name
                                    )));
                                }
                            }
                        } else {
                            return Err(RuntimeError::Message(format!(
                                "Dependency module \"{}\" has different type.",
                                loaded_import_module_name
                            )));
                        }
                    }
                    ModuleDependency::Runtime => {
                        return Err(RuntimeError::Message(format!(
                            "Dependency module \"{}\" has different type.",
                            loaded_import_module_name
                        )));
                    }
                    ModuleDependency::Module => unreachable!(),
                }
            }
        } else {
            dedup_module_items.push(loaded_item);
        }
    }

    // remove dangling modules (i.e., modules that are not referenced by any other module)

    // start traversing from the root node (i.e., the main module), and add
    // only non-dangling modules. (there may be modules introduced by modules that
    // have been deleted)
    let mut module_with_reference_count_items: Vec<(
        /* module name */ String,
        /* reference count */ usize,
    )> = vec![];

    let self_reference_module = ImportModuleEntry::self_reference_entry();

    let mut pending_module_entries: VecDeque<&ImageCommonEntry> = VecDeque::new();
    pending_module_entries.push_back(&main_module);

    while !pending_module_entries.is_empty() {
        let parent_module = pending_module_entries.pop_front().unwrap();

        for dependency_new in &parent_module.import_module_entries {
            // skip the self reference item
            if dependency_new == &self_reference_module {
                continue;
            }

            // find the existing item
            let index_opt = module_with_reference_count_items
                .iter()
                .position(|(name, _)| name == &dependency_new.name);

            if let Some(index) = index_opt {
                // inc the reference count
                module_with_reference_count_items[index].1 += 1;
            } else {
                // add reference count item
                module_with_reference_count_items.push((dependency_new.name.to_owned(), 1));

                // add to queue to calculate the depth of its subnodes,
                // i.e. subnodes of subnode.
                pending_module_entries.push_back(
                    &dedup_module_items
                        .iter()
                        .find(|item| item.image_common_entry.name == dependency_new.name)
                        .unwrap()
                        .image_common_entry,
                );
            }
        }
    }

    let effective_names = module_with_reference_count_items
        .iter()
        .map(|item| &item.0)
        .collect::<Vec<_>>();

    // remove dangling modules
    for idx in (0..dedup_module_items.len()).rev() {
        let name = &dedup_module_items[idx].image_common_entry.name;
        let existing = effective_names.iter().any(|item| *item == name);
        if !existing {
            println!("** REMOVE dangling moudle: {}", name);
            dedup_module_items.remove(idx);
        }
    }

    // generate dynamic_link_module_entries

    let mut dynamic_link_module_entries = vec![];

    for dedup_module_item in &dedup_module_items {
        let name = dedup_module_item.image_common_entry.name.clone();
        let module_location = match get_module_dependency_type(&dedup_module_item.module_dependency)
        {
            ModuleDependencyType::Local => ModuleLocation::Local(Box::new(ModuleLocationLocal {
                module_path: dedup_module_item.module_path.to_str().unwrap().to_owned(),
                hash: format_dependency_hash(&dedup_module_item.hash_opt.unwrap()),
            })),
            ModuleDependencyType::Remote => {
                ModuleLocation::Remote(Box::new(ModuleLocationRemote {
                    hash: format_dependency_hash(&dedup_module_item.hash_opt.unwrap()),
                }))
            }
            ModuleDependencyType::Share => ModuleLocation::Share(Box::new(ModuleLocationShare {
                version: dedup_module_item.image_common_entry.version.to_string(),
                hash: format_dependency_hash(&dedup_module_item.hash_opt.unwrap()),
            })),
            ModuleDependencyType::Runtime => ModuleLocation::Runtime,
            ModuleDependencyType::Module => unreachable!(),
        };

        dynamic_link_module_entries.push(DynamicLinkModuleEntry {
            name,
            module_location: Box::new(module_location),
        });
    }

    let mut image_common_entries = vec![];

    for DependencyBuildCompleteItem {
        image_common_entry, ..
    } in dedup_module_items
    {
        image_common_entries.push(image_common_entry);
    }

    Ok((image_common_entries, dynamic_link_module_entries))
}

fn add_import_module_entries_to_build_pending_items_with_rules_check(
    current_module_name: &str,  // for generating error message
    current_module_path: &Path, // for resolving relative path (in the module's dependency) of local module to absolute path
    current_module_dependency_type: ModuleDependencyType,
    new_import_module_entries: &[ImportModuleEntry],
    pending_import_module_items: &mut VecDeque<DependencyBuildPendingItem>,
) -> Result<(), RuntimeError> {
    // check the rules of dependency type
    // rules:
    // - "Remote" type does not allow "Local" type dependency.
    // - "Share" and "Runtime" types do not allow "Remote" and "Local" type dependency.
    for new_import_module_entry in new_import_module_entries {
        let new_import_module_dependency_type =
            get_module_dependency_type(&new_import_module_entry.module_dependency);
        let new_import_module_name = &new_import_module_entry.name;

        match current_module_dependency_type {
            ModuleDependencyType::Local => {
                // pass
            }
            ModuleDependencyType::Remote => {
                if new_import_module_dependency_type == ModuleDependencyType::Local {
                    return Err(RuntimeError::Message(format!(
                        "Remote type module \"{}\" contains a local type module \"{}\".",
                        current_module_name, new_import_module_name
                    )));
                }
            }
            ModuleDependencyType::Share => {
                if new_import_module_dependency_type == ModuleDependencyType::Local {
                    return Err(RuntimeError::Message(format!(
                        "Share type module \"{}\" contains a local type module \"{}\".",
                        current_module_name, new_import_module_name
                    )));
                } else if new_import_module_dependency_type == ModuleDependencyType::Remote {
                    return Err(RuntimeError::Message(format!(
                        "Share type module \"{}\" contains a remote type module \"{}\".",
                        current_module_name, new_import_module_name
                    )));
                }
            }
            ModuleDependencyType::Runtime => {
                if new_import_module_dependency_type == ModuleDependencyType::Local {
                    return Err(RuntimeError::Message(format!(
                        "Runtime type module \"{}\" contains a local type module \"{}\".",
                        current_module_name, new_import_module_name
                    )));
                } else if new_import_module_dependency_type == ModuleDependencyType::Remote {
                    return Err(RuntimeError::Message(format!(
                        "Runtime type module \"{}\" contains a remote type module \"{}\".",
                        current_module_name, new_import_module_name
                    )));
                }
            }
            ModuleDependencyType::Module => unreachable!(),
        }
    }

    for new_import_module_entry in new_import_module_entries {
        if matches!(
            new_import_module_entry.module_dependency.as_ref(),
            ModuleDependency::Module
        ) {
            continue;
        }

        // it's acceptable to add all imported items to the
        // pending list because the module is cached, so it is not
        // actually recompiled.
        pending_import_module_items.push_back(DependencyBuildPendingItem {
            parent_module_path_buf: current_module_path.to_path_buf(),
            import_module_entry: new_import_module_entry.to_owned(),
        });
    }

    Ok(())
}

fn save_module_config_meta(
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

fn assemble_by_file(
    import_module_entries: &[ImportModuleEntry],
    external_library_entries: &[ExternalLibraryEntry],
    submodule_full_name: &str,
    assembly_file_path: &Path,
) -> Result<ImageCommonEntry, RuntimeError> {
    let source_code = std::fs::read_to_string(assembly_file_path)
        .map_err(|e| RuntimeError::Message(format!("{}", e)))?;

    assemble(
        import_module_entries,
        external_library_entries,
        submodule_full_name,
        &source_code,
    )
}

fn assemble(
    import_module_entries: &[ImportModuleEntry],
    external_library_entries: &[ExternalLibraryEntry],
    submodule_full_name: &str,
    source_code: &str,
) -> Result<ImageCommonEntry, RuntimeError> {
    let module_node = parse_from_str(source_code)
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

fn save_object_meta(
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
    target_module_version: &EffectiveVersion,
    submodule_entries: &[ImageCommonEntry],
) -> Result<ImageCommonEntry, RuntimeError> {
    static_link(
        target_module_name,
        target_module_version,
        true,
        submodule_entries,
    )
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
fn index(
    image_common_entries: &mut [ImageCommonEntry],
    dynamic_link_module_entries: &[DynamicLinkModuleEntry],
) -> Result<ImageIndexEntry, RuntimeError> {
    sort_modules_by_dependent_deepth(image_common_entries)
        .map_err(|e| RuntimeError::Message(format!("{}", e)))?;
    dynamic_link(image_common_entries, dynamic_link_module_entries)
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

struct DependencyBuildCompleteItem {
    // module folder path
    module_path: PathBuf,

    // the dependency info of the current module
    module_dependency: ModuleDependency,
    hash_opt: Option<DependencyHash>,
    image_common_entry: ImageCommonEntry,
}

struct DependencyBuildPendingItem {
    // for resolving relative path (in the module's dependency) of local module to absolute path
    parent_module_path_buf: PathBuf,
    import_module_entry: ImportModuleEntry,
}

fn get_module_dependency_type(module_dependency: &ModuleDependency) -> ModuleDependencyType {
    match module_dependency {
        ModuleDependency::Local(_) => ModuleDependencyType::Local,
        ModuleDependency::Remote(_) => ModuleDependencyType::Remote,
        ModuleDependency::Share(_) => ModuleDependencyType::Share,
        ModuleDependency::Runtime => ModuleDependencyType::Runtime,
        ModuleDependency::Module => ModuleDependencyType::Module,
    }
}

pub fn load_inline_config_from_single_file_application_source(
    source_code: &str,
) -> Result<Option<ModuleConfig>, RuntimeError> {
    // search the "/*   @config {...}   */"
    //                ^               ^
    //                |               |
    //              whitespaces (' ', '\t', '\n') are allowed

    let mut chars = source_code.chars();
    let mut iter = PeekableIter::new(&mut chars, 2);

    while let Some(prev_char) = iter.next() {
        match prev_char {
            '/' if matches!(iter.peek(0), Some('/')) => {
                // line comment
                iter.next(); // consume '/'

                // consume to line end
                while let Some(c) = iter.next() {
                    if c == '\n' {
                        break;
                    }
                }
            }
            '/' if matches!(iter.peek(0), Some('*')) => {
                // block comment
                iter.next(); // consume '*'

                if let Some(comment_text) = parse_block_comment(&mut iter) {
                    let trimmed_text = comment_text.trim();
                    if trimmed_text.starts_with(INLINE_CONFIG_MARK) {
                        let (_, config_text) = trimmed_text.split_at("config!".len());
                        let module_config = ason::from_str(config_text)
                            .map_err(|e| RuntimeError::Message(e.with_source(config_text)))?;
                        return Ok(Some(module_config));
                    }
                }
            }
            _ => {
                // consume
            }
        }
    }
    Ok(None)
}

fn parse_block_comment(iter: &mut PeekableIter<char>) -> Option<String> {
    // /*? ... */?
    //   ^       ^
    //   |       |--> consumes to here
    //   |--> starts from here

    let mut nested = 1;
    let mut ss = String::new();

    while let Some(prev_char) = iter.next() {
        match prev_char {
            '/' if matches!(iter.peek(0), Some('/')) => {
                // line comment
                iter.next(); // consume '/'

                // consume to line end
                while let Some(c) = iter.next() {
                    if c == '\n' {
                        break;
                    }
                }
            }
            '/' if matches!(iter.peek(0), Some('*')) => {
                // start nested block comment
                iter.next(); // consume '*'
                nested += 1;
            }
            '*' if matches!(iter.peek(0), Some('/')) => {
                // end nested block comment
                iter.next(); // consume '/'
                nested -= 1;

                if nested == 0 {
                    return Some(ss);
                }
            }
            _ => {
                if nested == 1 {
                    ss.push(prev_char);
                }
            }
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use anc_image::DEPENDENCY_HASH_ZERO;
    use anc_isa::ModuleDependencyType;
    use resolve_path::PathResolveExt;

    use crate::{
        builder::{build_application_by_dependency_tree, load_or_build_module},
        entry::RuntimeProperty,
    };

    use super::{build_application_by_single_file, build_module};

    fn get_resources_path_buf() -> PathBuf {
        // `std::env::current_dir()` returns the current Rust project's root folder
        let mut pwd = std::env::current_dir().unwrap();
        pwd.push("tests");
        pwd.push("resources");
        pwd
    }

    fn get_runtime_property() -> RuntimeProperty {
        let runtime_home_relative = PathBuf::from("~/.anc");
        let runtime_home = runtime_home_relative.try_resolve().unwrap();
        if !runtime_home.exists() {
            std::fs::create_dir_all(&runtime_home).unwrap();
        }

        let current_runtime_path_relative = PathBuf::from("~/.anc/runtimes/2025");
        let current_runtime_path = current_runtime_path_relative.try_resolve().unwrap();

        RuntimeProperty::from_custom(
            &current_runtime_path.to_path_buf(),
            &runtime_home.to_path_buf(),
        )
    }

    #[test]
    fn test_build_module() {
        let hash_opt = Some(&DEPENDENCY_HASH_ZERO);

        // single_module_app
        {
            let mut moudle_path_buf = get_resources_path_buf();
            moudle_path_buf.push("single_module_app");

            // load or rebuild
            let result0 = load_or_build_module(&moudle_path_buf, hash_opt, false, true);
            assert!(result0.is_ok());
            // todo: check entries

            // unchanged
            let result1 = build_module(&moudle_path_buf, &DEPENDENCY_HASH_ZERO, false);
            assert!(matches!(result1, Ok(None)));
        }

        // single_module_app_with_executable_units
        {
            let mut moudle_path_buf = get_resources_path_buf();
            moudle_path_buf.push("single_module_app_with_executable_units");

            // load or rebuild
            let result0 = load_or_build_module(&moudle_path_buf, hash_opt, false, true);
            assert!(result0.is_ok());
            // todo: check entries

            // unchanged
            let result1 = build_module(&moudle_path_buf, &DEPENDENCY_HASH_ZERO, false);
            assert!(matches!(result1, Ok(None)));
        }

        // single_module_with_unit_tests
        {
            let mut moudle_path_buf = get_resources_path_buf();
            moudle_path_buf.push("single_module_with_unit_tests");

            // load or rebuild without unit tests
            let result0 = load_or_build_module(&moudle_path_buf, hash_opt, false, true);
            assert!(result0.is_ok());
            // todo: check entries

            // load or rebuild with unit tests
            let result1 = load_or_build_module(&moudle_path_buf, hash_opt, true, true);
            assert!(result1.is_ok());
            // todo: check unit test entries

            // unchanged
            let result2 = build_module(&moudle_path_buf, &DEPENDENCY_HASH_ZERO, true);
            assert!(matches!(result2, Ok(None)));
        }
    }

    #[test]
    fn test_build_application_by_dependencies() {
        let runtime_property = get_runtime_property();

        // single_module_app
        {
            let mut moudle_path_buf = get_resources_path_buf();
            moudle_path_buf.push("single_module_app");

            let result0 = build_application_by_dependency_tree(
                &moudle_path_buf,
                ModuleDependencyType::Local,
                &runtime_property,
                true,
            );
            assert!(result0.is_ok());
            // todo: check entries
        }

        // single_module_app_with_executable_units
        {
            let mut moudle_path_buf = get_resources_path_buf();
            moudle_path_buf.push("single_module_app_with_executable_units");

            let result0 = build_application_by_dependency_tree(
                &moudle_path_buf,
                ModuleDependencyType::Local,
                &runtime_property,
                true,
            );
            assert!(result0.is_ok());
            // todo: check entries
        }

        // single_module_with_unit_tests
        {
            let mut moudle_path_buf = get_resources_path_buf();
            moudle_path_buf.push("single_module_with_unit_tests");

            let result0 = build_application_by_dependency_tree(
                &moudle_path_buf,
                ModuleDependencyType::Local,
                &runtime_property,
                true,
            );
            assert!(result0.is_ok());
            // todo: check entries
        }

        // multiple_module_app
        {
            let mut moudle_path_buf = get_resources_path_buf();
            moudle_path_buf.push("multiple_module_app");
            moudle_path_buf.push("cli");

            let result0 = build_application_by_dependency_tree(
                &moudle_path_buf,
                ModuleDependencyType::Local,
                &runtime_property,
                true,
            );

            assert!(result0.is_ok());
            // todo: check entries
        }
    }

    #[test]
    fn test_build_application_by_single_file() {
        let runtime_property = get_runtime_property();

        // no config
        {
            let mut script_file_path_buf = get_resources_path_buf();
            script_file_path_buf.push("single_file_app");
            script_file_path_buf.push("no_conf.anca");

            let result0 =
                build_application_by_single_file(&script_file_path_buf, &runtime_property);

            assert!(result0.is_ok());
            // todo: check entries
        }

        // with config
        {
            let mut script_file_path_buf = get_resources_path_buf();
            script_file_path_buf.push("single_file_app");
            script_file_path_buf.push("with_conf.anca");

            let result0 =
                build_application_by_single_file(&script_file_path_buf, &runtime_property);

            assert!(result0.is_ok());
            // todo: check entries
        }
    }
}
