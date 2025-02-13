// Copyright (c) 2025 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE, LICENSE.additional and CONTRIBUTING.

use std::{
    collections::HashMap,
    fs::File,
    io::ErrorKind,
    path::{Path, PathBuf},
};

use anc_image::entry::{ExternalLibraryEntry, ImportModuleEntry};
use anc_isa::{ExternalLibraryDependency, ModuleDependency};
use resolve_path::PathResolveExt;
use serde::{Deserialize, Serialize};

use crate::{
    RuntimeError, DIRECTORY_NAME_BIN, DIRECTORY_NAME_MODULES, DIRECTORY_NAME_REGISTRIES,
    DIRECTORY_NAME_REPOSITORIES, DIRECTORY_NAME_RUNTIMES, FILE_NAME_INITIAL_CONFIG,
    FILE_NAME_USER_CONFIG,
};

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct ModuleConfig {
    pub name: String,
    pub version: String,
    pub edition: String,

    /// Optional
    /// the default value is `false`
    #[serde(default)]
    pub seal: bool,

    /// Optional
    /// the default value is []
    #[serde(default)]
    pub properties: HashMap<String, PropertyValue>,

    /// Optional
    /// the default value is []
    #[serde(default)]
    pub modules: HashMap<String, ModuleDependency>,

    /// Optional
    /// the default value is []
    #[serde(default)]
    pub libraries: HashMap<String, ExternalLibraryDependency>,
    // /// Extra registries
    // /// Optional
    // /// the default value is []
    // #[serde(default)]
    // pub registries: Vec<String>,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename = "prop")]
pub enum PropertyValue {
    #[serde(rename = "string")]
    String(String),

    #[serde(rename = "number")]
    Number(i64),

    #[serde(rename = "bool")]
    Bool(bool),

    #[serde(rename = "set")]
    Set(/* default */ bool, /* includes */ Vec<String>),

    #[serde(rename = "eval")]
    Eval(String),
}

impl ModuleConfig {
    pub fn load(module_config_file_path: &Path) -> Result<ModuleConfig, RuntimeError> {
        let config_file = File::open(module_config_file_path)
            .map_err(|e| RuntimeError::Message(format!("{}", e)))?;

        ason::from_reader(config_file).map_err(|e| {
            RuntimeError::Message(match std::fs::read_to_string(module_config_file_path) {
                Ok(source_code) => e.with_source(&source_code),
                Err(e) => format!("{}", e),
            })
        })
    }

    pub fn get_dependencies_by_module_config(
        &self,
    ) -> (Vec<ImportModuleEntry>, Vec<ExternalLibraryEntry>) {
        let import_module_entries = self
            .modules
            .iter()
            .map(|(name, module_dependency)| {
                ImportModuleEntry::new(name.to_owned(), Box::new(module_dependency.to_owned()))
            })
            .collect::<Vec<_>>();

        let external_library_entries = self
            .libraries
            .iter()
            .map(|(name, external_library_dependency)| {
                ExternalLibraryEntry::new(
                    name.to_owned(),
                    Box::new(external_library_dependency.to_owned()),
                )
            })
            .collect::<Vec<_>>();

        (import_module_entries, external_library_entries)
    }
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct LauncherConfig {
    #[serde(default)]
    pub data_folder: String,

    #[serde(default)]
    pub runtime_registries: Vec<String>,
}

impl LauncherConfig {
    fn new() -> Self {
        // default: `~/.anc`
        let data_folder = "~/.anc".to_owned();

        // default:
        // - "https://github.com/hemashushu/anc_runtime_registry"
        // - "https://gitlab.com/hemashushu/anc_runtime_registry"
        let runtime_registries = vec![
            "https://github.com/hemashushu/anc_runtime_registry".to_owned(),
            "https://gitlab.com/hemashushu/anc_runtime_registry".to_owned(),
        ];

        Self {
            data_folder,
            runtime_registries,
        }
    }

    pub fn load(_filepath: &Path) -> Result<Self, RuntimeError> {
        let default = Self::new();
        // todo
        Ok(default)
    }
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct RuntimeConfig {
    #[serde(default)]
    pub registries: Vec<String>,
}

impl RuntimeConfig {
    fn new() -> Self {
        // default:
        // - "https://github.com/hemashushu/anc_module_registry"
        // - "https://gitlab.com/hemashushu/anc_module_registry"
        let registries = vec![
            "https://github.com/hemashushu/anc_module_registry".to_owned(),
            "https://gitlab.com/hemashushu/anc_module_registry".to_owned(),
        ];

        Self { registries }
    }

    pub fn load(_filepath: &Path) -> Result<Self, RuntimeError> {
        let default = Self::new();
        // todo
        Ok(default)
    }
}

pub struct RuntimeProperty {
    /// e.g. `/home/{user_name}/.anc`
    pub data_path: PathBuf,

    /// e.g. `{initial_path}/runtimes/2025`
    pub current_runtime_path: PathBuf,

    pub registries: Vec<String>,
}

impl RuntimeProperty {
    pub fn from_runtime_exec_file(extra_registries: &[String]) -> Result<Self, RuntimeError> {
        // e.g.
        // - `{initial_path}/runtimes/2025/ancrt`
        // - `{data_path}/runtimes/2025/ancrt`
        let mut exec_path = std::env::current_exe().unwrap();
        exec_path.pop(); // EDITION

        let current_runtime_path = exec_path.clone();

        exec_path.pop(); // `runtimes`
        exec_path.pop(); // `{initial_path|data_path}`

        let initial_path = exec_path.clone();
        let initial_config_path = initial_path.join(FILE_NAME_INITIAL_CONFIG);

        let data_path = if initial_config_path.exists() {
            // the runtime exec file is located in the `{initial_path}` directory.
            let initial_config = LauncherConfig::load(&initial_config_path)?;
            let data_path_string = &initial_config.data_folder;
            PathBuf::from(data_path_string)
                .resolve_in(&initial_path)
                .to_path_buf()
        } else {
            // the runtime exec file is located in the `{data_path}` directory.
            initial_path.clone()
        };

        let user_config_path = data_path.join(FILE_NAME_USER_CONFIG);
        let user_config = RuntimeConfig::load(&user_config_path)?;

        let mut registries = user_config.registries.clone();

        for registry in extra_registries {
            if !registries.iter().any(|item| item == registry) {
                registries.push(registry.to_owned());
            }
        }

        let runtime_property = RuntimeProperty {
            data_path,
            current_runtime_path,
            registries,
        };

        Ok(runtime_property)
    }

    pub fn from_custom(current_runtime_path: &Path, data_path: &Path) -> Self {
        let registries = vec![
            "https://github.com/hemashushu/anc_module_registry".to_owned(),
            "https://gitlab.com/hemashushu/anc_module_registry".to_owned(),
        ];

        let runtime_property = RuntimeProperty {
            data_path: data_path.to_path_buf(),
            current_runtime_path: current_runtime_path.to_path_buf(),
            registries,
        };

        runtime_property
    }

    // `{anc_data_path}/bin`
    pub fn get_bin_directory(&self) -> PathBuf {
        self.data_path.join(DIRECTORY_NAME_BIN)
    }

    // `{anc_data_path}/runtimes`
    pub fn get_runtimes_directory(&self) -> PathBuf {
        self.data_path.join(DIRECTORY_NAME_RUNTIMES)
    }

    // `{anc_data_path}/registries`
    pub fn get_registries_directory(&self) -> PathBuf {
        self.data_path.join(DIRECTORY_NAME_REGISTRIES)
    }

    // `{anc_data_path}/repositories`
    pub fn get_repositories_directory(&self) -> PathBuf {
        self.data_path.join(DIRECTORY_NAME_REPOSITORIES)
    }

    // `{anc_data_path}/modules`
    pub fn get_modules_directory(&self) -> PathBuf {
        self.data_path.join(DIRECTORY_NAME_MODULES)
    }

    // `{initial_path}/runtimes/EDITION/modules`
    pub fn get_builtin_modules_directory(&self) -> PathBuf {
        self.current_runtime_path.join(DIRECTORY_NAME_MODULES)
    }
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct FileMeta {
    // The last modified time of source file
    pub timestamp: Option<u64>,

    /// the default value is []
    #[serde(default)]
    pub dependencies: Vec<String>,
}

impl FileMeta {
    pub fn load(meta_file_path: &Path) -> Result<Option<FileMeta>, RuntimeError> {
        let config_file = match File::open(meta_file_path) {
            Ok(f) => f,
            Err(e) if e.kind() == ErrorKind::NotFound => {
                return Ok(None);
            }
            Err(e) => {
                return Err(RuntimeError::Message(format!("{}", e)));
            }
        };

        ason::from_reader(config_file)
            .map_err(|e| {
                RuntimeError::Message(match std::fs::read_to_string(meta_file_path) {
                    Ok(source_code) => e.with_source(&source_code),
                    Err(e) => format!("{}", e),
                })
            })
            .map(Some)
    }
}
