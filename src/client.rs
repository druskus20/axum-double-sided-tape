use reqwest::{RequestBuilder, Response};
use serde::{Serialize, de::DeserializeOwned};

use crate::server::{HttpMethod, Route};

pub trait TryFromHttpResonse
where
    Self: Sized,
{
    fn try_from_http_response(
        r: Response,
    ) -> impl std::future::Future<Output = Result<Self, reqwest::Error>> + Send;
}

// TryFromHttpResonse for values marked as Json<T>
impl<T> TryFromHttpResonse for T
where
    T: DeserializeOwned,
{
    async fn try_from_http_response(r: Response) -> Result<Self, reqwest::Error> {
        r.json().await
    }
}

pub struct Client {
    base_http_url: String,
    http_client: reqwest::Client,
}

impl Client {
    pub fn new(base_url: &str) -> Self {
        let base_http_url = format!("http://{base_url}");
        Self {
            base_http_url,
            http_client: reqwest::Client::new(),
        }
    }

    pub fn request_for<R: Route<()>>(&self) -> TransformRequest<R> {
        TransformRequest {
            url: format!("{}{}", self.base_http_url, R::route()),
            phantom: std::marker::PhantomData,
            http_client: self.http_client.clone(),
        }
    }
}

pub struct TransformRequest<R> {
    url: String,
    phantom: std::marker::PhantomData<R>,
    http_client: reqwest::Client,
}

impl<R> TransformRequest<R>
where
    R: Route<()>,
{
    pub fn with_args(self, args: R::RequestArgs) -> ReadyRequest<R, R::ResponseType> {
        ReadyRequest {
            url: self.url,
            args,
            phantom: std::marker::PhantomData,
            http_client: self.http_client,
        }
    }
}

pub struct ReadyRequest<R, RT>
where
    R: Route<()>,
{
    url: String,
    args: R::RequestArgs,
    phantom: std::marker::PhantomData<RT>,
    http_client: reqwest::Client,
}

impl<R, RT> ReadyRequest<R, RT>
where
    R: Route<(), ResponseType = RT>,
    RT: TryFromHttpResonse,
    R::RequestArgs: Serialize,
{
    pub async fn send(self) -> Result<RT, reqwest::Error> {
        let client = self.http_client;

        let method = match R::METHOD {
            HttpMethod::Post => reqwest::Client::post,
            HttpMethod::Get => reqwest::Client::get,
        };

        let args = match R::METHOD {
            HttpMethod::Post => RequestBuilder::json,
            HttpMethod::Get => RequestBuilder::query,
        };

        let request = args(method(&client, self.url), &self.args);

        let response = request.send().await?;
        dbg!(&response);

        R::ResponseType::try_from_http_response(response).await
    }
}
