use axum::{
    Json, Router,
    handler::Handler,
    response::IntoResponse,
    routing::{get, post},
};
use reqwest::StatusCode;
use serde::Serialize;

pub trait ToStatusCode {
    fn to_status_code(&self) -> StatusCode;
}

#[derive(Debug)]
pub enum HttpMethod {
    Post,
    Get,
}

pub struct TypedResponse<T> {
    pub status_code: StatusCode,
    pub payload: T,
}

pub trait IntoTypedResponse
where
    Self: Sized,
{
    fn typed(self) -> TypedResponse<Self>;
}

impl<T> IntoResponse for TypedResponse<T>
where
    T: Serialize,
{
    fn into_response(self) -> axum::response::Response {
        let status_code = self.status_code;
        (status_code, Json(self.payload)).into_response()
    }
}

/// Craft a Typed response from things that can be converted to a Result
impl<T, E> IntoTypedResponse for Result<T, E>
where
    T: Into<Result<T, E>>,
    E: Into<Result<T, E>>,
{
    fn typed(self) -> TypedResponse<Self> {
        match self {
            Ok(value) => TypedResponse {
                status_code: StatusCode::OK,
                payload: value.into(),
            },
            Err(err) => TypedResponse {
                status_code: StatusCode::INTERNAL_SERVER_ERROR,
                payload: err.into(),
            },
        }
    }
}

pub trait Route<S> {
    type HandlerOutput: IntoTypedResponse;
    type HandlerInput;
    type RequestArgs: Serialize;
    type ResponseType; // TODO: might not be needed now
    const METHOD: HttpMethod;

    fn route() -> String;
}

pub trait RouteAdder<S> {
    fn add_route<R, T, TH>(self, handler: TH) -> Self
    where
        // here we bind together the route and the handler's types
        R: Route<S>,
        TH: TypedHandler<R::HandlerInput, TypedResponse<R::HandlerOutput>>,
        TH: Handler<T, S>,
        T: 'static,
        S: Clone + Send + Sync + 'static;
}

impl<S> RouteAdder<S> for Router<S> {
    fn add_route<R, T, TH>(self, handler: TH) -> Self
    where
        R: Route<S>,
        TH: TypedHandler<R::HandlerInput, TypedResponse<R::HandlerOutput>>,
        TH: Handler<T, S>,
        T: 'static,
        S: Clone + Send + Sync + 'static,
    {
        let method = match R::METHOD {
            HttpMethod::Post => post,
            HttpMethod::Get => get,
        };

        self.route(&R::route(), method(handler))
    }
}

/// Handler with Input and Output type introspection
pub trait TypedHandler<I, O> {}

#[rustfmt::skip]
macro_rules! all_the_tuples {
    ($name:ident) => {
        $name!([], T1);
        $name!([T1], T2);
        $name!([T1, T2], T3);
        $name!([T1, T2, T3], T4);
        $name!([T1, T2, T3, T4], T5);
        $name!([T1, T2, T3, T4, T5], T6);
        $name!([T1, T2, T3, T4, T5, T6], T7);
        $name!([T1, T2, T3, T4, T5, T6, T7], T8);
        $name!([T1, T2, T3, T4, T5, T6, T7, T8], T9);
        $name!([T1, T2, T3, T4, T5, T6, T7, T8, T9], T10);
        $name!([T1, T2, T3, T4, T5, T6, T7, T8, T9, T10], T11);
        $name!([T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11], T12);
        $name!([T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12], T13);
        $name!([T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13], T14);
        $name!([T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14], T15);
        $name!([T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15], T16);
    };
}

macro_rules! impl_my_trait {
    ([$($Ts:ident),*], $Last:ident) => {
        impl<F, Fut, $($Ts,)* $Last, Ret> TypedHandler<($($Ts,)* $Last,), Ret> for F
        where
            F: FnOnce($($Ts,)* $Last) -> Fut,
            Fut: Future<Output = Ret> + Send,
        {}
    };
}

all_the_tuples!(impl_my_trait);
