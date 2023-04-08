# Wirewave
Wirewave is a request-response style protocol that allows clients communicate with Rustbase Database Server thought a regular TCP connection.
It is a binary protocol that is designed to be easy to implement in any programming language. It is also designed to be easy to implement in Rustbase itself.

## Protocol
Wirewave uses BSON to encode messages. BSON is a binary format that is similar to JSON.

## Requests
Each request must be a BSON document with the following fields:
-   `auth` - A basic authentication string. This is used to authenticate the client. (This can be empty if the server is not configured to require authentication.)
-   `body` - The body of the message. This is a BSON document.

## Response
Each response must be a BSON document with the following fields:

-   `body` - The body of the message. This is a BSON document and can be null.
-   `error` - The message to send to the client. This is a string and can be null.
-   `status` - The status of the response. This is a enum with the following values:
    - `Ok` - The request was successful.
    - `Error` - The request failed.
    - `NotFound` - The requested resource was not found.
    - `AlreadyExists` - The requested resource already exists.
    - `SyntaxError` - The request was malformed.
    - `InvalidQuery` - The query was invalid.
    - `InvalidBody` - The body was invalid.
    - `InvalidBson` - The BSON was invalid.
    - `InvalidAuth` - The authentication was invalid.
    - `NotAuthorized` - The client is not authorized to perform the requested action.
    - `Reserved` - Cannot be used.