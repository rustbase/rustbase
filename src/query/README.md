damn bruh, another query lang lol 💀
# Query Engine 🧑‍💻
Responsable to parse the RBQL (Rustbase Query Language) into a AST tree.

## Syntax
This language is not similar to SQL, but it is inspired by it.

The query has 5 main keywords: `insert`, `get`, `update`, `delete` and `list`.

### Insert
The `insert` keyword is used to insert some data into the database.

### Get
The `get` keyword is used to get some data from the database.

### Update
The `update` keyword is used to update some data in the database.

### Delete
The `delete` keyword is used to delete some data from the database.

### List
The `list` keyword is used to list keys from the database.

## Examples
```rbql
insert "some value" into some_key
```