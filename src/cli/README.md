# CLI
This component is responsible for the command line interface of the application.

## Modules
- [Snapshot](./snapshot.rs): This module is responsible for the snapshot command.
- [Upgrade](./upgrade.rs): This module is responsible for the upgrade command.

## Snapshot
### Creating a snapshot
To create a snapshot, run the following command:
```bash
$ rustbase_server snapshot create --db <database_name> --path <out-dir>
```

This will create a snapshot file on the specified path.

### Restoring a snapshot
To restore a snapshot, run the following command:
```bash
$ rustbase_server snapshot restore --db <database_name> --path <snapshot-file>
```

## Upgrade
### Upgrading the database

To upgrade the database, run the following command:
```bash
$ rustbase_server upgrade
```


### Upgrading with a specific version

To upgrade the database to a specific version, run the following command:
```bash
$ rustbase_server upgrade <version>
```

**To check all available versions see: https://github.com/rustbase/rustbase/releases (some versions only support certain platforms)**