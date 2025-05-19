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

        impl From<$success_enum> for std::result::Result<$success_enum, $error_enum> {
            fn from(success: $success_enum) -> Self {
                Ok(success)
            }
        }

        impl From<$error_enum> for std::result::Result<$success_enum, $error_enum> {
            fn from(error: $error_enum) -> Self {
                Err(error)
            }
        }

        pub struct $name {}

        impl<S> $crate::server::Route<S> for $name {
            type HandlerOutput = std::result::Result<$success_enum, $error_enum>;
            type HandlerInput = $handler_input;
            type RequestArgs = $request_args;
            type ResponseType = std::result::Result<$success_enum, $error_enum>;
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
}
