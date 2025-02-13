# The Runtime (The `ancrt` Command)

## Runner

- `ancrt run [--registry <URLs>] </path/to/application> <args>`
  Runs the specified application, which can be a:

  - Local file system path of module
  - Source file path (for single-file applications)

- `ancrt run [--registry <URLs>] </path/to/application:unit_name> <args>`
  Runs the specified executable unit of an application.

- `ancrt run [--registry <URLs>] [:unit_name]`
  Runs the application if the current directory is the root directory of a project.

## Creator

- `ancrt new <module_name>`
  Creates a new module project (application or shared library).

- `ancrt new -f <file_name>`
  Creates a new single-file application.

<!--
- `ancrt dep add <module_name>[@<version>] [--registry <URL>]`
  Adds a dependent shared module to the current module.

  - `ancrt dep add --path <path>`
    Adds a dependent local module to the current module.
  - `ancrt dep add --remote <url> --revision <tag_or_commit>`
    Adds a dependent remote module to the current module.
  - `ancrt dep add --library <library_name>`
    Adds a dependent shared library to the current module.
-->

- `ancrt test [unit_test_path_name_prefix]`
  Runs unit tests for the current module. The `unit_test_path_name_prefix` can be the name of a submodule (e.g. "client" for the namespace "tests::client"), or the partial path name of a unit test functions (e.g. "client::test_get" for functions "tests::client::test_get*").

- `ancrt build [/path/to/module]`
  Builds the binary image for the specified application or module. When building an application, all dependent modules and libraries will be automatically downloaded.

- `ancrt clean [/path/to/module]`
  Remove artifacts that builder generated.

- `ancrt package [--strip] -o </path/to/generate> [/path/to/application]`
  TODO::
  - only can be apply to application (can not pack a shared module)
  - static-link all dependencies except the build-in modules

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

- `ancrt debug </path/to/application>`
  Debug the specified application.

### Utilities provided by the builtin modules `anwriter`, `ansh` and `ancoreutils`

- `ancrt edit <file>`
  Builtin code editor.

- `ancrt shell`
  Builtin shell.

- `ancrt command <command>`
  Builtin shell commands, such as mount, umount, export, ls, echo, cat, more, vi, mkdir, ln, pwd, rm, mv, cp, chown, chmod, date and exit.

## Others

- `ancrt [-h|--help]`
- `ancrt --version`
