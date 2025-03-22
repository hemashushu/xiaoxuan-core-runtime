# The Launcher (The `anc` Command)

The launcher is user-scope runtimes manager and applications manager.

## Application Management

- `anc search [--registry <URLs>] <application_name>`
  Search the specified application from the central registry.

- `anc install [--registry <URLs>] <application_name>`
  - Installs an application from central registry, a local project folder, single source file, packaged module file, or remote Git URL.
  - Creates a shell script for quick launching.

- `anc run [--registry <URLs>] <application_name>`
  TODO

- `anc list`
  List installed applications.

- `anc info [--registry <URLs>] <application_name>`
  Shows information for both remote and local installed application.

- `anc update`
  Updates registry cache.

- `anc upgrade [application_name]`
  Upgrade all installed applications.

- `anc uninstall <application_name>`
  Removes an installed application.

- `anc gc`
  Removes unused modules or versions.

- `anc fetch [--registry <URLs>] <application_name> [-o <path>]`
  Download the source of application or module to the current directory or to the specified directory.

## Registry Management

- `anc registry [list]`

- `anc registry add <URL>`

- `anc registry remove <URL>`

- `anc registry top <URL>`

## Runtime Management

- `anc runtime list`
  List all available runtimes.

- `anc runtime add <edition>`
  Add the specified edition runtime.

- `anc runtime update [edition]`
  Updates all installed runtimes.

- `anc runtime remove <edition>`
  Removes an installed runtime edition.

## Self commands

- `anc me update`
  Updates the `anc` program itself.

- `ancrt [-h|--help]`
- `ancrt --version`

- `anc me manpage`
  Generates man pages.

- `anc me completion`
  Generates shell completion script.

## Runtime Command Redirecting

Sub-commands such as `new`, `test`, and `dump` etc. will be redirected to latest version of `ancrt`. e.g. `anc new myproject` executes `/path/to/ancrt new myproject`.
