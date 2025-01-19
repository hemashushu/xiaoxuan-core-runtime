# The Runtime (The `ancrt` Command)

## Runner

- `ancrt run <application> <args>`
  Runs the specified application, which can be a:

  - Module name
  - Project folder
  - Source file path (for single-file applications)
  - Remote Git URL

- `ancrt run <application.unit_name> <args>`
  Runs the specified executable unit of an application.

- `ancrt run [.unit_name]`
  Runs the application if the current directory is the root directory of a project.

## Creator

- `ancrt new <module_name>`
  Creates a new module project (application or shared library).

- `ancrt new -f <file_name>`
  Creates a new single-file application.

- `ancrt mod add <module_name>`
  Adds a dependent module to the current module.

- `ancrt lib add <library_name>`
  Adds a dependent library to the current module.

- `ancrt test [unit_test_name]`
  Runs unit tests for the current module. The `unit_test_name` can be the name of a submodule (e.g. "client" for the namespace "tests::client", note that the name of the module does not need to be specified), or the path name of a unit test function (e.g. "client::test_get" for the function "tests::client::test_get).

- `ancrt build <path/to/application>`
  Builds the binary image for the specified application or module. When building an application, all dependent modules and libraries will be automatically downloaded.

<!--
- `ancrt build -r <path/to/source>`
  Builds the intermediate representation (IR) (*.ancr) for the source file.

- `ancrt build -a <path/to/source>`
  Builds the assembly code (*.anca) for the source file or IR.

- `ancrt build -o <path/to/source>`
  Builds the object file (*.anco) for the source file, IR or assembly.

- `ancrt build -m <path/to/object_file...>`
  Links the object files to a module image (*.ancm).

- `ancrt build -i <path/to/module_image>`
  Generates the application image (*.anci) from the main module image.
-->

## Binutils

- `ancrt dump <image>`
  Displays the contents of a binary image (the type of image can be an application, shared module, and object file), the image can be a local file, a remote module, or a remote application.

- `ancrt dump -s <section_name> <image>`
  Displays the contents of a specific section.

- `ancrt dump -f <function_name> <image>`
  Disassembles a specific function.

- `ancrt dump -d <data_name> <image>`
  Displays the contents of a specific data.

- `ancrt dump -h <image>`
  Lists sections in the image.

## Builtin utilities

- `ancrt debug <application>`
  Debug the specified application.

Utilities provided by the builtin module `anwriter`, `ansh`, and `ancoreutils`.

- `ancrt edit <file>`
  Builtin code editor.

- `ancrt shell`
  Builtin shell.

- `ancrt command <command>`
  Builtin shell commands, such as mount, umount, export, ls, echo, cat, more, vi, mkdir, ln, pwd, rm, mv, cp, chown, chmod, date and exit.
