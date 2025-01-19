// Copyright (c) 2025 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE, LICENSE.additional and CONTRIBUTING.

use std::collections::HashMap;

use anc_isa::{ExternalLibraryDependency, ModuleDependency};
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct ModuleConfig {
    pub name: String,
    pub version: String,
    pub edition: String,

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

    /// Optional
    /// the default value is []
    #[serde(default)]
    pub module_repositories: HashMap<String, String>,

    /// Optional
    /// the default value is []
    #[serde(default)]
    pub library_repositories: HashMap<String, String>,
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

    #[serde(rename = "eval")]
    Eval(String),
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct RuntimeConfig {
    /// Default: `~/.local/lib/anc/repositories`
    ///
    /// the index cache directory of a specific repository
    /// would be `{repositories_index_cache_directory}/{remote_git_repo_name_path}`
    pub repositories_index_cache_directory: String,

    /// Optional
    /// the default value is []
    #[serde(default)]
    pub module_repositories: HashMap<String, String>,

    /// Optional
    /// the default value is []
    #[serde(default)]
    pub library_repositories: HashMap<String, String>,
}