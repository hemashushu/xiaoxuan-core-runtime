# The Runtime (The `ancrt` Command)

## Runner

- `ancrt run </path/to/application> [args]...`
  Runs the specified application, which can be a:

  - Local file system path of module
  - Path of source file  (for single-file applications)
  - Path of a wrapped application image file

- `ancrt run </path/to/application:unit_name> [args]...`
  Runs the specified executable unit of an application.

- `ancrt run --unit unit_name </path/to/application> [args]...`
  Runs the specified executable unit of an application.

- `ancrt run [:unit_name] [args]...`
  Runs the application if the current directory is the root directory of a project.

## Creator

- `ancrt new [--type type] <module_name> [location]`
  Creates a new module project (application or shared library).
  TODO:: type:
  - lib (default)
  - app
  - mix
  - bins
  - script

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

- `ancrt test [path_name_prefix] [/path/to/module]`
  Runs unit tests for the current module. The `path_name_prefix` can be the name of a submodule (e.g. "client" for the namespace "tests::client"), or the partial path name of unit test functions (e.g. "client::test_get" for functions "tests::client::test_get*").

- `ancrt build [--tests] [/path/to/module]`
  Builds the binary image for the specified application or module. When building an application, all dependent modules and libraries will be automatically downloaded.
  TODO:: --tests, include unit tests.

- `ancrt clean [/path/to/module]`
  Remove artifacts that builder generated.

- `ancrt package [--strip] [/path/to/module]`
  TODO:: Build and seal the module with default parameters.
  TODO:: `--strip`  remove source files

<!--
- `ancrt wrap [--strip] [/path/to/application]`
  TODO:: Staticly link all dependencies except the build-in modules (the packaged application file can be executed directly by set the `execute` file bit on Linux, but it requires the `anc` is installed and set the binfmt_misc with `anc` )
-->

## Binutils

- `ancrt dump <object_file>`
  Displays the contents of a binary image (the type of image can be an application, shared module, and object file).

- `ancrt dump -s <section_name> <object_file>`
  Displays the contents of a specific section.

- `ancrt dump -f <function_name> <object_file>`
  Disassembles a specific function.

- `ancrt dump -d <data_name> <object_file>`
  Displays the contents of a specific data.

- `ancrt dump -l <object_file>`
  Lists sections in the image.

## Builtin utilities

- `ancrt env [name]`
  Print the compilation environment variables.

- `ancrt debug </path/to/application>`
  Debug the specified application.

### Utilities provided by the builtin applications `xiaoxuan-editor`, `xiaoxuan-shell` and `xiaoxuan-base-utils`

- `ancrt edit <file>`
  Builtin code editor.

- `ancrt shell [command-line]`
  Builtin shell.

- `ancrt command <command-line>`
  Builtin shell commands, such as mount, umount, export, ls, echo, cat, more, vi, mkdir, ln, pwd, rm, mv, cp, chown, chmod, date and exit.

## Self commands

- `ancrt me manpage [out_dir]`
  Generates man pages.

- `ancrt me completion [--shell shell] [out_dir]`
  Generates shell completion script.
  `--shell` TODO:: bash, zsh, fish, nushell, powershell
