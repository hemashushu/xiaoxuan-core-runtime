# Module Folder Hierarchy

```text
MODULE_FOLDER
  |-- module.anc.ason       # module (package/project) manifest
  |-- README.md
  |-- LICENSE.md etc.
  |-- src
  |   |-- lib.anca          # top-level submodule
  |   |-- main.anca         # the default executable unit
  |   |-- foo.anca          # submodule
  |   |-- subfolder
  |       |-- bar.anca      # submodule under the subfolder
  |
  |-- app
  |   |-- cmd1.anca         # sub-executable unit
  |   |-- cmd2.anca         # sub-executable unit
  |
  |-- tests                 # unit test directory
  |   |-- test_name1.anca   # testing unit
  |   |-- test_name2.anca   # testing unit
  |   |-- subfolder
  |       |-- bar.anca      # submodule for unit testing only
  |
  |-- doc
  |   |-- README.md         # documentations
  |
  |-- output                # the building assets
```
