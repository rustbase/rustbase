# Wirewave
Wirewave is a request-response style protocol that allows clients communicate with Rustbase Database Server thought a regular TCP connection.
It is a binary protocol that is designed to be easy to implement in any programming language. It is also designed to be easy to implement in Rustbase itself.

## Protocol
Wirewave uses BSON to encode messages. BSON is a binary format that is similar to JSON.

## Requests
Each request must be a BSON document with the following fields:

-   `body` - The body of the message. This is a BSON document.

## Response
Each response must be a BSON document with the following fields:

-   `body` - The body of the message. This is a BSON document and can be null.
-   `error` - The message to send to the client. This is a string and can be null.
-   `status` - The status of the response. This is a enum with the following values:
    -   `Ok` - The request was successful.
    -   `Error` - The request failed.
    -   `DatabaseNotFound` - The database was not found.
    -   `KeyNotFound` - The key was not found.
    -   `KeyAlreadyExists` - The key already exists.
    -   `SyntaxError` - The syntax of the request was invalid.
    -   `InvalidRequest` - The request was invalid.
    -   `InvalidBody` - The body of the request was invalid.