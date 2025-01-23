# The `output` Folder

<!-- @import "[TOC]" {cmd="toc" depthFrom=2 depthTo=4 orderedList=false} -->

<!-- code_chunk_output -->

<!-- /code_chunk_output -->

An `output` folder is created to hold the object files and images after an application or shared module is compiled. A typical `output` folder are as follows:

```text
MODULE_FOLDER
  |-- module.anc.ason
  |-- src
  |-- app
  |-- tests
  |-- output
      |-- name.anci                     # application binary image
      |-- hash0
      |   |-- name.ancm                 # shared module binary image
      |   |-- asset
      |       |-- module.anc.meta.ason
      |       |-- object                # object files
      |       |   |-- lib.anco
      |       |   |-- lib.meta.ason
      |       |   |-- submodule.anco
      |       |   |-- submodule.meta.ason
      |       |
      |       |-- assembly              # assembly files
      |       |-- ir                    # IR files
      |
      |-- hash1
      |-- hash2
```

Note that for shared modules, multiple `hash` folders are generated because their object files vary depending on parameters and compilation environment variables. However, shared modules of type `Runtime` have a all zero `hash` folder, because runtime builtin modules are pre-compiled and their parameters cannot be customized.

Also note that single-file application do not have `output` folder, because it is recompiled (in memory) each time the application is started.
