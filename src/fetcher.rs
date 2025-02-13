// Copyright (c) 2025 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions,
// more details in file LICENSE, LICENSE.additional and CONTRIBUTING.

use std::path::{Path, PathBuf};

use anc_isa::EffectiveVersion;

use crate::RuntimeError;

pub struct RemoteRepositoryResourceLocation {
    pub url: String,
    pub revision: String,
}

impl RemoteRepositoryResourceLocation {
    pub fn new(url: &str, revision: &str) -> Self {
        Self {
            url: url.to_owned(),
            revision: revision.to_owned(),
        }
    }
}

/// Checking each registry and find the repository URL and revision
/// for the specified name and version.
pub fn get_shared_module_remote_location(
    _registries: &[String],
    _module_name: &str,
    _module_version: &EffectiveVersion,
) -> Result<RemoteRepositoryResourceLocation, RuntimeError> {
    todo!()
}

/// Download a module from the specified remote Git repository
/// and save to `{anc_data_path}/repositories` folder.
pub fn fetch_module(
    _remote_repository_url: &str,
    _repositories_directory: &Path,
) -> Result</* the `{anc_data_path}/repositories/{hash}` directory*/ PathBuf, RuntimeError> {
    todo!()
}

/// Checkout the specified revision and save to the
/// `{anc_data_path}/modules` folder.
pub fn checkout_module(
    _repository_path: &Path,
    _revision: &str,
) -> Result</* the `{anc_data_path}/modules/{name}` directory */ PathBuf, RuntimeError> {
    todo!()
}
