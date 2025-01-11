# The `output` Folder

<!-- @import "[TOC]" {cmd="toc" depthFrom=2 depthTo=4 orderedList=false} -->

<!-- code_chunk_output -->



<!-- /code_chunk_output -->

An `output` folder is created to hold the object files and images after an application or shared module is compiled. A typical `output` folder are as follows:

```text
module_folder
|
|-- module.anc.ason
|-- src
|-- output
    |-- hash0
    |   |-- objects
    |   |   |-- lib.anco
    |   |   |-- submodule0.anco
    |   |
    |   |-- name.ancm
    |
    |-- hash1
        |-- objects
        |   |-- lib.anco
        |   |-- submodule0.anco
        |
        |-- name.ancm

```

Note that for shared modules, multiple `hash` folders are generated because their object files vary depending on parameters and compilation environment variables. However, shared modules of type `Runtime` have a all zero `hash` folder, because runtime builtin modules are pre-compiled and their parameters cannot be customized.

Also note that single-file application do not have `output` folder, because it is recompiled (in memory) each time the application is started.
