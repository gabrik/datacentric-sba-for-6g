use async_trait::async_trait;
use futures::{
    future, future::BoxFuture, future::FutureExt, future::TryFutureExt, stream, stream::StreamExt,
    Stream,
};
use hyper::header::{HeaderName, HeaderValue, CONTENT_TYPE};
use hyper::{service::Service, Body, Request, Response, Uri};
use percent_encoding::{utf8_percent_encode, AsciiSet};
use std::borrow::Cow;
use std::convert::TryInto;
use std::error::Error;
use std::fmt;
use std::future::Future;
use std::io::{ErrorKind, Read};
use std::marker::PhantomData;
use std::path::Path;
use std::str;
use std::str::FromStr;
use std::string::ToString;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll};
use swagger::{ApiError, AuthData, BodyExt, Connector, DropContextService, Has, XSpanIdString};
use url::form_urlencoded;

use crate::header;
use crate::models;

/// https://url.spec.whatwg.org/#fragment-percent-encode-set
#[allow(dead_code)]
const FRAGMENT_ENCODE_SET: &AsciiSet = &percent_encoding::CONTROLS
    .add(b' ')
    .add(b'"')
    .add(b'<')
    .add(b'>')
    .add(b'`');

/// This encode set is used for object IDs
///
/// Aside from the special characters defined in the `PATH_SEGMENT_ENCODE_SET`,
/// the vertical bar (|) is encoded.
#[allow(dead_code)]
const ID_ENCODE_SET: &AsciiSet = &FRAGMENT_ENCODE_SET.add(b'|');

use crate::CallbackApi;
use crate::OnScpDomainRoutingInformationChangePostResponse;

/// A client that implements the API by making HTTP calls out to a server.
pub struct Client<S, C>
where
    S: Service<(Request<Body>, C), Response = Response<Body>, Error = hyper::Error>
        + Clone
        + Send
        + Sync,
    S::Future: Send + 'static,
    C: Clone + Send + Sync + 'static,
{
    /// Inner service
    client_service: S,

    /// Marker
    marker: PhantomData<fn(C)>,
}

impl<S, C> fmt::Debug for Client<S, C>
where
    S: Service<(Request<Body>, C), Response = Response<Body>, Error = hyper::Error>
        + Clone
        + Send
        + Sync,
    S::Future: Send + 'static,
    C: Clone + Send + Sync + 'static,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Client")
    }
}

impl<S, C> Clone for Client<S, C>
where
    S: Service<(Request<Body>, C), Response = Response<Body>, Error = hyper::Error>
        + Clone
        + Send
        + Sync,
    S::Future: Send + 'static,
    C: Clone + Send + Sync + 'static,
{
    fn clone(&self) -> Self {
        Self {
            client_service: self.client_service.clone(),
            marker: PhantomData,
        }
    }
}

impl<Connector, C> Client<DropContextService<hyper::client::Client<Connector, Body>, C>, C>
where
    Connector: hyper::client::connect::Connect + Clone + Send + Sync + 'static,
    C: Clone + Send + Sync + 'static,
{
    /// Create a client with a custom implementation of hyper::client::Connect.
    ///
    /// Intended for use with custom implementations of connect for e.g. protocol logging
    /// or similar functionality which requires wrapping the transport layer. When wrapping a TCP connection,
    /// this function should be used in conjunction with `swagger::Connector::builder()`.
    ///
    /// For ordinary tcp connections, prefer the use of `new_http`, `new_https`
    /// and `new_https_mutual`, to avoid introducing a dependency on the underlying transport layer.
    ///
    /// # Arguments
    ///
    /// * `connector` - Implementation of `hyper::client::Connect` to use for the client
    pub fn new_with_connector(connector: Connector) -> Self {
        let client_service = hyper::client::Client::builder().build(connector);
        let client_service = DropContextService::new(client_service);

        Self {
            client_service,
            marker: PhantomData,
        }
    }
}

impl<C> Client<DropContextService<hyper::client::Client<hyper::client::HttpConnector, Body>, C>, C>
where
    C: Clone + Send + Sync + 'static,
{
    /// Create an HTTP client.
    pub fn new_http() -> Self {
        let http_connector = Connector::builder().build();
        Self::new_with_connector(http_connector)
    }
}

#[cfg(any(target_os = "macos", target_os = "windows", target_os = "ios"))]
type HttpsConnector = hyper_tls::HttpsConnector<hyper::client::HttpConnector>;

#[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "ios")))]
type HttpsConnector = hyper_openssl::HttpsConnector<hyper::client::HttpConnector>;

impl<C> Client<DropContextService<hyper::client::Client<HttpsConnector, Body>, C>, C>
where
    C: Clone + Send + Sync + 'static,
{
    /// Create a client with a TLS connection to the server.
    #[cfg(any(target_os = "macos", target_os = "windows", target_os = "ios"))]
    pub fn new_https() -> Result<Self, native_tls::Error> {
        let https_connector = Connector::builder().https().build()?;
        Ok(Self::new_with_connector(https_connector))
    }

    /// Create a client with a TLS connection to the server.
    #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "ios")))]
    pub fn new_https() -> Result<Self, openssl::error::ErrorStack> {
        let https_connector = Connector::builder().https().build()?;
        Ok(Self::new_with_connector(https_connector))
    }

    /// Create a client with a TLS connection to the server, pinning the certificate
    ///
    /// # Arguments
    /// * `ca_certificate` - Path to CA certificate used to authenticate the server
    #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "ios")))]
    pub fn new_https_pinned<CA>(ca_certificate: CA) -> Result<Self, openssl::error::ErrorStack>
    where
        CA: AsRef<Path>,
    {
        let https_connector = Connector::builder()
            .https()
            .pin_server_certificate(ca_certificate)
            .build()?;
        Ok(Self::new_with_connector(https_connector))
    }

    /// Create a client with a mutually authenticated TLS connection to the server.
    ///
    /// # Arguments
    /// * `ca_certificate` - Path to CA certificate used to authenticate the server
    /// * `client_key` - Path to the client private key
    /// * `client_certificate` - Path to the client's public certificate associated with the private key
    #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "ios")))]
    pub fn new_https_mutual<CA, K, D>(
        ca_certificate: CA,
        client_key: K,
        client_certificate: D,
    ) -> Result<Self, openssl::error::ErrorStack>
    where
        CA: AsRef<Path>,
        K: AsRef<Path>,
        D: AsRef<Path>,
    {
        let https_connector = Connector::builder()
            .https()
            .pin_server_certificate(ca_certificate)
            .client_authentication(client_key, client_certificate)
            .build()?;
        Ok(Self::new_with_connector(https_connector))
    }
}

impl<S, C> Client<S, C>
where
    S: Service<(Request<Body>, C), Response = Response<Body>, Error = hyper::Error>
        + Clone
        + Send
        + Sync,
    S::Future: Send + 'static,
    C: Clone + Send + Sync + 'static,
{
    /// Constructor for creating a `Client` by passing in a pre-made `swagger::Service`
    ///
    /// This allows adding custom wrappers around the underlying transport, for example for logging.
    pub fn new_with_client_service(client_service: S) -> Self {
        Client {
            client_service,
            marker: PhantomData,
        }
    }
}

#[async_trait]
impl<S, C> CallbackApi<C> for Client<S, C>
where
    S: Service<(Request<Body>, C), Response = Response<Body>, Error = hyper::Error>
        + Clone
        + Send
        + Sync,
    S::Future: Send + 'static,
    S::Error: Into<crate::ServiceError> + fmt::Display,
    C: Has<XSpanIdString> + Has<Option<AuthData>> + Clone + Send + Sync,
{
    fn poll_ready(&self, cx: &mut Context) -> Poll<Result<(), crate::ServiceError>> {
        match self.client_service.clone().poll_ready(cx) {
            Poll::Ready(Err(e)) => Poll::Ready(Err(Box::new(e))),
            Poll::Ready(Ok(o)) => Poll::Ready(Ok(o)),
            Poll::Pending => Poll::Pending,
        }
    }

    async fn on_scp_domain_routing_information_change_post(
        &self,
        callback_request_body_callback_uri: String,
        param_content_encoding: Option<String>,
        param_scp_domain_routing_info_notification: Option<
            models::ScpDomainRoutingInfoNotification,
        >,
        context: &C,
    ) -> Result<OnScpDomainRoutingInformationChangePostResponse, ApiError> {
        let mut client_service = self.client_service.clone();
        let mut uri = format!(
            "{request_body_callback_uri}",
            request_body_callback_uri = callback_request_body_callback_uri
        );

        // Query parameters
        let query_string = {
            let mut query_string = form_urlencoded::Serializer::new("".to_owned());
            query_string.finish()
        };
        if !query_string.is_empty() {
            uri += "?";
            uri += &query_string;
        }

        let uri = match Uri::from_str(&uri) {
            Ok(uri) => uri,
            Err(err) => return Err(ApiError(format!("Unable to build URI: {}", err))),
        };

        let mut request = match Request::builder()
            .method("POST")
            .uri(uri)
            .body(Body::empty())
        {
            Ok(req) => req,
            Err(e) => return Err(ApiError(format!("Unable to create request: {}", e))),
        };

        // Body parameter
        let body = param_scp_domain_routing_info_notification
            .map(|ref body| serde_json::to_string(body).expect("impossible to fail to serialize"));

        if let Some(body) = body {
            *request.body_mut() = Body::from(body);
        }

        let header = "application/json";
        request.headers_mut().insert(
            CONTENT_TYPE,
            match HeaderValue::from_str(header) {
                Ok(h) => h,
                Err(e) => {
                    return Err(ApiError(format!(
                        "Unable to create header: {} - {}",
                        header, e
                    )))
                }
            },
        );

        let header = HeaderValue::from_str(Has::<XSpanIdString>::get(context).0.as_str());
        request.headers_mut().insert(
            HeaderName::from_static("x-span-id"),
            match header {
                Ok(h) => h,
                Err(e) => {
                    return Err(ApiError(format!(
                        "Unable to create X-Span ID header value: {}",
                        e
                    )))
                }
            },
        );

        // Header parameters
        match param_content_encoding {
            Some(param_content_encoding) => {
                request.headers_mut().append(
                    HeaderName::from_static("content-encoding"),
                    #[allow(clippy::redundant_clone)]
                    match header::IntoHeaderValue(param_content_encoding.clone()).try_into() {
                        Ok(header) => header,
                        Err(e) => {
                            return Err(ApiError(format!(
                                "Invalid header content_encoding - {}",
                                e
                            )));
                        }
                    },
                );
            }
            None => {}
        }

        let response = client_service
            .call((request, context.clone()))
            .map_err(|e| ApiError(format!("No response received: {}", e)))
            .await?;

        match response.status().as_u16() {
            204 => {
                let response_accept_encoding = match response
                    .headers()
                    .get(HeaderName::from_static("accept-encoding"))
                {
                    Some(response_accept_encoding) => {
                        let response_accept_encoding = response_accept_encoding.clone();
                        let response_accept_encoding =
                            match TryInto::<header::IntoHeaderValue<String>>::try_into(
                                response_accept_encoding,
                            ) {
                                Ok(value) => value,
                                Err(e) => {
                                    return Err(ApiError(format!(
                                    "Invalid response header Accept-Encoding for response 204 - {}",
                                    e
                                )));
                                }
                            };
                        Some(response_accept_encoding.0)
                    }
                    None => None,
                };

                Ok(
                    OnScpDomainRoutingInformationChangePostResponse::ExpectedResponseToASuccessfulCallbackProcessing
                    {
                        accept_encoding: response_accept_encoding,
                    }
                )
            }
            400 => {
                let body = response.into_body();
                let body = body
                    .into_raw()
                    .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                    .await?;
                let body = str::from_utf8(&body)
                    .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body = serde_json::from_str::<models::ProblemDetails>(body).map_err(|e| {
                    ApiError(format!("Response body did not match the schema: {}", e))
                })?;
                Ok(OnScpDomainRoutingInformationChangePostResponse::BadRequest(
                    body,
                ))
            }
            401 => {
                let body = response.into_body();
                let body = body
                    .into_raw()
                    .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                    .await?;
                let body = str::from_utf8(&body)
                    .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body = serde_json::from_str::<models::ProblemDetails>(body).map_err(|e| {
                    ApiError(format!("Response body did not match the schema: {}", e))
                })?;
                Ok(OnScpDomainRoutingInformationChangePostResponse::Unauthorized(body))
            }
            403 => {
                let body = response.into_body();
                let body = body
                    .into_raw()
                    .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                    .await?;
                let body = str::from_utf8(&body)
                    .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body = serde_json::from_str::<models::ProblemDetails>(body).map_err(|e| {
                    ApiError(format!("Response body did not match the schema: {}", e))
                })?;
                Ok(OnScpDomainRoutingInformationChangePostResponse::Forbidden(
                    body,
                ))
            }
            404 => {
                let body = response.into_body();
                let body = body
                    .into_raw()
                    .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                    .await?;
                let body = str::from_utf8(&body)
                    .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body = serde_json::from_str::<models::ProblemDetails>(body).map_err(|e| {
                    ApiError(format!("Response body did not match the schema: {}", e))
                })?;
                Ok(OnScpDomainRoutingInformationChangePostResponse::NotFound(
                    body,
                ))
            }
            411 => {
                let body = response.into_body();
                let body = body
                    .into_raw()
                    .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                    .await?;
                let body = str::from_utf8(&body)
                    .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body = serde_json::from_str::<models::ProblemDetails>(body).map_err(|e| {
                    ApiError(format!("Response body did not match the schema: {}", e))
                })?;
                Ok(OnScpDomainRoutingInformationChangePostResponse::LengthRequired(body))
            }
            413 => {
                let body = response.into_body();
                let body = body
                    .into_raw()
                    .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                    .await?;
                let body = str::from_utf8(&body)
                    .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body = serde_json::from_str::<models::ProblemDetails>(body).map_err(|e| {
                    ApiError(format!("Response body did not match the schema: {}", e))
                })?;
                Ok(OnScpDomainRoutingInformationChangePostResponse::PayloadTooLarge(body))
            }
            415 => {
                let body = response.into_body();
                let body = body
                    .into_raw()
                    .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                    .await?;
                let body = str::from_utf8(&body)
                    .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body = serde_json::from_str::<models::ProblemDetails>(body).map_err(|e| {
                    ApiError(format!("Response body did not match the schema: {}", e))
                })?;
                Ok(OnScpDomainRoutingInformationChangePostResponse::UnsupportedMediaType(body))
            }
            429 => {
                let body = response.into_body();
                let body = body
                    .into_raw()
                    .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                    .await?;
                let body = str::from_utf8(&body)
                    .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body = serde_json::from_str::<models::ProblemDetails>(body).map_err(|e| {
                    ApiError(format!("Response body did not match the schema: {}", e))
                })?;
                Ok(OnScpDomainRoutingInformationChangePostResponse::TooManyRequests(body))
            }
            500 => {
                let body = response.into_body();
                let body = body
                    .into_raw()
                    .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                    .await?;
                let body = str::from_utf8(&body)
                    .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body = serde_json::from_str::<models::ProblemDetails>(body).map_err(|e| {
                    ApiError(format!("Response body did not match the schema: {}", e))
                })?;
                Ok(OnScpDomainRoutingInformationChangePostResponse::InternalServerError(body))
            }
            501 => {
                let body = response.into_body();
                let body = body
                    .into_raw()
                    .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                    .await?;
                let body = str::from_utf8(&body)
                    .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body = serde_json::from_str::<models::ProblemDetails>(body).map_err(|e| {
                    ApiError(format!("Response body did not match the schema: {}", e))
                })?;
                Ok(OnScpDomainRoutingInformationChangePostResponse::NotImplemented(body))
            }
            503 => {
                let body = response.into_body();
                let body = body
                    .into_raw()
                    .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                    .await?;
                let body = str::from_utf8(&body)
                    .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body = serde_json::from_str::<models::ProblemDetails>(body).map_err(|e| {
                    ApiError(format!("Response body did not match the schema: {}", e))
                })?;
                Ok(OnScpDomainRoutingInformationChangePostResponse::ServiceUnavailable(body))
            }
            0 => Ok(OnScpDomainRoutingInformationChangePostResponse::GenericError),
            code => {
                let headers = response.headers().clone();
                let body = response.into_body().take(100).into_raw().await;
                Err(ApiError(format!(
                    "Unexpected response code {}:\n{:?}\n\n{}",
                    code,
                    headers,
                    match body {
                        Ok(body) => match String::from_utf8(body) {
                            Ok(body) => body,
                            Err(e) => format!("<Body was not UTF8: {:?}>", e),
                        },
                        Err(e) => format!("<Failed to read body: {}>", e),
                    }
                )))
            }
        }
    }
}
