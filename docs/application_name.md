# Application Name

Application name is applied to `anc` command, such as `anc run` and `anc install`.

## For the `run` command

### Application on local file system

Application on the current directory:

```sh
anc run
anc run :unit_name
anc run --unit unit_name
```

Application on a specific path:

```sh
anc run /path/to/module
anc run /path/to/module:unit_name
anc run --unit unit_name /path/to/module
```

Path can be either an absolute path, or a relative path, e.g. `~/under/homefolder/module`, `./subfolder/module`, `.`.

### Applications on registry

```sh
anc run module_name
anc run module_name:unit_name
anc run --unit unit_name module_name
```

e.g., TODO

With specified version:

```sh
anc run module_name@x.y.z
anc run --version x.y.z module_name
```

With both executable unit and version:

```sh
anc run module_name:unit_name@x.y.z
anc run --unit unit_name --version x.y.z module_name
```

With specific registry:

```sh
anc run registry.domain/path:module_name
anc run registry.domain/path:module_name:unit_name@x.y.z
anc run --registry registry.domain/path --unit unit_name --version x.y.z module_name
```

e.g., TODO

### Applications on remote host

With remote URL:

```sh
anc run --remote https://host.domain/path
anc run --remote --unit unit_name --revision tag_or_commit https://host.domain/path
```

e.g., TODO

### Application image files

```sh
anc run /path/to/image_name.ancp
anc run /path/to/image_name.ancp:unit_name
anc run --unit unit_name /path/to/image_name.ancp
```

TODO

### Installed applications

```sh
anc run module_name

anc run module_name@x.y.z
anc run module_name:unit_name@x.y.z

anc run module_name@remote
anc run module_name:unit_name@remote

anc run module_name@local
anc run module_name:unit_name@local
```

An application can be installed multiple versions from registry, as well as two special version: a `remote` which is installed from a remote URL, and a `local` which is installed from a local file system folder. When execute `run` command without specific version, e.g. `anc run hello`, the launcher will find and try to run the `local` version first, and then the `remote` one if there is no `local`, and at last, the latest version if there is no `remote` either.

(TODO:: auto install)

## For the `install/add` command

TODO::
