# The Launcher (The `anc` Command)

The launcher is user-scope runtimes manager and applications manager.

## Application Management

- `anc search [--registry <URLs>] <application_name>`
  Search the specified application from the central registry.

- `anc install|add [--registry <URLs>] <application_name_path>`
  - Installs an application from central registry, a local project folder, single source file, or remote Git URL.
  - Creates a shell script for quick launching.

- `anc run [--registry <URLs>] <application_name>`
  TODO

- `anc list`
  List installed applications.

- `anc update`
  Updates all installed applications.

- `anc uninstall|remove <application_name>`
  Removes an installed application.

- `anc gc`
  Removes unused modules or versions.

- `anc download [--registry <URLs>] <application_name_path> [-o <path>]`
  Download the source of application or module to the current directory or to the specified directory.

## Runtime Management

- `anc runtime list [--installed]`
  List all available runtime's edition and version.

- `anc runtime add <edition>`
  Add the specified edition runtime.

- `anc runtime update [edition]`
  Updates all installed runtimes.

- `anc runtime remove <edition>`
  Removes an installed runtime edition.

## Runtime Command Redirecting

Sub-commands such as `new`, `test`, and `dump` etc. will be redirected to latest version of `ancrt`. e.g.

- `anc new myproject`
  Executes `/path/to/ancrt new myproject`.

## Other

- `anc self update`
  Updates the `anc` command itself.
