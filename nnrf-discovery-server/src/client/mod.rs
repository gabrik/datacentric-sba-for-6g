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

use crate::{
    Api, RetrieveCompleteSearchResponse, RetrieveStoredSearchResponse,
    SCpDomainRoutingInfoGetResponse, ScpDomainRoutingInfoSubscribeResponse,
    ScpDomainRoutingInfoUnsubscribeResponse, SearchNfInstancesResponse,
};

pub mod callbacks;

/// Convert input into a base path, e.g. "http://example:123". Also checks the scheme as it goes.
fn into_base_path(
    input: impl TryInto<Uri, Error = hyper::http::uri::InvalidUri>,
    correct_scheme: Option<&'static str>,
) -> Result<String, ClientInitError> {
    // First convert to Uri, since a base path is a subset of Uri.
    let uri = input.try_into()?;

    let scheme = uri.scheme_str().ok_or(ClientInitError::InvalidScheme)?;

    // Check the scheme if necessary
    if let Some(correct_scheme) = correct_scheme {
        if scheme != correct_scheme {
            return Err(ClientInitError::InvalidScheme);
        }
    }

    let host = uri.host().ok_or(ClientInitError::MissingHost)?;
    let port = uri
        .port_u16()
        .map(|x| format!(":{}", x))
        .unwrap_or_default();
    Ok(format!(
        "{}://{}{}{}",
        scheme,
        host,
        port,
        uri.path().trim_end_matches('/')
    ))
}

/// A client that implements the API by making HTTP calls out to a server.
pub struct Client<S, C>
where
    S: Service<(Request<Body>, C), Response = Response<Body>> + Clone + Sync + Send + 'static,
    S::Future: Send + 'static,
    S::Error: Into<crate::ServiceError> + fmt::Display,
    C: Clone + Send + Sync + 'static,
{
    /// Inner service
    client_service: S,

    /// Base path of the API
    base_path: String,

    /// Marker
    marker: PhantomData<fn(C)>,
}

impl<S, C> fmt::Debug for Client<S, C>
where
    S: Service<(Request<Body>, C), Response = Response<Body>> + Clone + Sync + Send + 'static,
    S::Future: Send + 'static,
    S::Error: Into<crate::ServiceError> + fmt::Display,
    C: Clone + Send + Sync + 'static,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Client {{ base_path: {} }}", self.base_path)
    }
}

impl<S, C> Clone for Client<S, C>
where
    S: Service<(Request<Body>, C), Response = Response<Body>> + Clone + Sync + Send + 'static,
    S::Future: Send + 'static,
    S::Error: Into<crate::ServiceError> + fmt::Display,
    C: Clone + Send + Sync + 'static,
{
    fn clone(&self) -> Self {
        Self {
            client_service: self.client_service.clone(),
            base_path: self.base_path.clone(),
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
    /// For ordinary tcp connections, prefer the use of `try_new_http`, `try_new_https`
    /// and `try_new_https_mutual`, to avoid introducing a dependency on the underlying transport layer.
    ///
    /// # Arguments
    ///
    /// * `base_path` - base path of the client API, i.e. "http://www.my-api-implementation.com"
    /// * `protocol` - Which protocol to use when constructing the request url, e.g. `Some("http")`
    /// * `connector` - Implementation of `hyper::client::Connect` to use for the client
    pub fn try_new_with_connector(
        base_path: &str,
        protocol: Option<&'static str>,
        connector: Connector,
    ) -> Result<Self, ClientInitError> {
        let client_service = hyper::client::Client::builder().build(connector);
        let client_service = DropContextService::new(client_service);

        Ok(Self {
            client_service,
            base_path: into_base_path(base_path, protocol)?,
            marker: PhantomData,
        })
    }
}

#[derive(Debug, Clone)]
pub enum HyperClient {
    Http(hyper::client::Client<hyper::client::HttpConnector, Body>),
    Https(hyper::client::Client<HttpsConnector, Body>),
}

impl Service<Request<Body>> for HyperClient {
    type Response = Response<Body>;
    type Error = hyper::Error;
    type Future = hyper::client::ResponseFuture;

    fn poll_ready(&mut self, cx: &mut Context) -> Poll<Result<(), Self::Error>> {
        match self {
            HyperClient::Http(client) => client.poll_ready(cx),
            HyperClient::Https(client) => client.poll_ready(cx),
        }
    }

    fn call(&mut self, req: Request<Body>) -> Self::Future {
        match self {
            HyperClient::Http(client) => client.call(req),
            HyperClient::Https(client) => client.call(req),
        }
    }
}

impl<C> Client<DropContextService<HyperClient, C>, C>
where
    C: Clone + Send + Sync + 'static,
{
    /// Create an HTTP client.
    ///
    /// # Arguments
    /// * `base_path` - base path of the client API, i.e. "http://www.my-api-implementation.com"
    pub fn try_new(base_path: &str) -> Result<Self, ClientInitError> {
        let uri = Uri::from_str(base_path)?;

        let scheme = uri.scheme_str().ok_or(ClientInitError::InvalidScheme)?;
        let scheme = scheme.to_ascii_lowercase();

        let connector = Connector::builder();

        let client_service = match scheme.as_str() {
            "http" => HyperClient::Http(hyper::client::Client::builder().build(connector.build())),
            "https" => {
                let connector = connector
                    .https()
                    .build()
                    .map_err(ClientInitError::SslError)?;
                HyperClient::Https(hyper::client::Client::builder().build(connector))
            }
            _ => {
                return Err(ClientInitError::InvalidScheme);
            }
        };

        let client_service = DropContextService::new(client_service);

        Ok(Self {
            client_service,
            base_path: into_base_path(base_path, None)?,
            marker: PhantomData,
        })
    }
}

impl<C> Client<DropContextService<hyper::client::Client<hyper::client::HttpConnector, Body>, C>, C>
where
    C: Clone + Send + Sync + 'static,
{
    /// Create an HTTP client.
    ///
    /// # Arguments
    /// * `base_path` - base path of the client API, i.e. "http://www.my-api-implementation.com"
    pub fn try_new_http(base_path: &str) -> Result<Self, ClientInitError> {
        let http_connector = Connector::builder().build();

        Self::try_new_with_connector(base_path, Some("http"), http_connector)
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
    /// Create a client with a TLS connection to the server
    ///
    /// # Arguments
    /// * `base_path` - base path of the client API, i.e. "https://www.my-api-implementation.com"
    pub fn try_new_https(base_path: &str) -> Result<Self, ClientInitError> {
        let https_connector = Connector::builder()
            .https()
            .build()
            .map_err(ClientInitError::SslError)?;
        Self::try_new_with_connector(base_path, Some("https"), https_connector)
    }

    /// Create a client with a TLS connection to the server using a pinned certificate
    ///
    /// # Arguments
    /// * `base_path` - base path of the client API, i.e. "https://www.my-api-implementation.com"
    /// * `ca_certificate` - Path to CA certificate used to authenticate the server
    #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "ios")))]
    pub fn try_new_https_pinned<CA>(
        base_path: &str,
        ca_certificate: CA,
    ) -> Result<Self, ClientInitError>
    where
        CA: AsRef<Path>,
    {
        let https_connector = Connector::builder()
            .https()
            .pin_server_certificate(ca_certificate)
            .build()
            .map_err(ClientInitError::SslError)?;
        Self::try_new_with_connector(base_path, Some("https"), https_connector)
    }

    /// Create a client with a mutually authenticated TLS connection to the server.
    ///
    /// # Arguments
    /// * `base_path` - base path of the client API, i.e. "https://www.my-api-implementation.com"
    /// * `ca_certificate` - Path to CA certificate used to authenticate the server
    /// * `client_key` - Path to the client private key
    /// * `client_certificate` - Path to the client's public certificate associated with the private key
    #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "ios")))]
    pub fn try_new_https_mutual<CA, K, D>(
        base_path: &str,
        ca_certificate: CA,
        client_key: K,
        client_certificate: D,
    ) -> Result<Self, ClientInitError>
    where
        CA: AsRef<Path>,
        K: AsRef<Path>,
        D: AsRef<Path>,
    {
        let https_connector = Connector::builder()
            .https()
            .pin_server_certificate(ca_certificate)
            .client_authentication(client_key, client_certificate)
            .build()
            .map_err(ClientInitError::SslError)?;
        Self::try_new_with_connector(base_path, Some("https"), https_connector)
    }
}

impl<S, C> Client<S, C>
where
    S: Service<(Request<Body>, C), Response = Response<Body>> + Clone + Sync + Send + 'static,
    S::Future: Send + 'static,
    S::Error: Into<crate::ServiceError> + fmt::Display,
    C: Clone + Send + Sync + 'static,
{
    /// Constructor for creating a `Client` by passing in a pre-made `hyper::service::Service` /
    /// `tower::Service`
    ///
    /// This allows adding custom wrappers around the underlying transport, for example for logging.
    pub fn try_new_with_client_service(
        client_service: S,
        base_path: &str,
    ) -> Result<Self, ClientInitError> {
        Ok(Self {
            client_service,
            base_path: into_base_path(base_path, None)?,
            marker: PhantomData,
        })
    }
}

/// Error type failing to create a Client
#[derive(Debug)]
pub enum ClientInitError {
    /// Invalid URL Scheme
    InvalidScheme,

    /// Invalid URI
    InvalidUri(hyper::http::uri::InvalidUri),

    /// Missing Hostname
    MissingHost,

    /// SSL Connection Error
    #[cfg(any(target_os = "macos", target_os = "windows", target_os = "ios"))]
    SslError(native_tls::Error),

    /// SSL Connection Error
    #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "ios")))]
    SslError(openssl::error::ErrorStack),
}

impl From<hyper::http::uri::InvalidUri> for ClientInitError {
    fn from(err: hyper::http::uri::InvalidUri) -> ClientInitError {
        ClientInitError::InvalidUri(err)
    }
}

impl fmt::Display for ClientInitError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s: &dyn fmt::Debug = self;
        s.fmt(f)
    }
}

impl Error for ClientInitError {
    fn description(&self) -> &str {
        "Failed to produce a hyper client."
    }
}

#[async_trait]
impl<S, C> Api<C> for Client<S, C>
where
    S: Service<(Request<Body>, C), Response = Response<Body>> + Clone + Sync + Send + 'static,
    S::Future: Send + 'static,
    S::Error: Into<crate::ServiceError> + fmt::Display,
    C: Has<XSpanIdString> + Has<Option<AuthData>> + Clone + Send + Sync + 'static,
{
    fn poll_ready(&self, cx: &mut Context) -> Poll<Result<(), crate::ServiceError>> {
        match self.client_service.clone().poll_ready(cx) {
            Poll::Ready(Err(e)) => Poll::Ready(Err(e.into())),
            Poll::Ready(Ok(o)) => Poll::Ready(Ok(o)),
            Poll::Pending => Poll::Pending,
        }
    }

    async fn retrieve_complete_search(
        &self,
        param_search_id: String,
        param_accept_encoding: Option<String>,
        context: &C,
    ) -> Result<RetrieveCompleteSearchResponse, ApiError> {
        let mut client_service = self.client_service.clone();
        let mut uri = format!(
            "{}/nnrf-disc/v1/searches/{search_id}/complete",
            self.base_path,
            search_id = utf8_percent_encode(&param_search_id.to_string(), ID_ENCODE_SET)
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
            .method("GET")
            .uri(uri)
            .body(Body::empty())
        {
            Ok(req) => req,
            Err(e) => return Err(ApiError(format!("Unable to create request: {}", e))),
        };

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

        #[allow(clippy::collapsible_match)]
        if let Some(auth_data) = Has::<Option<AuthData>>::get(context).as_ref() {
            // Currently only authentication with Basic and Bearer are supported
            #[allow(clippy::single_match, clippy::match_single_binding)]
            match auth_data {
                &AuthData::Bearer(ref bearer_header) => {
                    let auth = swagger::auth::Header(bearer_header.clone());
                    let header = match HeaderValue::from_str(&format!("{}", auth)) {
                        Ok(h) => h,
                        Err(e) => {
                            return Err(ApiError(format!(
                                "Unable to create Authorization header: {}",
                                e
                            )))
                        }
                    };
                    request
                        .headers_mut()
                        .insert(hyper::header::AUTHORIZATION, header);
                }
                _ => {}
            }
        }

        // Header parameters
        match param_accept_encoding {
            Some(param_accept_encoding) => {
                request.headers_mut().append(
                    HeaderName::from_static("accept-encoding"),
                    #[allow(clippy::redundant_clone)]
                    match header::IntoHeaderValue(param_accept_encoding.clone()).try_into() {
                        Ok(header) => header,
                        Err(e) => {
                            return Err(ApiError(format!(
                                "Invalid header accept_encoding - {}",
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
            200 => {
                let response_cache_control = match response
                    .headers()
                    .get(HeaderName::from_static("cache-control"))
                {
                    Some(response_cache_control) => {
                        let response_cache_control = response_cache_control.clone();
                        let response_cache_control =
                            match TryInto::<header::IntoHeaderValue<String>>::try_into(
                                response_cache_control,
                            ) {
                                Ok(value) => value,
                                Err(e) => {
                                    return Err(ApiError(format!("Invalid response header Cache-Control for response 200 - {}", e)));
                                }
                            };
                        Some(response_cache_control.0)
                    }
                    None => None,
                };

                let response_e_tag = match response.headers().get(HeaderName::from_static("etag")) {
                    Some(response_e_tag) => {
                        let response_e_tag = response_e_tag.clone();
                        let response_e_tag =
                            match TryInto::<header::IntoHeaderValue<String>>::try_into(
                                response_e_tag,
                            ) {
                                Ok(value) => value,
                                Err(e) => {
                                    return Err(ApiError(format!(
                                        "Invalid response header ETag for response 200 - {}",
                                        e
                                    )));
                                }
                            };
                        Some(response_e_tag.0)
                    }
                    None => None,
                };

                let response_content_encoding = match response
                    .headers()
                    .get(HeaderName::from_static("content-encoding"))
                {
                    Some(response_content_encoding) => {
                        let response_content_encoding = response_content_encoding.clone();
                        let response_content_encoding = match TryInto::<
                            header::IntoHeaderValue<String>,
                        >::try_into(
                            response_content_encoding
                        ) {
                            Ok(value) => value,
                            Err(e) => {
                                return Err(ApiError(format!("Invalid response header Content-Encoding for response 200 - {}", e)));
                            }
                        };
                        Some(response_content_encoding.0)
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
                let body =
                    serde_json::from_str::<models::StoredSearchResult>(body).map_err(|e| {
                        ApiError(format!("Response body did not match the schema: {}", e))
                    })?;
                Ok(
                    RetrieveCompleteSearchResponse::ExpectedResponseToAValidRequest {
                        body,
                        cache_control: response_cache_control,
                        e_tag: response_e_tag,
                        content_encoding: response_content_encoding,
                    },
                )
            }
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
                Ok(RetrieveCompleteSearchResponse::TemporaryRedirect {
                    body,
                    location: response_location,
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
                Ok(RetrieveCompleteSearchResponse::PermanentRedirect {
                    body,
                    location: response_location,
                })
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

    async fn scp_domain_routing_info_unsubscribe(
        &self,
        param_subscription_id: String,
        context: &C,
    ) -> Result<ScpDomainRoutingInfoUnsubscribeResponse, ApiError> {
        let mut client_service = self.client_service.clone();
        let mut uri = format!(
            "{}/nnrf-disc/v1/scp-domain-routing-info-subs/{subscription_id}",
            self.base_path,
            subscription_id =
                utf8_percent_encode(&param_subscription_id.to_string(), ID_ENCODE_SET)
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
            .method("DELETE")
            .uri(uri)
            .body(Body::empty())
        {
            Ok(req) => req,
            Err(e) => return Err(ApiError(format!("Unable to create request: {}", e))),
        };

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

        #[allow(clippy::collapsible_match)]
        if let Some(auth_data) = Has::<Option<AuthData>>::get(context).as_ref() {
            // Currently only authentication with Basic and Bearer are supported
            #[allow(clippy::single_match, clippy::match_single_binding)]
            match auth_data {
                &AuthData::Bearer(ref bearer_header) => {
                    let auth = swagger::auth::Header(bearer_header.clone());
                    let header = match HeaderValue::from_str(&format!("{}", auth)) {
                        Ok(h) => h,
                        Err(e) => {
                            return Err(ApiError(format!(
                                "Unable to create Authorization header: {}",
                                e
                            )))
                        }
                    };
                    request
                        .headers_mut()
                        .insert(hyper::header::AUTHORIZATION, header);
                }
                _ => {}
            }
        }

        let response = client_service
            .call((request, context.clone()))
            .map_err(|e| ApiError(format!("No response received: {}", e)))
            .await?;

        match response.status().as_u16() {
            204 => {
                Ok(
                    ScpDomainRoutingInfoUnsubscribeResponse::ExpectedResponseToASuccessfulSubscriptionRemoval
                )
            }
            400 => {
                let body = response.into_body();
                let body = body
                        .into_raw()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e))).await?;
                let body = str::from_utf8(&body)
                    .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body = serde_json::from_str::<models::ProblemDetails>(body).map_err(|e| {
                    ApiError(format!("Response body did not match the schema: {}", e))
                })?;
                Ok(ScpDomainRoutingInfoUnsubscribeResponse::BadRequest
                    (body)
                )
            }
            401 => {
                let body = response.into_body();
                let body = body
                        .into_raw()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e))).await?;
                let body = str::from_utf8(&body)
                    .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body = serde_json::from_str::<models::ProblemDetails>(body).map_err(|e| {
                    ApiError(format!("Response body did not match the schema: {}", e))
                })?;
                Ok(ScpDomainRoutingInfoUnsubscribeResponse::Unauthorized
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
                let body = serde_json::from_str::<models::ProblemDetails>(body).map_err(|e| {
                    ApiError(format!("Response body did not match the schema: {}", e))
                })?;
                Ok(ScpDomainRoutingInfoUnsubscribeResponse::Forbidden
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
                let body = serde_json::from_str::<models::ProblemDetails>(body).map_err(|e| {
                    ApiError(format!("Response body did not match the schema: {}", e))
                })?;
                Ok(ScpDomainRoutingInfoUnsubscribeResponse::NotFound
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
                Ok(ScpDomainRoutingInfoUnsubscribeResponse::LengthRequired
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
                let body = serde_json::from_str::<models::ProblemDetails>(body).map_err(|e| {
                    ApiError(format!("Response body did not match the schema: {}", e))
                })?;
                Ok(ScpDomainRoutingInfoUnsubscribeResponse::PayloadTooLarge
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
                let body = serde_json::from_str::<models::ProblemDetails>(body).map_err(|e| {
                    ApiError(format!("Response body did not match the schema: {}", e))
                })?;
                Ok(ScpDomainRoutingInfoUnsubscribeResponse::UnsupportedMediaType
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
                let body = serde_json::from_str::<models::ProblemDetails>(body).map_err(|e| {
                    ApiError(format!("Response body did not match the schema: {}", e))
                })?;
                Ok(ScpDomainRoutingInfoUnsubscribeResponse::TooManyRequests
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
                let body = serde_json::from_str::<models::ProblemDetails>(body).map_err(|e| {
                    ApiError(format!("Response body did not match the schema: {}", e))
                })?;
                Ok(ScpDomainRoutingInfoUnsubscribeResponse::InternalServerError
                    (body)
                )
            }
            501 => {
                let body = response.into_body();
                let body = body
                        .into_raw()
                        .map_err(|e| ApiError(format!("Failed to read response: {}", e))).await?;
                let body = str::from_utf8(&body)
                    .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body = serde_json::from_str::<models::ProblemDetails>(body).map_err(|e| {
                    ApiError(format!("Response body did not match the schema: {}", e))
                })?;
                Ok(ScpDomainRoutingInfoUnsubscribeResponse::NotImplemented
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
                let body = serde_json::from_str::<models::ProblemDetails>(body).map_err(|e| {
                    ApiError(format!("Response body did not match the schema: {}", e))
                })?;
                Ok(ScpDomainRoutingInfoUnsubscribeResponse::ServiceUnavailable
                    (body)
                )
            }
            0 => {
                Ok(
                    ScpDomainRoutingInfoUnsubscribeResponse::GenericError
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

    async fn search_nf_instances(
        &self,
        param_target_nf_type: models::NfType,
        param_requester_nf_type: models::NfType,
        param_accept_encoding: Option<String>,
        param_preferred_collocated_nf_types: Option<&Vec<models::CollocatedNfType>>,
        param_requester_nf_instance_id: Option<uuid::Uuid>,
        param_service_names: Option<&Vec<models::ServiceName>>,
        param_requester_nf_instance_fqdn: Option<String>,
        param_target_plmn_list: Option<&Vec<models::PlmnId>>,
        param_requester_plmn_list: Option<&Vec<models::PlmnId>>,
        param_target_nf_instance_id: Option<uuid::Uuid>,
        param_target_nf_fqdn: Option<String>,
        param_hnrf_uri: Option<String>,
        param_snssais: Option<&Vec<models::Snssai>>,
        param_requester_snssais: Option<&Vec<models::ExtSnssai>>,
        param_plmn_specific_snssai_list: Option<&Vec<models::PlmnSnssai>>,
        param_requester_plmn_specific_snssai_list: Option<&Vec<models::PlmnSnssai>>,
        param_dnn: Option<String>,
        param_ipv4_index: Option<models::IpIndex>,
        param_ipv6_index: Option<models::IpIndex>,
        param_nsi_list: Option<&Vec<String>>,
        param_smf_serving_area: Option<String>,
        param_mbsmf_serving_area: Option<String>,
        param_tai: Option<models::Tai>,
        param_amf_region_id: Option<String>,
        param_amf_set_id: Option<String>,
        param_guami: Option<models::Guami>,
        param_supi: Option<String>,
        param_ue_ipv4_address: Option<String>,
        param_ip_domain: Option<String>,
        param_ue_ipv6_prefix: Option<models::Ipv6Prefix>,
        param_pgw_ind: Option<bool>,
        param_preferred_pgw_ind: Option<bool>,
        param_pgw: Option<String>,
        param_pgw_ip: Option<models::IpAddr>,
        param_gpsi: Option<String>,
        param_external_group_identity: Option<String>,
        param_internal_group_identity: Option<String>,
        param_pfd_data: Option<models::PfdData>,
        param_data_set: Option<models::DataSetId>,
        param_routing_indicator: Option<String>,
        param_group_id_list: Option<&Vec<models::NfGroupId>>,
        param_dnai_list: Option<&Vec<models::Dnai>>,
        param_pdu_session_types: Option<&Vec<models::PduSessionType>>,
        param_event_id_list: Option<&Vec<models::EventId>>,
        param_nwdaf_event_list: Option<&Vec<models::NwdafEvent>>,
        param_supported_features: Option<String>,
        param_upf_iwk_eps_ind: Option<bool>,
        param_chf_supported_plmn: Option<models::PlmnId>,
        param_preferred_locality: Option<String>,
        param_access_type: Option<models::AccessType>,
        param_limit: Option<i32>,
        param_required_features: Option<&Vec<models::SupportedFeatures>>,
        param_complex_query: Option<models::ComplexQuery>,
        param_max_payload_size: Option<i32>,
        param_max_payload_size_ext: Option<i32>,
        param_atsss_capability: Option<models::AtsssCapability>,
        param_upf_ue_ip_addr_ind: Option<bool>,
        param_client_type: Option<models::ExternalClientType>,
        param_lmf_id: Option<String>,
        param_an_node_type: Option<models::AnNodeType>,
        param_rat_type: Option<models::RatType>,
        param_preferred_tai: Option<models::Tai>,
        param_preferred_nf_instances: Option<&Vec<models::NfInstanceId>>,
        param_if_none_match: Option<String>,
        param_target_snpn: Option<models::PlmnIdNid>,
        param_requester_snpn_list: Option<&Vec<models::PlmnIdNid>>,
        param_af_ee_data: Option<models::AfEventExposureData>,
        param_w_agf_info: Option<models::WAgfInfo>,
        param_tngf_info: Option<models::TngfInfo>,
        param_twif_info: Option<models::TwifInfo>,
        param_target_nf_set_id: Option<String>,
        param_target_nf_service_set_id: Option<String>,
        param_nef_id: Option<String>,
        param_notification_type: Option<models::NotificationType>,
        param_n1_msg_class: Option<models::N1MessageClass>,
        param_n2_info_class: Option<models::N2InformationClass>,
        param_serving_scope: Option<&Vec<String>>,
        param_imsi: Option<String>,
        param_ims_private_identity: Option<String>,
        param_ims_public_identity: Option<String>,
        param_msisdn: Option<String>,
        param_preferred_api_versions: Option<std::collections::HashMap<String, String>>,
        param_v2x_support_ind: Option<bool>,
        param_redundant_gtpu: Option<bool>,
        param_redundant_transport: Option<bool>,
        param_ipups: Option<bool>,
        param_scp_domain_list: Option<&Vec<String>>,
        param_address_domain: Option<String>,
        param_ipv4_addr: Option<String>,
        param_ipv6_prefix: Option<models::Ipv6Prefix>,
        param_served_nf_set_id: Option<String>,
        param_remote_plmn_id: Option<models::PlmnId>,
        param_remote_snpn_id: Option<models::PlmnIdNid>,
        param_data_forwarding: Option<bool>,
        param_preferred_full_plmn: Option<bool>,
        param_requester_features: Option<String>,
        param_realm_id: Option<String>,
        param_storage_id: Option<String>,
        param_vsmf_support_ind: Option<bool>,
        param_ismf_support_ind: Option<bool>,
        param_nrf_disc_uri: Option<String>,
        param_preferred_vendor_specific_features: Option<
            std::collections::HashMap<
                String,
                std::collections::HashMap<String, Vec<models::VendorSpecificFeature>>,
            >,
        >,
        param_preferred_vendor_specific_nf_features: Option<
            std::collections::HashMap<String, Vec<models::VendorSpecificFeature>>,
        >,
        param_required_pfcp_features: Option<String>,
        param_home_pub_key_id: Option<i32>,
        param_prose_support_ind: Option<bool>,
        param_analytics_aggregation_ind: Option<bool>,
        param_serving_nf_set_id: Option<String>,
        param_serving_nf_type: Option<models::NfType>,
        param_ml_analytics_info_list: Option<&Vec<models::MlAnalyticsInfo>>,
        param_analytics_metadata_prov_ind: Option<bool>,
        param_nsacf_capability: Option<models::NsacfCapability>,
        param_mbs_session_id_list: Option<&Vec<models::MbsSessionId>>,
        param_area_session_id: Option<i32>,
        param_gmlc_number: Option<String>,
        param_upf_n6_ip: Option<models::IpAddr>,
        param_tai_list: Option<&Vec<models::Tai>>,
        param_preferences_precedence: Option<&Vec<String>>,
        param_support_onboarding_capability: Option<bool>,
        param_uas_nf_functionality_ind: Option<bool>,
        param_v2x_capability: Option<models::V2xCapability>,
        param_prose_capability: Option<models::ProSeCapability>,
        param_shared_data_id: Option<String>,
        param_target_hni: Option<String>,
        param_target_nw_resolution: Option<bool>,
        param_exclude_nfinst_list: Option<&Vec<models::NfInstanceId>>,
        param_exclude_nfservinst_list: Option<&Vec<models::NfServiceInstance>>,
        param_exclude_nfserviceset_list: Option<&Vec<models::NfServiceSetId>>,
        param_exclude_nfset_list: Option<&Vec<models::NfSetId>>,
        param_preferred_analytics_delays: Option<
            std::collections::HashMap<String, models::DurationSec>,
        >,
        context: &C,
    ) -> Result<SearchNfInstancesResponse, ApiError> {
        let mut client_service = self.client_service.clone();
        let mut uri = format!("{}/nnrf-disc/v1/nf-instances", self.base_path);

        // Query parameters
        let query_string = {
            let mut query_string = form_urlencoded::Serializer::new("".to_owned());
            query_string.append_pair("target-nf-type", &param_target_nf_type.to_string());
            query_string.append_pair("requester-nf-type", &param_requester_nf_type.to_string());
            if let Some(param_preferred_collocated_nf_types) = param_preferred_collocated_nf_types {
                query_string.append_pair(
                    "preferred-collocated-nf-types",
                    &param_preferred_collocated_nf_types
                        .iter()
                        .map(ToString::to_string)
                        .collect::<Vec<String>>()
                        .join(","),
                );
            }
            if let Some(param_requester_nf_instance_id) = param_requester_nf_instance_id {
                query_string.append_pair(
                    "requester-nf-instance-id",
                    &param_requester_nf_instance_id.to_string(),
                );
            }
            if let Some(param_service_names) = param_service_names {
                query_string.append_pair(
                    "service-names",
                    &param_service_names
                        .iter()
                        .map(ToString::to_string)
                        .collect::<Vec<String>>()
                        .join(","),
                );
            }
            if let Some(param_requester_nf_instance_fqdn) = param_requester_nf_instance_fqdn {
                query_string.append_pair(
                    "requester-nf-instance-fqdn",
                    &param_requester_nf_instance_fqdn,
                );
            }
            if let Some(param_target_plmn_list) = param_target_plmn_list {
                query_string.append_pair(
                    "target-plmn-list",
                    &match serde_json::to_string(&param_target_plmn_list) {
                        Ok(str) => str,
                        Err(e) => {
                            return Err(ApiError(format!(
                                "Unable to serialize target_plmn_list to string: {}",
                                e
                            )))
                        }
                    },
                );
            }
            if let Some(param_requester_plmn_list) = param_requester_plmn_list {
                query_string.append_pair(
                    "requester-plmn-list",
                    &match serde_json::to_string(&param_requester_plmn_list) {
                        Ok(str) => str,
                        Err(e) => {
                            return Err(ApiError(format!(
                                "Unable to serialize requester_plmn_list to string: {}",
                                e
                            )))
                        }
                    },
                );
            }
            if let Some(param_target_nf_instance_id) = param_target_nf_instance_id {
                query_string.append_pair(
                    "target-nf-instance-id",
                    &param_target_nf_instance_id.to_string(),
                );
            }
            if let Some(param_target_nf_fqdn) = param_target_nf_fqdn {
                query_string.append_pair("target-nf-fqdn", &param_target_nf_fqdn);
            }
            if let Some(param_hnrf_uri) = param_hnrf_uri {
                query_string.append_pair("hnrf-uri", &param_hnrf_uri);
            }
            if let Some(param_snssais) = param_snssais {
                query_string.append_pair(
                    "snssais",
                    &match serde_json::to_string(&param_snssais) {
                        Ok(str) => str,
                        Err(e) => {
                            return Err(ApiError(format!(
                                "Unable to serialize snssais to string: {}",
                                e
                            )))
                        }
                    },
                );
            }
            if let Some(param_requester_snssais) = param_requester_snssais {
                query_string.append_pair(
                    "requester-snssais",
                    &match serde_json::to_string(&param_requester_snssais) {
                        Ok(str) => str,
                        Err(e) => {
                            return Err(ApiError(format!(
                                "Unable to serialize requester_snssais to string: {}",
                                e
                            )))
                        }
                    },
                );
            }
            if let Some(param_plmn_specific_snssai_list) = param_plmn_specific_snssai_list {
                query_string.append_pair(
                    "plmn-specific-snssai-list",
                    &match serde_json::to_string(&param_plmn_specific_snssai_list) {
                        Ok(str) => str,
                        Err(e) => {
                            return Err(ApiError(format!(
                                "Unable to serialize plmn_specific_snssai_list to string: {}",
                                e
                            )))
                        }
                    },
                );
            }
            if let Some(param_requester_plmn_specific_snssai_list) =
                param_requester_plmn_specific_snssai_list
            {
                query_string.append_pair(
                    "requester-plmn-specific-snssai-list",
                    &match serde_json::to_string(&param_requester_plmn_specific_snssai_list) {
                        Ok(str) => str,
                        Err(e) => {
                            return Err(ApiError(format!(
                            "Unable to serialize requester_plmn_specific_snssai_list to string: {}",
                            e
                        )))
                        }
                    },
                );
            }
            if let Some(param_dnn) = param_dnn {
                query_string.append_pair("dnn", &param_dnn);
            }
            if let Some(param_ipv4_index) = param_ipv4_index {
                query_string.append_pair(
                    "ipv4-index",
                    &match serde_json::to_string(&param_ipv4_index) {
                        Ok(str) => str,
                        Err(e) => {
                            return Err(ApiError(format!(
                                "Unable to serialize ipv4_index to string: {}",
                                e
                            )))
                        }
                    },
                );
            }
            if let Some(param_ipv6_index) = param_ipv6_index {
                query_string.append_pair(
                    "ipv6-index",
                    &match serde_json::to_string(&param_ipv6_index) {
                        Ok(str) => str,
                        Err(e) => {
                            return Err(ApiError(format!(
                                "Unable to serialize ipv6_index to string: {}",
                                e
                            )))
                        }
                    },
                );
            }
            if let Some(param_nsi_list) = param_nsi_list {
                query_string.append_pair(
                    "nsi-list",
                    &param_nsi_list
                        .iter()
                        .map(ToString::to_string)
                        .collect::<Vec<String>>()
                        .join(","),
                );
            }
            if let Some(param_smf_serving_area) = param_smf_serving_area {
                query_string.append_pair("smf-serving-area", &param_smf_serving_area);
            }
            if let Some(param_mbsmf_serving_area) = param_mbsmf_serving_area {
                query_string.append_pair("mbsmf-serving-area", &param_mbsmf_serving_area);
            }
            if let Some(param_tai) = param_tai {
                query_string.append_pair(
                    "tai",
                    &match serde_json::to_string(&param_tai) {
                        Ok(str) => str,
                        Err(e) => {
                            return Err(ApiError(format!(
                                "Unable to serialize tai to string: {}",
                                e
                            )))
                        }
                    },
                );
            }
            if let Some(param_amf_region_id) = param_amf_region_id {
                query_string.append_pair("amf-region-id", &param_amf_region_id);
            }
            if let Some(param_amf_set_id) = param_amf_set_id {
                query_string.append_pair("amf-set-id", &param_amf_set_id);
            }
            if let Some(param_guami) = param_guami {
                query_string.append_pair(
                    "guami",
                    &match serde_json::to_string(&param_guami) {
                        Ok(str) => str,
                        Err(e) => {
                            return Err(ApiError(format!(
                                "Unable to serialize guami to string: {}",
                                e
                            )))
                        }
                    },
                );
            }
            if let Some(param_supi) = param_supi {
                query_string.append_pair("supi", &param_supi);
            }
            if let Some(param_ue_ipv4_address) = param_ue_ipv4_address {
                query_string.append_pair("ue-ipv4-address", &param_ue_ipv4_address);
            }
            if let Some(param_ip_domain) = param_ip_domain {
                query_string.append_pair("ip-domain", &param_ip_domain);
            }
            if let Some(param_ue_ipv6_prefix) = param_ue_ipv6_prefix {
                query_string.append_pair("ue-ipv6-prefix", &param_ue_ipv6_prefix.to_string());
            }
            if let Some(param_pgw_ind) = param_pgw_ind {
                query_string.append_pair("pgw-ind", &param_pgw_ind.to_string());
            }
            if let Some(param_preferred_pgw_ind) = param_preferred_pgw_ind {
                query_string.append_pair("preferred-pgw-ind", &param_preferred_pgw_ind.to_string());
            }
            if let Some(param_pgw) = param_pgw {
                query_string.append_pair("pgw", &param_pgw);
            }
            if let Some(param_pgw_ip) = param_pgw_ip {
                query_string.append_pair(
                    "pgw-ip",
                    &match serde_json::to_string(&param_pgw_ip) {
                        Ok(str) => str,
                        Err(e) => {
                            return Err(ApiError(format!(
                                "Unable to serialize pgw_ip to string: {}",
                                e
                            )))
                        }
                    },
                );
            }
            if let Some(param_gpsi) = param_gpsi {
                query_string.append_pair("gpsi", &param_gpsi);
            }
            if let Some(param_external_group_identity) = param_external_group_identity {
                query_string.append_pair("external-group-identity", &param_external_group_identity);
            }
            if let Some(param_internal_group_identity) = param_internal_group_identity {
                query_string.append_pair("internal-group-identity", &param_internal_group_identity);
            }
            if let Some(param_pfd_data) = param_pfd_data {
                query_string.append_pair(
                    "pfd-data",
                    &match serde_json::to_string(&param_pfd_data) {
                        Ok(str) => str,
                        Err(e) => {
                            return Err(ApiError(format!(
                                "Unable to serialize pfd_data to string: {}",
                                e
                            )))
                        }
                    },
                );
            }
            if let Some(param_data_set) = param_data_set {
                query_string.append_pair("data-set", &param_data_set.to_string());
            }
            if let Some(param_routing_indicator) = param_routing_indicator {
                query_string.append_pair("routing-indicator", &param_routing_indicator);
            }
            if let Some(param_group_id_list) = param_group_id_list {
                query_string.append_pair(
                    "group-id-list",
                    &param_group_id_list
                        .iter()
                        .map(ToString::to_string)
                        .collect::<Vec<String>>()
                        .join(","),
                );
            }
            if let Some(param_dnai_list) = param_dnai_list {
                query_string.append_pair(
                    "dnai-list",
                    &param_dnai_list
                        .iter()
                        .map(ToString::to_string)
                        .collect::<Vec<String>>()
                        .join(","),
                );
            }
            if let Some(param_pdu_session_types) = param_pdu_session_types {
                query_string.append_pair(
                    "pdu-session-types",
                    &param_pdu_session_types
                        .iter()
                        .map(ToString::to_string)
                        .collect::<Vec<String>>()
                        .join(","),
                );
            }
            if let Some(param_event_id_list) = param_event_id_list {
                query_string.append_pair(
                    "event-id-list",
                    &param_event_id_list
                        .iter()
                        .map(ToString::to_string)
                        .collect::<Vec<String>>()
                        .join(","),
                );
            }
            if let Some(param_nwdaf_event_list) = param_nwdaf_event_list {
                query_string.append_pair(
                    "nwdaf-event-list",
                    &param_nwdaf_event_list
                        .iter()
                        .map(ToString::to_string)
                        .collect::<Vec<String>>()
                        .join(","),
                );
            }
            if let Some(param_supported_features) = param_supported_features {
                query_string.append_pair("supported-features", &param_supported_features);
            }
            if let Some(param_upf_iwk_eps_ind) = param_upf_iwk_eps_ind {
                query_string.append_pair("upf-iwk-eps-ind", &param_upf_iwk_eps_ind.to_string());
            }
            if let Some(param_chf_supported_plmn) = param_chf_supported_plmn {
                query_string.append_pair(
                    "chf-supported-plmn",
                    &match serde_json::to_string(&param_chf_supported_plmn) {
                        Ok(str) => str,
                        Err(e) => {
                            return Err(ApiError(format!(
                                "Unable to serialize chf_supported_plmn to string: {}",
                                e
                            )))
                        }
                    },
                );
            }
            if let Some(param_preferred_locality) = param_preferred_locality {
                query_string.append_pair("preferred-locality", &param_preferred_locality);
            }
            if let Some(param_access_type) = param_access_type {
                query_string.append_pair("access-type", &param_access_type.to_string());
            }
            if let Some(param_limit) = param_limit {
                query_string.append_pair("limit", &param_limit.to_string());
            }
            if let Some(param_required_features) = param_required_features {
                query_string.append_pair(
                    "required-features",
                    &param_required_features
                        .iter()
                        .map(ToString::to_string)
                        .collect::<Vec<String>>()
                        .join(","),
                );
            }
            if let Some(param_complex_query) = param_complex_query {
                query_string.append_pair(
                    "complex-query",
                    &match serde_json::to_string(&param_complex_query) {
                        Ok(str) => str,
                        Err(e) => {
                            return Err(ApiError(format!(
                                "Unable to serialize complex_query to string: {}",
                                e
                            )))
                        }
                    },
                );
            }
            if let Some(param_max_payload_size) = param_max_payload_size {
                query_string.append_pair("max-payload-size", &param_max_payload_size.to_string());
            }
            if let Some(param_max_payload_size_ext) = param_max_payload_size_ext {
                query_string.append_pair(
                    "max-payload-size-ext",
                    &param_max_payload_size_ext.to_string(),
                );
            }
            if let Some(param_atsss_capability) = param_atsss_capability {
                query_string.append_pair(
                    "atsss-capability",
                    &match serde_json::to_string(&param_atsss_capability) {
                        Ok(str) => str,
                        Err(e) => {
                            return Err(ApiError(format!(
                                "Unable to serialize atsss_capability to string: {}",
                                e
                            )))
                        }
                    },
                );
            }
            if let Some(param_upf_ue_ip_addr_ind) = param_upf_ue_ip_addr_ind {
                query_string
                    .append_pair("upf-ue-ip-addr-ind", &param_upf_ue_ip_addr_ind.to_string());
            }
            if let Some(param_client_type) = param_client_type {
                query_string.append_pair(
                    "client-type",
                    &match serde_json::to_string(&param_client_type) {
                        Ok(str) => str,
                        Err(e) => {
                            return Err(ApiError(format!(
                                "Unable to serialize client_type to string: {}",
                                e
                            )))
                        }
                    },
                );
            }
            if let Some(param_lmf_id) = param_lmf_id {
                query_string.append_pair(
                    "lmf-id",
                    &match serde_json::to_string(&param_lmf_id) {
                        Ok(str) => str,
                        Err(e) => {
                            return Err(ApiError(format!(
                                "Unable to serialize lmf_id to string: {}",
                                e
                            )))
                        }
                    },
                );
            }
            if let Some(param_an_node_type) = param_an_node_type {
                query_string.append_pair(
                    "an-node-type",
                    &match serde_json::to_string(&param_an_node_type) {
                        Ok(str) => str,
                        Err(e) => {
                            return Err(ApiError(format!(
                                "Unable to serialize an_node_type to string: {}",
                                e
                            )))
                        }
                    },
                );
            }
            if let Some(param_rat_type) = param_rat_type {
                query_string.append_pair(
                    "rat-type",
                    &match serde_json::to_string(&param_rat_type) {
                        Ok(str) => str,
                        Err(e) => {
                            return Err(ApiError(format!(
                                "Unable to serialize rat_type to string: {}",
                                e
                            )))
                        }
                    },
                );
            }
            if let Some(param_preferred_tai) = param_preferred_tai {
                query_string.append_pair(
                    "preferred-tai",
                    &match serde_json::to_string(&param_preferred_tai) {
                        Ok(str) => str,
                        Err(e) => {
                            return Err(ApiError(format!(
                                "Unable to serialize preferred_tai to string: {}",
                                e
                            )))
                        }
                    },
                );
            }
            if let Some(param_preferred_nf_instances) = param_preferred_nf_instances {
                query_string.append_pair(
                    "preferred-nf-instances",
                    &param_preferred_nf_instances
                        .iter()
                        .map(ToString::to_string)
                        .collect::<Vec<String>>()
                        .join(","),
                );
            }
            if let Some(param_target_snpn) = param_target_snpn {
                query_string.append_pair(
                    "target-snpn",
                    &match serde_json::to_string(&param_target_snpn) {
                        Ok(str) => str,
                        Err(e) => {
                            return Err(ApiError(format!(
                                "Unable to serialize target_snpn to string: {}",
                                e
                            )))
                        }
                    },
                );
            }
            if let Some(param_requester_snpn_list) = param_requester_snpn_list {
                query_string.append_pair(
                    "requester-snpn-list",
                    &match serde_json::to_string(&param_requester_snpn_list) {
                        Ok(str) => str,
                        Err(e) => {
                            return Err(ApiError(format!(
                                "Unable to serialize requester_snpn_list to string: {}",
                                e
                            )))
                        }
                    },
                );
            }
            if let Some(param_af_ee_data) = param_af_ee_data {
                query_string.append_pair(
                    "af-ee-data",
                    &match serde_json::to_string(&param_af_ee_data) {
                        Ok(str) => str,
                        Err(e) => {
                            return Err(ApiError(format!(
                                "Unable to serialize af_ee_data to string: {}",
                                e
                            )))
                        }
                    },
                );
            }
            if let Some(param_w_agf_info) = param_w_agf_info {
                query_string.append_pair(
                    "w-agf-info",
                    &match serde_json::to_string(&param_w_agf_info) {
                        Ok(str) => str,
                        Err(e) => {
                            return Err(ApiError(format!(
                                "Unable to serialize w_agf_info to string: {}",
                                e
                            )))
                        }
                    },
                );
            }
            if let Some(param_tngf_info) = param_tngf_info {
                query_string.append_pair(
                    "tngf-info",
                    &match serde_json::to_string(&param_tngf_info) {
                        Ok(str) => str,
                        Err(e) => {
                            return Err(ApiError(format!(
                                "Unable to serialize tngf_info to string: {}",
                                e
                            )))
                        }
                    },
                );
            }
            if let Some(param_twif_info) = param_twif_info {
                query_string.append_pair(
                    "twif-info",
                    &match serde_json::to_string(&param_twif_info) {
                        Ok(str) => str,
                        Err(e) => {
                            return Err(ApiError(format!(
                                "Unable to serialize twif_info to string: {}",
                                e
                            )))
                        }
                    },
                );
            }
            if let Some(param_target_nf_set_id) = param_target_nf_set_id {
                query_string.append_pair("target-nf-set-id", &param_target_nf_set_id);
            }
            if let Some(param_target_nf_service_set_id) = param_target_nf_service_set_id {
                query_string
                    .append_pair("target-nf-service-set-id", &param_target_nf_service_set_id);
            }
            if let Some(param_nef_id) = param_nef_id {
                query_string.append_pair("nef-id", &param_nef_id);
            }
            if let Some(param_notification_type) = param_notification_type {
                query_string.append_pair("notification-type", &param_notification_type.to_string());
            }
            if let Some(param_n1_msg_class) = param_n1_msg_class {
                query_string.append_pair("n1-msg-class", &param_n1_msg_class.to_string());
            }
            if let Some(param_n2_info_class) = param_n2_info_class {
                query_string.append_pair("n2-info-class", &param_n2_info_class.to_string());
            }
            if let Some(param_serving_scope) = param_serving_scope {
                query_string.append_pair(
                    "serving-scope",
                    &param_serving_scope
                        .iter()
                        .map(ToString::to_string)
                        .collect::<Vec<String>>()
                        .join(","),
                );
            }
            if let Some(param_imsi) = param_imsi {
                query_string.append_pair("imsi", &param_imsi);
            }
            if let Some(param_ims_private_identity) = param_ims_private_identity {
                query_string.append_pair("ims-private-identity", &param_ims_private_identity);
            }
            if let Some(param_ims_public_identity) = param_ims_public_identity {
                query_string.append_pair("ims-public-identity", &param_ims_public_identity);
            }
            if let Some(param_msisdn) = param_msisdn {
                query_string.append_pair("msisdn", &param_msisdn);
            }
            if let Some(param_preferred_api_versions) = param_preferred_api_versions {
                query_string.append_pair(
                    "preferred-api-versions",
                    &match serde_json::to_string(&param_preferred_api_versions) {
                        Ok(str) => str,
                        Err(e) => {
                            return Err(ApiError(format!(
                                "Unable to serialize preferred_api_versions to string: {}",
                                e
                            )))
                        }
                    },
                );
            }
            if let Some(param_v2x_support_ind) = param_v2x_support_ind {
                query_string.append_pair("v2x-support-ind", &param_v2x_support_ind.to_string());
            }
            if let Some(param_redundant_gtpu) = param_redundant_gtpu {
                query_string.append_pair("redundant-gtpu", &param_redundant_gtpu.to_string());
            }
            if let Some(param_redundant_transport) = param_redundant_transport {
                query_string.append_pair(
                    "redundant-transport",
                    &param_redundant_transport.to_string(),
                );
            }
            if let Some(param_ipups) = param_ipups {
                query_string.append_pair("ipups", &param_ipups.to_string());
            }
            if let Some(param_scp_domain_list) = param_scp_domain_list {
                query_string.append_pair(
                    "scp-domain-list",
                    &param_scp_domain_list
                        .iter()
                        .map(ToString::to_string)
                        .collect::<Vec<String>>()
                        .join(","),
                );
            }
            if let Some(param_address_domain) = param_address_domain {
                query_string.append_pair("address-domain", &param_address_domain);
            }
            if let Some(param_ipv4_addr) = param_ipv4_addr {
                query_string.append_pair("ipv4-addr", &param_ipv4_addr);
            }
            if let Some(param_ipv6_prefix) = param_ipv6_prefix {
                query_string.append_pair("ipv6-prefix", &param_ipv6_prefix.to_string());
            }
            if let Some(param_served_nf_set_id) = param_served_nf_set_id {
                query_string.append_pair("served-nf-set-id", &param_served_nf_set_id);
            }
            if let Some(param_remote_plmn_id) = param_remote_plmn_id {
                query_string.append_pair(
                    "remote-plmn-id",
                    &match serde_json::to_string(&param_remote_plmn_id) {
                        Ok(str) => str,
                        Err(e) => {
                            return Err(ApiError(format!(
                                "Unable to serialize remote_plmn_id to string: {}",
                                e
                            )))
                        }
                    },
                );
            }
            if let Some(param_remote_snpn_id) = param_remote_snpn_id {
                query_string.append_pair(
                    "remote-snpn-id",
                    &match serde_json::to_string(&param_remote_snpn_id) {
                        Ok(str) => str,
                        Err(e) => {
                            return Err(ApiError(format!(
                                "Unable to serialize remote_snpn_id to string: {}",
                                e
                            )))
                        }
                    },
                );
            }
            if let Some(param_data_forwarding) = param_data_forwarding {
                query_string.append_pair("data-forwarding", &param_data_forwarding.to_string());
            }
            if let Some(param_preferred_full_plmn) = param_preferred_full_plmn {
                query_string.append_pair(
                    "preferred-full-plmn",
                    &param_preferred_full_plmn.to_string(),
                );
            }
            if let Some(param_requester_features) = param_requester_features {
                query_string.append_pair("requester-features", &param_requester_features);
            }
            if let Some(param_realm_id) = param_realm_id {
                query_string.append_pair("realm-id", &param_realm_id);
            }
            if let Some(param_storage_id) = param_storage_id {
                query_string.append_pair("storage-id", &param_storage_id);
            }
            if let Some(param_vsmf_support_ind) = param_vsmf_support_ind {
                query_string.append_pair("vsmf-support-ind", &param_vsmf_support_ind.to_string());
            }
            if let Some(param_ismf_support_ind) = param_ismf_support_ind {
                query_string.append_pair("ismf-support-ind", &param_ismf_support_ind.to_string());
            }
            if let Some(param_nrf_disc_uri) = param_nrf_disc_uri {
                query_string.append_pair("nrf-disc-uri", &param_nrf_disc_uri);
            }
            if let Some(param_preferred_vendor_specific_features) =
                param_preferred_vendor_specific_features
            {
                query_string.append_pair(
                    "preferred-vendor-specific-features",
                    &match serde_json::to_string(&param_preferred_vendor_specific_features) {
                        Ok(str) => str,
                        Err(e) => {
                            return Err(ApiError(format!(
                            "Unable to serialize preferred_vendor_specific_features to string: {}",
                            e
                        )))
                        }
                    },
                );
            }
            if let Some(param_preferred_vendor_specific_nf_features) =
                param_preferred_vendor_specific_nf_features
            {
                query_string.append_pair("preferred-vendor-specific-nf-features",
                    &match serde_json::to_string(&param_preferred_vendor_specific_nf_features) {
                        Ok(str) => str,
                        Err(e) => return Err(ApiError(format!("Unable to serialize preferred_vendor_specific_nf_features to string: {}", e))),
                    });
            }
            if let Some(param_required_pfcp_features) = param_required_pfcp_features {
                query_string.append_pair("required-pfcp-features", &param_required_pfcp_features);
            }
            if let Some(param_home_pub_key_id) = param_home_pub_key_id {
                query_string.append_pair("home-pub-key-id", &param_home_pub_key_id.to_string());
            }
            if let Some(param_prose_support_ind) = param_prose_support_ind {
                query_string.append_pair("prose-support-ind", &param_prose_support_ind.to_string());
            }
            if let Some(param_analytics_aggregation_ind) = param_analytics_aggregation_ind {
                query_string.append_pair(
                    "analytics-aggregation-ind",
                    &param_analytics_aggregation_ind.to_string(),
                );
            }
            if let Some(param_serving_nf_set_id) = param_serving_nf_set_id {
                query_string.append_pair("serving-nf-set-id", &param_serving_nf_set_id);
            }
            if let Some(param_serving_nf_type) = param_serving_nf_type {
                query_string.append_pair("serving-nf-type", &param_serving_nf_type.to_string());
            }
            if let Some(param_ml_analytics_info_list) = param_ml_analytics_info_list {
                query_string.append_pair(
                    "ml-analytics-info-list",
                    &match serde_json::to_string(&param_ml_analytics_info_list) {
                        Ok(str) => str,
                        Err(e) => {
                            return Err(ApiError(format!(
                                "Unable to serialize ml_analytics_info_list to string: {}",
                                e
                            )))
                        }
                    },
                );
            }
            if let Some(param_analytics_metadata_prov_ind) = param_analytics_metadata_prov_ind {
                query_string.append_pair(
                    "analytics-metadata-prov-ind",
                    &param_analytics_metadata_prov_ind.to_string(),
                );
            }
            if let Some(param_nsacf_capability) = param_nsacf_capability {
                query_string.append_pair("nsacf-capability", &param_nsacf_capability.to_string());
            }
            if let Some(param_mbs_session_id_list) = param_mbs_session_id_list {
                query_string.append_pair(
                    "mbs-session-id-list",
                    &match serde_json::to_string(&param_mbs_session_id_list) {
                        Ok(str) => str,
                        Err(e) => {
                            return Err(ApiError(format!(
                                "Unable to serialize mbs_session_id_list to string: {}",
                                e
                            )))
                        }
                    },
                );
            }
            if let Some(param_area_session_id) = param_area_session_id {
                query_string.append_pair("area-session-id", &param_area_session_id.to_string());
            }
            if let Some(param_gmlc_number) = param_gmlc_number {
                query_string.append_pair("gmlc-number", &param_gmlc_number);
            }
            if let Some(param_upf_n6_ip) = param_upf_n6_ip {
                query_string.append_pair(
                    "upf-n6-ip",
                    &match serde_json::to_string(&param_upf_n6_ip) {
                        Ok(str) => str,
                        Err(e) => {
                            return Err(ApiError(format!(
                                "Unable to serialize upf_n6_ip to string: {}",
                                e
                            )))
                        }
                    },
                );
            }
            if let Some(param_tai_list) = param_tai_list {
                query_string.append_pair(
                    "tai-list",
                    &match serde_json::to_string(&param_tai_list) {
                        Ok(str) => str,
                        Err(e) => {
                            return Err(ApiError(format!(
                                "Unable to serialize tai_list to string: {}",
                                e
                            )))
                        }
                    },
                );
            }
            if let Some(param_preferences_precedence) = param_preferences_precedence {
                query_string.append_pair(
                    "preferences-precedence",
                    &param_preferences_precedence
                        .iter()
                        .map(ToString::to_string)
                        .collect::<Vec<String>>()
                        .join(","),
                );
            }
            if let Some(param_support_onboarding_capability) = param_support_onboarding_capability {
                query_string.append_pair(
                    "support-onboarding-capability",
                    &param_support_onboarding_capability.to_string(),
                );
            }
            if let Some(param_uas_nf_functionality_ind) = param_uas_nf_functionality_ind {
                query_string.append_pair(
                    "uas-nf-functionality-ind",
                    &param_uas_nf_functionality_ind.to_string(),
                );
            }
            if let Some(param_v2x_capability) = param_v2x_capability {
                query_string.append_pair(
                    "v2x-capability",
                    &match serde_json::to_string(&param_v2x_capability) {
                        Ok(str) => str,
                        Err(e) => {
                            return Err(ApiError(format!(
                                "Unable to serialize v2x_capability to string: {}",
                                e
                            )))
                        }
                    },
                );
            }
            if let Some(param_prose_capability) = param_prose_capability {
                query_string.append_pair(
                    "prose-capability",
                    &match serde_json::to_string(&param_prose_capability) {
                        Ok(str) => str,
                        Err(e) => {
                            return Err(ApiError(format!(
                                "Unable to serialize prose_capability to string: {}",
                                e
                            )))
                        }
                    },
                );
            }
            if let Some(param_shared_data_id) = param_shared_data_id {
                query_string.append_pair("shared-data-id", &param_shared_data_id);
            }
            if let Some(param_target_hni) = param_target_hni {
                query_string.append_pair("target-hni", &param_target_hni);
            }
            if let Some(param_target_nw_resolution) = param_target_nw_resolution {
                query_string.append_pair(
                    "target-nw-resolution",
                    &param_target_nw_resolution.to_string(),
                );
            }
            if let Some(param_exclude_nfinst_list) = param_exclude_nfinst_list {
                query_string.append_pair(
                    "exclude-nfinst-list",
                    &param_exclude_nfinst_list
                        .iter()
                        .map(ToString::to_string)
                        .collect::<Vec<String>>()
                        .join(","),
                );
            }
            if let Some(param_exclude_nfservinst_list) = param_exclude_nfservinst_list {
                query_string.append_pair(
                    "exclude-nfservinst-list",
                    &match serde_json::to_string(&param_exclude_nfservinst_list) {
                        Ok(str) => str,
                        Err(e) => {
                            return Err(ApiError(format!(
                                "Unable to serialize exclude_nfservinst_list to string: {}",
                                e
                            )))
                        }
                    },
                );
            }
            if let Some(param_exclude_nfserviceset_list) = param_exclude_nfserviceset_list {
                query_string.append_pair(
                    "exclude-nfserviceset-list",
                    &param_exclude_nfserviceset_list
                        .iter()
                        .map(ToString::to_string)
                        .collect::<Vec<String>>()
                        .join(","),
                );
            }
            if let Some(param_exclude_nfset_list) = param_exclude_nfset_list {
                query_string.append_pair(
                    "exclude-nfset-list",
                    &param_exclude_nfset_list
                        .iter()
                        .map(ToString::to_string)
                        .collect::<Vec<String>>()
                        .join(","),
                );
            }
            if let Some(param_preferred_analytics_delays) = param_preferred_analytics_delays {
                query_string.append_pair(
                    "preferred-analytics-delays",
                    &match serde_json::to_string(&param_preferred_analytics_delays) {
                        Ok(str) => str,
                        Err(e) => {
                            return Err(ApiError(format!(
                                "Unable to serialize preferred_analytics_delays to string: {}",
                                e
                            )))
                        }
                    },
                );
            }
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
            .method("GET")
            .uri(uri)
            .body(Body::empty())
        {
            Ok(req) => req,
            Err(e) => return Err(ApiError(format!("Unable to create request: {}", e))),
        };

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

        #[allow(clippy::collapsible_match)]
        if let Some(auth_data) = Has::<Option<AuthData>>::get(context).as_ref() {
            // Currently only authentication with Basic and Bearer are supported
            #[allow(clippy::single_match, clippy::match_single_binding)]
            match auth_data {
                &AuthData::Bearer(ref bearer_header) => {
                    let auth = swagger::auth::Header(bearer_header.clone());
                    let header = match HeaderValue::from_str(&format!("{}", auth)) {
                        Ok(h) => h,
                        Err(e) => {
                            return Err(ApiError(format!(
                                "Unable to create Authorization header: {}",
                                e
                            )))
                        }
                    };
                    request
                        .headers_mut()
                        .insert(hyper::header::AUTHORIZATION, header);
                }
                _ => {}
            }
        }

        // Header parameters
        match param_accept_encoding {
            Some(param_accept_encoding) => {
                request.headers_mut().append(
                    HeaderName::from_static("accept-encoding"),
                    #[allow(clippy::redundant_clone)]
                    match header::IntoHeaderValue(param_accept_encoding.clone()).try_into() {
                        Ok(header) => header,
                        Err(e) => {
                            return Err(ApiError(format!(
                                "Invalid header accept_encoding - {}",
                                e
                            )));
                        }
                    },
                );
            }
            None => {}
        }

        match param_if_none_match {
            Some(param_if_none_match) => {
                request.headers_mut().append(
                    HeaderName::from_static("if-none-match"),
                    #[allow(clippy::redundant_clone)]
                    match header::IntoHeaderValue(param_if_none_match.clone()).try_into() {
                        Ok(header) => header,
                        Err(e) => {
                            return Err(ApiError(format!("Invalid header if_none_match - {}", e)));
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
        log::info!("Response is {response:?}");
        match response.status().as_u16() {
            200 => {
                let response_cache_control = match response
                    .headers()
                    .get(HeaderName::from_static("cache-control"))
                {
                    Some(response_cache_control) => {
                        let response_cache_control = response_cache_control.clone();
                        let response_cache_control =
                            match TryInto::<header::IntoHeaderValue<String>>::try_into(
                                response_cache_control,
                            ) {
                                Ok(value) => value,
                                Err(e) => {
                                    return Err(ApiError(format!("Invalid response header Cache-Control for response 200 - {}", e)));
                                }
                            };
                        Some(response_cache_control.0)
                    }
                    None => None,
                };

                let response_e_tag = match response.headers().get(HeaderName::from_static("etag")) {
                    Some(response_e_tag) => {
                        let response_e_tag = response_e_tag.clone();
                        let response_e_tag =
                            match TryInto::<header::IntoHeaderValue<String>>::try_into(
                                response_e_tag,
                            ) {
                                Ok(value) => value,
                                Err(e) => {
                                    return Err(ApiError(format!(
                                        "Invalid response header ETag for response 200 - {}",
                                        e
                                    )));
                                }
                            };
                        Some(response_e_tag.0)
                    }
                    None => None,
                };

                let response_content_encoding = match response
                    .headers()
                    .get(HeaderName::from_static("content-encoding"))
                {
                    Some(response_content_encoding) => {
                        let response_content_encoding = response_content_encoding.clone();
                        let response_content_encoding = match TryInto::<
                            header::IntoHeaderValue<String>,
                        >::try_into(
                            response_content_encoding
                        ) {
                            Ok(value) => value,
                            Err(e) => {
                                return Err(ApiError(format!("Invalid response header Content-Encoding for response 200 - {}", e)));
                            }
                        };
                        Some(response_content_encoding.0)
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
                let body = serde_json::from_str::<models::SearchResult>(body).map_err(|e| {
                    ApiError(format!("Response body did not match the schema: {}", e))
                })?;
                Ok(SearchNfInstancesResponse::ExpectedResponseToAValidRequest {
                    body,
                    cache_control: response_cache_control,
                    e_tag: response_e_tag,
                    content_encoding: response_content_encoding,
                })
            }
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
                Ok(SearchNfInstancesResponse::TemporaryRedirect {
                    body,
                    location: response_location,
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
                Ok(SearchNfInstancesResponse::PermanentRedirect {
                    body,
                    location: response_location,
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
                let body = serde_json::from_str::<models::ProblemDetails>(body).map_err(|e| {
                    ApiError(format!("Response body did not match the schema: {}", e))
                })?;
                Ok(SearchNfInstancesResponse::BadRequest(body))
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
                Ok(SearchNfInstancesResponse::Unauthorized(body))
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
                Ok(SearchNfInstancesResponse::Forbidden(body))
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
                Ok(SearchNfInstancesResponse::NotFound(body))
            }
            406 => Ok(SearchNfInstancesResponse::Status406),
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
                Ok(SearchNfInstancesResponse::LengthRequired(body))
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
                Ok(SearchNfInstancesResponse::PayloadTooLarge(body))
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
                Ok(SearchNfInstancesResponse::UnsupportedMediaType(body))
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
                Ok(SearchNfInstancesResponse::TooManyRequests(body))
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
                Ok(SearchNfInstancesResponse::InternalServerError(body))
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
                Ok(SearchNfInstancesResponse::NotImplemented(body))
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
                Ok(SearchNfInstancesResponse::ServiceUnavailable(body))
            }
            0 => Ok(SearchNfInstancesResponse::GenericError),
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

    async fn scp_domain_routing_info_get(
        &self,
        param_local: Option<bool>,
        param_accept_encoding: Option<String>,
        context: &C,
    ) -> Result<SCpDomainRoutingInfoGetResponse, ApiError> {
        let mut client_service = self.client_service.clone();
        let mut uri = format!("{}/nnrf-disc/v1/scp-domain-routing-info", self.base_path);

        // Query parameters
        let query_string = {
            let mut query_string = form_urlencoded::Serializer::new("".to_owned());
            if let Some(param_local) = param_local {
                query_string.append_pair("local", &param_local.to_string());
            }
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
            .method("GET")
            .uri(uri)
            .body(Body::empty())
        {
            Ok(req) => req,
            Err(e) => return Err(ApiError(format!("Unable to create request: {}", e))),
        };

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

        #[allow(clippy::collapsible_match)]
        if let Some(auth_data) = Has::<Option<AuthData>>::get(context).as_ref() {
            // Currently only authentication with Basic and Bearer are supported
            #[allow(clippy::single_match, clippy::match_single_binding)]
            match auth_data {
                &AuthData::Bearer(ref bearer_header) => {
                    let auth = swagger::auth::Header(bearer_header.clone());
                    let header = match HeaderValue::from_str(&format!("{}", auth)) {
                        Ok(h) => h,
                        Err(e) => {
                            return Err(ApiError(format!(
                                "Unable to create Authorization header: {}",
                                e
                            )))
                        }
                    };
                    request
                        .headers_mut()
                        .insert(hyper::header::AUTHORIZATION, header);
                }
                _ => {}
            }
        }

        // Header parameters
        match param_accept_encoding {
            Some(param_accept_encoding) => {
                request.headers_mut().append(
                    HeaderName::from_static("accept-encoding"),
                    #[allow(clippy::redundant_clone)]
                    match header::IntoHeaderValue(param_accept_encoding.clone()).try_into() {
                        Ok(header) => header,
                        Err(e) => {
                            return Err(ApiError(format!(
                                "Invalid header accept_encoding - {}",
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
            200 => {
                let response_content_encoding = match response
                    .headers()
                    .get(HeaderName::from_static("content-encoding"))
                {
                    Some(response_content_encoding) => {
                        let response_content_encoding = response_content_encoding.clone();
                        let response_content_encoding = match TryInto::<
                            header::IntoHeaderValue<String>,
                        >::try_into(
                            response_content_encoding
                        ) {
                            Ok(value) => value,
                            Err(e) => {
                                return Err(ApiError(format!("Invalid response header Content-Encoding for response 200 - {}", e)));
                            }
                        };
                        Some(response_content_encoding.0)
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
                let body = serde_json::from_str::<models::ScpDomainRoutingInformation>(body)
                    .map_err(|e| {
                        ApiError(format!("Response body did not match the schema: {}", e))
                    })?;
                Ok(
                    SCpDomainRoutingInfoGetResponse::ExpectedResponseToAValidRequest {
                        body,
                        content_encoding: response_content_encoding,
                    },
                )
            }
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

                Ok(SCpDomainRoutingInfoGetResponse::TemporaryRedirect {
                    location: response_location,
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
                let body = serde_json::from_str::<models::ProblemDetails>(body).map_err(|e| {
                    ApiError(format!("Response body did not match the schema: {}", e))
                })?;
                Ok(SCpDomainRoutingInfoGetResponse::BadRequest(body))
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
                Ok(SCpDomainRoutingInfoGetResponse::Unauthorized(body))
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
                Ok(SCpDomainRoutingInfoGetResponse::Forbidden(body))
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
                Ok(SCpDomainRoutingInfoGetResponse::NotFound(body))
            }
            406 => Ok(SCpDomainRoutingInfoGetResponse::Status406),
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
                Ok(SCpDomainRoutingInfoGetResponse::LengthRequired(body))
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
                Ok(SCpDomainRoutingInfoGetResponse::PayloadTooLarge(body))
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
                Ok(SCpDomainRoutingInfoGetResponse::UnsupportedMediaType(body))
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
                Ok(SCpDomainRoutingInfoGetResponse::TooManyRequests(body))
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
                Ok(SCpDomainRoutingInfoGetResponse::InternalServerError(body))
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
                Ok(SCpDomainRoutingInfoGetResponse::NotImplemented(body))
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
                Ok(SCpDomainRoutingInfoGetResponse::ServiceUnavailable(body))
            }
            0 => Ok(SCpDomainRoutingInfoGetResponse::GenericError),
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

    async fn scp_domain_routing_info_subscribe(
        &self,
        param_scp_domain_routing_info_subscription: models::ScpDomainRoutingInfoSubscription,
        param_content_encoding: Option<String>,
        param_accept_encoding: Option<String>,
        context: &C,
    ) -> Result<ScpDomainRoutingInfoSubscribeResponse, ApiError> {
        let mut client_service = self.client_service.clone();
        let mut uri = format!(
            "{}/nnrf-disc/v1/scp-domain-routing-info-subs",
            self.base_path
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
        let body = serde_json::to_string(&param_scp_domain_routing_info_subscription)
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

        #[allow(clippy::collapsible_match)]
        if let Some(auth_data) = Has::<Option<AuthData>>::get(context).as_ref() {
            // Currently only authentication with Basic and Bearer are supported
            #[allow(clippy::single_match, clippy::match_single_binding)]
            match auth_data {
                &AuthData::Bearer(ref bearer_header) => {
                    let auth = swagger::auth::Header(bearer_header.clone());
                    let header = match HeaderValue::from_str(&format!("{}", auth)) {
                        Ok(h) => h,
                        Err(e) => {
                            return Err(ApiError(format!(
                                "Unable to create Authorization header: {}",
                                e
                            )))
                        }
                    };
                    request
                        .headers_mut()
                        .insert(hyper::header::AUTHORIZATION, header);
                }
                _ => {}
            }
        }

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

        match param_accept_encoding {
            Some(param_accept_encoding) => {
                request.headers_mut().append(
                    HeaderName::from_static("accept-encoding"),
                    #[allow(clippy::redundant_clone)]
                    match header::IntoHeaderValue(param_accept_encoding.clone()).try_into() {
                        Ok(header) => header,
                        Err(e) => {
                            return Err(ApiError(format!(
                                "Invalid header accept_encoding - {}",
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
            201 => {
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
                                        "Invalid response header Location for response 201 - {}",
                                        e
                                    )));
                                    }
                                };
                            response_location.0
                        }
                        None => {
                            return Err(ApiError(String::from(
                                "Required response header Location for response 201 was not found.",
                            )))
                        }
                    };

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
                                    "Invalid response header Accept-Encoding for response 201 - {}",
                                    e
                                )));
                                }
                            };
                        Some(response_accept_encoding.0)
                    }
                    None => None,
                };

                let response_content_encoding = match response
                    .headers()
                    .get(HeaderName::from_static("content-encoding"))
                {
                    Some(response_content_encoding) => {
                        let response_content_encoding = response_content_encoding.clone();
                        let response_content_encoding = match TryInto::<
                            header::IntoHeaderValue<String>,
                        >::try_into(
                            response_content_encoding
                        ) {
                            Ok(value) => value,
                            Err(e) => {
                                return Err(ApiError(format!("Invalid response header Content-Encoding for response 201 - {}", e)));
                            }
                        };
                        Some(response_content_encoding.0)
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
                let body = serde_json::from_str::<models::ScpDomainRoutingInfoSubscription>(body)
                    .map_err(|e| {
                    ApiError(format!("Response body did not match the schema: {}", e))
                })?;
                Ok(
                    ScpDomainRoutingInfoSubscribeResponse::ExpectedResponseToAValidRequest {
                        body,
                        location: response_location,
                        accept_encoding: response_accept_encoding,
                        content_encoding: response_content_encoding,
                    },
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
                Ok(ScpDomainRoutingInfoSubscribeResponse::BadRequest(body))
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
                Ok(ScpDomainRoutingInfoSubscribeResponse::Unauthorized(body))
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
                Ok(ScpDomainRoutingInfoSubscribeResponse::Forbidden(body))
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
                Ok(ScpDomainRoutingInfoSubscribeResponse::NotFound(body))
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
                Ok(ScpDomainRoutingInfoSubscribeResponse::LengthRequired(body))
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
                Ok(ScpDomainRoutingInfoSubscribeResponse::PayloadTooLarge(body))
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
                Ok(ScpDomainRoutingInfoSubscribeResponse::UnsupportedMediaType(
                    body,
                ))
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
                Ok(ScpDomainRoutingInfoSubscribeResponse::TooManyRequests(body))
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
                Ok(ScpDomainRoutingInfoSubscribeResponse::InternalServerError(
                    body,
                ))
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
                Ok(ScpDomainRoutingInfoSubscribeResponse::NotImplemented(body))
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
                Ok(ScpDomainRoutingInfoSubscribeResponse::ServiceUnavailable(
                    body,
                ))
            }
            0 => Ok(ScpDomainRoutingInfoSubscribeResponse::GenericError),
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

    async fn retrieve_stored_search(
        &self,
        param_search_id: String,
        param_accept_encoding: Option<String>,
        context: &C,
    ) -> Result<RetrieveStoredSearchResponse, ApiError> {
        let mut client_service = self.client_service.clone();
        let mut uri = format!(
            "{}/nnrf-disc/v1/searches/{search_id}",
            self.base_path,
            search_id = utf8_percent_encode(&param_search_id.to_string(), ID_ENCODE_SET)
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
            .method("GET")
            .uri(uri)
            .body(Body::empty())
        {
            Ok(req) => req,
            Err(e) => return Err(ApiError(format!("Unable to create request: {}", e))),
        };

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

        #[allow(clippy::collapsible_match)]
        if let Some(auth_data) = Has::<Option<AuthData>>::get(context).as_ref() {
            // Currently only authentication with Basic and Bearer are supported
            #[allow(clippy::single_match, clippy::match_single_binding)]
            match auth_data {
                &AuthData::Bearer(ref bearer_header) => {
                    let auth = swagger::auth::Header(bearer_header.clone());
                    let header = match HeaderValue::from_str(&format!("{}", auth)) {
                        Ok(h) => h,
                        Err(e) => {
                            return Err(ApiError(format!(
                                "Unable to create Authorization header: {}",
                                e
                            )))
                        }
                    };
                    request
                        .headers_mut()
                        .insert(hyper::header::AUTHORIZATION, header);
                }
                _ => {}
            }
        }

        // Header parameters
        match param_accept_encoding {
            Some(param_accept_encoding) => {
                request.headers_mut().append(
                    HeaderName::from_static("accept-encoding"),
                    #[allow(clippy::redundant_clone)]
                    match header::IntoHeaderValue(param_accept_encoding.clone()).try_into() {
                        Ok(header) => header,
                        Err(e) => {
                            return Err(ApiError(format!(
                                "Invalid header accept_encoding - {}",
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
            200 => {
                let response_cache_control = match response
                    .headers()
                    .get(HeaderName::from_static("cache-control"))
                {
                    Some(response_cache_control) => {
                        let response_cache_control = response_cache_control.clone();
                        let response_cache_control =
                            match TryInto::<header::IntoHeaderValue<String>>::try_into(
                                response_cache_control,
                            ) {
                                Ok(value) => value,
                                Err(e) => {
                                    return Err(ApiError(format!("Invalid response header Cache-Control for response 200 - {}", e)));
                                }
                            };
                        Some(response_cache_control.0)
                    }
                    None => None,
                };

                let response_e_tag = match response.headers().get(HeaderName::from_static("etag")) {
                    Some(response_e_tag) => {
                        let response_e_tag = response_e_tag.clone();
                        let response_e_tag =
                            match TryInto::<header::IntoHeaderValue<String>>::try_into(
                                response_e_tag,
                            ) {
                                Ok(value) => value,
                                Err(e) => {
                                    return Err(ApiError(format!(
                                        "Invalid response header ETag for response 200 - {}",
                                        e
                                    )));
                                }
                            };
                        Some(response_e_tag.0)
                    }
                    None => None,
                };

                let response_content_encoding = match response
                    .headers()
                    .get(HeaderName::from_static("content-encoding"))
                {
                    Some(response_content_encoding) => {
                        let response_content_encoding = response_content_encoding.clone();
                        let response_content_encoding = match TryInto::<
                            header::IntoHeaderValue<String>,
                        >::try_into(
                            response_content_encoding
                        ) {
                            Ok(value) => value,
                            Err(e) => {
                                return Err(ApiError(format!("Invalid response header Content-Encoding for response 200 - {}", e)));
                            }
                        };
                        Some(response_content_encoding.0)
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
                let body =
                    serde_json::from_str::<models::StoredSearchResult>(body).map_err(|e| {
                        ApiError(format!("Response body did not match the schema: {}", e))
                    })?;
                Ok(
                    RetrieveStoredSearchResponse::ExpectedResponseToAValidRequest {
                        body,
                        cache_control: response_cache_control,
                        e_tag: response_e_tag,
                        content_encoding: response_content_encoding,
                    },
                )
            }
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
                Ok(RetrieveStoredSearchResponse::TemporaryRedirect {
                    body,
                    location: response_location,
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
                Ok(RetrieveStoredSearchResponse::PermanentRedirect {
                    body,
                    location: response_location,
                })
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
