// Copyright (c) 2025 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE, LICENSE.additional and CONTRIBUTING.

use std::path::Path;

use anc_isa::EffectiveVersion;

use crate::{entry::RuntimeConfig, RuntimeError};

pub struct RemoteLocation {
    pub url: String,
    pub revision: String,
}

impl RemoteLocation {
    pub fn new(url: &str, revision: &str) -> Self {
        Self {
            url: url.to_owned(),
            revision: revision.to_owned(),
        }
    }
}

/// Download a module from the specified remote Git repository.
pub fn download_module(
    remote_location: &RemoteLocation,
    output_directory: &Path,
) -> Result<(), RuntimeError> {
    todo!()
}

pub fn get_shared_module_remote_location(
    runtime_config: &RuntimeConfig,
    module_name: &str,
    module_version: &EffectiveVersion,
) -> Result<RemoteLocation, RuntimeError> {
    todo!()
}
