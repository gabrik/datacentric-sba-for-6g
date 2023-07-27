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

use hyper_0_10::header::{ContentType, Headers};
use mime_multipart::{generate_boundary, write_multipart, Node, Part};

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
use crate::ModifyPduSessionIsmfResponse;
use crate::ModifyPduSessionResponse;
use crate::NotifyStatusIsfmResponse;
use crate::NotifyStatusResponse;
use crate::SmContextStatusNotificationPostResponse;
use crate::TransferMtDataIsmfResponse;
use crate::TransferMtDataResponse;

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

    async fn notify_status(
        &self,
        callback_request_body_vsmf_pdu_session_uri: String,
        param_status_notification: models::StatusNotification,
        context: &C,
    ) -> Result<NotifyStatusResponse, ApiError> {
        let mut client_service = self.client_service.clone();
        let mut uri = format!(
            "{request_body_vsmf_pdu_session_uri}",
            request_body_vsmf_pdu_session_uri = callback_request_body_vsmf_pdu_session_uri
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
        let body = serde_json::to_string(&param_status_notification)
            .expect("impossible to fail to serialize");

        *request.body_mut() = Body::from(body);

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

        let response = client_service
            .call((request, context.clone()))
            .map_err(|e| ApiError(format!("No response received: {}", e)))
            .await?;

        match response.status().as_u16() {
            204 => Ok(NotifyStatusResponse::SuccessfulNotificationofTheStatusChange),
            307 => {
                let response_location =
                    match response.headers().get(HeaderName::from_static("location")) {
                        Some(response_location) => {
                            let response_location = response_location.clone();
                            let response_location =
                                match TryInto::<header::IntoHeaderValue<String>>::try_into(
                                    response_location,
                                ) {
                                    Ok(value) => value,
                                    Err(e) => {
                                        return Err(ApiError(format!(
                                        "Invalid response header Location for response 307 - {}",
                                        e
                                    )));
                                    }
                                };
                            response_location.0
                        }
                        None => {
                            return Err(ApiError(String::from(
                                "Required response header Location for response 307 was not found.",
                            )))
                        }
                    };

                let response_param_3gpp_sbi_target_nf_id = match response
                    .headers()
                    .get(HeaderName::from_static("3gpp-sbi-target-nf-id"))
                {
                    Some(response_param_3gpp_sbi_target_nf_id) => {
                        let response_param_3gpp_sbi_target_nf_id =
                            response_param_3gpp_sbi_target_nf_id.clone();
                        let response_param_3gpp_sbi_target_nf_id = match TryInto::<
                            header::IntoHeaderValue<String>,
                        >::try_into(
                            response_param_3gpp_sbi_target_nf_id,
                        ) {
                            Ok(value) => value,
                            Err(e) => {
                                return Err(ApiError(format!("Invalid response header 3gpp-Sbi-Target-Nf-Id for response 307 - {}", e)));
                            }
                        };
                        Some(response_param_3gpp_sbi_target_nf_id.0)
                    }
                    None => None,
                };

                let body = response.into_body();
                let body = body
                    .into_raw()
                    .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                    .await?;
                let body = str::from_utf8(&body)
                    .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body = serde_json::from_str::<models::RedirectResponse>(body).map_err(|e| {
                    ApiError(format!("Response body did not match the schema: {}", e))
                })?;
                Ok(NotifyStatusResponse::TemporaryRedirect {
                    body,
                    location: response_location,
                    param_3gpp_sbi_target_nf_id: response_param_3gpp_sbi_target_nf_id,
                })
            }
            308 => {
                let response_location =
                    match response.headers().get(HeaderName::from_static("location")) {
                        Some(response_location) => {
                            let response_location = response_location.clone();
                            let response_location =
                                match TryInto::<header::IntoHeaderValue<String>>::try_into(
                                    response_location,
                                ) {
                                    Ok(value) => value,
                                    Err(e) => {
                                        return Err(ApiError(format!(
                                        "Invalid response header Location for response 308 - {}",
                                        e
                                    )));
                                    }
                                };
                            response_location.0
                        }
                        None => {
                            return Err(ApiError(String::from(
                                "Required response header Location for response 308 was not found.",
                            )))
                        }
                    };

                let response_param_3gpp_sbi_target_nf_id = match response
                    .headers()
                    .get(HeaderName::from_static("3gpp-sbi-target-nf-id"))
                {
                    Some(response_param_3gpp_sbi_target_nf_id) => {
                        let response_param_3gpp_sbi_target_nf_id =
                            response_param_3gpp_sbi_target_nf_id.clone();
                        let response_param_3gpp_sbi_target_nf_id = match TryInto::<
                            header::IntoHeaderValue<String>,
                        >::try_into(
                            response_param_3gpp_sbi_target_nf_id,
                        ) {
                            Ok(value) => value,
                            Err(e) => {
                                return Err(ApiError(format!("Invalid response header 3gpp-Sbi-Target-Nf-Id for response 308 - {}", e)));
                            }
                        };
                        Some(response_param_3gpp_sbi_target_nf_id.0)
                    }
                    None => None,
                };

                let body = response.into_body();
                let body = body
                    .into_raw()
                    .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                    .await?;
                let body = str::from_utf8(&body)
                    .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body = serde_json::from_str::<models::RedirectResponse>(body).map_err(|e| {
                    ApiError(format!("Response body did not match the schema: {}", e))
                })?;
                Ok(NotifyStatusResponse::PermanentRedirect {
                    body,
                    location: response_location,
                    param_3gpp_sbi_target_nf_id: response_param_3gpp_sbi_target_nf_id,
                })
            }
            400 => {
                let body = response.into_body();
                let body = body
                    .into_raw()
                    .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                    .await?;
                let body = str::from_utf8(&body)
                    .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body =
                    serde_json::from_str::<models::ExtProblemDetails>(body).map_err(|e| {
                        ApiError(format!("Response body did not match the schema: {}", e))
                    })?;
                Ok(NotifyStatusResponse::BadRequest(body))
            }
            403 => {
                let body = response.into_body();
                let body = body
                    .into_raw()
                    .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                    .await?;
                let body = str::from_utf8(&body)
                    .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body =
                    serde_json::from_str::<models::ExtProblemDetails>(body).map_err(|e| {
                        ApiError(format!("Response body did not match the schema: {}", e))
                    })?;
                Ok(NotifyStatusResponse::Forbidden(body))
            }
            404 => {
                let body = response.into_body();
                let body = body
                    .into_raw()
                    .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                    .await?;
                let body = str::from_utf8(&body)
                    .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body =
                    serde_json::from_str::<models::ExtProblemDetails>(body).map_err(|e| {
                        ApiError(format!("Response body did not match the schema: {}", e))
                    })?;
                Ok(NotifyStatusResponse::NotFound(body))
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
                Ok(NotifyStatusResponse::LengthRequired(body))
            }
            413 => {
                let body = response.into_body();
                let body = body
                    .into_raw()
                    .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                    .await?;
                let body = str::from_utf8(&body)
                    .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body =
                    serde_json::from_str::<models::ExtProblemDetails>(body).map_err(|e| {
                        ApiError(format!("Response body did not match the schema: {}", e))
                    })?;
                Ok(NotifyStatusResponse::PayloadTooLarge(body))
            }
            415 => {
                let body = response.into_body();
                let body = body
                    .into_raw()
                    .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                    .await?;
                let body = str::from_utf8(&body)
                    .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body =
                    serde_json::from_str::<models::ExtProblemDetails>(body).map_err(|e| {
                        ApiError(format!("Response body did not match the schema: {}", e))
                    })?;
                Ok(NotifyStatusResponse::UnsupportedMediaType(body))
            }
            429 => {
                let body = response.into_body();
                let body = body
                    .into_raw()
                    .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                    .await?;
                let body = str::from_utf8(&body)
                    .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body =
                    serde_json::from_str::<models::ExtProblemDetails>(body).map_err(|e| {
                        ApiError(format!("Response body did not match the schema: {}", e))
                    })?;
                Ok(NotifyStatusResponse::TooManyRequests(body))
            }
            500 => {
                let body = response.into_body();
                let body = body
                    .into_raw()
                    .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                    .await?;
                let body = str::from_utf8(&body)
                    .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body =
                    serde_json::from_str::<models::ExtProblemDetails>(body).map_err(|e| {
                        ApiError(format!("Response body did not match the schema: {}", e))
                    })?;
                Ok(NotifyStatusResponse::InternalServerError(body))
            }
            503 => {
                let body = response.into_body();
                let body = body
                    .into_raw()
                    .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                    .await?;
                let body = str::from_utf8(&body)
                    .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body =
                    serde_json::from_str::<models::ExtProblemDetails>(body).map_err(|e| {
                        ApiError(format!("Response body did not match the schema: {}", e))
                    })?;
                Ok(NotifyStatusResponse::ServiceUnavailable(body))
            }
            0 => Ok(NotifyStatusResponse::GenericError),
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

    async fn notify_status_isfm(
        &self,
        callback_request_body_ismf_pdu_session_uri: String,
        param_status_notification: models::StatusNotification,
        context: &C,
    ) -> Result<NotifyStatusIsfmResponse, ApiError> {
        let mut client_service = self.client_service.clone();
        let mut uri = format!(
            "{request_body_ismf_pdu_session_uri}",
            request_body_ismf_pdu_session_uri = callback_request_body_ismf_pdu_session_uri
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
        let body = serde_json::to_string(&param_status_notification)
            .expect("impossible to fail to serialize");

        *request.body_mut() = Body::from(body);

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

        let response = client_service
            .call((request, context.clone()))
            .map_err(|e| ApiError(format!("No response received: {}", e)))
            .await?;

        match response.status().as_u16() {
            204 => Ok(NotifyStatusIsfmResponse::SuccessfulNotificationofTheStatusChange),
            307 => {
                let response_location =
                    match response.headers().get(HeaderName::from_static("location")) {
                        Some(response_location) => {
                            let response_location = response_location.clone();
                            let response_location =
                                match TryInto::<header::IntoHeaderValue<String>>::try_into(
                                    response_location,
                                ) {
                                    Ok(value) => value,
                                    Err(e) => {
                                        return Err(ApiError(format!(
                                        "Invalid response header Location for response 307 - {}",
                                        e
                                    )));
                                    }
                                };
                            response_location.0
                        }
                        None => {
                            return Err(ApiError(String::from(
                                "Required response header Location for response 307 was not found.",
                            )))
                        }
                    };

                let response_param_3gpp_sbi_target_nf_id = match response
                    .headers()
                    .get(HeaderName::from_static("3gpp-sbi-target-nf-id"))
                {
                    Some(response_param_3gpp_sbi_target_nf_id) => {
                        let response_param_3gpp_sbi_target_nf_id =
                            response_param_3gpp_sbi_target_nf_id.clone();
                        let response_param_3gpp_sbi_target_nf_id = match TryInto::<
                            header::IntoHeaderValue<String>,
                        >::try_into(
                            response_param_3gpp_sbi_target_nf_id,
                        ) {
                            Ok(value) => value,
                            Err(e) => {
                                return Err(ApiError(format!("Invalid response header 3gpp-Sbi-Target-Nf-Id for response 307 - {}", e)));
                            }
                        };
                        Some(response_param_3gpp_sbi_target_nf_id.0)
                    }
                    None => None,
                };

                let body = response.into_body();
                let body = body
                    .into_raw()
                    .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                    .await?;
                let body = str::from_utf8(&body)
                    .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body = serde_json::from_str::<models::RedirectResponse>(body).map_err(|e| {
                    ApiError(format!("Response body did not match the schema: {}", e))
                })?;
                Ok(NotifyStatusIsfmResponse::TemporaryRedirect {
                    body,
                    location: response_location,
                    param_3gpp_sbi_target_nf_id: response_param_3gpp_sbi_target_nf_id,
                })
            }
            308 => {
                let response_location =
                    match response.headers().get(HeaderName::from_static("location")) {
                        Some(response_location) => {
                            let response_location = response_location.clone();
                            let response_location =
                                match TryInto::<header::IntoHeaderValue<String>>::try_into(
                                    response_location,
                                ) {
                                    Ok(value) => value,
                                    Err(e) => {
                                        return Err(ApiError(format!(
                                        "Invalid response header Location for response 308 - {}",
                                        e
                                    )));
                                    }
                                };
                            response_location.0
                        }
                        None => {
                            return Err(ApiError(String::from(
                                "Required response header Location for response 308 was not found.",
                            )))
                        }
                    };

                let response_param_3gpp_sbi_target_nf_id = match response
                    .headers()
                    .get(HeaderName::from_static("3gpp-sbi-target-nf-id"))
                {
                    Some(response_param_3gpp_sbi_target_nf_id) => {
                        let response_param_3gpp_sbi_target_nf_id =
                            response_param_3gpp_sbi_target_nf_id.clone();
                        let response_param_3gpp_sbi_target_nf_id = match TryInto::<
                            header::IntoHeaderValue<String>,
                        >::try_into(
                            response_param_3gpp_sbi_target_nf_id,
                        ) {
                            Ok(value) => value,
                            Err(e) => {
                                return Err(ApiError(format!("Invalid response header 3gpp-Sbi-Target-Nf-Id for response 308 - {}", e)));
                            }
                        };
                        Some(response_param_3gpp_sbi_target_nf_id.0)
                    }
                    None => None,
                };

                let body = response.into_body();
                let body = body
                    .into_raw()
                    .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                    .await?;
                let body = str::from_utf8(&body)
                    .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body = serde_json::from_str::<models::RedirectResponse>(body).map_err(|e| {
                    ApiError(format!("Response body did not match the schema: {}", e))
                })?;
                Ok(NotifyStatusIsfmResponse::PermanentRedirect {
                    body,
                    location: response_location,
                    param_3gpp_sbi_target_nf_id: response_param_3gpp_sbi_target_nf_id,
                })
            }
            400 => {
                let body = response.into_body();
                let body = body
                    .into_raw()
                    .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                    .await?;
                let body = str::from_utf8(&body)
                    .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body =
                    serde_json::from_str::<models::ExtProblemDetails>(body).map_err(|e| {
                        ApiError(format!("Response body did not match the schema: {}", e))
                    })?;
                Ok(NotifyStatusIsfmResponse::BadRequest(body))
            }
            403 => {
                let body = response.into_body();
                let body = body
                    .into_raw()
                    .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                    .await?;
                let body = str::from_utf8(&body)
                    .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body =
                    serde_json::from_str::<models::ExtProblemDetails>(body).map_err(|e| {
                        ApiError(format!("Response body did not match the schema: {}", e))
                    })?;
                Ok(NotifyStatusIsfmResponse::Forbidden(body))
            }
            404 => {
                let body = response.into_body();
                let body = body
                    .into_raw()
                    .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                    .await?;
                let body = str::from_utf8(&body)
                    .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body =
                    serde_json::from_str::<models::ExtProblemDetails>(body).map_err(|e| {
                        ApiError(format!("Response body did not match the schema: {}", e))
                    })?;
                Ok(NotifyStatusIsfmResponse::NotFound(body))
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
                Ok(NotifyStatusIsfmResponse::LengthRequired(body))
            }
            413 => {
                let body = response.into_body();
                let body = body
                    .into_raw()
                    .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                    .await?;
                let body = str::from_utf8(&body)
                    .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body =
                    serde_json::from_str::<models::ExtProblemDetails>(body).map_err(|e| {
                        ApiError(format!("Response body did not match the schema: {}", e))
                    })?;
                Ok(NotifyStatusIsfmResponse::PayloadTooLarge(body))
            }
            415 => {
                let body = response.into_body();
                let body = body
                    .into_raw()
                    .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                    .await?;
                let body = str::from_utf8(&body)
                    .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body =
                    serde_json::from_str::<models::ExtProblemDetails>(body).map_err(|e| {
                        ApiError(format!("Response body did not match the schema: {}", e))
                    })?;
                Ok(NotifyStatusIsfmResponse::UnsupportedMediaType(body))
            }
            429 => {
                let body = response.into_body();
                let body = body
                    .into_raw()
                    .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                    .await?;
                let body = str::from_utf8(&body)
                    .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body =
                    serde_json::from_str::<models::ExtProblemDetails>(body).map_err(|e| {
                        ApiError(format!("Response body did not match the schema: {}", e))
                    })?;
                Ok(NotifyStatusIsfmResponse::TooManyRequests(body))
            }
            500 => {
                let body = response.into_body();
                let body = body
                    .into_raw()
                    .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                    .await?;
                let body = str::from_utf8(&body)
                    .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body =
                    serde_json::from_str::<models::ExtProblemDetails>(body).map_err(|e| {
                        ApiError(format!("Response body did not match the schema: {}", e))
                    })?;
                Ok(NotifyStatusIsfmResponse::InternalServerError(body))
            }
            503 => {
                let body = response.into_body();
                let body = body
                    .into_raw()
                    .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                    .await?;
                let body = str::from_utf8(&body)
                    .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body =
                    serde_json::from_str::<models::ExtProblemDetails>(body).map_err(|e| {
                        ApiError(format!("Response body did not match the schema: {}", e))
                    })?;
                Ok(NotifyStatusIsfmResponse::ServiceUnavailable(body))
            }
            0 => Ok(NotifyStatusIsfmResponse::GenericError),
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

    async fn modify_pdu_session(
        &self,
        callback_request_body_vsmf_pdu_session_uri: String,
        param_vsmf_update_data: models::VsmfUpdateData,
        context: &C,
    ) -> Result<ModifyPduSessionResponse, ApiError> {
        let mut client_service = self.client_service.clone();
        let mut uri = format!(
            "{request_body_vsmf_pdu_session_uri}/modify",
            request_body_vsmf_pdu_session_uri = callback_request_body_vsmf_pdu_session_uri
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
        let body = serde_json::to_string(&param_vsmf_update_data)
            .expect("impossible to fail to serialize");

        *request.body_mut() = Body::from(body);

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

        let response = client_service
            .call((request, context.clone()))
            .map_err(|e| ApiError(format!("No response received: {}", e)))
            .await?;

        match response.status().as_u16() {
            200 => {
                let body = response.into_body();
                let body = body
                    .into_raw()
                    .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                    .await?;
                let body = str::from_utf8(&body)
                    .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body = serde_json::from_str::<models::VsmfUpdatedData>(body).map_err(|e| {
                    ApiError(format!("Response body did not match the schema: {}", e))
                })?;
                Ok(
                    ModifyPduSessionResponse::SuccessfulUpdateOfAPDUSessionWithContentInTheResponse(
                        body,
                    ),
                )
            }
            204 => Ok(
                ModifyPduSessionResponse::SuccessfulUpdateOfAPDUSessionWithoutContentInTheResponse,
            ),
            307 => {
                let response_location =
                    match response.headers().get(HeaderName::from_static("location")) {
                        Some(response_location) => {
                            let response_location = response_location.clone();
                            let response_location =
                                match TryInto::<header::IntoHeaderValue<String>>::try_into(
                                    response_location,
                                ) {
                                    Ok(value) => value,
                                    Err(e) => {
                                        return Err(ApiError(format!(
                                        "Invalid response header Location for response 307 - {}",
                                        e
                                    )));
                                    }
                                };
                            response_location.0
                        }
                        None => {
                            return Err(ApiError(String::from(
                                "Required response header Location for response 307 was not found.",
                            )))
                        }
                    };

                let response_param_3gpp_sbi_target_nf_id = match response
                    .headers()
                    .get(HeaderName::from_static("3gpp-sbi-target-nf-id"))
                {
                    Some(response_param_3gpp_sbi_target_nf_id) => {
                        let response_param_3gpp_sbi_target_nf_id =
                            response_param_3gpp_sbi_target_nf_id.clone();
                        let response_param_3gpp_sbi_target_nf_id = match TryInto::<
                            header::IntoHeaderValue<String>,
                        >::try_into(
                            response_param_3gpp_sbi_target_nf_id,
                        ) {
                            Ok(value) => value,
                            Err(e) => {
                                return Err(ApiError(format!("Invalid response header 3gpp-Sbi-Target-Nf-Id for response 307 - {}", e)));
                            }
                        };
                        Some(response_param_3gpp_sbi_target_nf_id.0)
                    }
                    None => None,
                };

                let body = response.into_body();
                let body = body
                    .into_raw()
                    .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                    .await?;
                let body = str::from_utf8(&body)
                    .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body = serde_json::from_str::<models::RedirectResponse>(body).map_err(|e| {
                    ApiError(format!("Response body did not match the schema: {}", e))
                })?;
                Ok(ModifyPduSessionResponse::TemporaryRedirect {
                    body,
                    location: response_location,
                    param_3gpp_sbi_target_nf_id: response_param_3gpp_sbi_target_nf_id,
                })
            }
            308 => {
                let response_location =
                    match response.headers().get(HeaderName::from_static("location")) {
                        Some(response_location) => {
                            let response_location = response_location.clone();
                            let response_location =
                                match TryInto::<header::IntoHeaderValue<String>>::try_into(
                                    response_location,
                                ) {
                                    Ok(value) => value,
                                    Err(e) => {
                                        return Err(ApiError(format!(
                                        "Invalid response header Location for response 308 - {}",
                                        e
                                    )));
                                    }
                                };
                            response_location.0
                        }
                        None => {
                            return Err(ApiError(String::from(
                                "Required response header Location for response 308 was not found.",
                            )))
                        }
                    };

                let response_param_3gpp_sbi_target_nf_id = match response
                    .headers()
                    .get(HeaderName::from_static("3gpp-sbi-target-nf-id"))
                {
                    Some(response_param_3gpp_sbi_target_nf_id) => {
                        let response_param_3gpp_sbi_target_nf_id =
                            response_param_3gpp_sbi_target_nf_id.clone();
                        let response_param_3gpp_sbi_target_nf_id = match TryInto::<
                            header::IntoHeaderValue<String>,
                        >::try_into(
                            response_param_3gpp_sbi_target_nf_id,
                        ) {
                            Ok(value) => value,
                            Err(e) => {
                                return Err(ApiError(format!("Invalid response header 3gpp-Sbi-Target-Nf-Id for response 308 - {}", e)));
                            }
                        };
                        Some(response_param_3gpp_sbi_target_nf_id.0)
                    }
                    None => None,
                };

                let body = response.into_body();
                let body = body
                    .into_raw()
                    .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                    .await?;
                let body = str::from_utf8(&body)
                    .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body = serde_json::from_str::<models::RedirectResponse>(body).map_err(|e| {
                    ApiError(format!("Response body did not match the schema: {}", e))
                })?;
                Ok(ModifyPduSessionResponse::PermanentRedirect {
                    body,
                    location: response_location,
                    param_3gpp_sbi_target_nf_id: response_param_3gpp_sbi_target_nf_id,
                })
            }
            400 => {
                let body = response.into_body();
                let body = body
                    .into_raw()
                    .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                    .await?;
                let body = str::from_utf8(&body)
                    .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body = serde_json::from_str::<models::VsmfUpdateError>(body).map_err(|e| {
                    ApiError(format!("Response body did not match the schema: {}", e))
                })?;
                Ok(ModifyPduSessionResponse::UnsuccessfulUpdateOfAPDUSession(
                    body,
                ))
            }
            403 => {
                let body = response.into_body();
                let body = body
                    .into_raw()
                    .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                    .await?;
                let body = str::from_utf8(&body)
                    .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body = serde_json::from_str::<models::VsmfUpdateError>(body).map_err(|e| {
                    ApiError(format!("Response body did not match the schema: {}", e))
                })?;
                Ok(ModifyPduSessionResponse::UnsuccessfulUpdateOfAPDUSession_2(
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
                let body = serde_json::from_str::<models::VsmfUpdateError>(body).map_err(|e| {
                    ApiError(format!("Response body did not match the schema: {}", e))
                })?;
                Ok(ModifyPduSessionResponse::UnsuccessfulUpdateOfAPDUSession_3(
                    body,
                ))
            }
            409 => {
                let body = response.into_body();
                let body = body
                    .into_raw()
                    .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                    .await?;
                let body = str::from_utf8(&body)
                    .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body = serde_json::from_str::<models::VsmfUpdateError>(body).map_err(|e| {
                    ApiError(format!("Response body did not match the schema: {}", e))
                })?;
                Ok(ModifyPduSessionResponse::UnsuccessfulUpdateOfAPDUSession_4(
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
                Ok(ModifyPduSessionResponse::LengthRequired(body))
            }
            413 => {
                let body = response.into_body();
                let body = body
                    .into_raw()
                    .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                    .await?;
                let body = str::from_utf8(&body)
                    .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body =
                    serde_json::from_str::<models::ExtProblemDetails>(body).map_err(|e| {
                        ApiError(format!("Response body did not match the schema: {}", e))
                    })?;
                Ok(ModifyPduSessionResponse::PayloadTooLarge(body))
            }
            415 => {
                let body = response.into_body();
                let body = body
                    .into_raw()
                    .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                    .await?;
                let body = str::from_utf8(&body)
                    .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body =
                    serde_json::from_str::<models::ExtProblemDetails>(body).map_err(|e| {
                        ApiError(format!("Response body did not match the schema: {}", e))
                    })?;
                Ok(ModifyPduSessionResponse::UnsupportedMediaType(body))
            }
            429 => {
                let body = response.into_body();
                let body = body
                    .into_raw()
                    .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                    .await?;
                let body = str::from_utf8(&body)
                    .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body =
                    serde_json::from_str::<models::ExtProblemDetails>(body).map_err(|e| {
                        ApiError(format!("Response body did not match the schema: {}", e))
                    })?;
                Ok(ModifyPduSessionResponse::TooManyRequests(body))
            }
            500 => {
                let body = response.into_body();
                let body = body
                    .into_raw()
                    .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                    .await?;
                let body = str::from_utf8(&body)
                    .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body = serde_json::from_str::<models::VsmfUpdateError>(body).map_err(|e| {
                    ApiError(format!("Response body did not match the schema: {}", e))
                })?;
                Ok(ModifyPduSessionResponse::UnsuccessfulUpdateOfAPDUSession_5(
                    body,
                ))
            }
            503 => {
                let body = response.into_body();
                let body = body
                    .into_raw()
                    .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                    .await?;
                let body = str::from_utf8(&body)
                    .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body = serde_json::from_str::<models::VsmfUpdateError>(body).map_err(|e| {
                    ApiError(format!("Response body did not match the schema: {}", e))
                })?;
                Ok(ModifyPduSessionResponse::UnsuccessfulUpdateOfAPDUSession_6(
                    body,
                ))
            }
            504 => {
                let body = response.into_body();
                let body = body
                    .into_raw()
                    .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                    .await?;
                let body = str::from_utf8(&body)
                    .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body = serde_json::from_str::<models::VsmfUpdateError>(body).map_err(|e| {
                    ApiError(format!("Response body did not match the schema: {}", e))
                })?;
                Ok(ModifyPduSessionResponse::UnsuccessfulUpdateOfAPDUSession_7(
                    body,
                ))
            }
            0 => Ok(ModifyPduSessionResponse::GenericError),
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

    async fn modify_pdu_session_ismf(
        &self,
        callback_request_body_ismf_pdu_session_uri: String,
        param_vsmf_update_data: models::VsmfUpdateData,
        context: &C,
    ) -> Result<ModifyPduSessionIsmfResponse, ApiError> {
        let mut client_service = self.client_service.clone();
        let mut uri = format!(
            "{request_body_ismf_pdu_session_uri}/modify",
            request_body_ismf_pdu_session_uri = callback_request_body_ismf_pdu_session_uri
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
        let body = serde_json::to_string(&param_vsmf_update_data)
            .expect("impossible to fail to serialize");

        *request.body_mut() = Body::from(body);

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

        let response = client_service
            .call((request, context.clone()))
            .map_err(|e| ApiError(format!("No response received: {}", e)))
            .await?;

        match response.status().as_u16() {
            200 => {
                let body = response.into_body();
                let body = body
                        .into_raw()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e))).await?;
                let body = str::from_utf8(&body)
                    .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body = serde_json::from_str::<models::VsmfUpdatedData>(body).map_err(|e| {
                    ApiError(format!("Response body did not match the schema: {}", e))
                })?;
                Ok(ModifyPduSessionIsmfResponse::SuccessfulUpdateOfAPDUSessionWithContentInTheResponse
                    (body)
                )
            }
            204 => {
                Ok(
                    ModifyPduSessionIsmfResponse::SuccessfulUpdateOfAPDUSessionWithoutContentInTheResponse
                )
            }
            307 => {
                let response_location = match response.headers().get(HeaderName::from_static("location")) {
                    Some(response_location) => {
                        let response_location = response_location.clone();
                        let response_location = match TryInto::<header::IntoHeaderValue<String>>::try_into(response_location) {
                            Ok(value) => value,
                            Err(e) => {
                                return Err(ApiError(format!("Invalid response header Location for response 307 - {}", e)));
                            },
                        };
                        response_location.0
                        },
                    None => return Err(ApiError(String::from("Required response header Location for response 307 was not found."))),
                };

                let response_param_3gpp_sbi_target_nf_id = match response.headers().get(HeaderName::from_static("3gpp-sbi-target-nf-id")) {
                    Some(response_param_3gpp_sbi_target_nf_id) => {
                        let response_param_3gpp_sbi_target_nf_id = response_param_3gpp_sbi_target_nf_id.clone();
                        let response_param_3gpp_sbi_target_nf_id = match TryInto::<header::IntoHeaderValue<String>>::try_into(response_param_3gpp_sbi_target_nf_id) {
                            Ok(value) => value,
                            Err(e) => {
                                return Err(ApiError(format!("Invalid response header 3gpp-Sbi-Target-Nf-Id for response 307 - {}", e)));
                            },
                        };
                        Some(response_param_3gpp_sbi_target_nf_id.0)
                        },
                    None => None,
                };

                let body = response.into_body();
                let body = body
                        .into_raw()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e))).await?;
                let body = str::from_utf8(&body)
                    .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body = serde_json::from_str::<models::RedirectResponse>(body).map_err(|e| {
                    ApiError(format!("Response body did not match the schema: {}", e))
                })?;
                Ok(ModifyPduSessionIsmfResponse::TemporaryRedirect
                    {
                        body,
                        location: response_location,
                        param_3gpp_sbi_target_nf_id: response_param_3gpp_sbi_target_nf_id,
                    }
                )
            }
            308 => {
                let response_location = match response.headers().get(HeaderName::from_static("location")) {
                    Some(response_location) => {
                        let response_location = response_location.clone();
                        let response_location = match TryInto::<header::IntoHeaderValue<String>>::try_into(response_location) {
                            Ok(value) => value,
                            Err(e) => {
                                return Err(ApiError(format!("Invalid response header Location for response 308 - {}", e)));
                            },
                        };
                        response_location.0
                        },
                    None => return Err(ApiError(String::from("Required response header Location for response 308 was not found."))),
                };

                let response_param_3gpp_sbi_target_nf_id = match response.headers().get(HeaderName::from_static("3gpp-sbi-target-nf-id")) {
                    Some(response_param_3gpp_sbi_target_nf_id) => {
                        let response_param_3gpp_sbi_target_nf_id = response_param_3gpp_sbi_target_nf_id.clone();
                        let response_param_3gpp_sbi_target_nf_id = match TryInto::<header::IntoHeaderValue<String>>::try_into(response_param_3gpp_sbi_target_nf_id) {
                            Ok(value) => value,
                            Err(e) => {
                                return Err(ApiError(format!("Invalid response header 3gpp-Sbi-Target-Nf-Id for response 308 - {}", e)));
                            },
                        };
                        Some(response_param_3gpp_sbi_target_nf_id.0)
                        },
                    None => None,
                };

                let body = response.into_body();
                let body = body
                        .into_raw()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e))).await?;
                let body = str::from_utf8(&body)
                    .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body = serde_json::from_str::<models::RedirectResponse>(body).map_err(|e| {
                    ApiError(format!("Response body did not match the schema: {}", e))
                })?;
                Ok(ModifyPduSessionIsmfResponse::PermanentRedirect
                    {
                        body,
                        location: response_location,
                        param_3gpp_sbi_target_nf_id: response_param_3gpp_sbi_target_nf_id,
                    }
                )
            }
            400 => {
                let body = response.into_body();
                let body = body
                        .into_raw()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e))).await?;
                let body = str::from_utf8(&body)
                    .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body = serde_json::from_str::<models::VsmfUpdateError>(body).map_err(|e| {
                    ApiError(format!("Response body did not match the schema: {}", e))
                })?;
                Ok(ModifyPduSessionIsmfResponse::UnsuccessfulUpdateOfAPDUSession
                    (body)
                )
            }
            403 => {
                let body = response.into_body();
                let body = body
                        .into_raw()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e))).await?;
                let body = str::from_utf8(&body)
                    .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body = serde_json::from_str::<models::VsmfUpdateError>(body).map_err(|e| {
                    ApiError(format!("Response body did not match the schema: {}", e))
                })?;
                Ok(ModifyPduSessionIsmfResponse::UnsuccessfulUpdateOfAPDUSession_2
                    (body)
                )
            }
            404 => {
                let body = response.into_body();
                let body = body
                        .into_raw()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e))).await?;
                let body = str::from_utf8(&body)
                    .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body = serde_json::from_str::<models::VsmfUpdateError>(body).map_err(|e| {
                    ApiError(format!("Response body did not match the schema: {}", e))
                })?;
                Ok(ModifyPduSessionIsmfResponse::UnsuccessfulUpdateOfAPDUSession_3
                    (body)
                )
            }
            409 => {
                let body = response.into_body();
                let body = body
                        .into_raw()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e))).await?;
                let body = str::from_utf8(&body)
                    .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body = serde_json::from_str::<models::VsmfUpdateError>(body).map_err(|e| {
                    ApiError(format!("Response body did not match the schema: {}", e))
                })?;
                Ok(ModifyPduSessionIsmfResponse::UnsuccessfulUpdateOfAPDUSession_4
                    (body)
                )
            }
            411 => {
                let body = response.into_body();
                let body = body
                        .into_raw()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e))).await?;
                let body = str::from_utf8(&body)
                    .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body = serde_json::from_str::<models::ProblemDetails>(body).map_err(|e| {
                    ApiError(format!("Response body did not match the schema: {}", e))
                })?;
                Ok(ModifyPduSessionIsmfResponse::LengthRequired
                    (body)
                )
            }
            413 => {
                let body = response.into_body();
                let body = body
                        .into_raw()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e))).await?;
                let body = str::from_utf8(&body)
                    .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body = serde_json::from_str::<models::ExtProblemDetails>(body).map_err(|e| {
                    ApiError(format!("Response body did not match the schema: {}", e))
                })?;
                Ok(ModifyPduSessionIsmfResponse::PayloadTooLarge
                    (body)
                )
            }
            415 => {
                let body = response.into_body();
                let body = body
                        .into_raw()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e))).await?;
                let body = str::from_utf8(&body)
                    .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body = serde_json::from_str::<models::ExtProblemDetails>(body).map_err(|e| {
                    ApiError(format!("Response body did not match the schema: {}", e))
                })?;
                Ok(ModifyPduSessionIsmfResponse::UnsupportedMediaType
                    (body)
                )
            }
            429 => {
                let body = response.into_body();
                let body = body
                        .into_raw()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e))).await?;
                let body = str::from_utf8(&body)
                    .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body = serde_json::from_str::<models::ExtProblemDetails>(body).map_err(|e| {
                    ApiError(format!("Response body did not match the schema: {}", e))
                })?;
                Ok(ModifyPduSessionIsmfResponse::TooManyRequests
                    (body)
                )
            }
            500 => {
                let body = response.into_body();
                let body = body
                        .into_raw()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e))).await?;
                let body = str::from_utf8(&body)
                    .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body = serde_json::from_str::<models::VsmfUpdateError>(body).map_err(|e| {
                    ApiError(format!("Response body did not match the schema: {}", e))
                })?;
                Ok(ModifyPduSessionIsmfResponse::UnsuccessfulUpdateOfAPDUSession_5
                    (body)
                )
            }
            503 => {
                let body = response.into_body();
                let body = body
                        .into_raw()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e))).await?;
                let body = str::from_utf8(&body)
                    .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body = serde_json::from_str::<models::VsmfUpdateError>(body).map_err(|e| {
                    ApiError(format!("Response body did not match the schema: {}", e))
                })?;
                Ok(ModifyPduSessionIsmfResponse::UnsuccessfulUpdateOfAPDUSession_6
                    (body)
                )
            }
            504 => {
                let body = response.into_body();
                let body = body
                        .into_raw()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e))).await?;
                let body = str::from_utf8(&body)
                    .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body = serde_json::from_str::<models::VsmfUpdateError>(body).map_err(|e| {
                    ApiError(format!("Response body did not match the schema: {}", e))
                })?;
                Ok(ModifyPduSessionIsmfResponse::UnsuccessfulUpdateOfAPDUSession_7
                    (body)
                )
            }
            0 => {
                Ok(
                    ModifyPduSessionIsmfResponse::GenericError
                )
            }
            code => {
                let headers = response.headers().clone();
                let body = response.into_body()
                       .take(100)
                       .into_raw().await;
                Err(ApiError(format!("Unexpected response code {}:\n{:?}\n\n{}",
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

    async fn transfer_mt_data(
        &self,
        callback_request_body_vsmf_pdu_session_uri: String,
        param_json_data: Option<models::TransferMtDataReqData>,
        param_binary_mt_data: Option<swagger::ByteArray>,
        context: &C,
    ) -> Result<TransferMtDataResponse, ApiError> {
        let mut client_service = self.client_service.clone();
        let mut uri = format!(
            "{request_body_vsmf_pdu_session_uri}/transfer-mt-data",
            request_body_vsmf_pdu_session_uri = callback_request_body_vsmf_pdu_session_uri
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

        // Construct the Body for a multipart/related request. The mime 0.2.6 library
        // does not parse quoted-string parameters correctly. The boundary doesn't
        // need to be a quoted string if it does not contain a '/', hence ensure
        // no such boundary is used.
        let mut boundary = generate_boundary();
        for b in boundary.iter_mut() {
            if b == &(b'/') {
                *b = b'=';
            }
        }

        let mut body_parts = vec![];

        if let Some(json_data) = param_json_data {
            let part = Node::Part(Part {
                headers: {
                    let mut h = Headers::new();
                    h.set(ContentType("application/json".parse().unwrap()));
                    h.set_raw("Content-ID", vec![b"jsonData".to_vec()]);
                    h
                },
                body: serde_json::to_string(&json_data)
                    .expect("Impossible to fail to serialize")
                    .into_bytes(),
            });
            body_parts.push(part);
        }

        if let Some(binary_mt_data) = param_binary_mt_data {
            let part = Node::Part(Part {
                headers: {
                    let mut h = Headers::new();
                    h.set(ContentType("application/vnd.3gpp.5gnas".parse().unwrap()));
                    h.set_raw("Content-ID", vec![b"binaryMtData".to_vec()]);
                    h
                },
                body: binary_mt_data.0,
            });
            body_parts.push(part);
        }

        // Write the body into a vec.
        let mut body: Vec<u8> = vec![];
        write_multipart(&mut body, &boundary, &body_parts).expect("Failed to write multipart body");

        // Add the message body to the request object.
        *request.body_mut() = Body::from(body);

        let header = "multipart/related";
        request.headers_mut().insert(
            CONTENT_TYPE,
            match HeaderValue::from_bytes(
                &[
                    header.as_bytes(),
                    "; boundary=".as_bytes(),
                    &boundary,
                    "; type=\"application/json\"".as_bytes(),
                ]
                .concat(),
            ) {
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

        let response = client_service
            .call((request, context.clone()))
            .map_err(|e| ApiError(format!("No response received: {}", e)))
            .await?;

        match response.status().as_u16() {
            204 => Ok(TransferMtDataResponse::SuccessfulTransferingOfMTData),
            307 => {
                let response_location =
                    match response.headers().get(HeaderName::from_static("location")) {
                        Some(response_location) => {
                            let response_location = response_location.clone();
                            let response_location =
                                match TryInto::<header::IntoHeaderValue<String>>::try_into(
                                    response_location,
                                ) {
                                    Ok(value) => value,
                                    Err(e) => {
                                        return Err(ApiError(format!(
                                        "Invalid response header Location for response 307 - {}",
                                        e
                                    )));
                                    }
                                };
                            response_location.0
                        }
                        None => {
                            return Err(ApiError(String::from(
                                "Required response header Location for response 307 was not found.",
                            )))
                        }
                    };

                let response_param_3gpp_sbi_target_nf_id = match response
                    .headers()
                    .get(HeaderName::from_static("3gpp-sbi-target-nf-id"))
                {
                    Some(response_param_3gpp_sbi_target_nf_id) => {
                        let response_param_3gpp_sbi_target_nf_id =
                            response_param_3gpp_sbi_target_nf_id.clone();
                        let response_param_3gpp_sbi_target_nf_id = match TryInto::<
                            header::IntoHeaderValue<String>,
                        >::try_into(
                            response_param_3gpp_sbi_target_nf_id,
                        ) {
                            Ok(value) => value,
                            Err(e) => {
                                return Err(ApiError(format!("Invalid response header 3gpp-Sbi-Target-Nf-Id for response 307 - {}", e)));
                            }
                        };
                        Some(response_param_3gpp_sbi_target_nf_id.0)
                    }
                    None => None,
                };

                let body = response.into_body();
                let body = body
                    .into_raw()
                    .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                    .await?;
                let body = str::from_utf8(&body)
                    .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body = serde_json::from_str::<models::RedirectResponse>(body).map_err(|e| {
                    ApiError(format!("Response body did not match the schema: {}", e))
                })?;
                Ok(TransferMtDataResponse::TemporaryRedirect {
                    body,
                    location: response_location,
                    param_3gpp_sbi_target_nf_id: response_param_3gpp_sbi_target_nf_id,
                })
            }
            308 => {
                let response_location =
                    match response.headers().get(HeaderName::from_static("location")) {
                        Some(response_location) => {
                            let response_location = response_location.clone();
                            let response_location =
                                match TryInto::<header::IntoHeaderValue<String>>::try_into(
                                    response_location,
                                ) {
                                    Ok(value) => value,
                                    Err(e) => {
                                        return Err(ApiError(format!(
                                        "Invalid response header Location for response 308 - {}",
                                        e
                                    )));
                                    }
                                };
                            response_location.0
                        }
                        None => {
                            return Err(ApiError(String::from(
                                "Required response header Location for response 308 was not found.",
                            )))
                        }
                    };

                let response_param_3gpp_sbi_target_nf_id = match response
                    .headers()
                    .get(HeaderName::from_static("3gpp-sbi-target-nf-id"))
                {
                    Some(response_param_3gpp_sbi_target_nf_id) => {
                        let response_param_3gpp_sbi_target_nf_id =
                            response_param_3gpp_sbi_target_nf_id.clone();
                        let response_param_3gpp_sbi_target_nf_id = match TryInto::<
                            header::IntoHeaderValue<String>,
                        >::try_into(
                            response_param_3gpp_sbi_target_nf_id,
                        ) {
                            Ok(value) => value,
                            Err(e) => {
                                return Err(ApiError(format!("Invalid response header 3gpp-Sbi-Target-Nf-Id for response 308 - {}", e)));
                            }
                        };
                        Some(response_param_3gpp_sbi_target_nf_id.0)
                    }
                    None => None,
                };

                let body = response.into_body();
                let body = body
                    .into_raw()
                    .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                    .await?;
                let body = str::from_utf8(&body)
                    .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body = serde_json::from_str::<models::RedirectResponse>(body).map_err(|e| {
                    ApiError(format!("Response body did not match the schema: {}", e))
                })?;
                Ok(TransferMtDataResponse::PermanentRedirect {
                    body,
                    location: response_location,
                    param_3gpp_sbi_target_nf_id: response_param_3gpp_sbi_target_nf_id,
                })
            }
            400 => {
                let body = response.into_body();
                let body = body
                    .into_raw()
                    .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                    .await?;
                let body = str::from_utf8(&body)
                    .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body =
                    serde_json::from_str::<models::ExtProblemDetails>(body).map_err(|e| {
                        ApiError(format!("Response body did not match the schema: {}", e))
                    })?;
                Ok(TransferMtDataResponse::BadRequest(body))
            }
            401 => {
                let body = response.into_body();
                let body = body
                    .into_raw()
                    .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                    .await?;
                let body = str::from_utf8(&body)
                    .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body =
                    serde_json::from_str::<models::ExtProblemDetails>(body).map_err(|e| {
                        ApiError(format!("Response body did not match the schema: {}", e))
                    })?;
                Ok(TransferMtDataResponse::Unauthorized(body))
            }
            403 => {
                let body = response.into_body();
                let body = body
                    .into_raw()
                    .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                    .await?;
                let body = str::from_utf8(&body)
                    .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body =
                    serde_json::from_str::<models::ExtProblemDetails>(body).map_err(|e| {
                        ApiError(format!("Response body did not match the schema: {}", e))
                    })?;
                Ok(TransferMtDataResponse::Forbidden(body))
            }
            404 => {
                let body = response.into_body();
                let body = body
                    .into_raw()
                    .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                    .await?;
                let body = str::from_utf8(&body)
                    .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body =
                    serde_json::from_str::<models::ExtProblemDetails>(body).map_err(|e| {
                        ApiError(format!("Response body did not match the schema: {}", e))
                    })?;
                Ok(TransferMtDataResponse::NotFound(body))
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
                Ok(TransferMtDataResponse::LengthRequired(body))
            }
            413 => {
                let body = response.into_body();
                let body = body
                    .into_raw()
                    .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                    .await?;
                let body = str::from_utf8(&body)
                    .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body =
                    serde_json::from_str::<models::ExtProblemDetails>(body).map_err(|e| {
                        ApiError(format!("Response body did not match the schema: {}", e))
                    })?;
                Ok(TransferMtDataResponse::PayloadTooLarge(body))
            }
            415 => {
                let body = response.into_body();
                let body = body
                    .into_raw()
                    .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                    .await?;
                let body = str::from_utf8(&body)
                    .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body =
                    serde_json::from_str::<models::ExtProblemDetails>(body).map_err(|e| {
                        ApiError(format!("Response body did not match the schema: {}", e))
                    })?;
                Ok(TransferMtDataResponse::UnsupportedMediaType(body))
            }
            429 => {
                let body = response.into_body();
                let body = body
                    .into_raw()
                    .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                    .await?;
                let body = str::from_utf8(&body)
                    .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body =
                    serde_json::from_str::<models::ExtProblemDetails>(body).map_err(|e| {
                        ApiError(format!("Response body did not match the schema: {}", e))
                    })?;
                Ok(TransferMtDataResponse::TooManyRequests(body))
            }
            500 => {
                let body = response.into_body();
                let body = body
                    .into_raw()
                    .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                    .await?;
                let body = str::from_utf8(&body)
                    .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body =
                    serde_json::from_str::<models::ExtProblemDetails>(body).map_err(|e| {
                        ApiError(format!("Response body did not match the schema: {}", e))
                    })?;
                Ok(TransferMtDataResponse::InternalServerError(body))
            }
            503 => {
                let body = response.into_body();
                let body = body
                    .into_raw()
                    .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                    .await?;
                let body = str::from_utf8(&body)
                    .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body =
                    serde_json::from_str::<models::ExtProblemDetails>(body).map_err(|e| {
                        ApiError(format!("Response body did not match the schema: {}", e))
                    })?;
                Ok(TransferMtDataResponse::ServiceUnavailable(body))
            }
            504 => {
                let body = response.into_body();
                let body = body
                    .into_raw()
                    .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                    .await?;
                let body = str::from_utf8(&body)
                    .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body =
                    serde_json::from_str::<models::TransferMtDataError>(body).map_err(|e| {
                        ApiError(format!("Response body did not match the schema: {}", e))
                    })?;
                Ok(TransferMtDataResponse::UnsuccessfulDeliveryOfMobileTerminatedData(body))
            }
            0 => Ok(TransferMtDataResponse::GenericError),
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

    async fn transfer_mt_data_ismf(
        &self,
        callback_request_body_ismf_pdu_session_uri: String,
        param_json_data: Option<models::TransferMtDataReqData>,
        param_binary_mt_data: Option<swagger::ByteArray>,
        context: &C,
    ) -> Result<TransferMtDataIsmfResponse, ApiError> {
        let mut client_service = self.client_service.clone();
        let mut uri = format!(
            "{request_body_ismf_pdu_session_uri}/transfer-mt-data",
            request_body_ismf_pdu_session_uri = callback_request_body_ismf_pdu_session_uri
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

        // Construct the Body for a multipart/related request. The mime 0.2.6 library
        // does not parse quoted-string parameters correctly. The boundary doesn't
        // need to be a quoted string if it does not contain a '/', hence ensure
        // no such boundary is used.
        let mut boundary = generate_boundary();
        for b in boundary.iter_mut() {
            if b == &(b'/') {
                *b = b'=';
            }
        }

        let mut body_parts = vec![];

        if let Some(json_data) = param_json_data {
            let part = Node::Part(Part {
                headers: {
                    let mut h = Headers::new();
                    h.set(ContentType("application/json".parse().unwrap()));
                    h.set_raw("Content-ID", vec![b"jsonData".to_vec()]);
                    h
                },
                body: serde_json::to_string(&json_data)
                    .expect("Impossible to fail to serialize")
                    .into_bytes(),
            });
            body_parts.push(part);
        }

        if let Some(binary_mt_data) = param_binary_mt_data {
            let part = Node::Part(Part {
                headers: {
                    let mut h = Headers::new();
                    h.set(ContentType("application/vnd.3gpp.5gnas".parse().unwrap()));
                    h.set_raw("Content-ID", vec![b"binaryMtData".to_vec()]);
                    h
                },
                body: binary_mt_data.0,
            });
            body_parts.push(part);
        }

        // Write the body into a vec.
        let mut body: Vec<u8> = vec![];
        write_multipart(&mut body, &boundary, &body_parts).expect("Failed to write multipart body");

        // Add the message body to the request object.
        *request.body_mut() = Body::from(body);

        let header = "multipart/related";
        request.headers_mut().insert(
            CONTENT_TYPE,
            match HeaderValue::from_bytes(
                &[
                    header.as_bytes(),
                    "; boundary=".as_bytes(),
                    &boundary,
                    "; type=\"application/json\"".as_bytes(),
                ]
                .concat(),
            ) {
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

        let response = client_service
            .call((request, context.clone()))
            .map_err(|e| ApiError(format!("No response received: {}", e)))
            .await?;

        match response.status().as_u16() {
            204 => Ok(TransferMtDataIsmfResponse::SuccessfulTransferingOfMTData),
            307 => {
                let response_location =
                    match response.headers().get(HeaderName::from_static("location")) {
                        Some(response_location) => {
                            let response_location = response_location.clone();
                            let response_location =
                                match TryInto::<header::IntoHeaderValue<String>>::try_into(
                                    response_location,
                                ) {
                                    Ok(value) => value,
                                    Err(e) => {
                                        return Err(ApiError(format!(
                                        "Invalid response header Location for response 307 - {}",
                                        e
                                    )));
                                    }
                                };
                            response_location.0
                        }
                        None => {
                            return Err(ApiError(String::from(
                                "Required response header Location for response 307 was not found.",
                            )))
                        }
                    };

                let response_param_3gpp_sbi_target_nf_id = match response
                    .headers()
                    .get(HeaderName::from_static("3gpp-sbi-target-nf-id"))
                {
                    Some(response_param_3gpp_sbi_target_nf_id) => {
                        let response_param_3gpp_sbi_target_nf_id =
                            response_param_3gpp_sbi_target_nf_id.clone();
                        let response_param_3gpp_sbi_target_nf_id = match TryInto::<
                            header::IntoHeaderValue<String>,
                        >::try_into(
                            response_param_3gpp_sbi_target_nf_id,
                        ) {
                            Ok(value) => value,
                            Err(e) => {
                                return Err(ApiError(format!("Invalid response header 3gpp-Sbi-Target-Nf-Id for response 307 - {}", e)));
                            }
                        };
                        Some(response_param_3gpp_sbi_target_nf_id.0)
                    }
                    None => None,
                };

                let body = response.into_body();
                let body = body
                    .into_raw()
                    .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                    .await?;
                let body = str::from_utf8(&body)
                    .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body = serde_json::from_str::<models::RedirectResponse>(body).map_err(|e| {
                    ApiError(format!("Response body did not match the schema: {}", e))
                })?;
                Ok(TransferMtDataIsmfResponse::TemporaryRedirect {
                    body,
                    location: response_location,
                    param_3gpp_sbi_target_nf_id: response_param_3gpp_sbi_target_nf_id,
                })
            }
            308 => {
                let response_location =
                    match response.headers().get(HeaderName::from_static("location")) {
                        Some(response_location) => {
                            let response_location = response_location.clone();
                            let response_location =
                                match TryInto::<header::IntoHeaderValue<String>>::try_into(
                                    response_location,
                                ) {
                                    Ok(value) => value,
                                    Err(e) => {
                                        return Err(ApiError(format!(
                                        "Invalid response header Location for response 308 - {}",
                                        e
                                    )));
                                    }
                                };
                            response_location.0
                        }
                        None => {
                            return Err(ApiError(String::from(
                                "Required response header Location for response 308 was not found.",
                            )))
                        }
                    };

                let response_param_3gpp_sbi_target_nf_id = match response
                    .headers()
                    .get(HeaderName::from_static("3gpp-sbi-target-nf-id"))
                {
                    Some(response_param_3gpp_sbi_target_nf_id) => {
                        let response_param_3gpp_sbi_target_nf_id =
                            response_param_3gpp_sbi_target_nf_id.clone();
                        let response_param_3gpp_sbi_target_nf_id = match TryInto::<
                            header::IntoHeaderValue<String>,
                        >::try_into(
                            response_param_3gpp_sbi_target_nf_id,
                        ) {
                            Ok(value) => value,
                            Err(e) => {
                                return Err(ApiError(format!("Invalid response header 3gpp-Sbi-Target-Nf-Id for response 308 - {}", e)));
                            }
                        };
                        Some(response_param_3gpp_sbi_target_nf_id.0)
                    }
                    None => None,
                };

                let body = response.into_body();
                let body = body
                    .into_raw()
                    .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                    .await?;
                let body = str::from_utf8(&body)
                    .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body = serde_json::from_str::<models::RedirectResponse>(body).map_err(|e| {
                    ApiError(format!("Response body did not match the schema: {}", e))
                })?;
                Ok(TransferMtDataIsmfResponse::PermanentRedirect {
                    body,
                    location: response_location,
                    param_3gpp_sbi_target_nf_id: response_param_3gpp_sbi_target_nf_id,
                })
            }
            400 => {
                let body = response.into_body();
                let body = body
                    .into_raw()
                    .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                    .await?;
                let body = str::from_utf8(&body)
                    .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body =
                    serde_json::from_str::<models::ExtProblemDetails>(body).map_err(|e| {
                        ApiError(format!("Response body did not match the schema: {}", e))
                    })?;
                Ok(TransferMtDataIsmfResponse::BadRequest(body))
            }
            401 => {
                let body = response.into_body();
                let body = body
                    .into_raw()
                    .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                    .await?;
                let body = str::from_utf8(&body)
                    .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body =
                    serde_json::from_str::<models::ExtProblemDetails>(body).map_err(|e| {
                        ApiError(format!("Response body did not match the schema: {}", e))
                    })?;
                Ok(TransferMtDataIsmfResponse::Unauthorized(body))
            }
            403 => {
                let body = response.into_body();
                let body = body
                    .into_raw()
                    .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                    .await?;
                let body = str::from_utf8(&body)
                    .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body =
                    serde_json::from_str::<models::ExtProblemDetails>(body).map_err(|e| {
                        ApiError(format!("Response body did not match the schema: {}", e))
                    })?;
                Ok(TransferMtDataIsmfResponse::Forbidden(body))
            }
            404 => {
                let body = response.into_body();
                let body = body
                    .into_raw()
                    .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                    .await?;
                let body = str::from_utf8(&body)
                    .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body =
                    serde_json::from_str::<models::ExtProblemDetails>(body).map_err(|e| {
                        ApiError(format!("Response body did not match the schema: {}", e))
                    })?;
                Ok(TransferMtDataIsmfResponse::NotFound(body))
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
                Ok(TransferMtDataIsmfResponse::LengthRequired(body))
            }
            413 => {
                let body = response.into_body();
                let body = body
                    .into_raw()
                    .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                    .await?;
                let body = str::from_utf8(&body)
                    .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body =
                    serde_json::from_str::<models::ExtProblemDetails>(body).map_err(|e| {
                        ApiError(format!("Response body did not match the schema: {}", e))
                    })?;
                Ok(TransferMtDataIsmfResponse::PayloadTooLarge(body))
            }
            415 => {
                let body = response.into_body();
                let body = body
                    .into_raw()
                    .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                    .await?;
                let body = str::from_utf8(&body)
                    .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body =
                    serde_json::from_str::<models::ExtProblemDetails>(body).map_err(|e| {
                        ApiError(format!("Response body did not match the schema: {}", e))
                    })?;
                Ok(TransferMtDataIsmfResponse::UnsupportedMediaType(body))
            }
            429 => {
                let body = response.into_body();
                let body = body
                    .into_raw()
                    .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                    .await?;
                let body = str::from_utf8(&body)
                    .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body =
                    serde_json::from_str::<models::ExtProblemDetails>(body).map_err(|e| {
                        ApiError(format!("Response body did not match the schema: {}", e))
                    })?;
                Ok(TransferMtDataIsmfResponse::TooManyRequests(body))
            }
            500 => {
                let body = response.into_body();
                let body = body
                    .into_raw()
                    .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                    .await?;
                let body = str::from_utf8(&body)
                    .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body =
                    serde_json::from_str::<models::ExtProblemDetails>(body).map_err(|e| {
                        ApiError(format!("Response body did not match the schema: {}", e))
                    })?;
                Ok(TransferMtDataIsmfResponse::InternalServerError(body))
            }
            503 => {
                let body = response.into_body();
                let body = body
                    .into_raw()
                    .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                    .await?;
                let body = str::from_utf8(&body)
                    .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body =
                    serde_json::from_str::<models::ExtProblemDetails>(body).map_err(|e| {
                        ApiError(format!("Response body did not match the schema: {}", e))
                    })?;
                Ok(TransferMtDataIsmfResponse::ServiceUnavailable(body))
            }
            504 => {
                let body = response.into_body();
                let body = body
                    .into_raw()
                    .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                    .await?;
                let body = str::from_utf8(&body)
                    .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body =
                    serde_json::from_str::<models::TransferMtDataError>(body).map_err(|e| {
                        ApiError(format!("Response body did not match the schema: {}", e))
                    })?;
                Ok(TransferMtDataIsmfResponse::UnsuccessfulDeliveryOfMobileTerminatedData(body))
            }
            0 => Ok(TransferMtDataIsmfResponse::GenericError),
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

    async fn sm_context_status_notification_post(
        &self,
        callback_request_body_sm_context_status_uri: String,
        param_sm_context_status_notification: models::SmContextStatusNotification,
        context: &C,
    ) -> Result<SmContextStatusNotificationPostResponse, ApiError> {
        let mut client_service = self.client_service.clone();
        let mut uri = format!(
            "{request_body_sm_context_status_uri}",
            request_body_sm_context_status_uri = callback_request_body_sm_context_status_uri
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
        let body = serde_json::to_string(&param_sm_context_status_notification)
            .expect("impossible to fail to serialize");

        *request.body_mut() = Body::from(body);

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

        let response = client_service
            .call((request, context.clone()))
            .map_err(|e| ApiError(format!("No response received: {}", e)))
            .await?;

        match response.status().as_u16() {
            204 => Ok(SmContextStatusNotificationPostResponse::SuccessfulNotification),
            307 => {
                let response_location =
                    match response.headers().get(HeaderName::from_static("location")) {
                        Some(response_location) => {
                            let response_location = response_location.clone();
                            let response_location =
                                match TryInto::<header::IntoHeaderValue<String>>::try_into(
                                    response_location,
                                ) {
                                    Ok(value) => value,
                                    Err(e) => {
                                        return Err(ApiError(format!(
                                        "Invalid response header Location for response 307 - {}",
                                        e
                                    )));
                                    }
                                };
                            response_location.0
                        }
                        None => {
                            return Err(ApiError(String::from(
                                "Required response header Location for response 307 was not found.",
                            )))
                        }
                    };

                let response_param_3gpp_sbi_target_nf_id = match response
                    .headers()
                    .get(HeaderName::from_static("3gpp-sbi-target-nf-id"))
                {
                    Some(response_param_3gpp_sbi_target_nf_id) => {
                        let response_param_3gpp_sbi_target_nf_id =
                            response_param_3gpp_sbi_target_nf_id.clone();
                        let response_param_3gpp_sbi_target_nf_id = match TryInto::<
                            header::IntoHeaderValue<String>,
                        >::try_into(
                            response_param_3gpp_sbi_target_nf_id,
                        ) {
                            Ok(value) => value,
                            Err(e) => {
                                return Err(ApiError(format!("Invalid response header 3gpp-Sbi-Target-Nf-Id for response 307 - {}", e)));
                            }
                        };
                        Some(response_param_3gpp_sbi_target_nf_id.0)
                    }
                    None => None,
                };

                let body = response.into_body();
                let body = body
                    .into_raw()
                    .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                    .await?;
                let body = str::from_utf8(&body)
                    .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body = serde_json::from_str::<models::RedirectResponse>(body).map_err(|e| {
                    ApiError(format!("Response body did not match the schema: {}", e))
                })?;
                Ok(SmContextStatusNotificationPostResponse::TemporaryRedirect {
                    body,
                    location: response_location,
                    param_3gpp_sbi_target_nf_id: response_param_3gpp_sbi_target_nf_id,
                })
            }
            308 => {
                let response_location =
                    match response.headers().get(HeaderName::from_static("location")) {
                        Some(response_location) => {
                            let response_location = response_location.clone();
                            let response_location =
                                match TryInto::<header::IntoHeaderValue<String>>::try_into(
                                    response_location,
                                ) {
                                    Ok(value) => value,
                                    Err(e) => {
                                        return Err(ApiError(format!(
                                        "Invalid response header Location for response 308 - {}",
                                        e
                                    )));
                                    }
                                };
                            response_location.0
                        }
                        None => {
                            return Err(ApiError(String::from(
                                "Required response header Location for response 308 was not found.",
                            )))
                        }
                    };

                let response_param_3gpp_sbi_target_nf_id = match response
                    .headers()
                    .get(HeaderName::from_static("3gpp-sbi-target-nf-id"))
                {
                    Some(response_param_3gpp_sbi_target_nf_id) => {
                        let response_param_3gpp_sbi_target_nf_id =
                            response_param_3gpp_sbi_target_nf_id.clone();
                        let response_param_3gpp_sbi_target_nf_id = match TryInto::<
                            header::IntoHeaderValue<String>,
                        >::try_into(
                            response_param_3gpp_sbi_target_nf_id,
                        ) {
                            Ok(value) => value,
                            Err(e) => {
                                return Err(ApiError(format!("Invalid response header 3gpp-Sbi-Target-Nf-Id for response 308 - {}", e)));
                            }
                        };
                        Some(response_param_3gpp_sbi_target_nf_id.0)
                    }
                    None => None,
                };

                let body = response.into_body();
                let body = body
                    .into_raw()
                    .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                    .await?;
                let body = str::from_utf8(&body)
                    .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body = serde_json::from_str::<models::RedirectResponse>(body).map_err(|e| {
                    ApiError(format!("Response body did not match the schema: {}", e))
                })?;
                Ok(SmContextStatusNotificationPostResponse::PermanentRedirect {
                    body,
                    location: response_location,
                    param_3gpp_sbi_target_nf_id: response_param_3gpp_sbi_target_nf_id,
                })
            }
            400 => {
                let body = response.into_body();
                let body = body
                    .into_raw()
                    .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                    .await?;
                let body = str::from_utf8(&body)
                    .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body =
                    serde_json::from_str::<models::ExtProblemDetails>(body).map_err(|e| {
                        ApiError(format!("Response body did not match the schema: {}", e))
                    })?;
                Ok(SmContextStatusNotificationPostResponse::BadRequest(body))
            }
            403 => {
                let body = response.into_body();
                let body = body
                    .into_raw()
                    .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                    .await?;
                let body = str::from_utf8(&body)
                    .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body =
                    serde_json::from_str::<models::ExtProblemDetails>(body).map_err(|e| {
                        ApiError(format!("Response body did not match the schema: {}", e))
                    })?;
                Ok(SmContextStatusNotificationPostResponse::Forbidden(body))
            }
            404 => {
                let body = response.into_body();
                let body = body
                    .into_raw()
                    .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                    .await?;
                let body = str::from_utf8(&body)
                    .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body =
                    serde_json::from_str::<models::ExtProblemDetails>(body).map_err(|e| {
                        ApiError(format!("Response body did not match the schema: {}", e))
                    })?;
                Ok(SmContextStatusNotificationPostResponse::NotFound(body))
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
                Ok(SmContextStatusNotificationPostResponse::LengthRequired(
                    body,
                ))
            }
            413 => {
                let body = response.into_body();
                let body = body
                    .into_raw()
                    .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                    .await?;
                let body = str::from_utf8(&body)
                    .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body =
                    serde_json::from_str::<models::ExtProblemDetails>(body).map_err(|e| {
                        ApiError(format!("Response body did not match the schema: {}", e))
                    })?;
                Ok(SmContextStatusNotificationPostResponse::PayloadTooLarge(
                    body,
                ))
            }
            415 => {
                let body = response.into_body();
                let body = body
                    .into_raw()
                    .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                    .await?;
                let body = str::from_utf8(&body)
                    .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body =
                    serde_json::from_str::<models::ExtProblemDetails>(body).map_err(|e| {
                        ApiError(format!("Response body did not match the schema: {}", e))
                    })?;
                Ok(SmContextStatusNotificationPostResponse::UnsupportedMediaType(body))
            }
            429 => {
                let body = response.into_body();
                let body = body
                    .into_raw()
                    .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                    .await?;
                let body = str::from_utf8(&body)
                    .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body =
                    serde_json::from_str::<models::ExtProblemDetails>(body).map_err(|e| {
                        ApiError(format!("Response body did not match the schema: {}", e))
                    })?;
                Ok(SmContextStatusNotificationPostResponse::TooManyRequests(
                    body,
                ))
            }
            500 => {
                let body = response.into_body();
                let body = body
                    .into_raw()
                    .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                    .await?;
                let body = str::from_utf8(&body)
                    .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body =
                    serde_json::from_str::<models::ExtProblemDetails>(body).map_err(|e| {
                        ApiError(format!("Response body did not match the schema: {}", e))
                    })?;
                Ok(SmContextStatusNotificationPostResponse::InternalServerError(body))
            }
            503 => {
                let body = response.into_body();
                let body = body
                    .into_raw()
                    .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                    .await?;
                let body = str::from_utf8(&body)
                    .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body =
                    serde_json::from_str::<models::ExtProblemDetails>(body).map_err(|e| {
                        ApiError(format!("Response body did not match the schema: {}", e))
                    })?;
                Ok(SmContextStatusNotificationPostResponse::ServiceUnavailable(
                    body,
                ))
            }
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
