damn bruh, another query lang lol 💀
# Query Engine 🧑‍💻
Responsable to parse the RBQL (Rustbase Query Language) into a AST tree.

# RBQL (Rustbase Query Language) 🧑‍🎓
## Inserting data
You can insert data into a database using the keyword `insert` statement and the given data.
RBQL support JSON data format and the data must be a valid JSON object.

```rbql
insert {
    "name": "John Doe",
    "email": "johndoe@example.com",
} into customer_0
```

The data will be inserted into the database with the given key (`customer_0`).

## Getting data
You can get data from a database using the keyword `get` statement and the given key.

```rbql
get customer_0
```

The data will be returned as a JSON object.

## Updating data
You can update data from a database using the keyword `update` statement and the given key and data.

```rbql
update {
    "name": "John Doe",
    "email": "another@example.com",
} into customer_0
```

The data will be updated into the database with the given key (`customer_0`).

## Deleting data
You can delete data from a database using the keyword `delete` statement and the given key.

```rbql
delete customer_0
```

The data will be deleted from the database with the given key (`customer_0`).

## Variables
You can use variables to store data and use it later.

```rbql
name = "John Doe";
email = "johndoe@example.com";
```


### Getting data from variables
If you want to get data from a variable, you can use the `$` symbol before the variable name.
```rbql
name = "John Doe";
get $name;


# Deleting the variable
customer = "customer_0";

delete $customer;
```

### Reassigning variables
```rbql
name = "John Doe";
name = "Another Name";
```