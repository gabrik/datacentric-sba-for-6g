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

use crate::{
    Api, PostPduSessionsResponse, PostSmContextsResponse, ReleasePduSessionResponse,
    ReleaseSmContextResponse, RetrievePduSessionResponse, RetrieveSmContextResponse,
    SendMoDataResponse, TransferMoDataResponse, UpdatePduSessionResponse, UpdateSmContextResponse,
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

    async fn release_pdu_session(
        &self,
        param_pdu_session_ref: String,
        param_release_data: Option<models::ReleaseData>,
        context: &C,
    ) -> Result<ReleasePduSessionResponse, ApiError> {
        let mut client_service = self.client_service.clone();
        let mut uri = format!(
            "{}/nsmf-pdusession/v1/pdu-sessions/{pdu_session_ref}/release",
            self.base_path,
            pdu_session_ref =
                utf8_percent_encode(&param_pdu_session_ref.to_string(), ID_ENCODE_SET)
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
        let body = param_release_data
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
            200 => {
                let body = response.into_body();
                let body = body
                    .into_raw()
                    .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                    .await?;
                let body = str::from_utf8(&body)
                    .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body = serde_json::from_str::<models::ReleasedData>(body).map_err(|e| {
                    ApiError(format!("Response body did not match the schema: {}", e))
                })?;
                Ok(ReleasePduSessionResponse::SuccessfulReleaseOfAPDUSessionWithContentInTheResponse
                    (body)
                )
            }
            204 => Ok(ReleasePduSessionResponse::SuccessfulReleaseOfAPDUSession),
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
                Ok(ReleasePduSessionResponse::TemporaryRedirect {
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
                Ok(ReleasePduSessionResponse::PermanentRedirect {
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
                Ok(ReleasePduSessionResponse::BadRequest(body))
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
                Ok(ReleasePduSessionResponse::Forbidden(body))
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
                Ok(ReleasePduSessionResponse::NotFound(body))
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
                Ok(ReleasePduSessionResponse::LengthRequired(body))
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
                Ok(ReleasePduSessionResponse::PayloadTooLarge(body))
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
                Ok(ReleasePduSessionResponse::UnsupportedMediaType(body))
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
                Ok(ReleasePduSessionResponse::TooManyRequests(body))
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
                Ok(ReleasePduSessionResponse::InternalServerError(body))
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
                Ok(ReleasePduSessionResponse::ServiceUnavailable(body))
            }
            0 => Ok(ReleasePduSessionResponse::GenericError),
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

    async fn retrieve_pdu_session(
        &self,
        param_pdu_session_ref: String,
        param_retrieve_data: models::RetrieveData,
        context: &C,
    ) -> Result<RetrievePduSessionResponse, ApiError> {
        let mut client_service = self.client_service.clone();
        let mut uri = format!(
            "{}/nsmf-pdusession/v1/pdu-sessions/{pdu_session_ref}/retrieve",
            self.base_path,
            pdu_session_ref =
                utf8_percent_encode(&param_pdu_session_ref.to_string(), ID_ENCODE_SET)
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

        let body =
            serde_json::to_string(&param_retrieve_data).expect("impossible to fail to serialize");
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
                let body = serde_json::from_str::<models::RetrievedData>(body).map_err(|e| {
                    ApiError(format!("Response body did not match the schema: {}", e))
                })?;
                Ok(RetrievePduSessionResponse::SuccessfulInformationRetrieval(
                    body,
                ))
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
                Ok(RetrievePduSessionResponse::TemporaryRedirect {
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
                Ok(RetrievePduSessionResponse::PermanentRedirect {
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
                Ok(RetrievePduSessionResponse::BadRequest(body))
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
                Ok(RetrievePduSessionResponse::Forbidden(body))
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
                Ok(RetrievePduSessionResponse::NotFound(body))
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
                Ok(RetrievePduSessionResponse::LengthRequired(body))
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
                Ok(RetrievePduSessionResponse::PayloadTooLarge(body))
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
                Ok(RetrievePduSessionResponse::UnsupportedMediaType(body))
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
                Ok(RetrievePduSessionResponse::TooManyRequests(body))
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
                Ok(RetrievePduSessionResponse::InternalServerError(body))
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
                Ok(RetrievePduSessionResponse::ServiceUnavailable(body))
            }
            504 => {
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
                Ok(RetrievePduSessionResponse::GatewayTimeout(body))
            }
            0 => Ok(RetrievePduSessionResponse::GenericError),
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

    async fn transfer_mo_data(
        &self,
        param_pdu_session_ref: String,
        param_json_data: Option<models::TransferMoDataReqData>,
        param_binary_mo_data: Option<swagger::ByteArray>,
        context: &C,
    ) -> Result<TransferMoDataResponse, ApiError> {
        let mut client_service = self.client_service.clone();
        let mut uri = format!(
            "{}/nsmf-pdusession/v1/pdu-sessions/{pdu_session_ref}/transfer-mo-data",
            self.base_path,
            pdu_session_ref =
                utf8_percent_encode(&param_pdu_session_ref.to_string(), ID_ENCODE_SET)
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

        if let Some(binary_mo_data) = param_binary_mo_data {
            let part = Node::Part(Part {
                headers: {
                    let mut h = Headers::new();
                    h.set(ContentType("application/vnd.3gpp.5gnas".parse().unwrap()));
                    h.set_raw("Content-ID", vec![b"binaryMoData".to_vec()]);
                    h
                },
                body: binary_mo_data.0,
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
            204 => Ok(TransferMoDataResponse::SuccessfulTransferingOfMOData),
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
                Ok(TransferMoDataResponse::TemporaryRedirect {
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
                Ok(TransferMoDataResponse::PermanentRedirect {
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
                Ok(TransferMoDataResponse::BadRequest(body))
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
                Ok(TransferMoDataResponse::Unauthorized(body))
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
                Ok(TransferMoDataResponse::Forbidden(body))
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
                Ok(TransferMoDataResponse::NotFound(body))
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
                Ok(TransferMoDataResponse::LengthRequired(body))
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
                Ok(TransferMoDataResponse::PayloadTooLarge(body))
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
                Ok(TransferMoDataResponse::UnsupportedMediaType(body))
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
                Ok(TransferMoDataResponse::TooManyRequests(body))
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
                Ok(TransferMoDataResponse::InternalServerError(body))
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
                Ok(TransferMoDataResponse::ServiceUnavailable(body))
            }
            0 => Ok(TransferMoDataResponse::GenericError),
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

    async fn update_pdu_session(
        &self,
        param_pdu_session_ref: String,
        param_hsmf_update_data: models::HsmfUpdateData,
        context: &C,
    ) -> Result<UpdatePduSessionResponse, ApiError> {
        let mut client_service = self.client_service.clone();
        let mut uri = format!(
            "{}/nsmf-pdusession/v1/pdu-sessions/{pdu_session_ref}/modify",
            self.base_path,
            pdu_session_ref =
                utf8_percent_encode(&param_pdu_session_ref.to_string(), ID_ENCODE_SET)
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

        let body = serde_json::to_string(&param_hsmf_update_data)
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
                let body = serde_json::from_str::<models::HsmfUpdatedData>(body).map_err(|e| {
                    ApiError(format!("Response body did not match the schema: {}", e))
                })?;
                Ok(
                    UpdatePduSessionResponse::SuccessfulUpdateOfAPDUSessionWithContentInTheResponse(
                        body,
                    ),
                )
            }
            204 => Ok(
                UpdatePduSessionResponse::SuccessfulUpdateOfAPDUSessionWithoutContentInTheResponse,
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
                Ok(UpdatePduSessionResponse::TemporaryRedirect {
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
                Ok(UpdatePduSessionResponse::PermanentRedirect {
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
                let body = serde_json::from_str::<models::HsmfUpdateError>(body).map_err(|e| {
                    ApiError(format!("Response body did not match the schema: {}", e))
                })?;
                Ok(UpdatePduSessionResponse::UnsuccessfulUpdateOfAPDUSession(
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
                let body = serde_json::from_str::<models::HsmfUpdateError>(body).map_err(|e| {
                    ApiError(format!("Response body did not match the schema: {}", e))
                })?;
                Ok(UpdatePduSessionResponse::UnsuccessfulUpdateOfAPDUSession_2(
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
                let body = serde_json::from_str::<models::HsmfUpdateError>(body).map_err(|e| {
                    ApiError(format!("Response body did not match the schema: {}", e))
                })?;
                Ok(UpdatePduSessionResponse::UnsuccessfulUpdateOfAPDUSession_3(
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
                Ok(UpdatePduSessionResponse::LengthRequired(body))
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
                Ok(UpdatePduSessionResponse::PayloadTooLarge(body))
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
                Ok(UpdatePduSessionResponse::UnsupportedMediaType(body))
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
                Ok(UpdatePduSessionResponse::TooManyRequests(body))
            }
            500 => {
                let body = response.into_body();
                let body = body
                    .into_raw()
                    .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                    .await?;
                let body = str::from_utf8(&body)
                    .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body = serde_json::from_str::<models::HsmfUpdateError>(body).map_err(|e| {
                    ApiError(format!("Response body did not match the schema: {}", e))
                })?;
                Ok(UpdatePduSessionResponse::UnsuccessfulUpdateOfAPDUSession_4(
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
                let body = serde_json::from_str::<models::HsmfUpdateError>(body).map_err(|e| {
                    ApiError(format!("Response body did not match the schema: {}", e))
                })?;
                Ok(UpdatePduSessionResponse::UnsuccessfulUpdateOfAPDUSession_5(
                    body,
                ))
            }
            0 => Ok(UpdatePduSessionResponse::GenericError),
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

    async fn release_sm_context(
        &self,
        param_sm_context_ref: String,
        param_sm_context_release_data: Option<models::SmContextReleaseData>,
        context: &C,
    ) -> Result<ReleaseSmContextResponse, ApiError> {
        let mut client_service = self.client_service.clone();
        let mut uri = format!(
            "{}/nsmf-pdusession/v1/sm-contexts/{sm_context_ref}/release",
            self.base_path,
            sm_context_ref = utf8_percent_encode(&param_sm_context_ref.to_string(), ID_ENCODE_SET)
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
        let body = param_sm_context_release_data
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
            200 => {
                let body = response.into_body();
                let body = body
                    .into_raw()
                    .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                    .await?;
                let body = str::from_utf8(&body)
                    .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body =
                    serde_json::from_str::<models::SmContextReleasedData>(body).map_err(|e| {
                        ApiError(format!("Response body did not match the schema: {}", e))
                    })?;
                Ok(ReleaseSmContextResponse::SuccessfulReleaseOfAPDUSessionWithContentInTheResponse
                    (body)
                )
            }
            204 => Ok(
                ReleaseSmContextResponse::SuccessfulReleaseOfAnSMContextWithoutContentInTheResponse,
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
                Ok(ReleaseSmContextResponse::TemporaryRedirect {
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
                Ok(ReleaseSmContextResponse::PermanentRedirect {
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
                Ok(ReleaseSmContextResponse::BadRequest(body))
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
                Ok(ReleaseSmContextResponse::Forbidden(body))
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
                Ok(ReleaseSmContextResponse::NotFound(body))
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
                Ok(ReleaseSmContextResponse::LengthRequired(body))
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
                Ok(ReleaseSmContextResponse::PayloadTooLarge(body))
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
                Ok(ReleaseSmContextResponse::UnsupportedMediaType(body))
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
                Ok(ReleaseSmContextResponse::TooManyRequests(body))
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
                Ok(ReleaseSmContextResponse::InternalServerError(body))
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
                Ok(ReleaseSmContextResponse::ServiceUnavailable(body))
            }
            0 => Ok(ReleaseSmContextResponse::GenericError),
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

    async fn retrieve_sm_context(
        &self,
        param_sm_context_ref: String,
        param_sm_context_retrieve_data: Option<models::SmContextRetrieveData>,
        context: &C,
    ) -> Result<RetrieveSmContextResponse, ApiError> {
        let mut client_service = self.client_service.clone();
        let mut uri = format!(
            "{}/nsmf-pdusession/v1/sm-contexts/{sm_context_ref}/retrieve",
            self.base_path,
            sm_context_ref = utf8_percent_encode(&param_sm_context_ref.to_string(), ID_ENCODE_SET)
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

        let body = param_sm_context_retrieve_data
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
            200 => {
                let body = response.into_body();
                let body = body
                    .into_raw()
                    .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                    .await?;
                let body = str::from_utf8(&body)
                    .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body =
                    serde_json::from_str::<models::SmContextRetrievedData>(body).map_err(|e| {
                        ApiError(format!("Response body did not match the schema: {}", e))
                    })?;
                Ok(RetrieveSmContextResponse::SuccessfulRetrievalOfAnSMContext(
                    body,
                ))
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
                Ok(RetrieveSmContextResponse::TemporaryRedirect {
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
                Ok(RetrieveSmContextResponse::PermanentRedirect {
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
                Ok(RetrieveSmContextResponse::BadRequest(body))
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
                Ok(RetrieveSmContextResponse::Forbidden(body))
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
                Ok(RetrieveSmContextResponse::NotFound(body))
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
                Ok(RetrieveSmContextResponse::LengthRequired(body))
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
                Ok(RetrieveSmContextResponse::PayloadTooLarge(body))
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
                Ok(RetrieveSmContextResponse::UnsupportedMediaType(body))
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
                Ok(RetrieveSmContextResponse::TooManyRequests(body))
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
                Ok(RetrieveSmContextResponse::InternalServerError(body))
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
                Ok(RetrieveSmContextResponse::ServiceUnavailable(body))
            }
            504 => {
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
                Ok(RetrieveSmContextResponse::GatewayTimeout(body))
            }
            0 => Ok(RetrieveSmContextResponse::GenericError),
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

    async fn send_mo_data(
        &self,
        param_sm_context_ref: String,
        param_json_data: Option<models::SendMoDataReqData>,
        param_binary_mo_data: Option<swagger::ByteArray>,
        context: &C,
    ) -> Result<SendMoDataResponse, ApiError> {
        let mut client_service = self.client_service.clone();
        let mut uri = format!(
            "{}/nsmf-pdusession/v1/sm-contexts/{sm_context_ref}/send-mo-data",
            self.base_path,
            sm_context_ref = utf8_percent_encode(&param_sm_context_ref.to_string(), ID_ENCODE_SET)
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

        if let Some(binary_mo_data) = param_binary_mo_data {
            let part = Node::Part(Part {
                headers: {
                    let mut h = Headers::new();
                    h.set(ContentType("application/vnd.3gpp.5gnas".parse().unwrap()));
                    h.set_raw("Content-ID", vec![b"binaryMoData".to_vec()]);
                    h
                },
                body: binary_mo_data.0,
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
            204 => Ok(SendMoDataResponse::SuccessfulSendingOfMOData),
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
                Ok(SendMoDataResponse::TemporaryRedirect {
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
                Ok(SendMoDataResponse::PermanentRedirect {
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
                Ok(SendMoDataResponse::BadRequest(body))
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
                Ok(SendMoDataResponse::Unauthorized(body))
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
                Ok(SendMoDataResponse::Forbidden(body))
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
                Ok(SendMoDataResponse::NotFound(body))
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
                Ok(SendMoDataResponse::LengthRequired(body))
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
                Ok(SendMoDataResponse::PayloadTooLarge(body))
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
                Ok(SendMoDataResponse::UnsupportedMediaType(body))
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
                Ok(SendMoDataResponse::TooManyRequests(body))
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
                Ok(SendMoDataResponse::InternalServerError(body))
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
                Ok(SendMoDataResponse::ServiceUnavailable(body))
            }
            0 => Ok(SendMoDataResponse::GenericError),
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

    async fn update_sm_context(
        &self,
        param_sm_context_ref: String,
        param_sm_context_update_data: models::SmContextUpdateData,
        context: &C,
    ) -> Result<UpdateSmContextResponse, ApiError> {
        let mut client_service = self.client_service.clone();
        let mut uri = format!(
            "{}/nsmf-pdusession/v1/sm-contexts/{sm_context_ref}/modify",
            self.base_path,
            sm_context_ref = utf8_percent_encode(&param_sm_context_ref.to_string(), ID_ENCODE_SET)
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

        let body = serde_json::to_string(&param_sm_context_update_data)
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
                let body =
                    serde_json::from_str::<models::SmContextUpdatedData>(body).map_err(|e| {
                        ApiError(format!("Response body did not match the schema: {}", e))
                    })?;
                Ok(
                    UpdateSmContextResponse::SuccessfulUpdateOfAnSMContextWithContentInTheResponse(
                        body,
                    ),
                )
            }
            204 => Ok(
                UpdateSmContextResponse::SuccessfulUpdateOfAnSMContextWithoutContentInTheResponse,
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
                Ok(UpdateSmContextResponse::TemporaryRedirect {
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
                Ok(UpdateSmContextResponse::PermanentRedirect {
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
                    serde_json::from_str::<models::SmContextUpdateError>(body).map_err(|e| {
                        ApiError(format!("Response body did not match the schema: {}", e))
                    })?;
                Ok(UpdateSmContextResponse::UnsuccessfulUpdateOfAnSMContext(
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
                let body =
                    serde_json::from_str::<models::SmContextUpdateError>(body).map_err(|e| {
                        ApiError(format!("Response body did not match the schema: {}", e))
                    })?;
                Ok(UpdateSmContextResponse::UnsuccessfulUpdateOfAnSMContext_2(
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
                let body =
                    serde_json::from_str::<models::SmContextUpdateError>(body).map_err(|e| {
                        ApiError(format!("Response body did not match the schema: {}", e))
                    })?;
                Ok(UpdateSmContextResponse::UnsuccessfulUpdateOfAnSMContext_3(
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
                Ok(UpdateSmContextResponse::LengthRequired(body))
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
                Ok(UpdateSmContextResponse::PayloadTooLarge(body))
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
                Ok(UpdateSmContextResponse::UnsupportedMediaType(body))
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
                Ok(UpdateSmContextResponse::TooManyRequests(body))
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
                    serde_json::from_str::<models::SmContextUpdateError>(body).map_err(|e| {
                        ApiError(format!("Response body did not match the schema: {}", e))
                    })?;
                Ok(UpdateSmContextResponse::UnsuccessfulUpdateOfAnSMContext_4(
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
                let body =
                    serde_json::from_str::<models::SmContextUpdateError>(body).map_err(|e| {
                        ApiError(format!("Response body did not match the schema: {}", e))
                    })?;
                Ok(UpdateSmContextResponse::UnsuccessfulUpdateOfAnSMContext_5(
                    body,
                ))
            }
            0 => Ok(UpdateSmContextResponse::GenericError),
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

    async fn post_pdu_sessions(
        &self,
        param_pdu_session_create_data: models::PduSessionCreateData,
        context: &C,
    ) -> Result<PostPduSessionsResponse, ApiError> {
        let mut client_service = self.client_service.clone();
        let mut uri = format!("{}/nsmf-pdusession/v1/pdu-sessions", self.base_path);

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
        let body = serde_json::to_string(&param_pdu_session_create_data)
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

                let body = response.into_body();
                let body = body
                    .into_raw()
                    .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                    .await?;
                let body = str::from_utf8(&body)
                    .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body =
                    serde_json::from_str::<models::PduSessionCreatedData>(body).map_err(|e| {
                        ApiError(format!("Response body did not match the schema: {}", e))
                    })?;
                Ok(PostPduSessionsResponse::SuccessfulCreationOfAPDUSession {
                    body,
                    location: response_location,
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
                Ok(PostPduSessionsResponse::TemporaryRedirect {
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
                Ok(PostPduSessionsResponse::PermanentRedirect {
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
                    serde_json::from_str::<models::PduSessionCreateError>(body).map_err(|e| {
                        ApiError(format!("Response body did not match the schema: {}", e))
                    })?;
                Ok(PostPduSessionsResponse::UnsuccessfulCreationOfAPDUSession(
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
                let body =
                    serde_json::from_str::<models::PduSessionCreateError>(body).map_err(|e| {
                        ApiError(format!("Response body did not match the schema: {}", e))
                    })?;
                Ok(PostPduSessionsResponse::UnsuccessfulCreationOfAPDUSession_2(body))
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
                    serde_json::from_str::<models::PduSessionCreateError>(body).map_err(|e| {
                        ApiError(format!("Response body did not match the schema: {}", e))
                    })?;
                Ok(PostPduSessionsResponse::UnsuccessfulCreationOfAPDUSession_3(body))
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
                Ok(PostPduSessionsResponse::LengthRequired(body))
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
                Ok(PostPduSessionsResponse::PayloadTooLarge(body))
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
                Ok(PostPduSessionsResponse::UnsupportedMediaType(body))
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
                Ok(PostPduSessionsResponse::TooManyRequests(body))
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
                    serde_json::from_str::<models::PduSessionCreateError>(body).map_err(|e| {
                        ApiError(format!("Response body did not match the schema: {}", e))
                    })?;
                Ok(PostPduSessionsResponse::UnsuccessfulCreationOfAPDUSession_4(body))
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
                    serde_json::from_str::<models::PduSessionCreateError>(body).map_err(|e| {
                        ApiError(format!("Response body did not match the schema: {}", e))
                    })?;
                Ok(PostPduSessionsResponse::UnsuccessfulCreationOfAPDUSession_5(body))
            }
            0 => Ok(PostPduSessionsResponse::GenericError),
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

    async fn post_sm_contexts(
        &self,
        param_json_data: Option<models::SmContextCreateData>,
        param_binary_data_n1_sm_message: Option<swagger::ByteArray>,
        param_binary_data_n2_sm_information: Option<swagger::ByteArray>,
        param_binary_data_n2_sm_information_ext1: Option<swagger::ByteArray>,
        context: &C,
    ) -> Result<PostSmContextsResponse, ApiError> {
        let mut client_service = self.client_service.clone();
        let mut uri = format!("{}/nsmf-pdusession/v1/sm-contexts", self.base_path);

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

        if let Some(binary_data_n1_sm_message) = param_binary_data_n1_sm_message {
            let part = Node::Part(Part {
                headers: {
                    let mut h = Headers::new();
                    h.set(ContentType("application/vnd.3gpp.5gnas".parse().unwrap()));
                    h.set_raw("Content-ID", vec![b"binaryDataN1SmMessage".to_vec()]);
                    h
                },
                body: binary_data_n1_sm_message.0,
            });
            body_parts.push(part);
        }

        if let Some(binary_data_n2_sm_information) = param_binary_data_n2_sm_information {
            let part = Node::Part(Part {
                headers: {
                    let mut h = Headers::new();
                    h.set(ContentType("application/vnd.3gpp.ngap".parse().unwrap()));
                    h.set_raw("Content-ID", vec![b"binaryDataN2SmInformation".to_vec()]);
                    h
                },
                body: binary_data_n2_sm_information.0,
            });
            body_parts.push(part);
        }

        if let Some(binary_data_n2_sm_information_ext1) = param_binary_data_n2_sm_information_ext1 {
            let part = Node::Part(Part {
                headers: {
                    let mut h = Headers::new();
                    h.set(ContentType("application/vnd.3gpp.ngap".parse().unwrap()));
                    h.set_raw(
                        "Content-ID",
                        vec![b"binaryDataN2SmInformationExt1".to_vec()],
                    );
                    h
                },
                body: binary_data_n2_sm_information_ext1.0,
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

                let body = response.into_body();
                let body = body
                    .into_raw()
                    .map_err(|e| ApiError(format!("Failed to read response: {}", e)))
                    .await?;
                let body = str::from_utf8(&body)
                    .map_err(|e| ApiError(format!("Response was not valid UTF8: {}", e)))?;
                let body =
                    serde_json::from_str::<models::SmContextCreatedData>(body).map_err(|e| {
                        ApiError(format!("Response body did not match the schema: {}", e))
                    })?;
                Ok(PostSmContextsResponse::SuccessfulCreationOfAnSMContext {
                    body,
                    location: response_location,
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
                Ok(PostSmContextsResponse::TemporaryRedirect {
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
                Ok(PostSmContextsResponse::PermanentRedirect {
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
                    serde_json::from_str::<models::SmContextCreateError>(body).map_err(|e| {
                        ApiError(format!("Response body did not match the schema: {}", e))
                    })?;
                Ok(PostSmContextsResponse::UnsuccessfulCreationOfAnSMContext(
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
                let body =
                    serde_json::from_str::<models::SmContextCreateError>(body).map_err(|e| {
                        ApiError(format!("Response body did not match the schema: {}", e))
                    })?;
                Ok(PostSmContextsResponse::UnsuccessfulCreationOfAnSMContext_2(
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
                let body =
                    serde_json::from_str::<models::SmContextCreateError>(body).map_err(|e| {
                        ApiError(format!("Response body did not match the schema: {}", e))
                    })?;
                Ok(PostSmContextsResponse::UnsuccessfulCreationOfAnSMContext_3(
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
                Ok(PostSmContextsResponse::LengthRequired(body))
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
                Ok(PostSmContextsResponse::PayloadTooLarge(body))
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
                Ok(PostSmContextsResponse::UnsupportedMediaType(body))
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
                Ok(PostSmContextsResponse::TooManyRequests(body))
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
                    serde_json::from_str::<models::SmContextCreateError>(body).map_err(|e| {
                        ApiError(format!("Response body did not match the schema: {}", e))
                    })?;
                Ok(PostSmContextsResponse::UnsuccessfulCreationOfAnSMContext_4(
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
                let body =
                    serde_json::from_str::<models::SmContextCreateError>(body).map_err(|e| {
                        ApiError(format!("Response body did not match the schema: {}", e))
                    })?;
                Ok(PostSmContextsResponse::UnsuccessfulCreationOfAnSMContext_5(
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
                let body =
                    serde_json::from_str::<models::SmContextCreateError>(body).map_err(|e| {
                        ApiError(format!("Response body did not match the schema: {}", e))
                    })?;
                Ok(PostSmContextsResponse::UnsuccessfulCreationOfAnSMContext_6(
                    body,
                ))
            }
            0 => Ok(PostSmContextsResponse::GenericError),
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
