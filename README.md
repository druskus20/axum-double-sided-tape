# axum-double-sided-tape

This is a weird contraption that provides a way to define an API shared between
a client and a server, type checked at compile time.

The API is defined in terms of `Route`s. A client is automatically generated
which can create requests for each Route. On the server side, the trait
`RouteAdder` provides an abstraction to automatically add routes to an
`axum::Router`.

All of this is done with the appropiate HTTP methods, and the responses from
the server are automatically parsed into the right types.

The name of the crate is intentionally bad, as you probably should not use it.
It is tailored to my specific use case - so it sacrifices some flexibility.

## Limitations

- Like in axum - there is no check for duplicate routes.
- Like in axum - there is no check that all routes are handled in the server.
- All responses are in JSON format. (limitation that arises from having the
client automatically parse the responses)
- GET parameters are in the the form of a URL query. POST parameters are sent
in the body as JSON.
- The API crate cannot check that the signature of a Route is a valid signature
for an Axum handler, but the compilation will fail when trying to implement the
server.


## Example

```rs
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct GetMsgQueryArgs {
    pub num: u32,
    pub prefix: String,
}

define_route!(
    GetMsg, // <-- Route
    Get,    // <-- Method
    "/get_msg", // <-- URL
    // Request arguments (client)
    GetMsgQueryArgs, 
    // Axum handler input (server) 
    (axum::extract::State<S>, axum::extract::Query<GetMsgQueryArgs>), 
    // Success and error responses
    GetMsgSuccess {
        Done { new_msg: String } => reqwest::StatusCode::CREATED,
        SuperGood => reqwest::StatusCode::ACCEPTED
    },
    GetMsgError {
        NotFound => reqwest::StatusCode::NOT_FOUND,
        OtherError => reqwest::StatusCode::INTERNAL_SERVER_ERROR
    }
);

define_route!(
    SetMsg,
    Post,
    "/set_msg",
    String,
    (axum::extract::State<S>, axum::extract::Json<String>),
    SetMsgSuccess {
        Done { new_msg: String } => reqwest::StatusCode::CREATED,
        SuperGood => reqwest::StatusCode::ACCEPTED
    },
    SetMsgError {
        NotFound => reqwest::StatusCode::NOT_FOUND,
        OtherError => reqwest::StatusCode::INTERNAL_SERVER_ERROR
    }
);
```
