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
use anc_isa::{EffectiveVersion, ModuleDependency, ModuleDependencyType, VersionCompatibility};
use anc_linker::{
    dynamic_linker::{dynamic_link, sort_modules_by_dependent_deepth},
    static_linker::static_link,
};
use anc_parser_asm::{parser::parse_from_str, NAME_PATH_SEPARATOR};
use resolve_path::PathResolveExt;

use crate::{
    common::{
        get_app_path, get_application_image_file_path, get_asset_assembly_path, get_asset_ir_path,
        get_asset_object_path, get_dependencies_by_module_config, get_file_timestamp,
        get_hash_asset_path, get_mata_file_path, get_mata_file_path_by_full_name,
        get_module_config_file_path, get_object_file_path, get_output_hash_path, get_output_path,
        get_shared_module_file_path, get_src_path, get_tests_path, list_assembly_files,
        load_file_meta, load_module_config, FileMeta, PathWithTimestamp, RuntimeProperty,
    },
    entry::RuntimeConfig,
    fetcher::{download_module, get_shared_module_remote_location, RemoteLocation},
    RuntimeError, DIRECTORY_NAME_NO_VERSION, MODULE_CONFIG_FILE_NAME,
};

/// Compile the specified module and generate the module image file.
/// The last modification time of source files is checked and no
/// module image is generated if all source files remain unchanged.
pub fn build_module(
    module_path: &Path,
    hash_opt: &Option<DependencyHash>,
    include_unit_tests: bool,
) -> Result<Option<ImageCommonEntry>, RuntimeError> {
    // module config
    let module_config_file_path = get_module_config_file_path(module_path);
    let module_config = load_module_config(&module_config_file_path)?;
    let (import_module_entries, external_library_entries) =
        get_dependencies_by_module_config(&module_config);

    // output folders
    let output_path = get_output_path(module_path);
    let hash_path = get_output_hash_path(&output_path, hash_opt);

    // asset folders
    let asset_path = get_hash_asset_path(&hash_path);
    let _ir_path = get_asset_ir_path(&asset_path);
    let _assembly_path = get_asset_assembly_path(&asset_path);
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

    // the building process
    //
    //      translate   compile        assemble
    // source0 ----> ir0 ----> assembly0 ----> object0--\  link
    // source1 ----> ir1 ----> assembly1 ----> object1--|-------> module
    // source2 ----> ir2 ----> assembly2 ----> object2--/
    //
    // the target of "pending source" file will be appended to the "pending ir",
    // as well as the target of "pending ir" file will be appended to the "pending assembly".
    let mut _pending_source_items: Vec<SourceBuildPendingItem> = vec![];
    let mut _ir_files: Vec<PathBuf> = vec![];

    let mut _pending_ir_items: Vec<SourceBuildPendingItem> = vec![];
    let mut _assembly_files: Vec<PathBuf> = vec![];

    let mut pending_assemble_items: Vec<SourceBuildPendingItem> = vec![];
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

            if is_module_config_changed // re-assemble when configuration changed
                || !is_object_file_exists // re-assemble when object file does not exist
                || is_assembly_file_changed
            {
                pending_assemble_items.push(SourceBuildPendingItem {
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
    let is_reassemble = !pending_assemble_items.is_empty();
    if is_reassemble {
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

            save_object_meta(
                pending_assemble_item.timestamp_opt,
                &pending_assemble_item.meta_file_path,
            )?;
        }
    }

    // link
    let shared_module_file_path = get_shared_module_file_path(&hash_path, &module_config.name);
    let is_shared_module_file_exist = shared_module_file_path.exists();

    let module_entry_opt = if !is_reassemble && is_shared_module_file_exist {
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

/// Recompile only if the module image (i.e. cache) does not exist.
/// Cache checking can be bypasswd with the parameter "force_check_modification",
/// it works like function `build_module` when "force_check_modification == true"  .
pub fn build_module_with_cache_check(
    module_path: &Path,
    hash_opt: &Option<DependencyHash>,
    include_unit_tests: bool,
    force_check_modification: bool,
) -> Result<ImageCommonEntry, RuntimeError> {
    // module config
    let module_config_file_path = get_module_config_file_path(module_path);
    let module_config = load_module_config(&module_config_file_path)?;

    // output folders
    let output_path = get_output_path(module_path);
    let hash_path = get_output_hash_path(&output_path, hash_opt);

    let shared_module_file_path = get_shared_module_file_path(&hash_path, &module_config.name);
    let is_shared_module_file_exist = shared_module_file_path.exists();

    let load_module = |module_file: &Path| -> Result<ImageCommonEntry, RuntimeError> {
        let module_binary =
            std::fs::read(module_file).map_err(|e| RuntimeError::Message(format!("{}", e)))?;
        read_object_file(&module_binary).map_err(|e| RuntimeError::Message(format!("{}", e)))
    };

    if is_shared_module_file_exist && !force_check_modification {
        load_module(&shared_module_file_path)
    } else {
        match build_module(module_path, hash_opt, include_unit_tests) {
            Ok(module_opt) => match module_opt {
                // rebuild
                Some(module) => Ok(module),
                // no changed
                None => load_module(&shared_module_file_path),
            },
            Err(e) => Err(e),
        }
    }
}

pub fn build_module_by_dependency(
    parent_module_path: &Path,
    import_module_entry: &ImportModuleEntry,
    runtime_property: &RuntimeProperty,
    runtime_config: &RuntimeConfig,
) -> Result<(PathBuf, Option<DependencyHash>, ImageCommonEntry), RuntimeError> {
    let ImportModuleEntry {
        name: module_name,
        module_dependency,
    } = import_module_entry;

    let (module_path, hash_opt, force_check_modification) = match module_dependency.as_ref() {
        ModuleDependency::Local(dependency_local) => {
            let path_buf = dependency_local
                .path
                .try_resolve_in(parent_module_path)
                .map_err(|e| RuntimeError::Message(format!("{}", e)))?;

            let hash = DEPENDENCY_HASH_ZERO; // todo:: calculate the hash
            (path_buf.to_path_buf(), Some(hash), true)
        }
        ModuleDependency::Remote(dependency_remote) => {
            // check existance
            let mut path_buf = runtime_property.get_modules_directory();
            path_buf.push(module_name);
            path_buf.push(DIRECTORY_NAME_NO_VERSION);

            // download
            let remote_location =
                RemoteLocation::new(&dependency_remote.url, &dependency_remote.reversion);
            download_module(&remote_location, &path_buf)?;

            let hash = DEPENDENCY_HASH_ZERO; // todo:: calculate the hash
            (path_buf, Some(hash), false)
        }
        ModuleDependency::Share(dependency_share) => {
            // check existance
            let mut path_buf = runtime_property.get_modules_directory();
            path_buf.push(module_name);
            path_buf.push(&dependency_share.version);

            // get remote location
            let remote_location_result = get_shared_module_remote_location(
                runtime_config,
                module_name,
                &EffectiveVersion::from_str(&dependency_share.version),
            );

            let remote_location = match remote_location_result {
                Ok(r) => r,
                Err(_e) /* if ... */ => {
                    // update module index if the cache does not exist.
                    // get remote location again
                    todo!()
                }
                // Err(e) => {return Err(e);}
            };

            // download
            download_module(&remote_location, &path_buf)?;

            let hash = DEPENDENCY_HASH_ZERO; // todo:: calculate the hash
            (path_buf, Some(hash), false)
        }
        ModuleDependency::Runtime => {
            let mut path_buf = runtime_property.get_builtin_modules_directory();
            path_buf.push(module_name);
            (path_buf, None, false)
        }
        ModuleDependency::Module => unreachable!(),
    };

    let image_common_entry =
        build_module_with_cache_check(&module_path, &hash_opt, false, force_check_modification)?;
    Ok((module_path, hash_opt, image_common_entry))
}

pub fn build_application_by_dependencies(
    module_path: &Path,
    module_dependency_type: ModuleDependencyType,
    runtime_property: &RuntimeProperty,
    runtime_config: &RuntimeConfig,
) -> Result<(ImageCommonEntry, ImageIndexEntry, PathBuf), RuntimeError> {
    struct LoadedItem {
        module_dependency: ModuleDependency,
        module_path: PathBuf,
        hash_opt: Option<DependencyHash>,
        image_common_entry: ImageCommonEntry,
    }

    struct PendingItem {
        parent_module_path_buf: PathBuf,
        import_module_entry: ImportModuleEntry,
    }

    let get_module_dependency_type = |module_dependency: &ModuleDependency| match module_dependency
    {
        ModuleDependency::Local(_) => ModuleDependencyType::Local,
        ModuleDependency::Remote(_) => ModuleDependencyType::Remote,
        ModuleDependency::Share(_) => ModuleDependencyType::Share,
        ModuleDependency::Runtime => ModuleDependencyType::Runtime,
        ModuleDependency::Module => ModuleDependencyType::Module,
    };

    let add_import_module_entries_to_pending_with_dependency_type_check =
        |current_module_name: &str,  // for generating error message
         current_module_path: &Path, // for resolve local dependent module path
         current_module_dependency_type: ModuleDependencyType,
         pending_import_module_items: &mut VecDeque<PendingItem>,
         new_import_module_entries: &[ImportModuleEntry]|
         -> Result<(), RuntimeError> {
            // check the dependency type of (new) import module entry
            //
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
                        } else if new_import_module_dependency_type == ModuleDependencyType::Remote
                        {
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
                        } else if new_import_module_dependency_type == ModuleDependencyType::Remote
                        {
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
                pending_import_module_items.push_back(PendingItem {
                    parent_module_path_buf: current_module_path.to_path_buf(),
                    import_module_entry: new_import_module_entry.to_owned(),
                });
            }

            Ok(())
        };

    let mut loaded_module_items: Vec<LoadedItem> = vec![];
    let mut pending_import_module_items = VecDeque::<PendingItem>::new();

    let main_hash = DEPENDENCY_HASH_ZERO; // todo :: calculate the hash

    let main_module = build_module_with_cache_check(
        module_path,
        &Some(main_hash),
        true,
        module_dependency_type == ModuleDependencyType::Local,
    )?;

    let module_name = main_module.name.clone();

    add_import_module_entries_to_pending_with_dependency_type_check(
        &module_name,
        module_path,
        module_dependency_type,
        &mut pending_import_module_items,
        &main_module.import_module_entries,
    )?;

    while !pending_import_module_items.is_empty() {
        let module_build_pending_item = pending_import_module_items.pop_front().unwrap();
        let (new_module_path, new_hash_opt, new_module_entry) = build_module_by_dependency(
            &module_build_pending_item.parent_module_path_buf,
            &module_build_pending_item.import_module_entry,
            runtime_property,
            runtime_config,
        )?;

        let new_module_denpendency = module_build_pending_item
            .import_module_entry
            .module_dependency
            .as_ref()
            .to_owned();
        let new_module_dependency_type = get_module_dependency_type(&new_module_denpendency);

        add_import_module_entries_to_pending_with_dependency_type_check(
            &new_module_entry.name,
            &new_module_path,
            new_module_dependency_type,
            &mut pending_import_module_items,
            &new_module_entry.import_module_entries,
        )?;

        loaded_module_items.push(LoadedItem {
            module_dependency: new_module_denpendency,
            module_path: new_module_path,
            hash_opt: new_hash_opt,
            image_common_entry: new_module_entry,
        });
    }

    // remove duplicated modules
    let mut dedup_module_items: Vec<LoadedItem> = vec![];
    for loaded_item in loaded_module_items {
        let loaded_import_module_name = &loaded_item.image_common_entry.name;

        let pos_dedup_opt = dedup_module_items.iter().position(|dedup_item| {
            &dedup_item.image_common_entry.name == loaded_import_module_name
        });

        if let Some(pos_dedup) = pos_dedup_opt {
            let dedup_item = &dedup_module_items[pos_dedup];

            if dedup_item.module_dependency == loaded_item.module_dependency {
                // identical
                continue;
            } else {
                match &loaded_item.module_dependency {
                    ModuleDependency::Local(_) => {
                        if matches!(dedup_item.module_dependency, ModuleDependency::Local(_)) {
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
                                    // keep:
                                    // the target item is newer than or equals to the source one.
                                    continue;
                                }
                                VersionCompatibility::GreaterThan => {
                                    // replace:
                                    // the target item is older than the source one
                                    dedup_module_items.remove(pos_dedup);
                                    dedup_module_items.push(loaded_item);
                                }
                                VersionCompatibility::Conflict => {
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

    // todo
    // start traversing from the root node (i.e., the main module), and add
    // only valid modules. (there may be modules introduced by modules that
    // have been deleted)

    let mut dynamic_link_module_entries = vec![DynamicLinkModuleEntry {
        name: module_name.clone(),
        module_location: Box::new(ModuleLocation::Embed),
    }];

    for dedup_module_item in &dedup_module_items {
        let name = dedup_module_item.image_common_entry.name.clone();
        let module_location = match get_module_dependency_type(&dedup_module_item.module_dependency)
        {
            ModuleDependencyType::Local => ModuleLocation::Local(Box::new(ModuleLocationLocal {
                path: dedup_module_item.module_path.to_str().unwrap().to_owned(),
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

    let mut image_common_entries = vec![main_module];
    for LoadedItem {
        image_common_entry, ..
    } in dedup_module_items
    {
        image_common_entries.push(image_common_entry);
    }

    let index_entry = index(&mut image_common_entries, &dynamic_link_module_entries)?;
    let common_entry = image_common_entries.remove(0);

    // output folders
    let output_path = get_output_path(module_path);
    let application_image_file_full_path =
        get_application_image_file_path(&output_path, &module_name);

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
    _runtime_config: &RuntimeConfig,
) -> Result<(), RuntimeError> {
    todo!()
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

struct SourceBuildPendingItem {
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

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use anc_image::DEPENDENCY_HASH_ZERO;
    use anc_isa::{ModuleDependencyType, RUNTIME_EDITION_STRING};
    use resolve_path::PathResolveExt;

    use crate::{
        builder::{build_application_by_dependencies, build_module_with_cache_check},
        common::RuntimeProperty,
        entry::RuntimeConfig,
    };

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
        let hash_opt = Some(DEPENDENCY_HASH_ZERO);

        // single_module_app
        {
            let mut moudle_path_buf = get_resources_path_buf();
            moudle_path_buf.push("single_module_app");

            // load or rebuild
            let result0 = build_module_with_cache_check(&moudle_path_buf, &hash_opt, false, true);
            assert!(result0.is_ok());
            // todo: check entries

            // unchanged
            let result1 = build_module(&moudle_path_buf, &hash_opt, false);
            assert!(matches!(result1, Ok(None)));
        }

        // single_module_app_with_executable_units
        {
            let mut moudle_path_buf = get_resources_path_buf();
            moudle_path_buf.push("single_module_app_with_executable_units");

            // load or rebuild
            let result0 = build_module_with_cache_check(&moudle_path_buf, &hash_opt, false, true);
            assert!(result0.is_ok());
            // todo: check entries

            // unchanged
            let result1 = build_module(&moudle_path_buf, &hash_opt, false);
            assert!(matches!(result1, Ok(None)));
        }

        // single_module_with_unit_tests
        {
            let mut moudle_path_buf = get_resources_path_buf();
            moudle_path_buf.push("single_module_with_unit_tests");

            // load or rebuild without unit tests
            let result0 = build_module_with_cache_check(&moudle_path_buf, &hash_opt, false, true);
            assert!(result0.is_ok());
            // todo: check entries

            // load or rebuild with unit tests
            let result1 = build_module_with_cache_check(&moudle_path_buf, &hash_opt, true, true);
            assert!(result1.is_ok());
            // todo: check unit test entries

            // unchanged
            let result2 = build_module(&moudle_path_buf, &hash_opt, true);
            assert!(matches!(result2, Ok(None)));
        }
    }

    #[test]
    fn test_build_application_by_dependencies() {
        let runtime_config = RuntimeConfig::load_and_merge_user_config().unwrap();
        let anc_root_path = runtime_config
            .user_anc_root_directory
            .try_resolve()
            .unwrap();

        if !anc_root_path.exists() {
            std::fs::create_dir_all(&anc_root_path).unwrap();
        }

        let runtime_property = RuntimeProperty::new(
            anc_root_path.to_path_buf(),
            RUNTIME_EDITION_STRING.to_owned(),
        );

        // single_module_app
        {
            let mut moudle_path_buf = get_resources_path_buf();
            moudle_path_buf.push("single_module_app");

            let result0 = build_application_by_dependencies(
                &moudle_path_buf,
                ModuleDependencyType::Local,
                &runtime_property,
                &runtime_config,
            );
            assert!(result0.is_ok());
            // todo: check entries
        }

        // single_module_app_with_executable_units
        {
            let mut moudle_path_buf = get_resources_path_buf();
            moudle_path_buf.push("single_module_app_with_executable_units");

            let result0 = build_application_by_dependencies(
                &moudle_path_buf,
                ModuleDependencyType::Local,
                &runtime_property,
                &runtime_config,
            );
            assert!(result0.is_ok());
            // todo: check entries
        }

        // single_module_with_unit_tests
        {
            let mut moudle_path_buf = get_resources_path_buf();
            moudle_path_buf.push("single_module_with_unit_tests");

            let result0 = build_application_by_dependencies(
                &moudle_path_buf,
                ModuleDependencyType::Local,
                &runtime_property,
                &runtime_config,
            );
            assert!(result0.is_ok());
            // todo: check entries
        }

        // multiple_module_app
        {
            let mut moudle_path_buf = get_resources_path_buf();
            moudle_path_buf.push("multiple_module_app");
            moudle_path_buf.push("cli");

            let result0 = build_application_by_dependencies(
                &moudle_path_buf,
                ModuleDependencyType::Local,
                &runtime_property,
                &runtime_config,
            );

            assert!(result0.is_ok());
            println!("{:#?}", result0.unwrap());
            // todo: check entries
        }
    }
}
