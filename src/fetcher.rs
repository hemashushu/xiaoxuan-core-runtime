// Copyright (c) 2025 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE, LICENSE.additional and CONTRIBUTING.

use anc_image::entry::ImportModuleEntry;

use crate::RuntimeError;

/// Download the specified module from center repository or remote Git repository.
pub fn download_module(import_module_entry:&ImportModuleEntry) -> Result<(), RuntimeError> {
    todo!()
}