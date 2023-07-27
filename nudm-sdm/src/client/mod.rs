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
    Api, CAgAckResponse, GetAmDataResponse, GetDataSetsResponse, GetEcrDataResponse,
    GetGroupIdentifiersResponse, GetIndividualSharedDataResponse, GetLcsBcaDataResponse,
    GetLcsMoDataResponse, GetLcsPrivacyDataResponse, GetMbsDataResponse,
    GetMultipleIdentifiersResponse, GetNssaiResponse, GetProseDataResponse, GetSharedDataResponse,
    GetSmDataResponse, GetSmfSelDataResponse, GetSmsDataResponse, GetSmsMngtDataResponse,
    GetSupiOrGpsiResponse, GetTraceConfigDataResponse, GetUcDataResponse,
    GetUeCtxInAmfDataResponse, GetUeCtxInSmfDataResponse, GetUeCtxInSmsfDataResponse,
    GetV2xDataResponse, ModifyResponse, ModifySharedDataSubsResponse, SNssaisAckResponse,
    SorAckInfoResponse, SubscribeResponse, SubscribeToSharedDataResponse,
    UnsubscribeForSharedDataResponse, UnsubscribeResponse, UpdateSorInfoResponse, UpuAckResponse,
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

    async fn get_am_data(
        &self,
        param_supi: String,
        param_supported_features: Option<String>,
        param_plmn_id: Option<models::PlmnIdNid>,
        param_adjacent_plmns: Option<&Vec<models::PlmnId>>,
        param_disaster_roaming_ind: Option<bool>,
        param_if_none_match: Option<String>,
        param_if_modified_since: Option<String>,
        context: &C,
    ) -> Result<GetAmDataResponse, ApiError> {
        let mut client_service = self.client_service.clone();
        let mut uri = format!(
            "{}/nudm-sdm/v2/{supi}/am-data",
            self.base_path,
            supi = utf8_percent_encode(&param_supi.to_string(), ID_ENCODE_SET)
        );

        // Query parameters
        let query_string = {
            let mut query_string = form_urlencoded::Serializer::new("".to_owned());
            if let Some(param_supported_features) = param_supported_features {
                query_string.append_pair("supported-features", &param_supported_features);
            }
            if let Some(param_plmn_id) = param_plmn_id {
                query_string.append_pair(
                    "plmn-id",
                    &match serde_json::to_string(&param_plmn_id) {
                        Ok(str) => str,
                        Err(e) => {
                            return Err(ApiError(format!(
                                "Unable to serialize plmn_id to string: {}",
                                e
                            )))
                        }
                    },
                );
            }
            if let Some(param_adjacent_plmns) = param_adjacent_plmns {
                query_string.append_pair(
                    "adjacent-plmns",
                    &param_adjacent_plmns
                        .iter()
                        .map(ToString::to_string)
                        .collect::<Vec<String>>()
                        .join(","),
                );
            }
            if let Some(param_disaster_roaming_ind) = param_disaster_roaming_ind {
                query_string.append_pair(
                    "disaster-roaming-ind",
                    &param_disaster_roaming_ind.to_string(),
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

        match param_if_modified_since {
            Some(param_if_modified_since) => {
                request.headers_mut().append(
                    HeaderName::from_static("if-modified-since"),
                    #[allow(clippy::redundant_clone)]
                    match header::IntoHeaderValue(param_if_modified_since.clone()).try_into() {
                        Ok(header) => header,
                        Err(e) => {
                            return Err(ApiError(format!(
                                "Invalid header if_modified_since - {}",
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

                let response_last_modified = match response
                    .headers()
                    .get(HeaderName::from_static("last-modified"))
                {
                    Some(response_last_modified) => {
                        let response_last_modified = response_last_modified.clone();
                        let response_last_modified =
                            match TryInto::<header::IntoHeaderValue<String>>::try_into(
                                response_last_modified,
                            ) {
                                Ok(value) => value,
                                Err(e) => {
                                    return Err(ApiError(format!("Invalid response header Last-Modified for response 200 - {}", e)));
                                }
                            };
                        Some(response_last_modified.0)
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
                let body = serde_json::from_str::<models::AccessAndMobilitySubscriptionData>(body)
                    .map_err(|e| {
                        ApiError(format!("Response body did not match the schema: {}", e))
                    })?;
                Ok(GetAmDataResponse::ExpectedResponseToAValidRequest {
                    body,
                    cache_control: response_cache_control,
                    e_tag: response_e_tag,
                    last_modified: response_last_modified,
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
                Ok(GetAmDataResponse::BadRequest(body))
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
                Ok(GetAmDataResponse::NotFound(body))
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
                Ok(GetAmDataResponse::InternalServerError(body))
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
                Ok(GetAmDataResponse::ServiceUnavailable(body))
            }
            0 => Ok(GetAmDataResponse::UnexpectedError),
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

    async fn get_mbs_data(
        &self,
        param_supi: String,
        param_supported_features: Option<String>,
        param_if_none_match: Option<String>,
        param_if_modified_since: Option<String>,
        context: &C,
    ) -> Result<GetMbsDataResponse, ApiError> {
        let mut client_service = self.client_service.clone();
        let mut uri = format!(
            "{}/nudm-sdm/v2/{supi}/5mbs-data",
            self.base_path,
            supi = utf8_percent_encode(&param_supi.to_string(), ID_ENCODE_SET)
        );

        // Query parameters
        let query_string = {
            let mut query_string = form_urlencoded::Serializer::new("".to_owned());
            if let Some(param_supported_features) = param_supported_features {
                query_string.append_pair("supported-features", &param_supported_features);
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

        match param_if_modified_since {
            Some(param_if_modified_since) => {
                request.headers_mut().append(
                    HeaderName::from_static("if-modified-since"),
                    #[allow(clippy::redundant_clone)]
                    match header::IntoHeaderValue(param_if_modified_since.clone()).try_into() {
                        Ok(header) => header,
                        Err(e) => {
                            return Err(ApiError(format!(
                                "Invalid header if_modified_since - {}",
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

                let response_last_modified = match response
                    .headers()
                    .get(HeaderName::from_static("last-modified"))
                {
                    Some(response_last_modified) => {
                        let response_last_modified = response_last_modified.clone();
                        let response_last_modified =
                            match TryInto::<header::IntoHeaderValue<String>>::try_into(
                                response_last_modified,
                            ) {
                                Ok(value) => value,
                                Err(e) => {
                                    return Err(ApiError(format!("Invalid response header Last-Modified for response 200 - {}", e)));
                                }
                            };
                        Some(response_last_modified.0)
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
                    serde_json::from_str::<models::MbsSubscriptionData>(body).map_err(|e| {
                        ApiError(format!("Response body did not match the schema: {}", e))
                    })?;
                Ok(GetMbsDataResponse::ExpectedResponseToAValidRequest {
                    body,
                    cache_control: response_cache_control,
                    e_tag: response_e_tag,
                    last_modified: response_last_modified,
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
                Ok(GetMbsDataResponse::BadRequest(body))
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
                Ok(GetMbsDataResponse::NotFound(body))
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
                Ok(GetMbsDataResponse::InternalServerError(body))
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
                Ok(GetMbsDataResponse::ServiceUnavailable(body))
            }
            0 => Ok(GetMbsDataResponse::UnexpectedError),
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

    async fn get_ecr_data(
        &self,
        param_supi: String,
        param_supported_features: Option<String>,
        param_if_none_match: Option<String>,
        param_if_modified_since: Option<String>,
        context: &C,
    ) -> Result<GetEcrDataResponse, ApiError> {
        let mut client_service = self.client_service.clone();
        let mut uri = format!(
            "{}/nudm-sdm/v2/{supi}/am-data/ecr-data",
            self.base_path,
            supi = utf8_percent_encode(&param_supi.to_string(), ID_ENCODE_SET)
        );

        // Query parameters
        let query_string = {
            let mut query_string = form_urlencoded::Serializer::new("".to_owned());
            if let Some(param_supported_features) = param_supported_features {
                query_string.append_pair("supported-features", &param_supported_features);
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

        match param_if_modified_since {
            Some(param_if_modified_since) => {
                request.headers_mut().append(
                    HeaderName::from_static("if-modified-since"),
                    #[allow(clippy::redundant_clone)]
                    match header::IntoHeaderValue(param_if_modified_since.clone()).try_into() {
                        Ok(header) => header,
                        Err(e) => {
                            return Err(ApiError(format!(
                                "Invalid header if_modified_since - {}",
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

                let response_last_modified = match response
                    .headers()
                    .get(HeaderName::from_static("last-modified"))
                {
                    Some(response_last_modified) => {
                        let response_last_modified = response_last_modified.clone();
                        let response_last_modified =
                            match TryInto::<header::IntoHeaderValue<String>>::try_into(
                                response_last_modified,
                            ) {
                                Ok(value) => value,
                                Err(e) => {
                                    return Err(ApiError(format!("Invalid response header Last-Modified for response 200 - {}", e)));
                                }
                            };
                        Some(response_last_modified.0)
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
                let body = serde_json::from_str::<models::EnhancedCoverageRestrictionData>(body)
                    .map_err(|e| {
                        ApiError(format!("Response body did not match the schema: {}", e))
                    })?;
                Ok(GetEcrDataResponse::ExpectedResponseToAValidRequest {
                    body,
                    cache_control: response_cache_control,
                    e_tag: response_e_tag,
                    last_modified: response_last_modified,
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
                Ok(GetEcrDataResponse::BadRequest(body))
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
                Ok(GetEcrDataResponse::NotFound(body))
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
                Ok(GetEcrDataResponse::InternalServerError(body))
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
                Ok(GetEcrDataResponse::ServiceUnavailable(body))
            }
            0 => Ok(GetEcrDataResponse::UnexpectedError),
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

    async fn get_supi_or_gpsi(
        &self,
        param_ue_id: String,
        param_supported_features: Option<String>,
        param_af_id: Option<String>,
        param_app_port_id: Option<models::AppPortId>,
        param_af_service_id: Option<String>,
        param_mtc_provider_info: Option<String>,
        param_requested_gpsi_type: Option<models::GpsiType>,
        param_if_none_match: Option<String>,
        param_if_modified_since: Option<String>,
        context: &C,
    ) -> Result<GetSupiOrGpsiResponse, ApiError> {
        let mut client_service = self.client_service.clone();
        let mut uri = format!(
            "{}/nudm-sdm/v2/{ue_id}/id-translation-result",
            self.base_path,
            ue_id = utf8_percent_encode(&param_ue_id.to_string(), ID_ENCODE_SET)
        );

        // Query parameters
        let query_string = {
            let mut query_string = form_urlencoded::Serializer::new("".to_owned());
            if let Some(param_supported_features) = param_supported_features {
                query_string.append_pair("supported-features", &param_supported_features);
            }
            if let Some(param_af_id) = param_af_id {
                query_string.append_pair("af-id", &param_af_id);
            }
            if let Some(param_app_port_id) = param_app_port_id {
                query_string.append_pair(
                    "app-port-id",
                    &match serde_json::to_string(&param_app_port_id) {
                        Ok(str) => str,
                        Err(e) => {
                            return Err(ApiError(format!(
                                "Unable to serialize app_port_id to string: {}",
                                e
                            )))
                        }
                    },
                );
            }
            if let Some(param_af_service_id) = param_af_service_id {
                query_string.append_pair("af-service-id", &param_af_service_id);
            }
            if let Some(param_mtc_provider_info) = param_mtc_provider_info {
                query_string.append_pair("mtc-provider-info", &param_mtc_provider_info);
            }
            if let Some(param_requested_gpsi_type) = param_requested_gpsi_type {
                query_string.append_pair(
                    "requested-gpsi-type",
                    &param_requested_gpsi_type.to_string(),
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

        match param_if_modified_since {
            Some(param_if_modified_since) => {
                request.headers_mut().append(
                    HeaderName::from_static("if-modified-since"),
                    #[allow(clippy::redundant_clone)]
                    match header::IntoHeaderValue(param_if_modified_since.clone()).try_into() {
                        Ok(header) => header,
                        Err(e) => {
                            return Err(ApiError(format!(
                                "Invalid header if_modified_since - {}",
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

                let response_last_modified = match response
                    .headers()
                    .get(HeaderName::from_static("last-modified"))
                {
                    Some(response_last_modified) => {
                        let response_last_modified = response_last_modified.clone();
                        let response_last_modified =
                            match TryInto::<header::IntoHeaderValue<String>>::try_into(
                                response_last_modified,
                            ) {
                                Ok(value) => value,
                                Err(e) => {
                                    return Err(ApiError(format!("Invalid response header Last-Modified for response 200 - {}", e)));
                                }
                            };
                        Some(response_last_modified.0)
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
                    serde_json::from_str::<models::IdTranslationResult>(body).map_err(|e| {
                        ApiError(format!("Response body did not match the schema: {}", e))
                    })?;
                Ok(GetSupiOrGpsiResponse::ExpectedResponseToAValidRequest {
                    body,
                    cache_control: response_cache_control,
                    e_tag: response_e_tag,
                    last_modified: response_last_modified,
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
                Ok(GetSupiOrGpsiResponse::BadRequest(body))
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
                Ok(GetSupiOrGpsiResponse::Forbidden(body))
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
                Ok(GetSupiOrGpsiResponse::NotFound(body))
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
                Ok(GetSupiOrGpsiResponse::InternalServerError(body))
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
                Ok(GetSupiOrGpsiResponse::ServiceUnavailable(body))
            }
            0 => Ok(GetSupiOrGpsiResponse::UnexpectedError),
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

    async fn get_group_identifiers(
        &self,
        param_ext_group_id: Option<String>,
        param_int_group_id: Option<String>,
        param_ue_id_ind: Option<bool>,
        param_supported_features: Option<String>,
        param_af_id: Option<String>,
        param_if_none_match: Option<String>,
        param_if_modified_since: Option<String>,
        context: &C,
    ) -> Result<GetGroupIdentifiersResponse, ApiError> {
        let mut client_service = self.client_service.clone();
        let mut uri = format!(
            "{}/nudm-sdm/v2/group-data/group-identifiers",
            self.base_path
        );

        // Query parameters
        let query_string = {
            let mut query_string = form_urlencoded::Serializer::new("".to_owned());
            if let Some(param_ext_group_id) = param_ext_group_id {
                query_string.append_pair("ext-group-id", &param_ext_group_id);
            }
            if let Some(param_int_group_id) = param_int_group_id {
                query_string.append_pair("int-group-id", &param_int_group_id);
            }
            if let Some(param_ue_id_ind) = param_ue_id_ind {
                query_string.append_pair("ue-id-ind", &param_ue_id_ind.to_string());
            }
            if let Some(param_supported_features) = param_supported_features {
                query_string.append_pair("supported-features", &param_supported_features);
            }
            if let Some(param_af_id) = param_af_id {
                query_string.append_pair("af-id", &param_af_id);
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

        match param_if_modified_since {
            Some(param_if_modified_since) => {
                request.headers_mut().append(
                    HeaderName::from_static("if-modified-since"),
                    #[allow(clippy::redundant_clone)]
                    match header::IntoHeaderValue(param_if_modified_since.clone()).try_into() {
                        Ok(header) => header,
                        Err(e) => {
                            return Err(ApiError(format!(
                                "Invalid header if_modified_since - {}",
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

                let response_last_modified = match response
                    .headers()
                    .get(HeaderName::from_static("last-modified"))
                {
                    Some(response_last_modified) => {
                        let response_last_modified = response_last_modified.clone();
                        let response_last_modified =
                            match TryInto::<header::IntoHeaderValue<String>>::try_into(
                                response_last_modified,
                            ) {
                                Ok(value) => value,
                                Err(e) => {
                                    return Err(ApiError(format!("Invalid response header Last-Modified for response 200 - {}", e)));
                                }
                            };
                        Some(response_last_modified.0)
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
                let body = serde_json::from_str::<models::GroupIdentifiers>(body).map_err(|e| {
                    ApiError(format!("Response body did not match the schema: {}", e))
                })?;
                Ok(
                    GetGroupIdentifiersResponse::ExpectedResponseToAValidRequest {
                        body,
                        cache_control: response_cache_control,
                        e_tag: response_e_tag,
                        last_modified: response_last_modified,
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
                Ok(GetGroupIdentifiersResponse::BadRequest(body))
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
                Ok(GetGroupIdentifiersResponse::Forbidden(body))
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
                Ok(GetGroupIdentifiersResponse::NotFound(body))
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
                Ok(GetGroupIdentifiersResponse::InternalServerError(body))
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
                Ok(GetGroupIdentifiersResponse::ServiceUnavailable(body))
            }
            0 => Ok(GetGroupIdentifiersResponse::UnexpectedError),
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

    async fn get_lcs_bca_data(
        &self,
        param_supi: String,
        param_supported_features: Option<String>,
        param_plmn_id: Option<models::PlmnId>,
        param_if_none_match: Option<String>,
        param_if_modified_since: Option<String>,
        context: &C,
    ) -> Result<GetLcsBcaDataResponse, ApiError> {
        let mut client_service = self.client_service.clone();
        let mut uri = format!(
            "{}/nudm-sdm/v2/{supi}/lcs-bca-data",
            self.base_path,
            supi = utf8_percent_encode(&param_supi.to_string(), ID_ENCODE_SET)
        );

        // Query parameters
        let query_string = {
            let mut query_string = form_urlencoded::Serializer::new("".to_owned());
            if let Some(param_supported_features) = param_supported_features {
                query_string.append_pair("supported-features", &param_supported_features);
            }
            if let Some(param_plmn_id) = param_plmn_id {
                query_string.append_pair(
                    "plmn-id",
                    &match serde_json::to_string(&param_plmn_id) {
                        Ok(str) => str,
                        Err(e) => {
                            return Err(ApiError(format!(
                                "Unable to serialize plmn_id to string: {}",
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

        match param_if_modified_since {
            Some(param_if_modified_since) => {
                request.headers_mut().append(
                    HeaderName::from_static("if-modified-since"),
                    #[allow(clippy::redundant_clone)]
                    match header::IntoHeaderValue(param_if_modified_since.clone()).try_into() {
                        Ok(header) => header,
                        Err(e) => {
                            return Err(ApiError(format!(
                                "Invalid header if_modified_since - {}",
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

                let response_last_modified = match response
                    .headers()
                    .get(HeaderName::from_static("last-modified"))
                {
                    Some(response_last_modified) => {
                        let response_last_modified = response_last_modified.clone();
                        let response_last_modified =
                            match TryInto::<header::IntoHeaderValue<String>>::try_into(
                                response_last_modified,
                            ) {
                                Ok(value) => value,
                                Err(e) => {
                                    return Err(ApiError(format!("Invalid response header Last-Modified for response 200 - {}", e)));
                                }
                            };
                        Some(response_last_modified.0)
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
                let body = serde_json::from_str::<models::LcsBroadcastAssistanceTypesData>(body)
                    .map_err(|e| {
                        ApiError(format!("Response body did not match the schema: {}", e))
                    })?;
                Ok(GetLcsBcaDataResponse::ExpectedResponseToAValidRequest {
                    body,
                    cache_control: response_cache_control,
                    e_tag: response_e_tag,
                    last_modified: response_last_modified,
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
                Ok(GetLcsBcaDataResponse::BadRequest(body))
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
                Ok(GetLcsBcaDataResponse::NotFound(body))
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
                Ok(GetLcsBcaDataResponse::InternalServerError(body))
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
                Ok(GetLcsBcaDataResponse::ServiceUnavailable(body))
            }
            0 => Ok(GetLcsBcaDataResponse::UnexpectedError),
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

    async fn get_lcs_mo_data(
        &self,
        param_supi: String,
        param_supported_features: Option<String>,
        param_if_none_match: Option<String>,
        param_if_modified_since: Option<String>,
        context: &C,
    ) -> Result<GetLcsMoDataResponse, ApiError> {
        let mut client_service = self.client_service.clone();
        let mut uri = format!(
            "{}/nudm-sdm/v2/{supi}/lcs-mo-data",
            self.base_path,
            supi = utf8_percent_encode(&param_supi.to_string(), ID_ENCODE_SET)
        );

        // Query parameters
        let query_string = {
            let mut query_string = form_urlencoded::Serializer::new("".to_owned());
            if let Some(param_supported_features) = param_supported_features {
                query_string.append_pair("supported-features", &param_supported_features);
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

        match param_if_modified_since {
            Some(param_if_modified_since) => {
                request.headers_mut().append(
                    HeaderName::from_static("if-modified-since"),
                    #[allow(clippy::redundant_clone)]
                    match header::IntoHeaderValue(param_if_modified_since.clone()).try_into() {
                        Ok(header) => header,
                        Err(e) => {
                            return Err(ApiError(format!(
                                "Invalid header if_modified_since - {}",
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

                let response_last_modified = match response
                    .headers()
                    .get(HeaderName::from_static("last-modified"))
                {
                    Some(response_last_modified) => {
                        let response_last_modified = response_last_modified.clone();
                        let response_last_modified =
                            match TryInto::<header::IntoHeaderValue<String>>::try_into(
                                response_last_modified,
                            ) {
                                Ok(value) => value,
                                Err(e) => {
                                    return Err(ApiError(format!("Invalid response header Last-Modified for response 200 - {}", e)));
                                }
                            };
                        Some(response_last_modified.0)
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
                let body = serde_json::from_str::<models::LcsMoData>(body).map_err(|e| {
                    ApiError(format!("Response body did not match the schema: {}", e))
                })?;
                Ok(GetLcsMoDataResponse::ExpectedResponseToAValidRequest {
                    body,
                    cache_control: response_cache_control,
                    e_tag: response_e_tag,
                    last_modified: response_last_modified,
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
                Ok(GetLcsMoDataResponse::BadRequest(body))
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
                Ok(GetLcsMoDataResponse::NotFound(body))
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
                Ok(GetLcsMoDataResponse::InternalServerError(body))
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
                Ok(GetLcsMoDataResponse::ServiceUnavailable(body))
            }
            0 => Ok(GetLcsMoDataResponse::UnexpectedError),
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

    async fn get_lcs_privacy_data(
        &self,
        param_ue_id: String,
        param_supported_features: Option<String>,
        param_if_none_match: Option<String>,
        param_if_modified_since: Option<String>,
        context: &C,
    ) -> Result<GetLcsPrivacyDataResponse, ApiError> {
        let mut client_service = self.client_service.clone();
        let mut uri = format!(
            "{}/nudm-sdm/v2/{ue_id}/lcs-privacy-data",
            self.base_path,
            ue_id = utf8_percent_encode(&param_ue_id.to_string(), ID_ENCODE_SET)
        );

        // Query parameters
        let query_string = {
            let mut query_string = form_urlencoded::Serializer::new("".to_owned());
            if let Some(param_supported_features) = param_supported_features {
                query_string.append_pair("supported-features", &param_supported_features);
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

        match param_if_modified_since {
            Some(param_if_modified_since) => {
                request.headers_mut().append(
                    HeaderName::from_static("if-modified-since"),
                    #[allow(clippy::redundant_clone)]
                    match header::IntoHeaderValue(param_if_modified_since.clone()).try_into() {
                        Ok(header) => header,
                        Err(e) => {
                            return Err(ApiError(format!(
                                "Invalid header if_modified_since - {}",
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

                let response_last_modified = match response
                    .headers()
                    .get(HeaderName::from_static("last-modified"))
                {
                    Some(response_last_modified) => {
                        let response_last_modified = response_last_modified.clone();
                        let response_last_modified =
                            match TryInto::<header::IntoHeaderValue<String>>::try_into(
                                response_last_modified,
                            ) {
                                Ok(value) => value,
                                Err(e) => {
                                    return Err(ApiError(format!("Invalid response header Last-Modified for response 200 - {}", e)));
                                }
                            };
                        Some(response_last_modified.0)
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
                let body = serde_json::from_str::<models::LcsPrivacyData>(body).map_err(|e| {
                    ApiError(format!("Response body did not match the schema: {}", e))
                })?;
                Ok(GetLcsPrivacyDataResponse::ExpectedResponseToAValidRequest {
                    body,
                    cache_control: response_cache_control,
                    e_tag: response_e_tag,
                    last_modified: response_last_modified,
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
                Ok(GetLcsPrivacyDataResponse::BadRequest(body))
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
                Ok(GetLcsPrivacyDataResponse::NotFound(body))
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
                Ok(GetLcsPrivacyDataResponse::InternalServerError(body))
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
                Ok(GetLcsPrivacyDataResponse::ServiceUnavailable(body))
            }
            0 => Ok(GetLcsPrivacyDataResponse::UnexpectedError),
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

    async fn get_multiple_identifiers(
        &self,
        param_gpsi_list: &Vec<models::Gpsi>,
        param_supported_features: Option<String>,
        context: &C,
    ) -> Result<GetMultipleIdentifiersResponse, ApiError> {
        let mut client_service = self.client_service.clone();
        let mut uri = format!("{}/nudm-sdm/v2/multiple-identifiers", self.base_path);

        // Query parameters
        let query_string = {
            let mut query_string = form_urlencoded::Serializer::new("".to_owned());
            if let Some(param_supported_features) = param_supported_features {
                query_string.append_pair("supported-features", &param_supported_features);
            }
            query_string.append_pair(
                "gpsi-list",
                &param_gpsi_list
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<String>>()
                    .join(","),
            );
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

                let response_last_modified = match response
                    .headers()
                    .get(HeaderName::from_static("last-modified"))
                {
                    Some(response_last_modified) => {
                        let response_last_modified = response_last_modified.clone();
                        let response_last_modified =
                            match TryInto::<header::IntoHeaderValue<String>>::try_into(
                                response_last_modified,
                            ) {
                                Ok(value) => value,
                                Err(e) => {
                                    return Err(ApiError(format!("Invalid response header Last-Modified for response 200 - {}", e)));
                                }
                            };
                        Some(response_last_modified.0)
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
                let body = serde_json::from_str::<
                    std::collections::HashMap<String, models::SupiInfo>,
                >(body)
                .map_err(|e| ApiError(format!("Response body did not match the schema: {}", e)))?;
                Ok(
                    GetMultipleIdentifiersResponse::ExpectedResponseToAValidRequest {
                        body,
                        cache_control: response_cache_control,
                        e_tag: response_e_tag,
                        last_modified: response_last_modified,
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
                Ok(GetMultipleIdentifiersResponse::BadRequest(body))
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
                Ok(GetMultipleIdentifiersResponse::Unauthorized(body))
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
                Ok(GetMultipleIdentifiersResponse::Forbidden(body))
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
                Ok(GetMultipleIdentifiersResponse::NotFound(body))
            }
            406 => Ok(GetMultipleIdentifiersResponse::Status406),
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
                Ok(GetMultipleIdentifiersResponse::TooManyRequests(body))
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
                Ok(GetMultipleIdentifiersResponse::InternalServerError(body))
            }
            502 => {
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
                Ok(GetMultipleIdentifiersResponse::BadGateway(body))
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
                Ok(GetMultipleIdentifiersResponse::ServiceUnavailable(body))
            }
            0 => Ok(GetMultipleIdentifiersResponse::UnexpectedError),
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

    async fn get_prose_data(
        &self,
        param_supi: String,
        param_supported_features: Option<String>,
        param_if_none_match: Option<String>,
        param_if_modified_since: Option<String>,
        context: &C,
    ) -> Result<GetProseDataResponse, ApiError> {
        let mut client_service = self.client_service.clone();
        let mut uri = format!(
            "{}/nudm-sdm/v2/{supi}/prose-data",
            self.base_path,
            supi = utf8_percent_encode(&param_supi.to_string(), ID_ENCODE_SET)
        );

        // Query parameters
        let query_string = {
            let mut query_string = form_urlencoded::Serializer::new("".to_owned());
            if let Some(param_supported_features) = param_supported_features {
                query_string.append_pair("supported-features", &param_supported_features);
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

        match param_if_modified_since {
            Some(param_if_modified_since) => {
                request.headers_mut().append(
                    HeaderName::from_static("if-modified-since"),
                    #[allow(clippy::redundant_clone)]
                    match header::IntoHeaderValue(param_if_modified_since.clone()).try_into() {
                        Ok(header) => header,
                        Err(e) => {
                            return Err(ApiError(format!(
                                "Invalid header if_modified_since - {}",
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

                let response_last_modified = match response
                    .headers()
                    .get(HeaderName::from_static("last-modified"))
                {
                    Some(response_last_modified) => {
                        let response_last_modified = response_last_modified.clone();
                        let response_last_modified =
                            match TryInto::<header::IntoHeaderValue<String>>::try_into(
                                response_last_modified,
                            ) {
                                Ok(value) => value,
                                Err(e) => {
                                    return Err(ApiError(format!("Invalid response header Last-Modified for response 200 - {}", e)));
                                }
                            };
                        Some(response_last_modified.0)
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
                    serde_json::from_str::<models::ProseSubscriptionData>(body).map_err(|e| {
                        ApiError(format!("Response body did not match the schema: {}", e))
                    })?;
                Ok(GetProseDataResponse::ExpectedResponseToAValidRequest {
                    body,
                    cache_control: response_cache_control,
                    e_tag: response_e_tag,
                    last_modified: response_last_modified,
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
                Ok(GetProseDataResponse::BadRequest(body))
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
                Ok(GetProseDataResponse::NotFound(body))
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
                Ok(GetProseDataResponse::InternalServerError(body))
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
                Ok(GetProseDataResponse::ServiceUnavailable(body))
            }
            0 => Ok(GetProseDataResponse::UnexpectedError),
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

    async fn cag_ack(
        &self,
        param_supi: String,
        param_acknowledge_info: Option<models::AcknowledgeInfo>,
        context: &C,
    ) -> Result<CAgAckResponse, ApiError> {
        let mut client_service = self.client_service.clone();
        let mut uri = format!(
            "{}/nudm-sdm/v2/{supi}/am-data/cag-ack",
            self.base_path,
            supi = utf8_percent_encode(&param_supi.to_string(), ID_ENCODE_SET)
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
            .method("PUT")
            .uri(uri)
            .body(Body::empty())
        {
            Ok(req) => req,
            Err(e) => return Err(ApiError(format!("Unable to create request: {}", e))),
        };

        // Body parameter
        let body = param_acknowledge_info
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
            204 => Ok(CAgAckResponse::SuccessfulAcknowledgement),
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
                Ok(CAgAckResponse::BadRequest(body))
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
                Ok(CAgAckResponse::InternalServerError(body))
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
                Ok(CAgAckResponse::ServiceUnavailable(body))
            }
            0 => Ok(CAgAckResponse::UnexpectedError),
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

    async fn s_nssais_ack(
        &self,
        param_supi: String,
        param_acknowledge_info: Option<models::AcknowledgeInfo>,
        context: &C,
    ) -> Result<SNssaisAckResponse, ApiError> {
        let mut client_service = self.client_service.clone();
        let mut uri = format!(
            "{}/nudm-sdm/v2/{supi}/am-data/subscribed-snssais-ack",
            self.base_path,
            supi = utf8_percent_encode(&param_supi.to_string(), ID_ENCODE_SET)
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
            .method("PUT")
            .uri(uri)
            .body(Body::empty())
        {
            Ok(req) => req,
            Err(e) => return Err(ApiError(format!("Unable to create request: {}", e))),
        };

        // Body parameter
        let body = param_acknowledge_info
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
            204 => Ok(SNssaisAckResponse::SuccessfulAcknowledgement),
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
                Ok(SNssaisAckResponse::BadRequest(body))
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
                Ok(SNssaisAckResponse::InternalServerError(body))
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
                Ok(SNssaisAckResponse::ServiceUnavailable(body))
            }
            0 => Ok(SNssaisAckResponse::UnexpectedError),
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

    async fn sor_ack_info(
        &self,
        param_supi: String,
        param_acknowledge_info: Option<models::AcknowledgeInfo>,
        context: &C,
    ) -> Result<SorAckInfoResponse, ApiError> {
        let mut client_service = self.client_service.clone();
        let mut uri = format!(
            "{}/nudm-sdm/v2/{supi}/am-data/sor-ack",
            self.base_path,
            supi = utf8_percent_encode(&param_supi.to_string(), ID_ENCODE_SET)
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
            .method("PUT")
            .uri(uri)
            .body(Body::empty())
        {
            Ok(req) => req,
            Err(e) => return Err(ApiError(format!("Unable to create request: {}", e))),
        };

        // Body parameter
        let body = param_acknowledge_info
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
            204 => Ok(SorAckInfoResponse::SuccessfulAcknowledgement),
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
                Ok(SorAckInfoResponse::BadRequest(body))
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
                Ok(SorAckInfoResponse::InternalServerError(body))
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
                Ok(SorAckInfoResponse::ServiceUnavailable(body))
            }
            0 => Ok(SorAckInfoResponse::UnexpectedError),
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

    async fn upu_ack(
        &self,
        param_supi: String,
        param_acknowledge_info: Option<models::AcknowledgeInfo>,
        context: &C,
    ) -> Result<UpuAckResponse, ApiError> {
        let mut client_service = self.client_service.clone();
        let mut uri = format!(
            "{}/nudm-sdm/v2/{supi}/am-data/upu-ack",
            self.base_path,
            supi = utf8_percent_encode(&param_supi.to_string(), ID_ENCODE_SET)
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
            .method("PUT")
            .uri(uri)
            .body(Body::empty())
        {
            Ok(req) => req,
            Err(e) => return Err(ApiError(format!("Unable to create request: {}", e))),
        };

        // Body parameter
        let body = param_acknowledge_info
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
            204 => Ok(UpuAckResponse::SuccessfulAcknowledgement),
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
                Ok(UpuAckResponse::BadRequest(body))
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
                Ok(UpuAckResponse::InternalServerError(body))
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
                Ok(UpuAckResponse::ServiceUnavailable(body))
            }
            0 => Ok(UpuAckResponse::UnexpectedError),
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

    async fn get_data_sets(
        &self,
        param_supi: String,
        param_dataset_names: &Vec<models::DataSetName>,
        param_plmn_id: Option<models::PlmnIdNid>,
        param_disaster_roaming_ind: Option<bool>,
        param_supported_features: Option<String>,
        param_if_none_match: Option<String>,
        param_if_modified_since: Option<String>,
        context: &C,
    ) -> Result<GetDataSetsResponse, ApiError> {
        let mut client_service = self.client_service.clone();
        let mut uri = format!(
            "{}/nudm-sdm/v2/{supi}",
            self.base_path,
            supi = utf8_percent_encode(&param_supi.to_string(), ID_ENCODE_SET)
        );

        // Query parameters
        let query_string = {
            let mut query_string = form_urlencoded::Serializer::new("".to_owned());
            query_string.append_pair(
                "dataset-names",
                &param_dataset_names
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<String>>()
                    .join(","),
            );
            if let Some(param_plmn_id) = param_plmn_id {
                query_string.append_pair(
                    "plmn-id",
                    &match serde_json::to_string(&param_plmn_id) {
                        Ok(str) => str,
                        Err(e) => {
                            return Err(ApiError(format!(
                                "Unable to serialize plmn_id to string: {}",
                                e
                            )))
                        }
                    },
                );
            }
            if let Some(param_disaster_roaming_ind) = param_disaster_roaming_ind {
                query_string.append_pair(
                    "disaster-roaming-ind",
                    &param_disaster_roaming_ind.to_string(),
                );
            }
            if let Some(param_supported_features) = param_supported_features {
                query_string.append_pair("supported-features", &param_supported_features);
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

        match param_if_modified_since {
            Some(param_if_modified_since) => {
                request.headers_mut().append(
                    HeaderName::from_static("if-modified-since"),
                    #[allow(clippy::redundant_clone)]
                    match header::IntoHeaderValue(param_if_modified_since.clone()).try_into() {
                        Ok(header) => header,
                        Err(e) => {
                            return Err(ApiError(format!(
                                "Invalid header if_modified_since - {}",
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

                let response_last_modified = match response
                    .headers()
                    .get(HeaderName::from_static("last-modified"))
                {
                    Some(response_last_modified) => {
                        let response_last_modified = response_last_modified.clone();
                        let response_last_modified =
                            match TryInto::<header::IntoHeaderValue<String>>::try_into(
                                response_last_modified,
                            ) {
                                Ok(value) => value,
                                Err(e) => {
                                    return Err(ApiError(format!("Invalid response header Last-Modified for response 200 - {}", e)));
                                }
                            };
                        Some(response_last_modified.0)
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
                    serde_json::from_str::<models::SubscriptionDataSets>(body).map_err(|e| {
                        ApiError(format!("Response body did not match the schema: {}", e))
                    })?;
                Ok(GetDataSetsResponse::ExpectedResponseToAValidRequest {
                    body,
                    cache_control: response_cache_control,
                    e_tag: response_e_tag,
                    last_modified: response_last_modified,
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
                Ok(GetDataSetsResponse::BadRequest(body))
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
                Ok(GetDataSetsResponse::NotFound(body))
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
                Ok(GetDataSetsResponse::InternalServerError(body))
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
                Ok(GetDataSetsResponse::ServiceUnavailable(body))
            }
            0 => Ok(GetDataSetsResponse::UnexpectedError),
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

    async fn get_shared_data(
        &self,
        param_shared_data_ids: &Vec<models::SharedDataId>,
        param_supported_features: Option<String>,
        param_supported_features2: Option<String>,
        param_if_none_match: Option<String>,
        param_if_modified_since: Option<String>,
        context: &C,
    ) -> Result<GetSharedDataResponse, ApiError> {
        let mut client_service = self.client_service.clone();
        let mut uri = format!("{}/nudm-sdm/v2/shared-data", self.base_path);

        // Query parameters
        let query_string = {
            let mut query_string = form_urlencoded::Serializer::new("".to_owned());
            query_string.append_pair(
                "shared-data-ids",
                &param_shared_data_ids
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<String>>()
                    .join(","),
            );
            if let Some(param_supported_features) = param_supported_features {
                query_string.append_pair("supportedFeatures", &param_supported_features);
            }
            if let Some(param_supported_features2) = param_supported_features2 {
                query_string.append_pair("supported-features", &param_supported_features2);
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

        match param_if_modified_since {
            Some(param_if_modified_since) => {
                request.headers_mut().append(
                    HeaderName::from_static("if-modified-since"),
                    #[allow(clippy::redundant_clone)]
                    match header::IntoHeaderValue(param_if_modified_since.clone()).try_into() {
                        Ok(header) => header,
                        Err(e) => {
                            return Err(ApiError(format!(
                                "Invalid header if_modified_since - {}",
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

                let response_last_modified = match response
                    .headers()
                    .get(HeaderName::from_static("last-modified"))
                {
                    Some(response_last_modified) => {
                        let response_last_modified = response_last_modified.clone();
                        let response_last_modified =
                            match TryInto::<header::IntoHeaderValue<String>>::try_into(
                                response_last_modified,
                            ) {
                                Ok(value) => value,
                                Err(e) => {
                                    return Err(ApiError(format!("Invalid response header Last-Modified for response 200 - {}", e)));
                                }
                            };
                        Some(response_last_modified.0)
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
                let body = serde_json::from_str::<Vec<models::SharedData>>(body).map_err(|e| {
                    ApiError(format!("Response body did not match the schema: {}", e))
                })?;
                Ok(GetSharedDataResponse::ExpectedResponseToAValidRequest {
                    body,
                    cache_control: response_cache_control,
                    e_tag: response_e_tag,
                    last_modified: response_last_modified,
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
                Ok(GetSharedDataResponse::BadRequest(body))
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
                Ok(GetSharedDataResponse::NotFound(body))
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
                Ok(GetSharedDataResponse::InternalServerError(body))
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
                Ok(GetSharedDataResponse::ServiceUnavailable(body))
            }
            0 => Ok(GetSharedDataResponse::UnexpectedError),
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

    async fn get_individual_shared_data(
        &self,
        param_shared_data_id: &Vec<models::SharedDataId>,
        param_supported_features: Option<String>,
        param_if_none_match: Option<String>,
        param_if_modified_since: Option<String>,
        context: &C,
    ) -> Result<GetIndividualSharedDataResponse, ApiError> {
        let mut client_service = self.client_service.clone();
        let mut uri = format!(
            "{}/nudm-sdm/v2/shared-data/{shared_data_id}",
            self.base_path,
            shared_data_id = utf8_percent_encode(
                &models::SharedDataId::to_vec_str(param_shared_data_id),
                ID_ENCODE_SET
            )
        );

        // Query parameters
        let query_string = {
            let mut query_string = form_urlencoded::Serializer::new("".to_owned());
            if let Some(param_supported_features) = param_supported_features {
                query_string.append_pair("supported-features", &param_supported_features);
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

        match param_if_modified_since {
            Some(param_if_modified_since) => {
                request.headers_mut().append(
                    HeaderName::from_static("if-modified-since"),
                    #[allow(clippy::redundant_clone)]
                    match header::IntoHeaderValue(param_if_modified_since.clone()).try_into() {
                        Ok(header) => header,
                        Err(e) => {
                            return Err(ApiError(format!(
                                "Invalid header if_modified_since - {}",
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

                let response_last_modified = match response
                    .headers()
                    .get(HeaderName::from_static("last-modified"))
                {
                    Some(response_last_modified) => {
                        let response_last_modified = response_last_modified.clone();
                        let response_last_modified =
                            match TryInto::<header::IntoHeaderValue<String>>::try_into(
                                response_last_modified,
                            ) {
                                Ok(value) => value,
                                Err(e) => {
                                    return Err(ApiError(format!("Invalid response header Last-Modified for response 200 - {}", e)));
                                }
                            };
                        Some(response_last_modified.0)
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
                let body = serde_json::from_str::<models::SharedData>(body).map_err(|e| {
                    ApiError(format!("Response body did not match the schema: {}", e))
                })?;
                Ok(
                    GetIndividualSharedDataResponse::ExpectedResponseToAValidRequest {
                        body,
                        cache_control: response_cache_control,
                        e_tag: response_e_tag,
                        last_modified: response_last_modified,
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
                Ok(GetIndividualSharedDataResponse::BadRequest(body))
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
                Ok(GetIndividualSharedDataResponse::NotFound(body))
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
                Ok(GetIndividualSharedDataResponse::InternalServerError(body))
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
                Ok(GetIndividualSharedDataResponse::ServiceUnavailable(body))
            }
            0 => Ok(GetIndividualSharedDataResponse::UnexpectedError),
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

    async fn get_smf_sel_data(
        &self,
        param_supi: String,
        param_supported_features: Option<String>,
        param_plmn_id: Option<models::PlmnId>,
        param_disaster_roaming_ind: Option<bool>,
        param_if_none_match: Option<String>,
        param_if_modified_since: Option<String>,
        context: &C,
    ) -> Result<GetSmfSelDataResponse, ApiError> {
        let mut client_service = self.client_service.clone();
        let mut uri = format!(
            "{}/nudm-sdm/v2/{supi}/smf-select-data",
            self.base_path,
            supi = utf8_percent_encode(&param_supi.to_string(), ID_ENCODE_SET)
        );

        // Query parameters
        let query_string = {
            let mut query_string = form_urlencoded::Serializer::new("".to_owned());
            if let Some(param_supported_features) = param_supported_features {
                query_string.append_pair("supported-features", &param_supported_features);
            }
            if let Some(param_plmn_id) = param_plmn_id {
                query_string.append_pair(
                    "plmn-id",
                    &match serde_json::to_string(&param_plmn_id) {
                        Ok(str) => str,
                        Err(e) => {
                            return Err(ApiError(format!(
                                "Unable to serialize plmn_id to string: {}",
                                e
                            )))
                        }
                    },
                );
            }
            if let Some(param_disaster_roaming_ind) = param_disaster_roaming_ind {
                query_string.append_pair(
                    "disaster-roaming-ind",
                    &param_disaster_roaming_ind.to_string(),
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

        match param_if_modified_since {
            Some(param_if_modified_since) => {
                request.headers_mut().append(
                    HeaderName::from_static("if-modified-since"),
                    #[allow(clippy::redundant_clone)]
                    match header::IntoHeaderValue(param_if_modified_since.clone()).try_into() {
                        Ok(header) => header,
                        Err(e) => {
                            return Err(ApiError(format!(
                                "Invalid header if_modified_since - {}",
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

                let response_last_modified = match response
                    .headers()
                    .get(HeaderName::from_static("last-modified"))
                {
                    Some(response_last_modified) => {
                        let response_last_modified = response_last_modified.clone();
                        let response_last_modified =
                            match TryInto::<header::IntoHeaderValue<String>>::try_into(
                                response_last_modified,
                            ) {
                                Ok(value) => value,
                                Err(e) => {
                                    return Err(ApiError(format!("Invalid response header Last-Modified for response 200 - {}", e)));
                                }
                            };
                        Some(response_last_modified.0)
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
                let body = serde_json::from_str::<models::SmfSelectionSubscriptionData>(body)
                    .map_err(|e| {
                        ApiError(format!("Response body did not match the schema: {}", e))
                    })?;
                Ok(GetSmfSelDataResponse::ExpectedResponseToAValidRequest {
                    body,
                    cache_control: response_cache_control,
                    e_tag: response_e_tag,
                    last_modified: response_last_modified,
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
                Ok(GetSmfSelDataResponse::BadRequest(body))
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
                Ok(GetSmfSelDataResponse::NotFound(body))
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
                Ok(GetSmfSelDataResponse::InternalServerError(body))
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
                Ok(GetSmfSelDataResponse::ServiceUnavailable(body))
            }
            0 => Ok(GetSmfSelDataResponse::UnexpectedError),
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

    async fn get_sms_mngt_data(
        &self,
        param_supi: String,
        param_supported_features: Option<String>,
        param_plmn_id: Option<models::PlmnId>,
        param_if_none_match: Option<String>,
        param_if_modified_since: Option<String>,
        context: &C,
    ) -> Result<GetSmsMngtDataResponse, ApiError> {
        let mut client_service = self.client_service.clone();
        let mut uri = format!(
            "{}/nudm-sdm/v2/{supi}/sms-mng-data",
            self.base_path,
            supi = utf8_percent_encode(&param_supi.to_string(), ID_ENCODE_SET)
        );

        // Query parameters
        let query_string = {
            let mut query_string = form_urlencoded::Serializer::new("".to_owned());
            if let Some(param_supported_features) = param_supported_features {
                query_string.append_pair("supported-features", &param_supported_features);
            }
            if let Some(param_plmn_id) = param_plmn_id {
                query_string.append_pair(
                    "plmn-id",
                    &match serde_json::to_string(&param_plmn_id) {
                        Ok(str) => str,
                        Err(e) => {
                            return Err(ApiError(format!(
                                "Unable to serialize plmn_id to string: {}",
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

        match param_if_modified_since {
            Some(param_if_modified_since) => {
                request.headers_mut().append(
                    HeaderName::from_static("if-modified-since"),
                    #[allow(clippy::redundant_clone)]
                    match header::IntoHeaderValue(param_if_modified_since.clone()).try_into() {
                        Ok(header) => header,
                        Err(e) => {
                            return Err(ApiError(format!(
                                "Invalid header if_modified_since - {}",
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

                let response_last_modified = match response
                    .headers()
                    .get(HeaderName::from_static("last-modified"))
                {
                    Some(response_last_modified) => {
                        let response_last_modified = response_last_modified.clone();
                        let response_last_modified =
                            match TryInto::<header::IntoHeaderValue<String>>::try_into(
                                response_last_modified,
                            ) {
                                Ok(value) => value,
                                Err(e) => {
                                    return Err(ApiError(format!("Invalid response header Last-Modified for response 200 - {}", e)));
                                }
                            };
                        Some(response_last_modified.0)
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
                let body = serde_json::from_str::<models::SmsManagementSubscriptionData>(body)
                    .map_err(|e| {
                        ApiError(format!("Response body did not match the schema: {}", e))
                    })?;
                Ok(GetSmsMngtDataResponse::ExpectedResponseToAValidRequest {
                    body,
                    cache_control: response_cache_control,
                    e_tag: response_e_tag,
                    last_modified: response_last_modified,
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
                Ok(GetSmsMngtDataResponse::BadRequest(body))
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
                Ok(GetSmsMngtDataResponse::NotFound(body))
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
                Ok(GetSmsMngtDataResponse::InternalServerError(body))
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
                Ok(GetSmsMngtDataResponse::ServiceUnavailable(body))
            }
            0 => Ok(GetSmsMngtDataResponse::UnexpectedError),
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

    async fn get_sms_data(
        &self,
        param_supi: String,
        param_supported_features: Option<String>,
        param_plmn_id: Option<models::PlmnId>,
        param_if_none_match: Option<String>,
        param_if_modified_since: Option<String>,
        context: &C,
    ) -> Result<GetSmsDataResponse, ApiError> {
        let mut client_service = self.client_service.clone();
        let mut uri = format!(
            "{}/nudm-sdm/v2/{supi}/sms-data",
            self.base_path,
            supi = utf8_percent_encode(&param_supi.to_string(), ID_ENCODE_SET)
        );

        // Query parameters
        let query_string = {
            let mut query_string = form_urlencoded::Serializer::new("".to_owned());
            if let Some(param_supported_features) = param_supported_features {
                query_string.append_pair("supported-features", &param_supported_features);
            }
            if let Some(param_plmn_id) = param_plmn_id {
                query_string.append_pair(
                    "plmn-id",
                    &match serde_json::to_string(&param_plmn_id) {
                        Ok(str) => str,
                        Err(e) => {
                            return Err(ApiError(format!(
                                "Unable to serialize plmn_id to string: {}",
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

        match param_if_modified_since {
            Some(param_if_modified_since) => {
                request.headers_mut().append(
                    HeaderName::from_static("if-modified-since"),
                    #[allow(clippy::redundant_clone)]
                    match header::IntoHeaderValue(param_if_modified_since.clone()).try_into() {
                        Ok(header) => header,
                        Err(e) => {
                            return Err(ApiError(format!(
                                "Invalid header if_modified_since - {}",
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

                let response_last_modified = match response
                    .headers()
                    .get(HeaderName::from_static("last-modified"))
                {
                    Some(response_last_modified) => {
                        let response_last_modified = response_last_modified.clone();
                        let response_last_modified =
                            match TryInto::<header::IntoHeaderValue<String>>::try_into(
                                response_last_modified,
                            ) {
                                Ok(value) => value,
                                Err(e) => {
                                    return Err(ApiError(format!("Invalid response header Last-Modified for response 200 - {}", e)));
                                }
                            };
                        Some(response_last_modified.0)
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
                    serde_json::from_str::<models::SmsSubscriptionData>(body).map_err(|e| {
                        ApiError(format!("Response body did not match the schema: {}", e))
                    })?;
                Ok(GetSmsDataResponse::ExpectedResponseToAValidRequest {
                    body,
                    cache_control: response_cache_control,
                    e_tag: response_e_tag,
                    last_modified: response_last_modified,
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
                Ok(GetSmsDataResponse::BadRequest(body))
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
                Ok(GetSmsDataResponse::NotFound(body))
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
                Ok(GetSmsDataResponse::InternalServerError(body))
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
                Ok(GetSmsDataResponse::ServiceUnavailable(body))
            }
            0 => Ok(GetSmsDataResponse::UnexpectedError),
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

    async fn get_sm_data(
        &self,
        param_supi: String,
        param_supported_features: Option<String>,
        param_single_nssai: Option<models::Snssai>,
        param_dnn: Option<String>,
        param_plmn_id: Option<models::PlmnId>,
        param_if_none_match: Option<String>,
        param_if_modified_since: Option<String>,
        context: &C,
    ) -> Result<GetSmDataResponse, ApiError> {
        let mut client_service = self.client_service.clone();
        let mut uri = format!(
            "{}/nudm-sdm/v2/{supi}/sm-data",
            self.base_path,
            supi = utf8_percent_encode(&param_supi.to_string(), ID_ENCODE_SET)
        );

        // Query parameters
        let query_string = {
            let mut query_string = form_urlencoded::Serializer::new("".to_owned());
            if let Some(param_supported_features) = param_supported_features {
                query_string.append_pair("supported-features", &param_supported_features);
            }
            if let Some(param_single_nssai) = param_single_nssai {
                query_string.append_pair(
                    "single-nssai",
                    &match serde_json::to_string(&param_single_nssai) {
                        Ok(str) => str,
                        Err(e) => {
                            return Err(ApiError(format!(
                                "Unable to serialize single_nssai to string: {}",
                                e
                            )))
                        }
                    },
                );
            }
            if let Some(param_dnn) = param_dnn {
                query_string.append_pair("dnn", &param_dnn);
            }
            if let Some(param_plmn_id) = param_plmn_id {
                query_string.append_pair(
                    "plmn-id",
                    &match serde_json::to_string(&param_plmn_id) {
                        Ok(str) => str,
                        Err(e) => {
                            return Err(ApiError(format!(
                                "Unable to serialize plmn_id to string: {}",
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

        match param_if_modified_since {
            Some(param_if_modified_since) => {
                request.headers_mut().append(
                    HeaderName::from_static("if-modified-since"),
                    #[allow(clippy::redundant_clone)]
                    match header::IntoHeaderValue(param_if_modified_since.clone()).try_into() {
                        Ok(header) => header,
                        Err(e) => {
                            return Err(ApiError(format!(
                                "Invalid header if_modified_since - {}",
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

                let response_last_modified = match response
                    .headers()
                    .get(HeaderName::from_static("last-modified"))
                {
                    Some(response_last_modified) => {
                        let response_last_modified = response_last_modified.clone();
                        let response_last_modified =
                            match TryInto::<header::IntoHeaderValue<String>>::try_into(
                                response_last_modified,
                            ) {
                                Ok(value) => value,
                                Err(e) => {
                                    return Err(ApiError(format!("Invalid response header Last-Modified for response 200 - {}", e)));
                                }
                            };
                        Some(response_last_modified.0)
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
                let body = serde_json::from_str::<models::SmSubsData>(body).map_err(|e| {
                    ApiError(format!("Response body did not match the schema: {}", e))
                })?;
                Ok(GetSmDataResponse::ExpectedResponseToAValidRequest {
                    body,
                    cache_control: response_cache_control,
                    e_tag: response_e_tag,
                    last_modified: response_last_modified,
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
                Ok(GetSmDataResponse::BadRequest(body))
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
                Ok(GetSmDataResponse::NotFound(body))
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
                Ok(GetSmDataResponse::InternalServerError(body))
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
                Ok(GetSmDataResponse::ServiceUnavailable(body))
            }
            0 => Ok(GetSmDataResponse::UnexpectedError),
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

    async fn get_nssai(
        &self,
        param_supi: String,
        param_supported_features: Option<String>,
        param_plmn_id: Option<models::PlmnId>,
        param_disaster_roaming_ind: Option<bool>,
        param_if_none_match: Option<String>,
        param_if_modified_since: Option<String>,
        context: &C,
    ) -> Result<GetNssaiResponse, ApiError> {
        let mut client_service = self.client_service.clone();
        let mut uri = format!(
            "{}/nudm-sdm/v2/{supi}/nssai",
            self.base_path,
            supi = utf8_percent_encode(&param_supi.to_string(), ID_ENCODE_SET)
        );

        // Query parameters
        let query_string = {
            let mut query_string = form_urlencoded::Serializer::new("".to_owned());
            if let Some(param_supported_features) = param_supported_features {
                query_string.append_pair("supported-features", &param_supported_features);
            }
            if let Some(param_plmn_id) = param_plmn_id {
                query_string.append_pair(
                    "plmn-id",
                    &match serde_json::to_string(&param_plmn_id) {
                        Ok(str) => str,
                        Err(e) => {
                            return Err(ApiError(format!(
                                "Unable to serialize plmn_id to string: {}",
                                e
                            )))
                        }
                    },
                );
            }
            if let Some(param_disaster_roaming_ind) = param_disaster_roaming_ind {
                query_string.append_pair(
                    "disaster-roaming-ind",
                    &param_disaster_roaming_ind.to_string(),
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

        match param_if_modified_since {
            Some(param_if_modified_since) => {
                request.headers_mut().append(
                    HeaderName::from_static("if-modified-since"),
                    #[allow(clippy::redundant_clone)]
                    match header::IntoHeaderValue(param_if_modified_since.clone()).try_into() {
                        Ok(header) => header,
                        Err(e) => {
                            return Err(ApiError(format!(
                                "Invalid header if_modified_since - {}",
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

                let response_last_modified = match response
                    .headers()
                    .get(HeaderName::from_static("last-modified"))
                {
                    Some(response_last_modified) => {
                        let response_last_modified = response_last_modified.clone();
                        let response_last_modified =
                            match TryInto::<header::IntoHeaderValue<String>>::try_into(
                                response_last_modified,
                            ) {
                                Ok(value) => value,
                                Err(e) => {
                                    return Err(ApiError(format!("Invalid response header Last-Modified for response 200 - {}", e)));
                                }
                            };
                        Some(response_last_modified.0)
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
                let body = serde_json::from_str::<models::Nssai>(body).map_err(|e| {
                    ApiError(format!("Response body did not match the schema: {}", e))
                })?;
                Ok(GetNssaiResponse::ExpectedResponseToAValidRequest {
                    body,
                    cache_control: response_cache_control,
                    e_tag: response_e_tag,
                    last_modified: response_last_modified,
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
                Ok(GetNssaiResponse::BadRequest(body))
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
                Ok(GetNssaiResponse::NotFound(body))
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
                Ok(GetNssaiResponse::InternalServerError(body))
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
                Ok(GetNssaiResponse::ServiceUnavailable(body))
            }
            0 => Ok(GetNssaiResponse::UnexpectedError),
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

    async fn subscribe(
        &self,
        param_ue_id: String,
        param_sdm_subscription: models::SdmSubscription,
        context: &C,
    ) -> Result<SubscribeResponse, ApiError> {
        let mut client_service = self.client_service.clone();
        let mut uri = format!(
            "{}/nudm-sdm/v2/{ue_id}/sdm-subscriptions",
            self.base_path,
            ue_id = utf8_percent_encode(&param_ue_id.to_string(), ID_ENCODE_SET)
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
        let body = serde_json::to_string(&param_sdm_subscription)
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
                let body = serde_json::from_str::<models::SdmSubscription>(body).map_err(|e| {
                    ApiError(format!("Response body did not match the schema: {}", e))
                })?;
                Ok(SubscribeResponse::ExpectedResponseToAValidRequest {
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
                Ok(SubscribeResponse::BadRequest(body))
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
                Ok(SubscribeResponse::NotFound(body))
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
                Ok(SubscribeResponse::InternalServerError(body))
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
                Ok(SubscribeResponse::NotImplemented(body))
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
                Ok(SubscribeResponse::ServiceUnavailable(body))
            }
            0 => Ok(SubscribeResponse::UnexpectedError),
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

    async fn subscribe_to_shared_data(
        &self,
        param_sdm_subscription: models::SdmSubscription,
        context: &C,
    ) -> Result<SubscribeToSharedDataResponse, ApiError> {
        let mut client_service = self.client_service.clone();
        let mut uri = format!("{}/nudm-sdm/v2/shared-data-subscriptions", self.base_path);

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
        let body = serde_json::to_string(&param_sdm_subscription)
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
                let body = serde_json::from_str::<models::SdmSubscription>(body).map_err(|e| {
                    ApiError(format!("Response body did not match the schema: {}", e))
                })?;
                Ok(
                    SubscribeToSharedDataResponse::ExpectedResponseToAValidRequest {
                        body,
                        location: response_location,
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
                Ok(SubscribeToSharedDataResponse::BadRequest(body))
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
                Ok(SubscribeToSharedDataResponse::NotFound(body))
            }
            0 => Ok(SubscribeToSharedDataResponse::UnexpectedError),
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

    async fn unsubscribe(
        &self,
        param_ue_id: String,
        param_subscription_id: String,
        context: &C,
    ) -> Result<UnsubscribeResponse, ApiError> {
        let mut client_service = self.client_service.clone();
        let mut uri = format!(
            "{}/nudm-sdm/v2/{ue_id}/sdm-subscriptions/{subscription_id}",
            self.base_path,
            ue_id = utf8_percent_encode(&param_ue_id.to_string(), ID_ENCODE_SET),
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
            204 => Ok(UnsubscribeResponse::SuccessfulResponse),
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
                Ok(UnsubscribeResponse::BadRequest(body))
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
                Ok(UnsubscribeResponse::NotFound(body))
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
                Ok(UnsubscribeResponse::InternalServerError(body))
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
                Ok(UnsubscribeResponse::ServiceUnavailable(body))
            }
            0 => Ok(UnsubscribeResponse::UnexpectedError),
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

    async fn unsubscribe_for_shared_data(
        &self,
        param_subscription_id: String,
        context: &C,
    ) -> Result<UnsubscribeForSharedDataResponse, ApiError> {
        let mut client_service = self.client_service.clone();
        let mut uri = format!(
            "{}/nudm-sdm/v2/shared-data-subscriptions/{subscription_id}",
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
            204 => Ok(UnsubscribeForSharedDataResponse::SuccessfulResponse),
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
                Ok(UnsubscribeForSharedDataResponse::BadRequest(body))
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
                Ok(UnsubscribeForSharedDataResponse::NotFound(body))
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
                Ok(UnsubscribeForSharedDataResponse::InternalServerError(body))
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
                Ok(UnsubscribeForSharedDataResponse::ServiceUnavailable(body))
            }
            0 => Ok(UnsubscribeForSharedDataResponse::UnexpectedError),
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

    async fn modify(
        &self,
        param_ue_id: String,
        param_subscription_id: String,
        param_sdm_subs_modification: models::SdmSubsModification,
        param_supported_features: Option<String>,
        context: &C,
    ) -> Result<ModifyResponse, ApiError> {
        let mut client_service = self.client_service.clone();
        let mut uri = format!(
            "{}/nudm-sdm/v2/{ue_id}/sdm-subscriptions/{subscription_id}",
            self.base_path,
            ue_id = utf8_percent_encode(&param_ue_id.to_string(), ID_ENCODE_SET),
            subscription_id =
                utf8_percent_encode(&param_subscription_id.to_string(), ID_ENCODE_SET)
        );

        // Query parameters
        let query_string = {
            let mut query_string = form_urlencoded::Serializer::new("".to_owned());
            if let Some(param_supported_features) = param_supported_features {
                query_string.append_pair("supported-features", &param_supported_features);
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
            .method("PATCH")
            .uri(uri)
            .body(Body::empty())
        {
            Ok(req) => req,
            Err(e) => return Err(ApiError(format!("Unable to create request: {}", e))),
        };

        // Body parameter
        let body = serde_json::to_string(&param_sdm_subs_modification)
            .expect("impossible to fail to serialize");
        *request.body_mut() = Body::from(body);

        let header = "application/merge-patch+json";
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
                    serde_json::from_str::<models::Modify200Response>(body).map_err(|e| {
                        ApiError(format!("Response body did not match the schema: {}", e))
                    })?;
                Ok(ModifyResponse::ExpectedResponseToAValidRequest(body))
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
                Ok(ModifyResponse::BadRequest(body))
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
                Ok(ModifyResponse::Forbidden(body))
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
                Ok(ModifyResponse::NotFound(body))
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
                Ok(ModifyResponse::InternalServerError(body))
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
                Ok(ModifyResponse::ServiceUnavailable(body))
            }
            0 => Ok(ModifyResponse::UnexpectedError),
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

    async fn modify_shared_data_subs(
        &self,
        param_subscription_id: String,
        param_sdm_subs_modification: models::SdmSubsModification,
        param_supported_features: Option<String>,
        context: &C,
    ) -> Result<ModifySharedDataSubsResponse, ApiError> {
        let mut client_service = self.client_service.clone();
        let mut uri = format!(
            "{}/nudm-sdm/v2/shared-data-subscriptions/{subscription_id}",
            self.base_path,
            subscription_id =
                utf8_percent_encode(&param_subscription_id.to_string(), ID_ENCODE_SET)
        );

        // Query parameters
        let query_string = {
            let mut query_string = form_urlencoded::Serializer::new("".to_owned());
            if let Some(param_supported_features) = param_supported_features {
                query_string.append_pair("supported-features", &param_supported_features);
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
            .method("PATCH")
            .uri(uri)
            .body(Body::empty())
        {
            Ok(req) => req,
            Err(e) => return Err(ApiError(format!("Unable to create request: {}", e))),
        };

        let body = serde_json::to_string(&param_sdm_subs_modification)
            .expect("impossible to fail to serialize");

        *request.body_mut() = Body::from(body);

        let header = "application/merge-patch+json";
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
                    serde_json::from_str::<models::Modify200Response>(body).map_err(|e| {
                        ApiError(format!("Response body did not match the schema: {}", e))
                    })?;
                Ok(ModifySharedDataSubsResponse::ExpectedResponseToAValidRequest(body))
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
                Ok(ModifySharedDataSubsResponse::BadRequest(body))
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
                Ok(ModifySharedDataSubsResponse::Forbidden(body))
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
                Ok(ModifySharedDataSubsResponse::NotFound(body))
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
                Ok(ModifySharedDataSubsResponse::InternalServerError(body))
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
                Ok(ModifySharedDataSubsResponse::ServiceUnavailable(body))
            }
            0 => Ok(ModifySharedDataSubsResponse::UnexpectedError),
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

    async fn get_trace_config_data(
        &self,
        param_supi: String,
        param_supported_features: Option<String>,
        param_plmn_id: Option<models::PlmnId>,
        param_if_none_match: Option<String>,
        param_if_modified_since: Option<String>,
        context: &C,
    ) -> Result<GetTraceConfigDataResponse, ApiError> {
        let mut client_service = self.client_service.clone();
        let mut uri = format!(
            "{}/nudm-sdm/v2/{supi}/trace-data",
            self.base_path,
            supi = utf8_percent_encode(&param_supi.to_string(), ID_ENCODE_SET)
        );

        // Query parameters
        let query_string = {
            let mut query_string = form_urlencoded::Serializer::new("".to_owned());
            if let Some(param_supported_features) = param_supported_features {
                query_string.append_pair("supported-features", &param_supported_features);
            }
            if let Some(param_plmn_id) = param_plmn_id {
                query_string.append_pair(
                    "plmn-id",
                    &match serde_json::to_string(&param_plmn_id) {
                        Ok(str) => str,
                        Err(e) => {
                            return Err(ApiError(format!(
                                "Unable to serialize plmn_id to string: {}",
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

        match param_if_modified_since {
            Some(param_if_modified_since) => {
                request.headers_mut().append(
                    HeaderName::from_static("if-modified-since"),
                    #[allow(clippy::redundant_clone)]
                    match header::IntoHeaderValue(param_if_modified_since.clone()).try_into() {
                        Ok(header) => header,
                        Err(e) => {
                            return Err(ApiError(format!(
                                "Invalid header if_modified_since - {}",
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

                let response_last_modified = match response
                    .headers()
                    .get(HeaderName::from_static("last-modified"))
                {
                    Some(response_last_modified) => {
                        let response_last_modified = response_last_modified.clone();
                        let response_last_modified =
                            match TryInto::<header::IntoHeaderValue<String>>::try_into(
                                response_last_modified,
                            ) {
                                Ok(value) => value,
                                Err(e) => {
                                    return Err(ApiError(format!("Invalid response header Last-Modified for response 200 - {}", e)));
                                }
                            };
                        Some(response_last_modified.0)
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
                    serde_json::from_str::<models::TraceDataResponse>(body).map_err(|e| {
                        ApiError(format!("Response body did not match the schema: {}", e))
                    })?;
                Ok(
                    GetTraceConfigDataResponse::ExpectedResponseToAValidRequest {
                        body,
                        cache_control: response_cache_control,
                        e_tag: response_e_tag,
                        last_modified: response_last_modified,
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
                Ok(GetTraceConfigDataResponse::BadRequest(body))
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
                Ok(GetTraceConfigDataResponse::NotFound(body))
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
                Ok(GetTraceConfigDataResponse::InternalServerError(body))
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
                Ok(GetTraceConfigDataResponse::ServiceUnavailable(body))
            }
            0 => Ok(GetTraceConfigDataResponse::UnexpectedError),
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

    async fn update_sor_info(
        &self,
        param_supi: String,
        param_sor_update_info: Option<models::SorUpdateInfo>,
        context: &C,
    ) -> Result<UpdateSorInfoResponse, ApiError> {
        let mut client_service = self.client_service.clone();
        let mut uri = format!(
            "{}/nudm-sdm/v2/{supi}/am-data/update-sor",
            self.base_path,
            supi = utf8_percent_encode(&param_supi.to_string(), ID_ENCODE_SET)
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
        let body = param_sor_update_info
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
                let body = serde_json::from_str::<models::SorInfo>(body).map_err(|e| {
                    ApiError(format!("Response body did not match the schema: {}", e))
                })?;
                Ok(UpdateSorInfoResponse::ExpectedResponseToAValidRequest(body))
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
                Ok(UpdateSorInfoResponse::BadRequest(body))
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
                Ok(UpdateSorInfoResponse::NotFound(body))
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
                Ok(UpdateSorInfoResponse::InternalServerError(body))
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
                Ok(UpdateSorInfoResponse::ServiceUnavailable(body))
            }
            0 => Ok(UpdateSorInfoResponse::UnexpectedError),
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

    async fn get_ue_ctx_in_amf_data(
        &self,
        param_supi: String,
        param_supported_features: Option<String>,
        context: &C,
    ) -> Result<GetUeCtxInAmfDataResponse, ApiError> {
        let mut client_service = self.client_service.clone();
        let mut uri = format!(
            "{}/nudm-sdm/v2/{supi}/ue-context-in-amf-data",
            self.base_path,
            supi = utf8_percent_encode(&param_supi.to_string(), ID_ENCODE_SET)
        );

        // Query parameters
        let query_string = {
            let mut query_string = form_urlencoded::Serializer::new("".to_owned());
            if let Some(param_supported_features) = param_supported_features {
                query_string.append_pair("supported-features", &param_supported_features);
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
                    serde_json::from_str::<models::UeContextInAmfData>(body).map_err(|e| {
                        ApiError(format!("Response body did not match the schema: {}", e))
                    })?;
                Ok(GetUeCtxInAmfDataResponse::ExpectedResponseToAValidRequest(
                    body,
                ))
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
                Ok(GetUeCtxInAmfDataResponse::BadRequest(body))
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
                Ok(GetUeCtxInAmfDataResponse::NotFound(body))
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
                Ok(GetUeCtxInAmfDataResponse::InternalServerError(body))
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
                Ok(GetUeCtxInAmfDataResponse::ServiceUnavailable(body))
            }
            0 => Ok(GetUeCtxInAmfDataResponse::UnexpectedError),
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

    async fn get_ue_ctx_in_smf_data(
        &self,
        param_supi: String,
        param_supported_features: Option<String>,
        context: &C,
    ) -> Result<GetUeCtxInSmfDataResponse, ApiError> {
        let mut client_service = self.client_service.clone();
        let mut uri = format!(
            "{}/nudm-sdm/v2/{supi}/ue-context-in-smf-data",
            self.base_path,
            supi = utf8_percent_encode(&param_supi.to_string(), ID_ENCODE_SET)
        );

        // Query parameters
        let query_string = {
            let mut query_string = form_urlencoded::Serializer::new("".to_owned());
            if let Some(param_supported_features) = param_supported_features {
                query_string.append_pair("supported-features", &param_supported_features);
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
                    serde_json::from_str::<models::UeContextInSmfData>(body).map_err(|e| {
                        ApiError(format!("Response body did not match the schema: {}", e))
                    })?;
                Ok(GetUeCtxInSmfDataResponse::ExpectedResponseToAValidRequest(
                    body,
                ))
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
                Ok(GetUeCtxInSmfDataResponse::BadRequest(body))
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
                Ok(GetUeCtxInSmfDataResponse::NotFound(body))
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
                Ok(GetUeCtxInSmfDataResponse::InternalServerError(body))
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
                Ok(GetUeCtxInSmfDataResponse::ServiceUnavailable(body))
            }
            0 => Ok(GetUeCtxInSmfDataResponse::UnexpectedError),
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

    async fn get_ue_ctx_in_smsf_data(
        &self,
        param_supi: String,
        param_supported_features: Option<String>,
        context: &C,
    ) -> Result<GetUeCtxInSmsfDataResponse, ApiError> {
        let mut client_service = self.client_service.clone();
        let mut uri = format!(
            "{}/nudm-sdm/v2/{supi}/ue-context-in-smsf-data",
            self.base_path,
            supi = utf8_percent_encode(&param_supi.to_string(), ID_ENCODE_SET)
        );

        // Query parameters
        let query_string = {
            let mut query_string = form_urlencoded::Serializer::new("".to_owned());
            if let Some(param_supported_features) = param_supported_features {
                query_string.append_pair("supported-features", &param_supported_features);
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
                    serde_json::from_str::<models::UeContextInSmsfData>(body).map_err(|e| {
                        ApiError(format!("Response body did not match the schema: {}", e))
                    })?;
                Ok(GetUeCtxInSmsfDataResponse::ExpectedResponseToAValidRequest(
                    body,
                ))
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
                Ok(GetUeCtxInSmsfDataResponse::BadRequest(body))
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
                Ok(GetUeCtxInSmsfDataResponse::NotFound(body))
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
                Ok(GetUeCtxInSmsfDataResponse::InternalServerError(body))
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
                Ok(GetUeCtxInSmsfDataResponse::ServiceUnavailable(body))
            }
            0 => Ok(GetUeCtxInSmsfDataResponse::UnexpectedError),
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

    async fn get_uc_data(
        &self,
        param_supi: String,
        param_supported_features: Option<String>,
        param_uc_purpose: Option<models::UcPurpose>,
        param_if_none_match: Option<String>,
        param_if_modified_since: Option<String>,
        context: &C,
    ) -> Result<GetUcDataResponse, ApiError> {
        let mut client_service = self.client_service.clone();
        let mut uri = format!(
            "{}/nudm-sdm/v2/{supi}/uc-data",
            self.base_path,
            supi = utf8_percent_encode(&param_supi.to_string(), ID_ENCODE_SET)
        );

        // Query parameters
        let query_string = {
            let mut query_string = form_urlencoded::Serializer::new("".to_owned());
            if let Some(param_supported_features) = param_supported_features {
                query_string.append_pair("supported-features", &param_supported_features);
            }
            if let Some(param_uc_purpose) = param_uc_purpose {
                query_string.append_pair("uc-purpose", &param_uc_purpose.to_string());
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

        match param_if_modified_since {
            Some(param_if_modified_since) => {
                request.headers_mut().append(
                    HeaderName::from_static("if-modified-since"),
                    #[allow(clippy::redundant_clone)]
                    match header::IntoHeaderValue(param_if_modified_since.clone()).try_into() {
                        Ok(header) => header,
                        Err(e) => {
                            return Err(ApiError(format!(
                                "Invalid header if_modified_since - {}",
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

                let response_last_modified = match response
                    .headers()
                    .get(HeaderName::from_static("last-modified"))
                {
                    Some(response_last_modified) => {
                        let response_last_modified = response_last_modified.clone();
                        let response_last_modified =
                            match TryInto::<header::IntoHeaderValue<String>>::try_into(
                                response_last_modified,
                            ) {
                                Ok(value) => value,
                                Err(e) => {
                                    return Err(ApiError(format!("Invalid response header Last-Modified for response 200 - {}", e)));
                                }
                            };
                        Some(response_last_modified.0)
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
                    serde_json::from_str::<models::UcSubscriptionData>(body).map_err(|e| {
                        ApiError(format!("Response body did not match the schema: {}", e))
                    })?;
                Ok(GetUcDataResponse::ExpectedResponseToAValidRequest {
                    body,
                    cache_control: response_cache_control,
                    e_tag: response_e_tag,
                    last_modified: response_last_modified,
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
                Ok(GetUcDataResponse::BadRequest(body))
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
                Ok(GetUcDataResponse::NotFound(body))
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
                Ok(GetUcDataResponse::InternalServerError(body))
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
                Ok(GetUcDataResponse::ServiceUnavailable(body))
            }
            0 => Ok(GetUcDataResponse::UnexpectedError),
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

    async fn get_v2x_data(
        &self,
        param_supi: String,
        param_supported_features: Option<String>,
        param_if_none_match: Option<String>,
        param_if_modified_since: Option<String>,
        context: &C,
    ) -> Result<GetV2xDataResponse, ApiError> {
        let mut client_service = self.client_service.clone();
        let mut uri = format!(
            "{}/nudm-sdm/v2/{supi}/v2x-data",
            self.base_path,
            supi = utf8_percent_encode(&param_supi.to_string(), ID_ENCODE_SET)
        );

        // Query parameters
        let query_string = {
            let mut query_string = form_urlencoded::Serializer::new("".to_owned());
            if let Some(param_supported_features) = param_supported_features {
                query_string.append_pair("supported-features", &param_supported_features);
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

        match param_if_modified_since {
            Some(param_if_modified_since) => {
                request.headers_mut().append(
                    HeaderName::from_static("if-modified-since"),
                    #[allow(clippy::redundant_clone)]
                    match header::IntoHeaderValue(param_if_modified_since.clone()).try_into() {
                        Ok(header) => header,
                        Err(e) => {
                            return Err(ApiError(format!(
                                "Invalid header if_modified_since - {}",
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

                let response_last_modified = match response
                    .headers()
                    .get(HeaderName::from_static("last-modified"))
                {
                    Some(response_last_modified) => {
                        let response_last_modified = response_last_modified.clone();
                        let response_last_modified =
                            match TryInto::<header::IntoHeaderValue<String>>::try_into(
                                response_last_modified,
                            ) {
                                Ok(value) => value,
                                Err(e) => {
                                    return Err(ApiError(format!("Invalid response header Last-Modified for response 200 - {}", e)));
                                }
                            };
                        Some(response_last_modified.0)
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
                    serde_json::from_str::<models::V2xSubscriptionData>(body).map_err(|e| {
                        ApiError(format!("Response body did not match the schema: {}", e))
                    })?;
                Ok(GetV2xDataResponse::ExpectedResponseToAValidRequest {
                    body,
                    cache_control: response_cache_control,
                    e_tag: response_e_tag,
                    last_modified: response_last_modified,
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
                Ok(GetV2xDataResponse::BadRequest(body))
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
                Ok(GetV2xDataResponse::NotFound(body))
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
                Ok(GetV2xDataResponse::InternalServerError(body))
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
                Ok(GetV2xDataResponse::ServiceUnavailable(body))
            }
            0 => Ok(GetV2xDataResponse::UnexpectedError),
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
