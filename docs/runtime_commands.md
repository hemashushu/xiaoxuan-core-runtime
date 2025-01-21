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

- `ancrt add <module_name>[@<version>]`
  Adds a dependent shared module to the current module.

- `ancrt add --path <path>` and `ancrt add --git <url>`
  Adds a dependent local or remote module to the current module.

- `ancrt add -l <library_name>`
  Adds a dependent shared library to the current module.

- `ancrt test [unit_test_name]`
  Runs unit tests for the current module. The `unit_test_name` can be the name of a submodule (e.g. "client" for the namespace "tests::client", note that the name of the module does not need to be specified), or the path name of a unit test function (e.g. "client::test_get" for the function "tests::client::test_get).

- `ancrt build [path/to/module]`
  Builds the binary image for the specified application or module. When building an application, all dependent modules and libraries will be automatically downloaded.

- `ancrt clean [path/to/module]`
  Remove artifacts that builder generated.

## Binutils

- `ancrt dump <image>`
  Displays the contents of a binary image (the type of image can be an application, shared module, and object file).

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
