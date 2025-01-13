# The Launcher (The `anc` Command)

## Application Management

- `anc search <application_name>`
  Search the specified application from the center repository.

- `anc install [--global] <application>`
  - Installs an application from center repository, a local project folder, single source file, remote Git repository.
  - Creates a shell script and symbolic link for quick launching.
  - Add `--global` parameter to install to the _global modules path_.
  - The runtime specified by the application will be installed automatically if it does not exist.

- `anc list [--global]`
  List installed applications.

- `anc update [--global] [application_name]`
  Updates all installed applications or a specified one.

- `anc uninstall|remove [--global] <application_name>`
  Removes an installed application.

- `anc download <application> [-o <path>]`
  Download the source of application.

## Runtime Management

- `anc runtime list [--installed]`
  List all available runtime's edition and version.

- `anc runtime install [--global] <edition>`
  Add the specified edition runtime.

- `anc runtime update [--global] [edition]`
  Download the latest runtime.

- `anc runtime default <edition>`
  Set the default edition.

- `anc runtime uninstall|remove [--global] <edition>`
  Removes an installed runtime edition.

## Other

- `anc self-upgrade`
  Updates the `anc` command itself.

## Redirecting

All other sub-commands will be redirected to `ancrt`. e.g.

`anc run helloworld` executes `ancrt run helloworld`, and `anc new myproject` executes `ancrt new myproject`.
