# The Runtime (The `ancrt` Command)

## Runner

- `ancrt run <application> <args>`

  Runs the specified application, which can be a:

  - Module name
  - Project folder
  - Source file path (for single-file applications)
  - Remote Git repository URL

- `ancrt run <application.unit> <args>`

  Runs the specified executable unit of an application.

## Creator

- `ancrt new <module_name>`
  Creates a new module project (application or shared library).

- `ancrt new -f <file_name>`
  Creates a new single-file application.

- `ancrt dep add <module_name>`
  Adds a dependent module to the current module.

- `ancrt dep add -l <library_name>`
  Adds a dependent library to the current module.

- `ancrt run .`
  Runs the current module if it's an application.

- `ancrt test`
  Runs unit tests for the current module.

- `ancrt test <unit_test_path_name>`
  Runs the specified unit test. (e.g., `math::test_add`, `http_client::test_get`)

## Builder

- `ancrt build <path/to/application>`
  Builds the binary image for the specified application or module.

- `ancrt build -r <path/to/source>`
  Builds the intermediate representation (IR) (*.ancr) for the source file.

- `ancrt build -a <path/to/source>`
  Builds the assembly code (*.anca) for the source file or IR.

- `ancrt build -o <path/to/source>`
  Builds the object file (*.anco) for the source file, IR or assembly.

- `ancrt build -m <path/to/object_file...>`
  Links the object files to a module image (*.ancm).

- `ancrt build -i <path/to/module_image>`
  Builds the application image (*.anci) from a module image.

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
