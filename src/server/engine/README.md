# Server Engine 🚂
The Rustbase Engine is responsable for work about queries, data, and the database itself. It is the core of the Rustbase project.

## Architecture
Rustbase Engine uses a similar architecture as [Nginx](https://www.nginx.com/).

The main process is the master process. It is responsible for managing the worker processes. The master process sends the worker
processes the tasks to do. The worker processes are responsible for executing the tasks. Including the queries, data and cache.