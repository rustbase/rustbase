# Config manager ⚙️
This component has a function to manage the configuration of the application

# Files
 - [schema.rs](./schema.rs) - Contains the schema of the configuration
 - [spec.rs](./spec.rs) - Contains some constants and the specification of the configuration
 - [mod.rs](./mod.rs) - Contains the functions to manage the configuration

# Config file
The configuration file is a JSON file with the following structure:
```json
{
    "net": {
        "host": "0.0.0.0", // The host of the server
        "port": "23561" // The port of the server
    },
    "database": {
        "path": "./data", // Path to the database
        "cache_size": 134217728, // Size of the cache of the database in bytes
        "threads": 12 // Number of threads to use for the database
    },
    // The following fields are optional
    "auth": {
        "username": "", // Username of the admin
        "password": "" // Password of the admin
    },
    "tls": {
        "type": "", // This is enum, can be "Required" or "Optional"
        "ca_file": "", // Path to the certificate
        "pem_key_file": "" // Path to the private key
    }
}

```


# Environment variables
 - `RUSTBASE_CONFIG_FILE` - (optional) The path to the configuration file. If not specified, the default configuration will be used. The path is relative based on the current working directory.
 - `RUSTBASE_INIT_USER` - (optional) The username of the initial user. [1]
 - `RUSTBASE_INIT_PASS` - (optional) The password of the initial user. [1]


[1] If one of these two environment variables is not specified, the Rustbase will not use authentication.