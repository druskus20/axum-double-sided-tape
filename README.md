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

The API crate can define the routes:

```rs
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct CreateUserArgs {
    pub name: String,
    pub surname: String,
}

define_route!(
    CreateUser,
    Get,
    "/create_user",
    CreateUserArgs,
    (State<S>, Json<CreateUserArgs>),
    CreateUserResponse {
        UserCreated { id: Uuid } => StatusCode::CREATED,
        AlreadyExists => StatusCode::NOT_FOUND,
    }
);

define_route!(
    GetUsers,
    Get,
    "/get_users",
    (),
    (State<S>,),
    GetUsersResponse {
        Users { users: Vec<User> } => StatusCode::OK,
    }
);
```

The server can implement the routes like this:

```rs

pub async fn create_user(
    State(state): State<AppState>,
    Json(CreateUserArgs { name, surname }): Json<CreateUserArgs>,
) -> TypedResponse<CreateUserResponse> {

    // ... do your normal axum stuff here

    CreateUserResponse::UserCreated { id }.typed()
}
```

The client can make requests like this:

```rs
let client = Client::new("localhost:8080");
let r = client
    .request_for::<CreateUser>()
    .with_args(CreateUserArgs { name, surname })
    .send()
    .await?;
  

let r = match r {
    CreateUserResponse::UserCreated { id } => id,
    CreateUserResponse::AlreadyExists => return Err(eyre!("User already exists")),
};

println!("User created with id: {r}");
```

