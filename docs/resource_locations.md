# Resource Locations

## Anc ROOT path

### the user ANC ROOT path, managed by normal/unprivileged user

default: `~/.local/lib/anc`

- user builtin modules path:
  `~/.local/lib/anc/EDITION/runtime/modules`
- user builtin libraries path:
  `~/.local/lib/anc/EDITION/runtime/libraries`
- user modules path:
  `~/.local/lib/anc/EDITION/modules`
- user libraries path:
  `~/.local/lib/anc/EDITION/libraries`

### the global ANC ROOT path, managed by root/privileged user

default: `/usr/local/lib/anc`

- global builtin modules path:
  `/usr/local/lib/anc/EDITION/runtime/modules`
- global builtin libraries path:
  `/usr/local/lib/anc/EDITION/runtime/libraries`
- global modules path:
  `/usr/local/lib/anc/EDITION/modules`
- global libraries path:
  `/usr/local/lib/anc/EDITION/libraries`

### the system ANC ROOT path, managed by system package manager

default: `/usr/lib/anc`

- system builtin modules path:
  `/usr/lib/anc/EDITION/runtime/modules`
- system builtin libraries path:
  `/usr/lib/anc/EDITION/runtime/libraries`
- system modules path:
  `/usr/lib/anc/EDITION/modules`
- system libraries path:
  `/usr/lib/anc/EDITION/libraries`

> The primary intent of global and system-wide applications are to be run by all users, not for development.

<!--
### Examples

- builtin module:
  `{BUILTIN_MODULE_PATH}/http-client/{src, tests, output}`
- builtin library:
  `{BUILTIN_LIBRARY_PATH}/lz4/{src, lib, include}`
- general module:
  `{MODULE_PATH}/foo/1.0.1/{src, tests, output}`
- general library:
  `{LIBRARY_PATH}/bar/1.0.2/{src, lib, include}`
-->

### Runtime isolation

The user, global, and system runtimes are independent of each other, for example, a shared module installed globally is not available to the user's application.

In particular, system-wide runtimes, modules, and applications are managed by the system's package manager, and users (either normal or privileged) cannot create, install, update, or remove system-wide runtimes, modules, and applications through the XiaoXuan Core runtime or launcher.

## Repository index cache

`~/.local/lib/anc/repositories/{remote_git_repo_name_path}`

## Configuration files

- System and global: `/etc/anc/config.ason`
- User: `~/.local/lib/anc/config.ason`

Configuration files are inherited, i.e. the user-type runtime will read both configuration files above and then override the global one using the user's values.

## Installed application scripts and symbolic links

- User
  shell scripts folder:
  `~/.local/lib/anc/applicatons`
  symbol links:
  `~/.local/bin/{app_name}`

- Global
  shell scripts folder:
  `/usr/local/lib/anc/applications`
  symbol links:
  `/usr/local/bin`

- System
  scripts folder:
  `/usr/lib/anc/applications`
  symbol links:
  `/usr/bin`
