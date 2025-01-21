# The Launcher (The `anc` Command)

## Application Management

- `anc search <application_name>`
  Search the specified application from the center repository.

- `anc install [--global] <application>`
  - Installs an application from center repository, a local project folder, single source file, or remote Git URL.
  - Creates a shell script and symbolic link for quick launching.
  - Add `--global` parameter to install to system instead of the current user.
  - The runtime specified by the application will be installed automatically if it does not exist.

- `anc list [--global]`
  List installed applications.

- `anc update [--global]`
  Updates all installed applications.

- `anc uninstall|remove [--global] <application_name>`
  Removes an installed application.

- `anc prune [--global]`
  Removes unused modules or versions.

- `anc download <module> [-o <path>]`
  Download the source of application or module to the current directory or to the specified directory.

## Runtime Management

- `anc runtime list [--installed]`
  List all available runtime's edition and version.

- `anc runtime install [--global] <edition>`
  Add the specified edition runtime.

- `anc runtime update [--global]`
  Updates all installed runtimes.

- `anc runtime default <edition>`
  Set the default edition.

- `anc runtime uninstall|remove [--global] <edition>`
  Removes an installed runtime edition.

## Other

- `anc self update`
  Updates the `anc` command itself.

## Redirecting

All other sub-commands will be redirected to `ancrt`. e.g.

`anc run hello` executes `ancrt run hello`, and `anc new myproject` executes `ancrt new myproject`.
