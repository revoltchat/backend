# Revolt Errors

Revolt's backend has a single error type which is used throughout the project, `revolt_result::Error`, all routes and general functions which can error should use this error when possible.

Authifier which handles the auth has its own error type `authifier::Error` as well but this should only be used when nessessary.

## `revolt_result`

`revolt_result` contains many useful functions and macros to make writing errors easier:

### `Error`

A wrapper around `ErrorType` which contains the error and the location of where it came from.

### `ErrorType`

An enum containing every type of error along with the data which the error is about.

### `create_error!`

A simple macro to quickly generate `Error`s along with the location, this macro takes an `ErrorType` variant by name. This should be your go to for generating error types.

```rust
create_error!(LabelMe)
```

### `create_database_error!`

Another error generating macro for database errors, util macro to fill the database op and collection.

```rust
create_database_error!("find", "users")
```

### `ToRevoltError`

This trait is implemented on `Option<T>` and `Result<T, E>` to more easily maniplate the error

#### `capture_error`

This method sends the error to sentry if the type contains an error.

No-op if the `sentry` feature is not enabled.

#### `to_internal_error`

This method is useful for when your working with 3rd party crates to easily wrap their errors, this should only be used when the error
is an internal error and not a user error, only use this for errors which should not happen under normal operation.

```rs
// converts it to `revolt_result::Error`, if it errors propegates the error as an internal error and reports it to sentry.
let value = my_3rd_party_crate_func().to_internal_error()?;
```

## Returning errors in Rocket request guards

Due to a Rocket.rs limitation their is no way to capture the error returned inside `FromRequest` and other request guards, to get around this
we store the error inside the request local cache via `request.local_cache(|| create_error!(LabelMe))`.

This is then picked up by the custom rocket `catcher`s to then display back to the user, it is important to do this otherwise the error might respond with either a html response or a rocket-style json error which cannot be read correctly by users of the api.