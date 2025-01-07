# The Launcher (The `anc` Command)

## Application Management

- `anc search <application_name>`
  Search the specified application from the center repository.

- `anc install <application>`
  - Installs an application from center repository, a local project folder, single source file, remote Git repository.
  - Creates a shell script and symbolic link for quick launching.

- `anc list`
  List installed applications.

- `anc update/upgrade [application_name]`
  Updates all installed applications or a specified one.

- `anc uninstall/remove <application_name>`
  Removes an installed application.

- `anc download <application> [-o <path>]`
  Download the source of application.

## Runtime Management

- `anc runtime all`
  List all available runtime's edition and version.

- `anc runtime update [edition]`
  Initializes the user environment, downloading the latest runtime.

- `anc runtime list`
  Lists installed runtime versions.

- `anc runtime default <edition>`
  Set the default edition.

- `anc runtime remove <version_number>`
  Removes an installed runtime version.

## Other

- `anc self-upgrade`
  Updates the `anc` command itself.

## Redirecting

All other sub-commands will be redirected to `ancrt`. e.g.

`anc run helloworld` executes `ancrt run helloworld`, and `anc new myproject` executes `ancrt new myproject`.
