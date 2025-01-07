# Pathes

## Anc Home Path

### the user ANC HOME path, managed by unprivileged user

default: `~/.local/lib/anc`

- builtin modules path:
  `~/.local/lib/anc/EDITION/runtime/modules`
- builtin libraries path:
  `~/.local/lib/anc/EDITION/runtime/libraries`
- system modules path:
  `~/.local/lib/anc/EDITION/modules`
- system libraries path:
  `~/.local/lib/anc/EDITION/libraries`

### the global ANC HOME path, managed by root user

default: `/usr/local/lib/anc`

- builtin modules path:
  `/usr/local/lib/anc/EDITION/runtime/modules`
- builtin libraries path:
  `/usr/local/lib/anc/EDITION/runtime/libraries`
- system modules path:
  `/usr/local/lib/anc/EDITION/modules`
- system libraries path:
  `/usr/local/lib/anc/EDITION/libraries`

### the system ANC HOME path, managed by system package manager

default: `/usr/lib/anc`

- builtin modules path:
  `/usr/lib/anc/EDITION/runtime/modules`
- builtin libraries path:
  `/usr/lib/anc/EDITION/runtime/libraries`
- system modules path:
  `/usr/lib/anc/EDITION/modules`
- system libraries path:
  `/usr/lib/anc/EDITION/libraries`

### Examples

- builtin module:
  `{BUILTIN_MODULE_PATH}/http-client/{src, tests, output}`
- builtin library:
  `{BUILTIN_LIBRARY_PATH}/lz4/{src, lib, include}`
- general module:
  `{MODULE_PATH}/foo/1.0.1/{src, tests, output}`
- general library:
  `{LIBRARY_PATH}/bar/1.0.2/{src, lib, include}`

### Searching order

1. User home folder
2. Global home folder
3. System home folder

## Repository index cache

`~/.local/lib/anc/repositories/{hash-of-remote-git-repo}`

## Remote applications cache

cache the remote applications and modules

default:  `/tmp/anc`

## Configuration files

- `~/.local/lib/anc/config.ason`
- `/etc/anc/config.ason`

## Installed application scripts and symbolic links

- scripts folder:
  `/usr/lib/anc/applications/{app_name}`
  symbol links:
  `/usr/bin`

- scripts folder:
  `/usr/local/lib/anc/applicatons/{app_name}`
  symbol links:
  `/usr/local/bin`

- scripts folder:
  `~/.local/lib/anc/applicatons/{app_name}`
  symbol links:
  `~/.local/bin/{app_name}`
