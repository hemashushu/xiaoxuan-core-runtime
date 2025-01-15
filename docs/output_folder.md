# The `output` Folder

<!-- @import "[TOC]" {cmd="toc" depthFrom=2 depthTo=4 orderedList=false} -->

<!-- code_chunk_output -->



<!-- /code_chunk_output -->

An `output` folder is created to hold the object files and images after an application or shared module is compiled. A typical `output` folder are as follows:

```text
module_source_folder
|-- module.anc.ason
|-- src
|-- app
|-- tests
|-- output
    |-- application_image.anci
    |-- hash0
    |   |-- shared_module_image.ancm
    |   |-- asset
    |       |-- module.anc.meta.ason
    |       |-- object
    |       |   |-- lib.anco
    |       |   |-- lib.meta.ason
    |       |   |-- submodule.anco
    |       |   |-- submodule.meta.ason
    |       |
    |       |-- assembly
    |       |-- ir
    |
    |-- hash1
    |-- hash2
```

Note that for shared modules, multiple `hash` folders are generated because their object files vary depending on parameters and compilation environment variables. However, shared modules of type `Runtime` have a all zero `hash` folder, because runtime builtin modules are pre-compiled and their parameters cannot be customized.

Also note that single-file application do not have `output` folder, because it is recompiled (in memory) each time the application is started.
