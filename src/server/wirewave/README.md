# Wirewave
Wirewave is a protocol to Rustbase communicates with the client. It is a binary protocol, and is designed to be as fast as possible.

## Request
Each request is a BSON document with the following fields:
- `body`: The body of the message. This is a BSON document.

## Response
Each response is a BSON document with the following fields:
- `body`: The body of the message. This is a BSON document.
- `message`: The message to send to the client. This is a string.
- `status`: The status of the response. This is a enum with the following values:
  - `Ok`: The request was successful.
  - `Error`: The request failed.
  - `DatabaseNotFound`: The database was not found.
  - `KeyNotFound`: The key was not found.
  - `KeyAlreadyExists`: The key already exists.
  - `SyntaxError`: The syntax of the request was invalid.
  - `InvalidRequest`: The request was invalid.
  - `InvalidBody`: The body of the request was invalid.
