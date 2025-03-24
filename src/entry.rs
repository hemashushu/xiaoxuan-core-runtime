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
    DIRECTORY_NAME_REPOSITORIES, DIRECTORY_NAME_RUNTIMES, FILE_NAME_DEFAULT_CONFIG,
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
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename = "prop")]
pub enum PropertyValue {
    #[serde(rename = "string")]
    String(String),

    #[serde(rename = "number")]
    Number(i64),

    #[serde(rename = "flag")]
    Flag(bool),
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
pub struct DefaultConfig {
    // the index of runtime executables
    #[serde(default)]
    pub runtime_registries: Vec<String>,

    // the index of modules
    #[serde(default)]
    pub registries: Vec<String>,

    #[serde(default)]
    pub runtime_home: String,
}

impl DefaultConfig {
    fn new() -> Self {
        // default:
        // - "https://github.com/hemashushu/anc_runtime_registry"
        // - "https://gitlab.com/hemashushu/anc_runtime_registry"
        let runtime_registries = vec![
            "https://github.com/hemashushu/anc_runtime_registry".to_owned(),
            "https://gitlab.com/hemashushu/anc_runtime_registry".to_owned(),
        ];

        // default:
        // - "https://github.com/hemashushu/anc_module_registry"
        // - "https://gitlab.com/hemashushu/anc_module_registry"
        let registries = vec![
            "https://github.com/hemashushu/anc_module_registry".to_owned(),
            "https://gitlab.com/hemashushu/anc_module_registry".to_owned(),
        ];

        // default: `~/.anc`
        let runtime_home = "~/.anc".to_owned();

        Self {
            runtime_registries,
            runtime_home,
            registries,
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
        let registries = vec![];
        Self { registries }
    }

    pub fn load(_filepath: &Path) -> Result<Self, RuntimeError> {
        let default = Self::new();
        // todo
        Ok(default)
    }
}

pub struct RuntimeProperty {
    /// default `~/.anc`
    pub runtime_home: PathBuf,

    /// e.g.
    /// - `{launcher_path}/runtimes/2025`
    /// - `{runtime_home}/runtimes/2025`
    pub runtime_path: PathBuf,

    /// the index of modules
    pub registries: Vec<String>,
}

impl RuntimeProperty {
    pub fn from_runtime_exec_file(/* extra_registries: &[String] */) -> Result<Self, RuntimeError> {
        // e.g.
        // - `{launcher_path}/runtimes/2025/ancrt`
        // - `{runtime_home}/runtimes/2025/ancrt`
        let mut exec_path = std::env::current_exe().unwrap();
        exec_path.pop(); // EDITION

        // the location of the current runtime execuable file
        let runtime_path = exec_path.clone();

        // find the default configuration file
        exec_path.pop(); // now in folder `runtimes`
        exec_path.pop(); // now in folder `{launcher_path}` or `{runtime_home}`
        let launcher_path = exec_path.clone();
        let default_config_path = launcher_path.join(FILE_NAME_DEFAULT_CONFIG);

        // the default config file only exists in the `{launcher_path}`
        let (default_config, runtime_home) = if default_config_path.exists() {
            // the runtime exec file is located in the `{launcher_path}` directory.
            let default_config = DefaultConfig::load(&default_config_path)?;
            let runtime_home = PathBuf::from(&default_config.runtime_home)
                .resolve_in(&launcher_path)
                .to_path_buf();
            (default_config, runtime_home)
        } else {
            // the runtime exec file is located in the `{runtime_home}` directory.
            (DefaultConfig::new(), launcher_path.clone())
        };

        let user_config_path = runtime_home.join(FILE_NAME_USER_CONFIG);
        let user_config =  if user_config_path.exists() {
            RuntimeConfig::load(&user_config_path)?
        }else {
            RuntimeConfig::new()
        };

        let mut registries = user_config.registries.clone();

        for registry in &default_config.registries {
            if !registries.iter().any(|item| item == registry) {
                registries.push(registry.to_owned());
            }
        }

        // for registry in extra_registries {
        //     if !registries.iter().any(|item| item == registry) {
        //         registries.push(registry.to_owned());
        //     }
        // }

        let runtime_property = RuntimeProperty {
            runtime_home,
            runtime_path,
            registries,
        };

        Ok(runtime_property)
    }

    pub fn from_custom(runtime_path: &Path, runtime_home: &Path) -> Self {
        let registries = vec![
            "https://github.com/hemashushu/anc_module_registry".to_owned(),
            "https://gitlab.com/hemashushu/anc_module_registry".to_owned(),
        ];

        let runtime_property = RuntimeProperty {
            runtime_home: runtime_home.to_path_buf(),
            runtime_path: runtime_path.to_path_buf(),
            registries,
        };

        runtime_property
    }

    // `{runtime_home}/bin`
    pub fn get_bin_directory(&self) -> PathBuf {
        self.runtime_home.join(DIRECTORY_NAME_BIN)
    }

    // `{runtime_home}/runtimes`
    pub fn get_runtimes_directory(&self) -> PathBuf {
        self.runtime_home.join(DIRECTORY_NAME_RUNTIMES)
    }

    // `{runtime_home}/registries`
    pub fn get_registries_directory(&self) -> PathBuf {
        self.runtime_home.join(DIRECTORY_NAME_REGISTRIES)
    }

    // `{runtime_home}/repositories`
    pub fn get_repositories_directory(&self) -> PathBuf {
        self.runtime_home.join(DIRECTORY_NAME_REPOSITORIES)
    }

    // `{runtime_home}/modules`
    pub fn get_modules_directory(&self) -> PathBuf {
        self.runtime_home.join(DIRECTORY_NAME_MODULES)
    }

    // `{launcher_path}/runtimes/EDITION/modules`
    pub fn get_builtin_modules_directory(&self) -> PathBuf {
        self.runtime_path.join(DIRECTORY_NAME_MODULES)
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
