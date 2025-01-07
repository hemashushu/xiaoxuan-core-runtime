# Anc Home folder hierarchy

## Example

```text
ANC_HOME_FOLDER
|-- anc
|-- 2025
|   |-- runtime
|   |   |-- ancrt
|   |   |-- libraries
|   |   |   |-- lz4
|   |   |       |-- lib
|   |   |       |   |-- liblz4.so.1
|   |   |       |-- include
|   |   |       |-- share
|   |   |       |-- src
|   |   |
|   |   |-- modules
|   |       |-- std
|   |       |   |-- module.anc.ason
|   |       |   |-- src
|   |       |       |-- lib.anc
|   |       |
|   |       |-- http
|   |           |-- module.anc.ason
|   |           |-- src
|   |               |-- lib.anc
|   |
|   |-- libraries
|   |   |-- zlib
|   |   |   |-- 1.2
|   |   |       |-- include
|   |   |       |   |-- zlib.h
|   |   |       |-- lib
|   |   |       |   |-- libz.so.1 -> libz.so.1.2.13
|   |   |       |   |-- libz.so.1.2.13
|   |   |       |-- share
|   |   |       |-- src
|   |   |-- sqlite3
|   |       |--1.0
|   |         |-- include
|   |         |   |-- sqlite3.h
|   |         |-- lib
|   |         |   |-- libsqlite3.so.0 -> libsqlite3.so.0.8.6
|   |         |   |-- libsqlite3.so.0.8.6
|   |         |-- share
|   |-- modules
|       |-- sha2
|           |-- 1.0
|               |-- module.anc.ason
|               |-- lib.anc
|-- 2030
```
