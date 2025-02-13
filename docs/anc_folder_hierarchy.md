# ANC Folder Hierarchy

## Overview

`anc_initial_path`, can be any path, it is determined by the anc and ancrt, in general it is `~/.anc` or `/usr/lib/anc`.

```text
anc_initial_path
  |
  |-- anc.ason          # initial configuration file
  |
  |-- bin
  |   |-- anc           # launcher
  |
  |-- runtimes          # built-in runtimes
  |   |-- 2025          # edition
  |   |   |-- ancrt     # the runtime executable file
  |   |   |-- modules   # built-in modules
  |   |
  |   |-- 2030          # another edition
  |   |   |-- ...
  |
```

`anc_data_path`, can be configurated by `anc.ason`, it is also can be the same as `anc_initial_path`.

```text
anc_data_path
  |
  |-- config.ason       # runtime configuration file
  |
  |-- bin
  |   |-- hello         # script file for launcher specific ANC application
  |   |-- ...
  |
  |-- runtimes          # new-added and updated runtimes
  |   |-- 2025          # the structure is the same as the builtin one
  |   |   |-- ancrt
  |   |   |-- modules
  |   |
  |   |-- 2030
  |       |-- ...
  |
  |-- registries
  |   |-- hash1         # module index, a Git repo, e.g. github.com/anc_registry/index
  |   |-- hash2         # another module index, a Git repo, e.g. gitlab.com/anc_registry/index
  |   |-- ...
  |
  |-- repositories
  |   |-- hash1         # module repository comes from registry index, a Git repo, e.g. https://github.com/username/module_name
  |   |-- hash2         # module repository comes from remote URL, a Git repo, e.g. https://host.domain/username/module_name
  |   |-- ...
  |
  |-- modules           # checkout the specific revision from the repository in the "repositories" folder and copy to this folder.
      |-- name1
      |-- name2
      |-- ...
```

## General Modules Example

```text
anc_data_path
  |
  |-- modules
  |   |-- name
  |   |   |-- local                  # module installed from local file system folder
  |   |   |   |-- module.anc.ason    # module manifest
  |   |   |   |-- src                # source code directory
  |   |   |   |-- output
  |   |   |
  |   |   |-- remote                 # module installed from remote URL
  |   |   |   |-- module.anc.ason
  |   |   |   |-- ...
  |   |   |
  |   |   |-- version1               # a specific version
  |   |   |   |-- module.anc.ason
  |   |   |   |-- ...
  |   |   |
  |   |   |-- version2
  |   |   |   |-- module.anc.ason
  |   |   |   |-- ...
  |   |   |
  |   |   |-- ...
```

## Builtin Modules Example

```text
anc_data_path
  |-- runtimes
  |   |-- 2025
  |   |   |-- modules
  |   |   |   |-- std
  |   |   |   |   |-- module.anc.ason
  |   |   |   |   |-- src
  |   |   |   |       |-- lib.anc
  |   |   |   |
  |   |   |   |-- http
  |   |   |       |-- module.anc.ason
  |   |   |       |-- src
  |   |   |           |-- lib.anc
```

<!-- TODO
## General Libraries Example

```text
ancc_data_path
  |
  |-- modules
  |   |-- zlib
  |   |   |-- 1.2.13
  |   |       |-- include
  |   |       |   |-- zlib.h
  |   |       |-- output
  |   |       |   |-- libz.so.1 -> libz.so.1.2.13
  |   |       |   |-- libz.so.1.2.13
  |   |       |-- share
  |   |       |-- src
  |   |
  |   |-- sqlite3
  |       |-- 0.8.6
  |           |-- include
  |           |   |-- sqlite3.h
  |           |-- output
  |           |   |-- libsqlite3.so.0 -> libsqlite3.so.0.8.6
  |           |   |-- libsqlite3.so.0.8.6
  |           |-- share
```
-->

<!-- TODO
## Builtin Libraries Example

```text
anc_data_path
  |-- runtimes
  |   |-- 2025
  |   |   |-- libraries
  |   |   |   |-- lz4
  |   |   |       |-- output
  |   |   |       |   |-- liblz4.so.1
  |   |   |       |-- include
  |   |   |       |-- share
  |   |   |       |-- src
```
-->

## ANC home folder details

### Runtime isolation

The user, global, and system runtimes are independent of each other, for example, a shared module installed globally is not available to the user's application.

In particular, system-wide runtimes, modules, and applications are managed by the system's package manager, and users (either normal or privileged) cannot create, install, update, or remove system-wide runtimes, modules, and applications through the XiaoXuan Core runtime or launcher.

### Configuration files

- initial configuration: `{anc_initial_path}/anc.ason`
- user configuration: `{anc_data_path}/config.ason`

TODO

### Installed application quick launching scripts

{anc_data_path}/bin/{app_name}

The content of script file:

```sh
#!/bin/sh
/home/USERNAME/.anc/runtimes/2025/ancrt APP_NAME
```
