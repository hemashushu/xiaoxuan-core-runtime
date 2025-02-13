# Application Name Path

An application name path is applied to both the `run` and `install/add` command.

## For the `run` command

### Application on local file system

Application on the current directory:

```sh
anc run
anc run :unitname
anc run --unit unitname
```

Application on a specific path:

```sh
anc run /path/to/module
anc run /path/to/module:unitname
anc run --unit unitname /path/to/module
```

Path can be either an absolute path, or a relative path, e.g. `~/under/homefolder/module`, `./subfolder/module`, `.`.

### Applications on registry

```sh
anc run modname
anc run modname:unitname
anc run --unit unitname modname
```

e.g., TODO

With specified version:

```sh
anc run modname@x.y.z
anc run --version x.y.z modname
```

With both executable unit and version:

```sh
anc run modname:unitname@x.y.z
anc run --unit unitname --version x.y.z modname
```

With specific registry:

```sh
anc run registry.domain/path:modname
anc run registry.domain/path:modname:unitname@x.y.z
anc run --registry registry.domain/path --unit unitname --version x.y.z modname
```

e.g., TODO

### Applications on remote host

With remote URL:

```sh
anc run --remote https://host.domain/path
anc run --remote --unit unitname --revision tag_or_commit https://host.domain/path
```

e.g., TODO

### Applications have been already installed

```sh
anc run modname

anc run modname@x.y.z
anc run modname:unitname@x.y.z

anc run modname@remote
anc run modname:unitname@remote

anc run modname@local
anc run modname:unitname@local
```

An application can be installed multiple versions from registry, as well as two special version: a `remote` which is installed from a remote URL, and a `local` which is installed from a local file system folder. When execute `run` command without specific version, e.g. `anc run hello`, the launcher will find and try to run the `local` version first, and then the `remote` one if there is no `local`, and at last, the latest version if there is no `remote` either.

(TODO:: auto install)

## For the `install/add` command

TODO::
