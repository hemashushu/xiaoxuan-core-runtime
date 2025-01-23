# Anc Root Folder Hierarchy

## Framework

```text
ANC_ROOT_DIRECTORY
  |-- anc                   # The launcher executable file
  |-- 2025                  # Edition
  |   |-- runtime           # Runtime directory
  |   |   |-- ancrt         # The runtime executable file
  |   |   |-- libraries     # Builtin libraries directory
  |   |   |-- modules       # Builtin modules directory
  |   |
  |   |-- libraries         # User libraries directory
  |   |-- modules           # User modules directory
```

## User Modules Example

```text
-- modules
   |-- sha2
       |-- 1.0.0                # a specific version
       |   |-- module.anc.ason  # module manifest
       |   |-- src              # source code directory
       |
       |-- noversion            # modules installed from local or remote URL
           |-- module.anc.ason
           |-- src
```

## User Libraries Example

```text
-- libraries
   |-- zlib
   |   |-- 1.2.13
   |       |-- include
   |       |   |-- zlib.h
   |       |-- lib
   |       |   |-- libz.so.1 -> libz.so.1.2.13
   |       |   |-- libz.so.1.2.13
   |       |-- share
   |       |-- src
   |
   |-- sqlite3
       |--0.8.6
         |-- include
         |   |-- sqlite3.h
         |-- lib
         |   |-- libsqlite3.so.0 -> libsqlite3.so.0.8.6
         |   |-- libsqlite3.so.0.8.6
         |-- share
```

## Builtin Modules Example

```text
-- modules
   |-- std
   |   |-- module.anc.ason
   |   |-- src
   |       |-- lib.anc
   |
   |-- http
       |-- module.anc.ason
       |-- src
           |-- lib.anc
```

## Builtin Libraries Example

```text
-- libraries
   |-- lz4
       |-- lib
       |   |-- liblz4.so.1
       |-- include
       |-- share
       |-- src
```
