// Copyright (c) 2025 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE, LICENSE.additional and CONTRIBUTING.

use std::fmt::Display;

mod entry;
mod fetcher;
mod locations;
mod peekableiter;
mod source_scanner;

pub mod builder;
pub mod runner;

// files in the launcher_path/runtime_home folder
pub const FILE_NAME_DEFAULT_CONFIG: &str = "default.ason";
pub const FILE_NAME_USER_CONFIG: &str = "config.ason";

// folders in the launcher_path/runtime_home folder
pub const DIRECTORY_NAME_DATA_FOLDER: &str = ".anc";
pub const DIRECTORY_NAME_BIN: &str = "bin";
pub const DIRECTORY_NAME_RUNTIMES: &str = "runtimes";
pub const DIRECTORY_NAME_REGISTRIES: &str = "registries";
pub const DIRECTORY_NAME_REPOSITORIES: &str = "repositories";
pub const DIRECTORY_NAME_MODULES: &str = "modules";
pub const DIRECTORY_NAME_VERSION_REMOTE: &str = "remote";
pub const DIRECTORY_NAME_VERSION_LOCAL: &str = "local";

// source files
pub const FILE_EXTENSION_SOURCE: &str = "anc";
pub const FILE_EXTENSION_IR: &str = "ancr";
pub const FILE_EXTENSION_ASSEMBLY: &str = "anca";

// files in a module
pub const FILE_NAME_MODULE_CONFIG: &str = "module.anc.ason";

// folders in a module
pub const DIRECTORY_NAME_SRC: &str = "src";
pub const DIRECTORY_NAME_APP: &str = "app";
pub const DIRECTORY_NAME_TESTS: &str = "tests";
pub const DIRECTORY_NAME_OUTPUT: &str = "output";

// building asset - files
pub const FILE_EXTENSION_OBJECT: &str = "anco";
pub const FILE_EXTENSION_MODULE: &str = "ancm";
pub const FILE_EXTENSION_IMAGE: &str = "anci";
pub const FILE_EXTENSION_META: &str = "meta.ason";

// building asset - folders
pub const DIRECTORY_NAME_IR: &str = "ir";
pub const DIRECTORY_NAME_ASSEMBLY: &str = "assembly";
pub const DIRECTORY_NAME_OBJECT: &str = "object";
pub const DIRECTORY_NAME_ASSET: &str = "asset";

#[derive(Debug, PartialEq, Clone)]
pub enum RuntimeError {
    Message(String),
}

impl Display for RuntimeError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            RuntimeError::Message(msg) => f.write_str(msg),
        }
    }
}

impl std::error::Error for RuntimeError {}
