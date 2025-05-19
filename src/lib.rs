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
        $response_enum:ident {
            $(
                $variant:ident $({ $($field:ident: $field_ty:ty),* $(,)? })? => $status:expr
            ),*
            $(,)?
        }
    ) => {
        #[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
        pub enum $response_enum {
            $(
                $variant $({ $($field: $field_ty),* })?
            ),*
        }

        impl $crate::server::IntoTypedResponse for $response_enum {
            fn typed(self) -> $crate::server::TypedResponse<Self> {
                match &self {
                    $(
                        $response_enum::$variant $({ $($field),* })? => $crate::server::TypedResponse {
                            status_code: $status,
                            payload: self,
                        },
                    )*
                }
            }
        }

        pub struct $name {}

        impl<S> $crate::server::Route<S> for $name {
            type HandlerOutput = $response_enum;
            type HandlerInput = $handler_input;
            type RequestArgs = $request_args;
            type ResponseType = Self::HandlerOutput;
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
            SuperGood => reqwest::StatusCode::ACCEPTED,
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
        }
    );
}
