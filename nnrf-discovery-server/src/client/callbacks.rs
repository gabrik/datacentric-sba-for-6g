use futures::{future, future::BoxFuture, future::FutureExt, stream, stream::TryStreamExt, Stream};
use hyper::header::{HeaderName, HeaderValue, CONTENT_TYPE};
use hyper::{Body, HeaderMap, Request, Response, StatusCode};
use log::warn;
#[allow(unused_imports)]
use std::convert::{TryFrom, TryInto};
use std::error::Error;
use std::future::Future;
use std::marker::PhantomData;
use std::task::{Context, Poll};
pub use swagger::auth::Authorization;
use swagger::auth::Scopes;
use swagger::{ApiError, BodyExt, Has, RequestParser, XSpanIdString};
use url::form_urlencoded;

use crate::header;
#[allow(unused_imports)]
use crate::models;

pub use crate::context;

type ServiceFuture = BoxFuture<'static, Result<Response<Body>, crate::ServiceError>>;

use crate::CallbackApi as Api;
use crate::OnScpDomainRoutingInformationChangePostResponse;

mod paths {
    use lazy_static::lazy_static;

    lazy_static! {
        pub static ref GLOBAL_REGEX_SET: regex::RegexSet =
            regex::RegexSet::new(vec![r"^/nnrf-disc/v1/(?P<request_body_callback_uri>.*)$"])
                .expect("Unable to create global regex set");
    }
    pub(crate) static ID_REQUEST_BODY_CALLBACKURI: usize = 0;
    lazy_static! {
        pub static ref REGEX_REQUEST_BODY_CALLBACKURI: regex::Regex =
            #[allow(clippy::invalid_regex)]
            regex::Regex::new(r"^/nnrf-disc/v1/(?P<request_body_callback_uri>.*)$")
                .expect("Unable to create regex for REQUEST_BODY_CALLBACKURI");
    }
}

pub struct MakeService<T, C>
where
    T: Api<C> + Clone + Send + 'static,
    C: Has<XSpanIdString> + Has<Option<Authorization>> + Send + Sync + 'static,
{
    api_impl: T,
    marker: PhantomData<C>,
}

impl<T, C> MakeService<T, C>
where
    T: Api<C> + Clone + Send + 'static,
    C: Has<XSpanIdString> + Has<Option<Authorization>> + Send + Sync + 'static,
{
    pub fn new(api_impl: T) -> Self {
        MakeService {
            api_impl,
            marker: PhantomData,
        }
    }
}

impl<T, C, Target> hyper::service::Service<Target> for MakeService<T, C>
where
    T: Api<C> + Clone + Send + 'static,
    C: Has<XSpanIdString> + Has<Option<Authorization>> + Send + Sync + 'static,
{
    type Response = Service<T, C>;
    type Error = crate::ServiceError;
    type Future = future::Ready<Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, target: Target) -> Self::Future {
        futures::future::ok(Service::new(self.api_impl.clone()))
    }
}

fn method_not_allowed() -> Result<Response<Body>, crate::ServiceError> {
    Ok(Response::builder()
        .status(StatusCode::METHOD_NOT_ALLOWED)
        .body(Body::empty())
        .expect("Unable to create Method Not Allowed response"))
}

pub struct Service<T, C>
where
    T: Api<C> + Clone + Send + 'static,
    C: Has<XSpanIdString> + Has<Option<Authorization>> + Send + Sync + 'static,
{
    api_impl: T,
    marker: PhantomData<C>,
}

impl<T, C> Service<T, C>
where
    T: Api<C> + Clone + Send + 'static,
    C: Has<XSpanIdString> + Has<Option<Authorization>> + Send + Sync + 'static,
{
    pub fn new(api_impl: T) -> Self {
        Service {
            api_impl,
            marker: PhantomData,
        }
    }
}

impl<T, C> Clone for Service<T, C>
where
    T: Api<C> + Clone + Send + 'static,
    C: Has<XSpanIdString> + Has<Option<Authorization>> + Send + Sync + 'static,
{
    fn clone(&self) -> Self {
        Service {
            api_impl: self.api_impl.clone(),
            marker: self.marker,
        }
    }
}

impl<T, C> hyper::service::Service<(Request<Body>, C)> for Service<T, C>
where
    T: Api<C> + Clone + Send + Sync + 'static,
    C: Has<XSpanIdString> + Has<Option<Authorization>> + Send + Sync + 'static,
{
    type Response = Response<Body>;
    type Error = crate::ServiceError;
    type Future = ServiceFuture;

    fn poll_ready(&mut self, cx: &mut Context) -> Poll<Result<(), Self::Error>> {
        self.api_impl.poll_ready(cx)
    }

    fn call(&mut self, req: (Request<Body>, C)) -> Self::Future {
        async fn run<T, C>(
            mut api_impl: T,
            req: (Request<Body>, C),
        ) -> Result<Response<Body>, crate::ServiceError>
        where
            T: Api<C> + Clone + Send + 'static,
            C: Has<XSpanIdString> + Has<Option<Authorization>> + Send + Sync + 'static,
        {
            let (request, context) = req;
            let (parts, body) = request.into_parts();
            let (method, uri, headers) = (parts.method, parts.uri, parts.headers);
            let path = paths::GLOBAL_REGEX_SET.matches(uri.path());

            match method {
                // OnScpDomainRoutingInformationChangePost - POST /{$request.body#/callbackUri}
                hyper::Method::POST if path.matched(paths::ID_REQUEST_BODY_CALLBACKURI) => {
                    // Path parameters
                    let path: &str = uri.path();
                    let path_params =
                    paths::REGEX_REQUEST_BODY_CALLBACKURI
                    .captures(path)
                    .unwrap_or_else(||
                        panic!("Path {} matched RE REQUEST_BODY_CALLBACKURI in set but failed match against \"{}\"", path, paths::REGEX_REQUEST_BODY_CALLBACKURI.as_str())
                    );

                    let callback_request_body_callback_uri =
                        path_params["request_body_callback_uri"].to_string();
                    // Header parameters
                    let param_content_encoding =
                        headers.get(HeaderName::from_static("content-encoding"));

                    let param_content_encoding = match param_content_encoding {
                        Some(v) => {
                            match header::IntoHeaderValue::<String>::try_from((*v).clone()) {
                                Ok(result) => Some(result.0),
                                Err(err) => {
                                    return Ok(Response::builder()
                                        .status(StatusCode::BAD_REQUEST)
                                        .body(Body::from(format!("Invalid header Content-Encoding - {}", err)))
                                        .expect("Unable to create Bad Request response for invalid header Content-Encoding"));
                                }
                            }
                        }
                        None => None,
                    };

                    // Body parameters (note that non-required body parameters will ignore garbage
                    // values, rather than causing a 400 response). Produce warning header and logs for
                    // any unused fields.
                    let result = body.into_raw().await;
                    match result {
                            Ok(body) => {
                                let mut unused_elements = Vec::new();
                                let param_scp_domain_routing_info_notification: Option<models::ScpDomainRoutingInfoNotification> = if !body.is_empty() {
                                    let deserializer = &mut serde_json::Deserializer::from_slice(&*body);
                                    match serde_ignored::deserialize(deserializer, |path| {
                                            warn!("Ignoring unknown field in body: {}", path);
                                            unused_elements.push(path.to_string());
                                    }) {
                                        Ok(param_scp_domain_routing_info_notification) => param_scp_domain_routing_info_notification,
                                        Err(_) => None,
                                    }
                                } else {
                                    None
                                };

                                let result = api_impl.on_scp_domain_routing_information_change_post(
                                            callback_request_body_callback_uri,
                                            param_content_encoding,
                                            param_scp_domain_routing_info_notification,
                                        &context
                                    ).await;
                                let mut response = Response::new(Body::empty());
                                response.headers_mut().insert(
                                            HeaderName::from_static("x-span-id"),
                                            HeaderValue::from_str((&context as &dyn Has<XSpanIdString>).get().0.clone().as_str())
                                                .expect("Unable to create X-Span-ID header value"));

                                        if !unused_elements.is_empty() {
                                            response.headers_mut().insert(
                                                HeaderName::from_static("warning"),
                                                HeaderValue::from_str(format!("Ignoring unknown fields in body: {:?}", unused_elements).as_str())
                                                    .expect("Unable to create Warning header value"));
                                        }

                                        match result {
                                            Ok(rsp) => match rsp {
                                                OnScpDomainRoutingInformationChangePostResponse::ExpectedResponseToASuccessfulCallbackProcessing
                                                    {
                                                        accept_encoding
                                                    }
                                                => {
                                                    if let Some(accept_encoding) = accept_encoding {
                                                    let accept_encoding = match header::IntoHeaderValue(accept_encoding).try_into() {
                                                        Ok(val) => val,
                                                        Err(e) => {
                                                            return Ok(Response::builder()
                                                                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                                                                    .body(Body::from(format!("An internal server error occurred handling accept_encoding header - {}", e)))
                                                                    .expect("Unable to create Internal Server Error for invalid response header"))
                                                        }
                                                    };

                                                    response.headers_mut().insert(
                                                        HeaderName::from_static("accept-encoding"),
                                                        accept_encoding
                                                    );
                                                    }
                                                    *response.status_mut() = StatusCode::from_u16(204).expect("Unable to turn 204 into a StatusCode");
                                                },
                                                OnScpDomainRoutingInformationChangePostResponse::BadRequest
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(400).expect("Unable to turn 400 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for ON_SCP_DOMAIN_ROUTING_INFORMATION_CHANGE_POST_BAD_REQUEST"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                OnScpDomainRoutingInformationChangePostResponse::Unauthorized
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(401).expect("Unable to turn 401 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for ON_SCP_DOMAIN_ROUTING_INFORMATION_CHANGE_POST_UNAUTHORIZED"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                OnScpDomainRoutingInformationChangePostResponse::Forbidden
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(403).expect("Unable to turn 403 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for ON_SCP_DOMAIN_ROUTING_INFORMATION_CHANGE_POST_FORBIDDEN"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                OnScpDomainRoutingInformationChangePostResponse::NotFound
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(404).expect("Unable to turn 404 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for ON_SCP_DOMAIN_ROUTING_INFORMATION_CHANGE_POST_NOT_FOUND"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                OnScpDomainRoutingInformationChangePostResponse::LengthRequired
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(411).expect("Unable to turn 411 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for ON_SCP_DOMAIN_ROUTING_INFORMATION_CHANGE_POST_LENGTH_REQUIRED"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                OnScpDomainRoutingInformationChangePostResponse::PayloadTooLarge
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(413).expect("Unable to turn 413 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for ON_SCP_DOMAIN_ROUTING_INFORMATION_CHANGE_POST_PAYLOAD_TOO_LARGE"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                OnScpDomainRoutingInformationChangePostResponse::UnsupportedMediaType
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(415).expect("Unable to turn 415 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for ON_SCP_DOMAIN_ROUTING_INFORMATION_CHANGE_POST_UNSUPPORTED_MEDIA_TYPE"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                OnScpDomainRoutingInformationChangePostResponse::TooManyRequests
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(429).expect("Unable to turn 429 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for ON_SCP_DOMAIN_ROUTING_INFORMATION_CHANGE_POST_TOO_MANY_REQUESTS"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                OnScpDomainRoutingInformationChangePostResponse::InternalServerError
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(500).expect("Unable to turn 500 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for ON_SCP_DOMAIN_ROUTING_INFORMATION_CHANGE_POST_INTERNAL_SERVER_ERROR"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                OnScpDomainRoutingInformationChangePostResponse::NotImplemented
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(501).expect("Unable to turn 501 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for ON_SCP_DOMAIN_ROUTING_INFORMATION_CHANGE_POST_NOT_IMPLEMENTED"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                OnScpDomainRoutingInformationChangePostResponse::ServiceUnavailable
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(503).expect("Unable to turn 503 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for ON_SCP_DOMAIN_ROUTING_INFORMATION_CHANGE_POST_SERVICE_UNAVAILABLE"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                OnScpDomainRoutingInformationChangePostResponse::GenericError
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(0).expect("Unable to turn 0 into a StatusCode");
                                                },
                                            },
                                            Err(_) => {
                                                // Application code returned an error. This should not happen, as the implementation should
                                                // return a valid response.
                                                *response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
                                                *response.body_mut() = Body::from("An internal error occurred");
                                            },
                                        }

                                        Ok(response)
                            },
                            Err(e) => Ok(Response::builder()
                                                .status(StatusCode::BAD_REQUEST)
                                                .body(Body::from(format!("Couldn't read body parameter ScpDomainRoutingInfoNotification: {}", e)))
                                                .expect("Unable to create Bad Request response due to unable to read body parameter ScpDomainRoutingInfoNotification")),
                        }
                }

                _ if path.matched(paths::ID_REQUEST_BODY_CALLBACKURI) => method_not_allowed(),
                _ => Ok(Response::builder()
                    .status(StatusCode::NOT_FOUND)
                    .body(Body::empty())
                    .expect("Unable to create Not Found response")),
            }
        }
        Box::pin(run(self.api_impl.clone(), req))
    }
}

/// Request parser for `Api`.
pub struct ApiRequestParser;
impl<T> RequestParser<T> for ApiRequestParser {
    fn parse_operation_id(request: &Request<T>) -> Option<&'static str> {
        let path = paths::GLOBAL_REGEX_SET.matches(request.uri().path());
        match *request.method() {
            // OnScpDomainRoutingInformationChangePost - POST /{$request.body#/callbackUri}
            hyper::Method::POST if path.matched(paths::ID_REQUEST_BODY_CALLBACKURI) => {
                Some("OnScpDomainRoutingInformationChangePost")
            }
            _ => None,
        }
    }
}
