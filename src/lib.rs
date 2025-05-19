pub mod client;
pub mod server;

#[macro_export]
macro_rules! define_route {
    (
        $name:ident,
        $method:ident,
        $route:expr,
        $request_args:ty,
        $handler_input:ty,
        $result_type:ident,
        $success_enum:ident {
            $(
                $success_variant:ident $({ $($success_field:ident: $success_field_ty:ty),* $(,)? })? => $success_status:expr
            ),*
            $(,)?
        },
        $error_enum:ident {
            $(
                $error_variant:ident => $error_status:expr
            ),*
            $(,)?
        }
    ) => {
        pub type $result_type = std::result::Result<$success_enum, $error_enum>;

        #[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
        pub enum $success_enum {
            $(
                $success_variant $({ $($success_field: $success_field_ty),* })?
            ),*
        }

        #[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
        pub enum $error_enum {
            $($error_variant),*
        }

        impl std::fmt::Display for $error_enum {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match self {
                    $(
                        Self::$error_variant => write!(f, stringify!($error_variant))
                    ),*
                }
            }
        }

        impl std::error::Error for $error_enum {}

        impl $crate::server::IntoTypedResponse for $result_type {
            fn typed(self) -> $crate::server::TypedResponse<Self> {
                match self {
                    Ok(result) => match result {
                        $(
                            $success_enum::$success_variant $({ $($success_field),* })? => $crate::server::TypedResponse {
                                status_code: $success_status,
                                payload: Ok($success_enum::$success_variant $({ $($success_field),* })?),
                            }
                        ),*
                    },
                    Err(err) => match err {
                        $(
                            $error_enum::$error_variant => $crate::server::TypedResponse {
                                status_code: $error_status,
                                payload: Err($error_enum::$error_variant),
                            }
                        ),*
                    },
                }
            }
        }

        pub struct $name {}

        impl<S> $crate::server::Route<S> for $name {
            type HandlerOutput = $result_type;
            type HandlerInput = $handler_input;
            type RequestArgs = $request_args;
            type ResponseType = $result_type;
            const METHOD: $crate::server::HttpMethod = $crate::server::HttpMethod::$method;

            fn route() -> String {
                $route.to_string()
            }
        }
    };
}

#[cfg(test)]
mod tests {
    #[derive(Debug, serde::Serialize, serde::Deserialize)]
    pub struct GetMsgQueryArgs {
        pub num: u32,
        pub prefix: String,
    }
    define_route!(
        GetMsg,
        Get,
        "/get_msg",
        GetMsgQueryArgs,
        (axum::extract::State<S>, axum::extract::Query<GetMsgQueryArgs>),
        GetMsgResult,
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
        SetMsgResult,
        SetMsgSuccess {
            Done { new_msg: String } => reqwest::StatusCode::CREATED,
            SuperGood => reqwest::StatusCode::ACCEPTED
        },
        SetMsgError {
            NotFound => reqwest::StatusCode::NOT_FOUND,
            OtherError => reqwest::StatusCode::INTERNAL_SERVER_ERROR
        }
    );
}
