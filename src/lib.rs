// Copyright (c) 2025 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE, LICENSE.additional and CONTRIBUTING.

use std::fmt::Display;

pub mod builder;
pub mod creator;
pub mod dumpper;
pub mod entry;
pub mod common;
pub mod runner;

pub const MODULE_CONFIG_FILE_NAME: &str = "module.anc.ason";
pub const MODULE_DIRECTORY_NAME_SRC: &str = "src";
pub const MODULE_DIRECTORY_NAME_APP: &str = "app";
pub const MODULE_DIRECTORY_NAME_TESTS: &str = "tests";

pub const FILE_EXTENSION_SOURCE: &str = "anc";
pub const FILE_EXTENSION_IR: &str = "ancr";
pub const FILE_EXTENSION_ASSEMBLY: &str = "anca";
pub const FILE_EXTENSION_OBJECT: &str = "anco";
pub const FILE_EXTENSION_MODULE: &str = "ancm";
pub const FILE_EXTENSION_IMAGE: &str = "anci";

pub const DIRECTORY_NAME_OUTPUT: &str = "output";
pub const DIRECTORY_NAME_IR: &str = "ir";
pub const DIRECTORY_NAME_ASSEMBLY: &str = "assembly";
pub const DIRECTORY_NAME_OBJECT: &str = "object";

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
