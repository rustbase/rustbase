# Config manager ⚙️
This component has a function to manage the configuration of the application

# Files
 - [schema.rs](./schema.rs) - Contains the schema of the configuration
 - [spec.rs](./spec.rs) - Contains some constants and the specification of the configuration
 - [mod.rs](./mod.rs) - Contains the functions to manage the configuration

# Core configuration
 - **threads**: The number of threads to use for the database 
 - **cache_size**: The size of the cache (in bytes)

# Network configuration
 - **host**: The host to bind the server to
 - **port**: The port to bind the server to
 - **tls**: The TLS configuration (see [below](#tls-configuration))

# Storage configuration
 - **path**: The path to the database file
 - **dustdata**: DustData configuration (see [below](#dustdata-configuration))

# Authentication configuration
 - **enable_auth_bypass**: Whether to enable authentication bypass
 - **auth_type**: The type of authentication to use (currently only `scram-sha-256` is supported)

# DustData configuration
 - **flush_threshold**: The number of writes to the database before flushing the data to disk

# TLS configuration
 - **ca_file**: The path to the certificate file
 - **pem_key_file**: The path to the key file