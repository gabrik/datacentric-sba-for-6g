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

use crate::{
    Api, RetrieveCompleteSearchResponse, RetrieveStoredSearchResponse,
    SCpDomainRoutingInfoGetResponse, ScpDomainRoutingInfoSubscribeResponse,
    ScpDomainRoutingInfoUnsubscribeResponse, SearchNfInstancesResponse,
};

pub mod callbacks;

mod paths {
    use lazy_static::lazy_static;

    lazy_static! {
        pub static ref GLOBAL_REGEX_SET: regex::RegexSet = regex::RegexSet::new(vec![
            r"^/nnrf-disc/v1/nf-instances$",
            r"^/nnrf-disc/v1/scp-domain-routing-info$",
            r"^/nnrf-disc/v1/scp-domain-routing-info-subs$",
            r"^/nnrf-disc/v1/scp-domain-routing-info-subs/(?P<subscriptionID>[^/?#]*)$",
            r"^/nnrf-disc/v1/searches/(?P<searchId>[^/?#]*)$",
            r"^/nnrf-disc/v1/searches/(?P<searchId>[^/?#]*)/complete$"
        ])
        .expect("Unable to create global regex set");
    }
    pub(crate) static ID_NF_INSTANCES: usize = 0;
    pub(crate) static ID_SCP_DOMAIN_ROUTING_INFO: usize = 1;
    pub(crate) static ID_SCP_DOMAIN_ROUTING_INFO_SUBS: usize = 2;
    pub(crate) static ID_SCP_DOMAIN_ROUTING_INFO_SUBS_SUBSCRIPTIONID: usize = 3;
    lazy_static! {
        pub static ref REGEX_SCP_DOMAIN_ROUTING_INFO_SUBS_SUBSCRIPTIONID: regex::Regex =
            #[allow(clippy::invalid_regex)]
            regex::Regex::new(
                r"^/nnrf-disc/v1/scp-domain-routing-info-subs/(?P<subscriptionID>[^/?#]*)$"
            )
            .expect("Unable to create regex for SCP_DOMAIN_ROUTING_INFO_SUBS_SUBSCRIPTIONID");
    }
    pub(crate) static ID_SEARCHES_SEARCHID: usize = 4;
    lazy_static! {
        pub static ref REGEX_SEARCHES_SEARCHID: regex::Regex =
            #[allow(clippy::invalid_regex)]
            regex::Regex::new(r"^/nnrf-disc/v1/searches/(?P<searchId>[^/?#]*)$")
                .expect("Unable to create regex for SEARCHES_SEARCHID");
    }
    pub(crate) static ID_SEARCHES_SEARCHID_COMPLETE: usize = 5;
    lazy_static! {
        pub static ref REGEX_SEARCHES_SEARCHID_COMPLETE: regex::Regex =
            #[allow(clippy::invalid_regex)]
            regex::Regex::new(r"^/nnrf-disc/v1/searches/(?P<searchId>[^/?#]*)/complete$")
                .expect("Unable to create regex for SEARCHES_SEARCHID_COMPLETE");
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
                // RetrieveCompleteSearch - GET /searches/{searchId}/complete
                hyper::Method::GET if path.matched(paths::ID_SEARCHES_SEARCHID_COMPLETE) => {
                    {
                        let authorization = match *(&context as &dyn Has<Option<Authorization>>)
                            .get()
                        {
                            Some(ref authorization) => authorization,
                            None => {
                                return Ok(Response::builder()
                                    .status(StatusCode::FORBIDDEN)
                                    .body(Body::from("Unauthenticated"))
                                    .expect("Unable to create Authentication Forbidden response"))
                            }
                        };

                        // Authorization
                        if let Scopes::Some(ref scopes) = authorization.scopes {
                            let required_scopes: std::collections::BTreeSet<String> = vec![
                                "nnrf-disc".to_string(), // Access to the Nnrf_NFDiscovery API
                            ]
                            .into_iter()
                            .collect();

                            if !required_scopes.is_subset(scopes) {
                                let missing_scopes = required_scopes.difference(scopes);
                                return Ok(Response::builder()
                                    .status(StatusCode::FORBIDDEN)
                                    .body(Body::from(missing_scopes.fold(
                                        "Insufficient authorization, missing scopes".to_string(),
                                        |s, scope| format!("{} {}", s, scope),
                                    )))
                                    .expect(
                                        "Unable to create Authentication Insufficient response",
                                    ));
                            }
                        }
                    }

                    // Path parameters
                    let path: &str = uri.path();
                    let path_params =
                    paths::REGEX_SEARCHES_SEARCHID_COMPLETE
                    .captures(path)
                    .unwrap_or_else(||
                        panic!("Path {} matched RE SEARCHES_SEARCHID_COMPLETE in set but failed match against \"{}\"", path, paths::REGEX_SEARCHES_SEARCHID_COMPLETE.as_str())
                    );

                    let param_search_id = match percent_encoding::percent_decode(path_params["searchId"].as_bytes()).decode_utf8() {
                    Ok(param_search_id) => match param_search_id.parse::<String>() {
                        Ok(param_search_id) => param_search_id,
                        Err(e) => return Ok(Response::builder()
                                        .status(StatusCode::BAD_REQUEST)
                                        .body(Body::from(format!("Couldn't parse path parameter searchId: {}", e)))
                                        .expect("Unable to create Bad Request response for invalid path parameter")),
                    },
                    Err(_) => return Ok(Response::builder()
                                        .status(StatusCode::BAD_REQUEST)
                                        .body(Body::from(format!("Couldn't percent-decode path parameter as UTF-8: {}", &path_params["searchId"])))
                                        .expect("Unable to create Bad Request response for invalid percent decode"))
                };

                    // Header parameters
                    let param_accept_encoding =
                        headers.get(HeaderName::from_static("accept-encoding"));

                    let param_accept_encoding = match param_accept_encoding {
                        Some(v) => {
                            match header::IntoHeaderValue::<String>::try_from((*v).clone()) {
                                Ok(result) => Some(result.0),
                                Err(err) => {
                                    return Ok(Response::builder()
                                        .status(StatusCode::BAD_REQUEST)
                                        .body(Body::from(format!("Invalid header Accept-Encoding - {}", err)))
                                        .expect("Unable to create Bad Request response for invalid header Accept-Encoding"));
                                }
                            }
                        }
                        None => None,
                    };

                    let result = api_impl
                        .retrieve_complete_search(param_search_id, param_accept_encoding, &context)
                        .await;
                    let mut response = Response::new(Body::empty());
                    response.headers_mut().insert(
                        HeaderName::from_static("x-span-id"),
                        HeaderValue::from_str(
                            (&context as &dyn Has<XSpanIdString>)
                                .get()
                                .0
                                .clone()
                                .as_str(),
                        )
                        .expect("Unable to create X-Span-ID header value"),
                    );

                    match result {
                        Ok(rsp) => match rsp {
                            RetrieveCompleteSearchResponse::ExpectedResponseToAValidRequest {
                                body,
                                cache_control,
                                e_tag,
                                content_encoding,
                            } => {
                                if let Some(cache_control) = cache_control {
                                    let cache_control = match header::IntoHeaderValue(cache_control).try_into() {
                                                        Ok(val) => val,
                                                        Err(e) => {
                                                            return Ok(Response::builder()
                                                                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                                                                    .body(Body::from(format!("An internal server error occurred handling cache_control header - {}", e)))
                                                                    .expect("Unable to create Internal Server Error for invalid response header"))
                                                        }
                                                    };

                                    response.headers_mut().insert(
                                        HeaderName::from_static("cache-control"),
                                        cache_control,
                                    );
                                }
                                if let Some(e_tag) = e_tag {
                                    let e_tag = match header::IntoHeaderValue(e_tag).try_into() {
                                                        Ok(val) => val,
                                                        Err(e) => {
                                                            return Ok(Response::builder()
                                                                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                                                                    .body(Body::from(format!("An internal server error occurred handling e_tag header - {}", e)))
                                                                    .expect("Unable to create Internal Server Error for invalid response header"))
                                                        }
                                                    };

                                    response
                                        .headers_mut()
                                        .insert(HeaderName::from_static("etag"), e_tag);
                                }
                                if let Some(content_encoding) = content_encoding {
                                    let content_encoding = match header::IntoHeaderValue(content_encoding).try_into() {
                                                        Ok(val) => val,
                                                        Err(e) => {
                                                            return Ok(Response::builder()
                                                                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                                                                    .body(Body::from(format!("An internal server error occurred handling content_encoding header - {}", e)))
                                                                    .expect("Unable to create Internal Server Error for invalid response header"))
                                                        }
                                                    };

                                    response.headers_mut().insert(
                                        HeaderName::from_static("content-encoding"),
                                        content_encoding,
                                    );
                                }
                                *response.status_mut() = StatusCode::from_u16(200)
                                    .expect("Unable to turn 200 into a StatusCode");
                                response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for RETRIEVE_COMPLETE_SEARCH_EXPECTED_RESPONSE_TO_A_VALID_REQUEST"));
                                let body = serde_json::to_string(&body)
                                    .expect("impossible to fail to serialize");
                                *response.body_mut() = Body::from(body);
                            }
                            RetrieveCompleteSearchResponse::TemporaryRedirect {
                                body,
                                location,
                            } => {
                                let location = match header::IntoHeaderValue(location).try_into() {
                                                        Ok(val) => val,
                                                        Err(e) => {
                                                            return Ok(Response::builder()
                                                                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                                                                    .body(Body::from(format!("An internal server error occurred handling location header - {}", e)))
                                                                    .expect("Unable to create Internal Server Error for invalid response header"))
                                                        }
                                                    };

                                response
                                    .headers_mut()
                                    .insert(HeaderName::from_static("location"), location);
                                *response.status_mut() = StatusCode::from_u16(307)
                                    .expect("Unable to turn 307 into a StatusCode");
                                response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for RETRIEVE_COMPLETE_SEARCH_TEMPORARY_REDIRECT"));
                                let body = serde_json::to_string(&body)
                                    .expect("impossible to fail to serialize");
                                *response.body_mut() = Body::from(body);
                            }
                            RetrieveCompleteSearchResponse::PermanentRedirect {
                                body,
                                location,
                            } => {
                                let location = match header::IntoHeaderValue(location).try_into() {
                                                        Ok(val) => val,
                                                        Err(e) => {
                                                            return Ok(Response::builder()
                                                                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                                                                    .body(Body::from(format!("An internal server error occurred handling location header - {}", e)))
                                                                    .expect("Unable to create Internal Server Error for invalid response header"))
                                                        }
                                                    };

                                response
                                    .headers_mut()
                                    .insert(HeaderName::from_static("location"), location);
                                *response.status_mut() = StatusCode::from_u16(308)
                                    .expect("Unable to turn 308 into a StatusCode");
                                response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for RETRIEVE_COMPLETE_SEARCH_PERMANENT_REDIRECT"));
                                let body = serde_json::to_string(&body)
                                    .expect("impossible to fail to serialize");
                                *response.body_mut() = Body::from(body);
                            }
                        },
                        Err(_) => {
                            // Application code returned an error. This should not happen, as the implementation should
                            // return a valid response.
                            *response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
                            *response.body_mut() = Body::from("An internal error occurred");
                        }
                    }

                    Ok(response)
                }

                // ScpDomainRoutingInfoUnsubscribe - DELETE /scp-domain-routing-info-subs/{subscriptionID}
                hyper::Method::DELETE
                    if path.matched(paths::ID_SCP_DOMAIN_ROUTING_INFO_SUBS_SUBSCRIPTIONID) =>
                {
                    {
                        let authorization = match *(&context as &dyn Has<Option<Authorization>>)
                            .get()
                        {
                            Some(ref authorization) => authorization,
                            None => {
                                return Ok(Response::builder()
                                    .status(StatusCode::FORBIDDEN)
                                    .body(Body::from("Unauthenticated"))
                                    .expect("Unable to create Authentication Forbidden response"))
                            }
                        };

                        // Authorization
                        if let Scopes::Some(ref scopes) = authorization.scopes {
                            let required_scopes: std::collections::BTreeSet<String> = vec![
                                "nnrf-disc".to_string(), // Access to the Nnrf_NFDiscovery API
                            ]
                            .into_iter()
                            .collect();

                            if !required_scopes.is_subset(scopes) {
                                let missing_scopes = required_scopes.difference(scopes);
                                return Ok(Response::builder()
                                    .status(StatusCode::FORBIDDEN)
                                    .body(Body::from(missing_scopes.fold(
                                        "Insufficient authorization, missing scopes".to_string(),
                                        |s, scope| format!("{} {}", s, scope),
                                    )))
                                    .expect(
                                        "Unable to create Authentication Insufficient response",
                                    ));
                            }
                        }
                    }

                    // Path parameters
                    let path: &str = uri.path();
                    let path_params =
                    paths::REGEX_SCP_DOMAIN_ROUTING_INFO_SUBS_SUBSCRIPTIONID
                    .captures(path)
                    .unwrap_or_else(||
                        panic!("Path {} matched RE SCP_DOMAIN_ROUTING_INFO_SUBS_SUBSCRIPTIONID in set but failed match against \"{}\"", path, paths::REGEX_SCP_DOMAIN_ROUTING_INFO_SUBS_SUBSCRIPTIONID.as_str())
                    );

                    let param_subscription_id = match percent_encoding::percent_decode(path_params["subscriptionID"].as_bytes()).decode_utf8() {
                    Ok(param_subscription_id) => match param_subscription_id.parse::<String>() {
                        Ok(param_subscription_id) => param_subscription_id,
                        Err(e) => return Ok(Response::builder()
                                        .status(StatusCode::BAD_REQUEST)
                                        .body(Body::from(format!("Couldn't parse path parameter subscriptionID: {}", e)))
                                        .expect("Unable to create Bad Request response for invalid path parameter")),
                    },
                    Err(_) => return Ok(Response::builder()
                                        .status(StatusCode::BAD_REQUEST)
                                        .body(Body::from(format!("Couldn't percent-decode path parameter as UTF-8: {}", &path_params["subscriptionID"])))
                                        .expect("Unable to create Bad Request response for invalid percent decode"))
                };

                    let result = api_impl
                        .scp_domain_routing_info_unsubscribe(param_subscription_id, &context)
                        .await;
                    let mut response = Response::new(Body::empty());
                    response.headers_mut().insert(
                        HeaderName::from_static("x-span-id"),
                        HeaderValue::from_str(
                            (&context as &dyn Has<XSpanIdString>)
                                .get()
                                .0
                                .clone()
                                .as_str(),
                        )
                        .expect("Unable to create X-Span-ID header value"),
                    );

                    match result {
                                            Ok(rsp) => match rsp {
                                                ScpDomainRoutingInfoUnsubscribeResponse::ExpectedResponseToASuccessfulSubscriptionRemoval
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(204).expect("Unable to turn 204 into a StatusCode");
                                                },
                                                ScpDomainRoutingInfoUnsubscribeResponse::BadRequest
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(400).expect("Unable to turn 400 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for SCP_DOMAIN_ROUTING_INFO_UNSUBSCRIBE_BAD_REQUEST"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                ScpDomainRoutingInfoUnsubscribeResponse::Unauthorized
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(401).expect("Unable to turn 401 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for SCP_DOMAIN_ROUTING_INFO_UNSUBSCRIBE_UNAUTHORIZED"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                ScpDomainRoutingInfoUnsubscribeResponse::Forbidden
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(403).expect("Unable to turn 403 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for SCP_DOMAIN_ROUTING_INFO_UNSUBSCRIBE_FORBIDDEN"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                ScpDomainRoutingInfoUnsubscribeResponse::NotFound
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(404).expect("Unable to turn 404 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for SCP_DOMAIN_ROUTING_INFO_UNSUBSCRIBE_NOT_FOUND"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                ScpDomainRoutingInfoUnsubscribeResponse::LengthRequired
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(411).expect("Unable to turn 411 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for SCP_DOMAIN_ROUTING_INFO_UNSUBSCRIBE_LENGTH_REQUIRED"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                ScpDomainRoutingInfoUnsubscribeResponse::PayloadTooLarge
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(413).expect("Unable to turn 413 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for SCP_DOMAIN_ROUTING_INFO_UNSUBSCRIBE_PAYLOAD_TOO_LARGE"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                ScpDomainRoutingInfoUnsubscribeResponse::UnsupportedMediaType
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(415).expect("Unable to turn 415 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for SCP_DOMAIN_ROUTING_INFO_UNSUBSCRIBE_UNSUPPORTED_MEDIA_TYPE"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                ScpDomainRoutingInfoUnsubscribeResponse::TooManyRequests
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(429).expect("Unable to turn 429 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for SCP_DOMAIN_ROUTING_INFO_UNSUBSCRIBE_TOO_MANY_REQUESTS"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                ScpDomainRoutingInfoUnsubscribeResponse::InternalServerError
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(500).expect("Unable to turn 500 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for SCP_DOMAIN_ROUTING_INFO_UNSUBSCRIBE_INTERNAL_SERVER_ERROR"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                ScpDomainRoutingInfoUnsubscribeResponse::NotImplemented
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(501).expect("Unable to turn 501 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for SCP_DOMAIN_ROUTING_INFO_UNSUBSCRIBE_NOT_IMPLEMENTED"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                ScpDomainRoutingInfoUnsubscribeResponse::ServiceUnavailable
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(503).expect("Unable to turn 503 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for SCP_DOMAIN_ROUTING_INFO_UNSUBSCRIBE_SERVICE_UNAVAILABLE"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                ScpDomainRoutingInfoUnsubscribeResponse::GenericError
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
                }

                // SearchNfInstances - GET /nf-instances
                hyper::Method::GET if path.matched(paths::ID_NF_INSTANCES) => {
                    {
                        // Method called for service discovery.

                        // reply to a SFM looking for UDM
                        // done to uri: /nnrf-disc/v1/nf-instances?requester-features=20&requester-nf-type=SMF&service-names=nudm-sdm&target-nf-type=UDM
                        // {
                        //     "validityPeriod":	3600,
                        //     "nfInstances":	[{
                        //             "nfInstanceId":	"65a52dac-b832-41ed-ba6e-53c1c0b3ed51",
                        //             "nfType":	"UDM",
                        //             "nfStatus":	"REGISTERED",
                        //             "heartBeatTimer":	10,
                        //             "ipv4Addresses":	["172.22.0.13"],
                        //             "allowedNfTypes":	["AMF", "SMF", "AUSF", "SCP"],
                        //             "priority":	0,
                        //             "capacity":	100,
                        //             "load":	0,
                        //             "nfServiceList":	{
                        //                 "65a54148-b832-41ed-ba6e-53c1c0b3ed51":	{
                        //                     "serviceInstanceId":	"65a54148-b832-41ed-ba6e-53c1c0b3ed51",
                        //                     "serviceName":	"nudm-sdm",
                        //                     "versions":	[{
                        //                             "apiVersionInUri":	"v2",
                        //                             "apiFullVersion":	"2.0.0"
                        //                         }],
                        //                     "scheme":	"http",
                        //                     "nfServiceStatus":	"REGISTERED",
                        //                     "ipEndPoints":	[{
                        //                             "ipv4Address":	"172.22.0.13",
                        //                             "port":	7777
                        //                         }],
                        //                     "allowedNfTypes":	["AMF", "SMF"],
                        //                     "priority":	0,
                        //                     "capacity":	100,
                        //                     "load":	0
                        //                 }
                        //             },
                        //             "nfProfileChangesSupportInd":	true
                        //         }]
                        // }

                        log::info!("Received GET /nf-instances");
                        let authorization = match *(&context as &dyn Has<Option<Authorization>>)
                            .get()
                        {
                            Some(ref authorization) => authorization,
                            None => {
                                return Ok(Response::builder()
                                    .status(StatusCode::FORBIDDEN)
                                    .body(Body::from("Unauthenticated"))
                                    .expect("Unable to create Authentication Forbidden response"))
                            }
                        };

                        // Authorization
                        if let Scopes::Some(ref scopes) = authorization.scopes {
                            let required_scopes: std::collections::BTreeSet<String> = vec![
                                "nnrf-disc".to_string(), // Access to the Nnrf_NFDiscovery API
                            ]
                            .into_iter()
                            .collect();

                            if !required_scopes.is_subset(scopes) {
                                let missing_scopes = required_scopes.difference(scopes);
                                return Ok(Response::builder()
                                    .status(StatusCode::FORBIDDEN)
                                    .body(Body::from(missing_scopes.fold(
                                        "Insufficient authorization, missing scopes".to_string(),
                                        |s, scope| format!("{} {}", s, scope),
                                    )))
                                    .expect(
                                        "Unable to create Authentication Insufficient response",
                                    ));
                            }
                        }
                    }

                    // Header parameters
                    let param_accept_encoding =
                        headers.get(HeaderName::from_static("accept-encoding"));

                    let param_accept_encoding = match param_accept_encoding {
                        Some(v) => {
                            match header::IntoHeaderValue::<String>::try_from((*v).clone()) {
                                Ok(result) => Some(result.0),
                                Err(err) => {
                                    log::info!("param_accept_encoding: {param_accept_encoding:?} ");
                                    return Ok(Response::builder()
                                        .status(StatusCode::BAD_REQUEST)
                                        .body(Body::from(format!("Invalid header Accept-Encoding - {}", err)))
                                        .expect("Unable to create Bad Request response for invalid header Accept-Encoding"));
                                }
                            }
                        }
                        None => None,
                    };
                    let param_if_none_match = headers.get(HeaderName::from_static("if-none-match"));

                    let param_if_none_match = match param_if_none_match {
                        Some(v) => {
                            match header::IntoHeaderValue::<String>::try_from((*v).clone()) {
                                Ok(result) => Some(result.0),
                                Err(err) => {
                                    return Ok(Response::builder()
                                        .status(StatusCode::BAD_REQUEST)
                                        .body(Body::from(format!("Invalid header If-None-Match - {}", err)))
                                        .expect("Unable to create Bad Request response for invalid header If-None-Match"));
                                }
                            }
                        }
                        None => None,
                    };

                    // Query parameters (note that non-required or collection query parameters will ignore garbage values, rather than causing a 400 response)
                    let query_params =
                        form_urlencoded::parse(uri.query().unwrap_or_default().as_bytes())
                            .collect::<Vec<_>>();
                    let param_target_nf_type = query_params
                        .iter()
                        .filter(|e| e.0 == "target-nf-type")
                        .map(|e| e.1.to_owned())
                        .next();
                    let param_target_nf_type = match param_target_nf_type {
                        Some(param_target_nf_type) => {
                            let param_target_nf_type =
                                <models::NfType as std::str::FromStr>::from_str(
                                    &param_target_nf_type,
                                );
                            match param_target_nf_type {
                            Ok(param_target_nf_type) => Some(param_target_nf_type),
                            Err(e) => return Ok(Response::builder()
                                .status(StatusCode::BAD_REQUEST)
                                .body(Body::from(format!("Couldn't parse query parameter target-nf-type - doesn't match schema: {}", e)))
                                .expect("Unable to create Bad Request response for invalid query parameter target-nf-type")),
                        }
                        }
                        None => None,
                    };
                    let param_target_nf_type = match param_target_nf_type {
                    Some(param_target_nf_type) => param_target_nf_type,
                    None => return Ok(Response::builder()
                        .status(StatusCode::BAD_REQUEST)
                        .body(Body::from("Missing required query parameter target-nf-type"))
                        .expect("Unable to create Bad Request response for missing query parameter target-nf-type")),
                };
                    let param_requester_nf_type = query_params
                        .iter()
                        .filter(|e| e.0 == "requester-nf-type")
                        .map(|e| e.1.to_owned())
                        .next();
                    let param_requester_nf_type = match param_requester_nf_type {
                        Some(param_requester_nf_type) => {
                            let param_requester_nf_type =
                                <models::NfType as std::str::FromStr>::from_str(
                                    &param_requester_nf_type,
                                );
                            match param_requester_nf_type {
                            Ok(param_requester_nf_type) => Some(param_requester_nf_type),
                            Err(e) => return Ok(Response::builder()
                                .status(StatusCode::BAD_REQUEST)
                                .body(Body::from(format!("Couldn't parse query parameter requester-nf-type - doesn't match schema: {}", e)))
                                .expect("Unable to create Bad Request response for invalid query parameter requester-nf-type")),
                        }
                        }
                        None => None,
                    };
                    let param_requester_nf_type = match param_requester_nf_type {
                    Some(param_requester_nf_type) => param_requester_nf_type,
                    None => return Ok(Response::builder()
                        .status(StatusCode::BAD_REQUEST)
                        .body(Body::from("Missing required query parameter requester-nf-type"))
                        .expect("Unable to create Bad Request response for missing query parameter requester-nf-type")),
                };
                    let param_preferred_collocated_nf_types = query_params
                        .iter()
                        .filter(|e| e.0 == "preferred-collocated-nf-types")
                        .map(|e| e.1.to_owned())
                        .filter_map(|param_preferred_collocated_nf_types| {
                            param_preferred_collocated_nf_types.parse().ok()
                        })
                        .collect::<Vec<_>>();
                    let param_preferred_collocated_nf_types =
                        if !param_preferred_collocated_nf_types.is_empty() {
                            Some(param_preferred_collocated_nf_types)
                        } else {
                            None
                        };
                    let param_requester_nf_instance_id = query_params
                        .iter()
                        .filter(|e| e.0 == "requester-nf-instance-id")
                        .map(|e| e.1.to_owned())
                        .next();
                    let param_requester_nf_instance_id = match param_requester_nf_instance_id {
                        Some(param_requester_nf_instance_id) => {
                            let param_requester_nf_instance_id =
                                <uuid::Uuid as std::str::FromStr>::from_str(
                                    &param_requester_nf_instance_id,
                                );
                            match param_requester_nf_instance_id {
                            Ok(param_requester_nf_instance_id) => Some(param_requester_nf_instance_id),
                            Err(e) => return Ok(Response::builder()
                                .status(StatusCode::BAD_REQUEST)
                                .body(Body::from(format!("Couldn't parse query parameter requester-nf-instance-id - doesn't match schema: {}", e)))
                                .expect("Unable to create Bad Request response for invalid query parameter requester-nf-instance-id")),
                        }
                        }
                        None => None,
                    };
                    let param_service_names = query_params
                        .iter()
                        .filter(|e| e.0 == "service-names")
                        .map(|e| e.1.to_owned())
                        .filter_map(|param_service_names| param_service_names.parse().ok())
                        .collect::<Vec<_>>();
                    let param_service_names = if !param_service_names.is_empty() {
                        Some(param_service_names)
                    } else {
                        None
                    };
                    let param_requester_nf_instance_fqdn = query_params
                        .iter()
                        .filter(|e| e.0 == "requester-nf-instance-fqdn")
                        .map(|e| e.1.to_owned())
                        .next();
                    let param_requester_nf_instance_fqdn = match param_requester_nf_instance_fqdn {
                        Some(param_requester_nf_instance_fqdn) => {
                            let param_requester_nf_instance_fqdn =
                                <String as std::str::FromStr>::from_str(
                                    &param_requester_nf_instance_fqdn,
                                );
                            match param_requester_nf_instance_fqdn {
                            Ok(param_requester_nf_instance_fqdn) => Some(param_requester_nf_instance_fqdn),
                            Err(e) => return Ok(Response::builder()
                                .status(StatusCode::BAD_REQUEST)
                                .body(Body::from(format!("Couldn't parse query parameter requester-nf-instance-fqdn - doesn't match schema: {}", e)))
                                .expect("Unable to create Bad Request response for invalid query parameter requester-nf-instance-fqdn")),
                        }
                        }
                        None => None,
                    };
                    let param_target_plmn_list = query_params
                        .iter()
                        .filter(|e| e.0 == "target-plmn-list")
                        .map(|e| e.1.to_owned())
                        .next();
                    let param_target_plmn_list = match param_target_plmn_list {
                        Some(param_target_plmn_list) => {
                            let param_target_plmn_list = serde_json::from_str::<Vec<models::PlmnId>>(
                                &param_target_plmn_list,
                            );
                            match param_target_plmn_list {
                            Ok(param_target_plmn_list) => Some(param_target_plmn_list),
                            Err(e) => return Ok(Response::builder()
                                .status(StatusCode::BAD_REQUEST)
                                .body(Body::from(format!("Couldn't parse query parameter target-plmn-list - doesn't match schema: {}", e)))
                                .expect("Unable to create Bad Request response for invalid query parameter target-plmn-list")),
                        }
                        }
                        None => None,
                    };
                    let param_requester_plmn_list = query_params
                        .iter()
                        .filter(|e| e.0 == "requester-plmn-list")
                        .map(|e| e.1.to_owned())
                        .next();
                    let param_requester_plmn_list = match param_requester_plmn_list {
                        Some(param_requester_plmn_list) => {
                            let param_requester_plmn_list =
                                serde_json::from_str::<Vec<models::PlmnId>>(
                                    &param_requester_plmn_list,
                                );
                            match param_requester_plmn_list {
                            Ok(param_requester_plmn_list) => Some(param_requester_plmn_list),
                            Err(e) => return Ok(Response::builder()
                                .status(StatusCode::BAD_REQUEST)
                                .body(Body::from(format!("Couldn't parse query parameter requester-plmn-list - doesn't match schema: {}", e)))
                                .expect("Unable to create Bad Request response for invalid query parameter requester-plmn-list")),
                        }
                        }
                        None => None,
                    };
                    let param_target_nf_instance_id = query_params
                        .iter()
                        .filter(|e| e.0 == "target-nf-instance-id")
                        .map(|e| e.1.to_owned())
                        .next();
                    let param_target_nf_instance_id = match param_target_nf_instance_id {
                        Some(param_target_nf_instance_id) => {
                            let param_target_nf_instance_id =
                                <uuid::Uuid as std::str::FromStr>::from_str(
                                    &param_target_nf_instance_id,
                                );
                            match param_target_nf_instance_id {
                            Ok(param_target_nf_instance_id) => Some(param_target_nf_instance_id),
                            Err(e) => return Ok(Response::builder()
                                .status(StatusCode::BAD_REQUEST)
                                .body(Body::from(format!("Couldn't parse query parameter target-nf-instance-id - doesn't match schema: {}", e)))
                                .expect("Unable to create Bad Request response for invalid query parameter target-nf-instance-id")),
                        }
                        }
                        None => None,
                    };
                    let param_target_nf_fqdn = query_params
                        .iter()
                        .filter(|e| e.0 == "target-nf-fqdn")
                        .map(|e| e.1.to_owned())
                        .next();
                    let param_target_nf_fqdn = match param_target_nf_fqdn {
                        Some(param_target_nf_fqdn) => {
                            let param_target_nf_fqdn =
                                <String as std::str::FromStr>::from_str(&param_target_nf_fqdn);
                            match param_target_nf_fqdn {
                            Ok(param_target_nf_fqdn) => Some(param_target_nf_fqdn),
                            Err(e) => return Ok(Response::builder()
                                .status(StatusCode::BAD_REQUEST)
                                .body(Body::from(format!("Couldn't parse query parameter target-nf-fqdn - doesn't match schema: {}", e)))
                                .expect("Unable to create Bad Request response for invalid query parameter target-nf-fqdn")),
                        }
                        }
                        None => None,
                    };
                    let param_hnrf_uri = query_params
                        .iter()
                        .filter(|e| e.0 == "hnrf-uri")
                        .map(|e| e.1.to_owned())
                        .next();
                    let param_hnrf_uri = match param_hnrf_uri {
                        Some(param_hnrf_uri) => {
                            let param_hnrf_uri =
                                <String as std::str::FromStr>::from_str(&param_hnrf_uri);
                            match param_hnrf_uri {
                            Ok(param_hnrf_uri) => Some(param_hnrf_uri),
                            Err(e) => return Ok(Response::builder()
                                .status(StatusCode::BAD_REQUEST)
                                .body(Body::from(format!("Couldn't parse query parameter hnrf-uri - doesn't match schema: {}", e)))
                                .expect("Unable to create Bad Request response for invalid query parameter hnrf-uri")),
                        }
                        }
                        None => None,
                    };
                    let param_snssais = query_params
                        .iter()
                        .filter(|e| e.0 == "snssais")
                        .map(|e| e.1.to_owned())
                        .next();
                    let param_snssais = match param_snssais {
                        Some(param_snssais) => {
                            let param_snssais =
                                serde_json::from_str::<Vec<models::Snssai>>(&param_snssais);
                            match param_snssais {
                            Ok(param_snssais) => Some(param_snssais),
                            Err(e) => return Ok(Response::builder()
                                .status(StatusCode::BAD_REQUEST)
                                .body(Body::from(format!("Couldn't parse query parameter snssais - doesn't match schema: {}", e)))
                                .expect("Unable to create Bad Request response for invalid query parameter snssais")),
                        }
                        }
                        None => None,
                    };
                    let param_requester_snssais = query_params
                        .iter()
                        .filter(|e| e.0 == "requester-snssais")
                        .map(|e| e.1.to_owned())
                        .next();
                    let param_requester_snssais = match param_requester_snssais {
                        Some(param_requester_snssais) => {
                            let param_requester_snssais =
                                serde_json::from_str::<Vec<models::ExtSnssai>>(
                                    &param_requester_snssais,
                                );
                            match param_requester_snssais {
                            Ok(param_requester_snssais) => Some(param_requester_snssais),
                            Err(e) => return Ok(Response::builder()
                                .status(StatusCode::BAD_REQUEST)
                                .body(Body::from(format!("Couldn't parse query parameter requester-snssais - doesn't match schema: {}", e)))
                                .expect("Unable to create Bad Request response for invalid query parameter requester-snssais")),
                        }
                        }
                        None => None,
                    };
                    let param_plmn_specific_snssai_list = query_params
                        .iter()
                        .filter(|e| e.0 == "plmn-specific-snssai-list")
                        .map(|e| e.1.to_owned())
                        .next();
                    let param_plmn_specific_snssai_list = match param_plmn_specific_snssai_list {
                        Some(param_plmn_specific_snssai_list) => {
                            let param_plmn_specific_snssai_list =
                                serde_json::from_str::<Vec<models::PlmnSnssai>>(
                                    &param_plmn_specific_snssai_list,
                                );
                            match param_plmn_specific_snssai_list {
                            Ok(param_plmn_specific_snssai_list) => Some(param_plmn_specific_snssai_list),
                            Err(e) => return Ok(Response::builder()
                                .status(StatusCode::BAD_REQUEST)
                                .body(Body::from(format!("Couldn't parse query parameter plmn-specific-snssai-list - doesn't match schema: {}", e)))
                                .expect("Unable to create Bad Request response for invalid query parameter plmn-specific-snssai-list")),
                        }
                        }
                        None => None,
                    };
                    let param_requester_plmn_specific_snssai_list = query_params
                        .iter()
                        .filter(|e| e.0 == "requester-plmn-specific-snssai-list")
                        .map(|e| e.1.to_owned())
                        .next();
                    let param_requester_plmn_specific_snssai_list =
                        match param_requester_plmn_specific_snssai_list {
                            Some(param_requester_plmn_specific_snssai_list) => {
                                let param_requester_plmn_specific_snssai_list =
                                    serde_json::from_str::<Vec<models::PlmnSnssai>>(
                                        &param_requester_plmn_specific_snssai_list,
                                    );
                                match param_requester_plmn_specific_snssai_list {
                            Ok(param_requester_plmn_specific_snssai_list) => Some(param_requester_plmn_specific_snssai_list),
                            Err(e) => return Ok(Response::builder()
                                .status(StatusCode::BAD_REQUEST)
                                .body(Body::from(format!("Couldn't parse query parameter requester-plmn-specific-snssai-list - doesn't match schema: {}", e)))
                                .expect("Unable to create Bad Request response for invalid query parameter requester-plmn-specific-snssai-list")),
                        }
                            }
                            None => None,
                        };
                    let param_dnn = query_params
                        .iter()
                        .filter(|e| e.0 == "dnn")
                        .map(|e| e.1.to_owned())
                        .next();
                    let param_dnn = match param_dnn {
                        Some(param_dnn) => {
                            let param_dnn = <String as std::str::FromStr>::from_str(&param_dnn);
                            match param_dnn {
                            Ok(param_dnn) => Some(param_dnn),
                            Err(e) => return Ok(Response::builder()
                                .status(StatusCode::BAD_REQUEST)
                                .body(Body::from(format!("Couldn't parse query parameter dnn - doesn't match schema: {}", e)))
                                .expect("Unable to create Bad Request response for invalid query parameter dnn")),
                        }
                        }
                        None => None,
                    };
                    let param_ipv4_index = query_params
                        .iter()
                        .filter(|e| e.0 == "ipv4-index")
                        .map(|e| e.1.to_owned())
                        .next();
                    let param_ipv4_index = match param_ipv4_index {
                        Some(param_ipv4_index) => {
                            let param_ipv4_index =
                                serde_json::from_str::<models::IpIndex>(&param_ipv4_index);
                            match param_ipv4_index {
                            Ok(param_ipv4_index) => Some(param_ipv4_index),
                            Err(e) => return Ok(Response::builder()
                                .status(StatusCode::BAD_REQUEST)
                                .body(Body::from(format!("Couldn't parse query parameter ipv4-index - doesn't match schema: {}", e)))
                                .expect("Unable to create Bad Request response for invalid query parameter ipv4-index")),
                        }
                        }
                        None => None,
                    };
                    let param_ipv6_index = query_params
                        .iter()
                        .filter(|e| e.0 == "ipv6-index")
                        .map(|e| e.1.to_owned())
                        .next();
                    let param_ipv6_index = match param_ipv6_index {
                        Some(param_ipv6_index) => {
                            let param_ipv6_index =
                                serde_json::from_str::<models::IpIndex>(&param_ipv6_index);
                            match param_ipv6_index {
                            Ok(param_ipv6_index) => Some(param_ipv6_index),
                            Err(e) => return Ok(Response::builder()
                                .status(StatusCode::BAD_REQUEST)
                                .body(Body::from(format!("Couldn't parse query parameter ipv6-index - doesn't match schema: {}", e)))
                                .expect("Unable to create Bad Request response for invalid query parameter ipv6-index")),
                        }
                        }
                        None => None,
                    };
                    let param_nsi_list = query_params
                        .iter()
                        .filter(|e| e.0 == "nsi-list")
                        .map(|e| e.1.to_owned())
                        .filter_map(|param_nsi_list| param_nsi_list.parse().ok())
                        .collect::<Vec<_>>();
                    let param_nsi_list = if !param_nsi_list.is_empty() {
                        Some(param_nsi_list)
                    } else {
                        None
                    };
                    let param_smf_serving_area = query_params
                        .iter()
                        .filter(|e| e.0 == "smf-serving-area")
                        .map(|e| e.1.to_owned())
                        .next();
                    let param_smf_serving_area = match param_smf_serving_area {
                        Some(param_smf_serving_area) => {
                            let param_smf_serving_area =
                                <String as std::str::FromStr>::from_str(&param_smf_serving_area);
                            match param_smf_serving_area {
                            Ok(param_smf_serving_area) => Some(param_smf_serving_area),
                            Err(e) => return Ok(Response::builder()
                                .status(StatusCode::BAD_REQUEST)
                                .body(Body::from(format!("Couldn't parse query parameter smf-serving-area - doesn't match schema: {}", e)))
                                .expect("Unable to create Bad Request response for invalid query parameter smf-serving-area")),
                        }
                        }
                        None => None,
                    };
                    let param_mbsmf_serving_area = query_params
                        .iter()
                        .filter(|e| e.0 == "mbsmf-serving-area")
                        .map(|e| e.1.to_owned())
                        .next();
                    let param_mbsmf_serving_area = match param_mbsmf_serving_area {
                        Some(param_mbsmf_serving_area) => {
                            let param_mbsmf_serving_area =
                                <String as std::str::FromStr>::from_str(&param_mbsmf_serving_area);
                            match param_mbsmf_serving_area {
                            Ok(param_mbsmf_serving_area) => Some(param_mbsmf_serving_area),
                            Err(e) => return Ok(Response::builder()
                                .status(StatusCode::BAD_REQUEST)
                                .body(Body::from(format!("Couldn't parse query parameter mbsmf-serving-area - doesn't match schema: {}", e)))
                                .expect("Unable to create Bad Request response for invalid query parameter mbsmf-serving-area")),
                        }
                        }
                        None => None,
                    };
                    let param_tai = query_params
                        .iter()
                        .filter(|e| e.0 == "tai")
                        .map(|e| e.1.to_owned())
                        .next();
                    let param_tai = match param_tai {
                        Some(param_tai) => {
                            let param_tai = serde_json::from_str::<models::Tai>(&param_tai);
                            match param_tai {
                            Ok(param_tai) => Some(param_tai),
                            Err(e) => return Ok(Response::builder()
                                .status(StatusCode::BAD_REQUEST)
                                .body(Body::from(format!("Couldn't parse query parameter tai - doesn't match schema: {}", e)))
                                .expect("Unable to create Bad Request response for invalid query parameter tai")),
                        }
                        }
                        None => None,
                    };
                    let param_amf_region_id = query_params
                        .iter()
                        .filter(|e| e.0 == "amf-region-id")
                        .map(|e| e.1.to_owned())
                        .next();
                    let param_amf_region_id = match param_amf_region_id {
                        Some(param_amf_region_id) => {
                            let param_amf_region_id =
                                <String as std::str::FromStr>::from_str(&param_amf_region_id);
                            match param_amf_region_id {
                            Ok(param_amf_region_id) => Some(param_amf_region_id),
                            Err(e) => return Ok(Response::builder()
                                .status(StatusCode::BAD_REQUEST)
                                .body(Body::from(format!("Couldn't parse query parameter amf-region-id - doesn't match schema: {}", e)))
                                .expect("Unable to create Bad Request response for invalid query parameter amf-region-id")),
                        }
                        }
                        None => None,
                    };
                    let param_amf_set_id = query_params
                        .iter()
                        .filter(|e| e.0 == "amf-set-id")
                        .map(|e| e.1.to_owned())
                        .next();
                    let param_amf_set_id = match param_amf_set_id {
                        Some(param_amf_set_id) => {
                            let param_amf_set_id =
                                <String as std::str::FromStr>::from_str(&param_amf_set_id);
                            match param_amf_set_id {
                            Ok(param_amf_set_id) => Some(param_amf_set_id),
                            Err(e) => return Ok(Response::builder()
                                .status(StatusCode::BAD_REQUEST)
                                .body(Body::from(format!("Couldn't parse query parameter amf-set-id - doesn't match schema: {}", e)))
                                .expect("Unable to create Bad Request response for invalid query parameter amf-set-id")),
                        }
                        }
                        None => None,
                    };
                    let param_guami = query_params
                        .iter()
                        .filter(|e| e.0 == "guami")
                        .map(|e| e.1.to_owned())
                        .next();
                    let param_guami = match param_guami {
                        Some(param_guami) => {
                            let param_guami = serde_json::from_str::<models::Guami>(&param_guami);
                            match param_guami {
                            Ok(param_guami) => Some(param_guami),
                            Err(e) => return Ok(Response::builder()
                                .status(StatusCode::BAD_REQUEST)
                                .body(Body::from(format!("Couldn't parse query parameter guami - doesn't match schema: {}", e)))
                                .expect("Unable to create Bad Request response for invalid query parameter guami")),
                        }
                        }
                        None => None,
                    };
                    let param_supi = query_params
                        .iter()
                        .filter(|e| e.0 == "supi")
                        .map(|e| e.1.to_owned())
                        .next();
                    let param_supi = match param_supi {
                        Some(param_supi) => {
                            let param_supi = <String as std::str::FromStr>::from_str(&param_supi);
                            match param_supi {
                            Ok(param_supi) => Some(param_supi),
                            Err(e) => return Ok(Response::builder()
                                .status(StatusCode::BAD_REQUEST)
                                .body(Body::from(format!("Couldn't parse query parameter supi - doesn't match schema: {}", e)))
                                .expect("Unable to create Bad Request response for invalid query parameter supi")),
                        }
                        }
                        None => None,
                    };
                    let param_ue_ipv4_address = query_params
                        .iter()
                        .filter(|e| e.0 == "ue-ipv4-address")
                        .map(|e| e.1.to_owned())
                        .next();
                    let param_ue_ipv4_address = match param_ue_ipv4_address {
                        Some(param_ue_ipv4_address) => {
                            let param_ue_ipv4_address =
                                <String as std::str::FromStr>::from_str(&param_ue_ipv4_address);
                            match param_ue_ipv4_address {
                            Ok(param_ue_ipv4_address) => Some(param_ue_ipv4_address),
                            Err(e) => return Ok(Response::builder()
                                .status(StatusCode::BAD_REQUEST)
                                .body(Body::from(format!("Couldn't parse query parameter ue-ipv4-address - doesn't match schema: {}", e)))
                                .expect("Unable to create Bad Request response for invalid query parameter ue-ipv4-address")),
                        }
                        }
                        None => None,
                    };
                    let param_ip_domain = query_params
                        .iter()
                        .filter(|e| e.0 == "ip-domain")
                        .map(|e| e.1.to_owned())
                        .next();
                    let param_ip_domain = match param_ip_domain {
                        Some(param_ip_domain) => {
                            let param_ip_domain =
                                <String as std::str::FromStr>::from_str(&param_ip_domain);
                            match param_ip_domain {
                            Ok(param_ip_domain) => Some(param_ip_domain),
                            Err(e) => return Ok(Response::builder()
                                .status(StatusCode::BAD_REQUEST)
                                .body(Body::from(format!("Couldn't parse query parameter ip-domain - doesn't match schema: {}", e)))
                                .expect("Unable to create Bad Request response for invalid query parameter ip-domain")),
                        }
                        }
                        None => None,
                    };
                    let param_ue_ipv6_prefix = query_params
                        .iter()
                        .filter(|e| e.0 == "ue-ipv6-prefix")
                        .map(|e| e.1.to_owned())
                        .next();
                    let param_ue_ipv6_prefix = match param_ue_ipv6_prefix {
                        Some(param_ue_ipv6_prefix) => {
                            let param_ue_ipv6_prefix =
                                <models::Ipv6Prefix as std::str::FromStr>::from_str(
                                    &param_ue_ipv6_prefix,
                                );
                            match param_ue_ipv6_prefix {
                            Ok(param_ue_ipv6_prefix) => Some(param_ue_ipv6_prefix),
                            Err(e) => return Ok(Response::builder()
                                .status(StatusCode::BAD_REQUEST)
                                .body(Body::from(format!("Couldn't parse query parameter ue-ipv6-prefix - doesn't match schema: {}", e)))
                                .expect("Unable to create Bad Request response for invalid query parameter ue-ipv6-prefix")),
                        }
                        }
                        None => None,
                    };
                    let param_pgw_ind = query_params
                        .iter()
                        .filter(|e| e.0 == "pgw-ind")
                        .map(|e| e.1.to_owned())
                        .next();
                    let param_pgw_ind = match param_pgw_ind {
                        Some(param_pgw_ind) => {
                            let param_pgw_ind =
                                <bool as std::str::FromStr>::from_str(&param_pgw_ind);
                            match param_pgw_ind {
                            Ok(param_pgw_ind) => Some(param_pgw_ind),
                            Err(e) => return Ok(Response::builder()
                                .status(StatusCode::BAD_REQUEST)
                                .body(Body::from(format!("Couldn't parse query parameter pgw-ind - doesn't match schema: {}", e)))
                                .expect("Unable to create Bad Request response for invalid query parameter pgw-ind")),
                        }
                        }
                        None => None,
                    };
                    let param_preferred_pgw_ind = query_params
                        .iter()
                        .filter(|e| e.0 == "preferred-pgw-ind")
                        .map(|e| e.1.to_owned())
                        .next();
                    let param_preferred_pgw_ind = match param_preferred_pgw_ind {
                        Some(param_preferred_pgw_ind) => {
                            let param_preferred_pgw_ind =
                                <bool as std::str::FromStr>::from_str(&param_preferred_pgw_ind);
                            match param_preferred_pgw_ind {
                            Ok(param_preferred_pgw_ind) => Some(param_preferred_pgw_ind),
                            Err(e) => return Ok(Response::builder()
                                .status(StatusCode::BAD_REQUEST)
                                .body(Body::from(format!("Couldn't parse query parameter preferred-pgw-ind - doesn't match schema: {}", e)))
                                .expect("Unable to create Bad Request response for invalid query parameter preferred-pgw-ind")),
                        }
                        }
                        None => None,
                    };
                    let param_pgw = query_params
                        .iter()
                        .filter(|e| e.0 == "pgw")
                        .map(|e| e.1.to_owned())
                        .next();
                    let param_pgw = match param_pgw {
                        Some(param_pgw) => {
                            let param_pgw = <String as std::str::FromStr>::from_str(&param_pgw);
                            match param_pgw {
                            Ok(param_pgw) => Some(param_pgw),
                            Err(e) => return Ok(Response::builder()
                                .status(StatusCode::BAD_REQUEST)
                                .body(Body::from(format!("Couldn't parse query parameter pgw - doesn't match schema: {}", e)))
                                .expect("Unable to create Bad Request response for invalid query parameter pgw")),
                        }
                        }
                        None => None,
                    };
                    let param_pgw_ip = query_params
                        .iter()
                        .filter(|e| e.0 == "pgw-ip")
                        .map(|e| e.1.to_owned())
                        .next();
                    let param_pgw_ip = match param_pgw_ip {
                        Some(param_pgw_ip) => {
                            let param_pgw_ip =
                                serde_json::from_str::<models::IpAddr>(&param_pgw_ip);
                            match param_pgw_ip {
                            Ok(param_pgw_ip) => Some(param_pgw_ip),
                            Err(e) => return Ok(Response::builder()
                                .status(StatusCode::BAD_REQUEST)
                                .body(Body::from(format!("Couldn't parse query parameter pgw-ip - doesn't match schema: {}", e)))
                                .expect("Unable to create Bad Request response for invalid query parameter pgw-ip")),
                        }
                        }
                        None => None,
                    };
                    let param_gpsi = query_params
                        .iter()
                        .filter(|e| e.0 == "gpsi")
                        .map(|e| e.1.to_owned())
                        .next();
                    let param_gpsi = match param_gpsi {
                        Some(param_gpsi) => {
                            let param_gpsi = <String as std::str::FromStr>::from_str(&param_gpsi);
                            match param_gpsi {
                            Ok(param_gpsi) => Some(param_gpsi),
                            Err(e) => return Ok(Response::builder()
                                .status(StatusCode::BAD_REQUEST)
                                .body(Body::from(format!("Couldn't parse query parameter gpsi - doesn't match schema: {}", e)))
                                .expect("Unable to create Bad Request response for invalid query parameter gpsi")),
                        }
                        }
                        None => None,
                    };
                    let param_external_group_identity = query_params
                        .iter()
                        .filter(|e| e.0 == "external-group-identity")
                        .map(|e| e.1.to_owned())
                        .next();
                    let param_external_group_identity = match param_external_group_identity {
                        Some(param_external_group_identity) => {
                            let param_external_group_identity =
                                <String as std::str::FromStr>::from_str(
                                    &param_external_group_identity,
                                );
                            match param_external_group_identity {
                            Ok(param_external_group_identity) => Some(param_external_group_identity),
                            Err(e) => return Ok(Response::builder()
                                .status(StatusCode::BAD_REQUEST)
                                .body(Body::from(format!("Couldn't parse query parameter external-group-identity - doesn't match schema: {}", e)))
                                .expect("Unable to create Bad Request response for invalid query parameter external-group-identity")),
                        }
                        }
                        None => None,
                    };
                    let param_internal_group_identity = query_params
                        .iter()
                        .filter(|e| e.0 == "internal-group-identity")
                        .map(|e| e.1.to_owned())
                        .next();
                    let param_internal_group_identity = match param_internal_group_identity {
                        Some(param_internal_group_identity) => {
                            let param_internal_group_identity =
                                <String as std::str::FromStr>::from_str(
                                    &param_internal_group_identity,
                                );
                            match param_internal_group_identity {
                            Ok(param_internal_group_identity) => Some(param_internal_group_identity),
                            Err(e) => return Ok(Response::builder()
                                .status(StatusCode::BAD_REQUEST)
                                .body(Body::from(format!("Couldn't parse query parameter internal-group-identity - doesn't match schema: {}", e)))
                                .expect("Unable to create Bad Request response for invalid query parameter internal-group-identity")),
                        }
                        }
                        None => None,
                    };
                    let param_pfd_data = query_params
                        .iter()
                        .filter(|e| e.0 == "pfd-data")
                        .map(|e| e.1.to_owned())
                        .next();
                    let param_pfd_data = match param_pfd_data {
                        Some(param_pfd_data) => {
                            let param_pfd_data =
                                serde_json::from_str::<models::PfdData>(&param_pfd_data);
                            match param_pfd_data {
                            Ok(param_pfd_data) => Some(param_pfd_data),
                            Err(e) => return Ok(Response::builder()
                                .status(StatusCode::BAD_REQUEST)
                                .body(Body::from(format!("Couldn't parse query parameter pfd-data - doesn't match schema: {}", e)))
                                .expect("Unable to create Bad Request response for invalid query parameter pfd-data")),
                        }
                        }
                        None => None,
                    };
                    let param_data_set = query_params
                        .iter()
                        .filter(|e| e.0 == "data-set")
                        .map(|e| e.1.to_owned())
                        .next();
                    let param_data_set = match param_data_set {
                        Some(param_data_set) => {
                            let param_data_set =
                                <models::DataSetId as std::str::FromStr>::from_str(&param_data_set);
                            match param_data_set {
                            Ok(param_data_set) => Some(param_data_set),
                            Err(e) => return Ok(Response::builder()
                                .status(StatusCode::BAD_REQUEST)
                                .body(Body::from(format!("Couldn't parse query parameter data-set - doesn't match schema: {}", e)))
                                .expect("Unable to create Bad Request response for invalid query parameter data-set")),
                        }
                        }
                        None => None,
                    };
                    let param_routing_indicator = query_params
                        .iter()
                        .filter(|e| e.0 == "routing-indicator")
                        .map(|e| e.1.to_owned())
                        .next();
                    let param_routing_indicator = match param_routing_indicator {
                        Some(param_routing_indicator) => {
                            let param_routing_indicator =
                                <String as std::str::FromStr>::from_str(&param_routing_indicator);
                            match param_routing_indicator {
                            Ok(param_routing_indicator) => Some(param_routing_indicator),
                            Err(e) => return Ok(Response::builder()
                                .status(StatusCode::BAD_REQUEST)
                                .body(Body::from(format!("Couldn't parse query parameter routing-indicator - doesn't match schema: {}", e)))
                                .expect("Unable to create Bad Request response for invalid query parameter routing-indicator")),
                        }
                        }
                        None => None,
                    };
                    let param_group_id_list = query_params
                        .iter()
                        .filter(|e| e.0 == "group-id-list")
                        .map(|e| e.1.to_owned())
                        .filter_map(|param_group_id_list| param_group_id_list.parse().ok())
                        .collect::<Vec<_>>();
                    let param_group_id_list = if !param_group_id_list.is_empty() {
                        Some(param_group_id_list)
                    } else {
                        None
                    };
                    let param_dnai_list = query_params
                        .iter()
                        .filter(|e| e.0 == "dnai-list")
                        .map(|e| e.1.to_owned())
                        .filter_map(|param_dnai_list| param_dnai_list.parse().ok())
                        .collect::<Vec<_>>();
                    let param_dnai_list = if !param_dnai_list.is_empty() {
                        Some(param_dnai_list)
                    } else {
                        None
                    };
                    let param_pdu_session_types = query_params
                        .iter()
                        .filter(|e| e.0 == "pdu-session-types")
                        .map(|e| e.1.to_owned())
                        .filter_map(|param_pdu_session_types| param_pdu_session_types.parse().ok())
                        .collect::<Vec<_>>();
                    let param_pdu_session_types = if !param_pdu_session_types.is_empty() {
                        Some(param_pdu_session_types)
                    } else {
                        None
                    };
                    let param_event_id_list = query_params
                        .iter()
                        .filter(|e| e.0 == "event-id-list")
                        .map(|e| e.1.to_owned())
                        .filter_map(|param_event_id_list| param_event_id_list.parse().ok())
                        .collect::<Vec<_>>();
                    let param_event_id_list = if !param_event_id_list.is_empty() {
                        Some(param_event_id_list)
                    } else {
                        None
                    };
                    let param_nwdaf_event_list = query_params
                        .iter()
                        .filter(|e| e.0 == "nwdaf-event-list")
                        .map(|e| e.1.to_owned())
                        .filter_map(|param_nwdaf_event_list| param_nwdaf_event_list.parse().ok())
                        .collect::<Vec<_>>();
                    let param_nwdaf_event_list = if !param_nwdaf_event_list.is_empty() {
                        Some(param_nwdaf_event_list)
                    } else {
                        None
                    };
                    let param_supported_features = query_params
                        .iter()
                        .filter(|e| e.0 == "supported-features")
                        .map(|e| e.1.to_owned())
                        .next();
                    let param_supported_features = match param_supported_features {
                        Some(param_supported_features) => {
                            let param_supported_features =
                                <String as std::str::FromStr>::from_str(&param_supported_features);
                            match param_supported_features {
                            Ok(param_supported_features) => Some(param_supported_features),
                            Err(e) => return Ok(Response::builder()
                                .status(StatusCode::BAD_REQUEST)
                                .body(Body::from(format!("Couldn't parse query parameter supported-features - doesn't match schema: {}", e)))
                                .expect("Unable to create Bad Request response for invalid query parameter supported-features")),
                        }
                        }
                        None => None,
                    };
                    let param_upf_iwk_eps_ind = query_params
                        .iter()
                        .filter(|e| e.0 == "upf-iwk-eps-ind")
                        .map(|e| e.1.to_owned())
                        .next();
                    let param_upf_iwk_eps_ind = match param_upf_iwk_eps_ind {
                        Some(param_upf_iwk_eps_ind) => {
                            let param_upf_iwk_eps_ind =
                                <bool as std::str::FromStr>::from_str(&param_upf_iwk_eps_ind);
                            match param_upf_iwk_eps_ind {
                            Ok(param_upf_iwk_eps_ind) => Some(param_upf_iwk_eps_ind),
                            Err(e) => return Ok(Response::builder()
                                .status(StatusCode::BAD_REQUEST)
                                .body(Body::from(format!("Couldn't parse query parameter upf-iwk-eps-ind - doesn't match schema: {}", e)))
                                .expect("Unable to create Bad Request response for invalid query parameter upf-iwk-eps-ind")),
                        }
                        }
                        None => None,
                    };
                    let param_chf_supported_plmn = query_params
                        .iter()
                        .filter(|e| e.0 == "chf-supported-plmn")
                        .map(|e| e.1.to_owned())
                        .next();
                    let param_chf_supported_plmn = match param_chf_supported_plmn {
                        Some(param_chf_supported_plmn) => {
                            let param_chf_supported_plmn =
                                serde_json::from_str::<models::PlmnId>(&param_chf_supported_plmn);
                            match param_chf_supported_plmn {
                            Ok(param_chf_supported_plmn) => Some(param_chf_supported_plmn),
                            Err(e) => return Ok(Response::builder()
                                .status(StatusCode::BAD_REQUEST)
                                .body(Body::from(format!("Couldn't parse query parameter chf-supported-plmn - doesn't match schema: {}", e)))
                                .expect("Unable to create Bad Request response for invalid query parameter chf-supported-plmn")),
                        }
                        }
                        None => None,
                    };
                    let param_preferred_locality = query_params
                        .iter()
                        .filter(|e| e.0 == "preferred-locality")
                        .map(|e| e.1.to_owned())
                        .next();
                    let param_preferred_locality = match param_preferred_locality {
                        Some(param_preferred_locality) => {
                            let param_preferred_locality =
                                <String as std::str::FromStr>::from_str(&param_preferred_locality);
                            match param_preferred_locality {
                            Ok(param_preferred_locality) => Some(param_preferred_locality),
                            Err(e) => return Ok(Response::builder()
                                .status(StatusCode::BAD_REQUEST)
                                .body(Body::from(format!("Couldn't parse query parameter preferred-locality - doesn't match schema: {}", e)))
                                .expect("Unable to create Bad Request response for invalid query parameter preferred-locality")),
                        }
                        }
                        None => None,
                    };
                    let param_access_type = query_params
                        .iter()
                        .filter(|e| e.0 == "access-type")
                        .map(|e| e.1.to_owned())
                        .next();
                    let param_access_type = match param_access_type {
                        Some(param_access_type) => {
                            let param_access_type =
                                <models::AccessType as std::str::FromStr>::from_str(
                                    &param_access_type,
                                );
                            match param_access_type {
                            Ok(param_access_type) => Some(param_access_type),
                            Err(e) => return Ok(Response::builder()
                                .status(StatusCode::BAD_REQUEST)
                                .body(Body::from(format!("Couldn't parse query parameter access-type - doesn't match schema: {}", e)))
                                .expect("Unable to create Bad Request response for invalid query parameter access-type")),
                        }
                        }
                        None => None,
                    };
                    let param_limit = query_params
                        .iter()
                        .filter(|e| e.0 == "limit")
                        .map(|e| e.1.to_owned())
                        .next();
                    let param_limit = match param_limit {
                        Some(param_limit) => {
                            let param_limit = <i32 as std::str::FromStr>::from_str(&param_limit);
                            match param_limit {
                            Ok(param_limit) => Some(param_limit),
                            Err(e) => return Ok(Response::builder()
                                .status(StatusCode::BAD_REQUEST)
                                .body(Body::from(format!("Couldn't parse query parameter limit - doesn't match schema: {}", e)))
                                .expect("Unable to create Bad Request response for invalid query parameter limit")),
                        }
                        }
                        None => None,
                    };
                    let param_required_features = query_params
                        .iter()
                        .filter(|e| e.0 == "required-features")
                        .map(|e| e.1.to_owned())
                        .filter_map(|param_required_features| param_required_features.parse().ok())
                        .collect::<Vec<_>>();
                    let param_required_features = if !param_required_features.is_empty() {
                        Some(param_required_features)
                    } else {
                        None
                    };
                    let param_complex_query = query_params
                        .iter()
                        .filter(|e| e.0 == "complex-query")
                        .map(|e| e.1.to_owned())
                        .next();
                    let param_complex_query = match param_complex_query {
                        Some(param_complex_query) => {
                            let param_complex_query =
                                serde_json::from_str::<models::ComplexQuery>(&param_complex_query);
                            match param_complex_query {
                            Ok(param_complex_query) => Some(param_complex_query),
                            Err(e) => return Ok(Response::builder()
                                .status(StatusCode::BAD_REQUEST)
                                .body(Body::from(format!("Couldn't parse query parameter complex-query - doesn't match schema: {}", e)))
                                .expect("Unable to create Bad Request response for invalid query parameter complex-query")),
                        }
                        }
                        None => None,
                    };
                    let param_max_payload_size = query_params
                        .iter()
                        .filter(|e| e.0 == "max-payload-size")
                        .map(|e| e.1.to_owned())
                        .next();
                    let param_max_payload_size = match param_max_payload_size {
                        Some(param_max_payload_size) => {
                            let param_max_payload_size =
                                <i32 as std::str::FromStr>::from_str(&param_max_payload_size);
                            match param_max_payload_size {
                            Ok(param_max_payload_size) => Some(param_max_payload_size),
                            Err(e) => return Ok(Response::builder()
                                .status(StatusCode::BAD_REQUEST)
                                .body(Body::from(format!("Couldn't parse query parameter max-payload-size - doesn't match schema: {}", e)))
                                .expect("Unable to create Bad Request response for invalid query parameter max-payload-size")),
                        }
                        }
                        None => None,
                    };
                    let param_max_payload_size_ext = query_params
                        .iter()
                        .filter(|e| e.0 == "max-payload-size-ext")
                        .map(|e| e.1.to_owned())
                        .next();
                    let param_max_payload_size_ext = match param_max_payload_size_ext {
                        Some(param_max_payload_size_ext) => {
                            let param_max_payload_size_ext =
                                <i32 as std::str::FromStr>::from_str(&param_max_payload_size_ext);
                            match param_max_payload_size_ext {
                            Ok(param_max_payload_size_ext) => Some(param_max_payload_size_ext),
                            Err(e) => return Ok(Response::builder()
                                .status(StatusCode::BAD_REQUEST)
                                .body(Body::from(format!("Couldn't parse query parameter max-payload-size-ext - doesn't match schema: {}", e)))
                                .expect("Unable to create Bad Request response for invalid query parameter max-payload-size-ext")),
                        }
                        }
                        None => None,
                    };
                    let param_atsss_capability = query_params
                        .iter()
                        .filter(|e| e.0 == "atsss-capability")
                        .map(|e| e.1.to_owned())
                        .next();
                    let param_atsss_capability = match param_atsss_capability {
                        Some(param_atsss_capability) => {
                            let param_atsss_capability =
                                serde_json::from_str::<models::AtsssCapability>(
                                    &param_atsss_capability,
                                );
                            match param_atsss_capability {
                            Ok(param_atsss_capability) => Some(param_atsss_capability),
                            Err(e) => return Ok(Response::builder()
                                .status(StatusCode::BAD_REQUEST)
                                .body(Body::from(format!("Couldn't parse query parameter atsss-capability - doesn't match schema: {}", e)))
                                .expect("Unable to create Bad Request response for invalid query parameter atsss-capability")),
                        }
                        }
                        None => None,
                    };
                    let param_upf_ue_ip_addr_ind = query_params
                        .iter()
                        .filter(|e| e.0 == "upf-ue-ip-addr-ind")
                        .map(|e| e.1.to_owned())
                        .next();
                    let param_upf_ue_ip_addr_ind = match param_upf_ue_ip_addr_ind {
                        Some(param_upf_ue_ip_addr_ind) => {
                            let param_upf_ue_ip_addr_ind =
                                <bool as std::str::FromStr>::from_str(&param_upf_ue_ip_addr_ind);
                            match param_upf_ue_ip_addr_ind {
                            Ok(param_upf_ue_ip_addr_ind) => Some(param_upf_ue_ip_addr_ind),
                            Err(e) => return Ok(Response::builder()
                                .status(StatusCode::BAD_REQUEST)
                                .body(Body::from(format!("Couldn't parse query parameter upf-ue-ip-addr-ind - doesn't match schema: {}", e)))
                                .expect("Unable to create Bad Request response for invalid query parameter upf-ue-ip-addr-ind")),
                        }
                        }
                        None => None,
                    };
                    let param_client_type = query_params
                        .iter()
                        .filter(|e| e.0 == "client-type")
                        .map(|e| e.1.to_owned())
                        .next();
                    let param_client_type = match param_client_type {
                        Some(param_client_type) => {
                            let param_client_type =
                                serde_json::from_str::<models::ExternalClientType>(
                                    &param_client_type,
                                );
                            match param_client_type {
                            Ok(param_client_type) => Some(param_client_type),
                            Err(e) => return Ok(Response::builder()
                                .status(StatusCode::BAD_REQUEST)
                                .body(Body::from(format!("Couldn't parse query parameter client-type - doesn't match schema: {}", e)))
                                .expect("Unable to create Bad Request response for invalid query parameter client-type")),
                        }
                        }
                        None => None,
                    };
                    let param_lmf_id = query_params
                        .iter()
                        .filter(|e| e.0 == "lmf-id")
                        .map(|e| e.1.to_owned())
                        .next();
                    let param_lmf_id = match param_lmf_id {
                        Some(param_lmf_id) => {
                            let param_lmf_id = serde_json::from_str::<String>(&param_lmf_id);
                            match param_lmf_id {
                            Ok(param_lmf_id) => Some(param_lmf_id),
                            Err(e) => return Ok(Response::builder()
                                .status(StatusCode::BAD_REQUEST)
                                .body(Body::from(format!("Couldn't parse query parameter lmf-id - doesn't match schema: {}", e)))
                                .expect("Unable to create Bad Request response for invalid query parameter lmf-id")),
                        }
                        }
                        None => None,
                    };
                    let param_an_node_type = query_params
                        .iter()
                        .filter(|e| e.0 == "an-node-type")
                        .map(|e| e.1.to_owned())
                        .next();
                    let param_an_node_type = match param_an_node_type {
                        Some(param_an_node_type) => {
                            let param_an_node_type =
                                serde_json::from_str::<models::AnNodeType>(&param_an_node_type);
                            match param_an_node_type {
                            Ok(param_an_node_type) => Some(param_an_node_type),
                            Err(e) => return Ok(Response::builder()
                                .status(StatusCode::BAD_REQUEST)
                                .body(Body::from(format!("Couldn't parse query parameter an-node-type - doesn't match schema: {}", e)))
                                .expect("Unable to create Bad Request response for invalid query parameter an-node-type")),
                        }
                        }
                        None => None,
                    };
                    let param_rat_type = query_params
                        .iter()
                        .filter(|e| e.0 == "rat-type")
                        .map(|e| e.1.to_owned())
                        .next();
                    let param_rat_type = match param_rat_type {
                        Some(param_rat_type) => {
                            let param_rat_type =
                                serde_json::from_str::<models::RatType>(&param_rat_type);
                            match param_rat_type {
                            Ok(param_rat_type) => Some(param_rat_type),
                            Err(e) => return Ok(Response::builder()
                                .status(StatusCode::BAD_REQUEST)
                                .body(Body::from(format!("Couldn't parse query parameter rat-type - doesn't match schema: {}", e)))
                                .expect("Unable to create Bad Request response for invalid query parameter rat-type")),
                        }
                        }
                        None => None,
                    };
                    let param_preferred_tai = query_params
                        .iter()
                        .filter(|e| e.0 == "preferred-tai")
                        .map(|e| e.1.to_owned())
                        .next();
                    let param_preferred_tai = match param_preferred_tai {
                        Some(param_preferred_tai) => {
                            let param_preferred_tai =
                                serde_json::from_str::<models::Tai>(&param_preferred_tai);
                            match param_preferred_tai {
                            Ok(param_preferred_tai) => Some(param_preferred_tai),
                            Err(e) => return Ok(Response::builder()
                                .status(StatusCode::BAD_REQUEST)
                                .body(Body::from(format!("Couldn't parse query parameter preferred-tai - doesn't match schema: {}", e)))
                                .expect("Unable to create Bad Request response for invalid query parameter preferred-tai")),
                        }
                        }
                        None => None,
                    };
                    let param_preferred_nf_instances = query_params
                        .iter()
                        .filter(|e| e.0 == "preferred-nf-instances")
                        .map(|e| e.1.to_owned())
                        .filter_map(|param_preferred_nf_instances| {
                            param_preferred_nf_instances.parse().ok()
                        })
                        .collect::<Vec<_>>();
                    let param_preferred_nf_instances = if !param_preferred_nf_instances.is_empty() {
                        Some(param_preferred_nf_instances)
                    } else {
                        None
                    };
                    let param_target_snpn = query_params
                        .iter()
                        .filter(|e| e.0 == "target-snpn")
                        .map(|e| e.1.to_owned())
                        .next();
                    let param_target_snpn = match param_target_snpn {
                        Some(param_target_snpn) => {
                            let param_target_snpn =
                                serde_json::from_str::<models::PlmnIdNid>(&param_target_snpn);
                            match param_target_snpn {
                            Ok(param_target_snpn) => Some(param_target_snpn),
                            Err(e) => return Ok(Response::builder()
                                .status(StatusCode::BAD_REQUEST)
                                .body(Body::from(format!("Couldn't parse query parameter target-snpn - doesn't match schema: {}", e)))
                                .expect("Unable to create Bad Request response for invalid query parameter target-snpn")),
                        }
                        }
                        None => None,
                    };
                    let param_requester_snpn_list = query_params
                        .iter()
                        .filter(|e| e.0 == "requester-snpn-list")
                        .map(|e| e.1.to_owned())
                        .next();
                    let param_requester_snpn_list = match param_requester_snpn_list {
                        Some(param_requester_snpn_list) => {
                            let param_requester_snpn_list =
                                serde_json::from_str::<Vec<models::PlmnIdNid>>(
                                    &param_requester_snpn_list,
                                );
                            match param_requester_snpn_list {
                            Ok(param_requester_snpn_list) => Some(param_requester_snpn_list),
                            Err(e) => return Ok(Response::builder()
                                .status(StatusCode::BAD_REQUEST)
                                .body(Body::from(format!("Couldn't parse query parameter requester-snpn-list - doesn't match schema: {}", e)))
                                .expect("Unable to create Bad Request response for invalid query parameter requester-snpn-list")),
                        }
                        }
                        None => None,
                    };
                    let param_af_ee_data = query_params
                        .iter()
                        .filter(|e| e.0 == "af-ee-data")
                        .map(|e| e.1.to_owned())
                        .next();
                    let param_af_ee_data = match param_af_ee_data {
                        Some(param_af_ee_data) => {
                            let param_af_ee_data = serde_json::from_str::<
                                models::AfEventExposureData,
                            >(&param_af_ee_data);
                            match param_af_ee_data {
                            Ok(param_af_ee_data) => Some(param_af_ee_data),
                            Err(e) => return Ok(Response::builder()
                                .status(StatusCode::BAD_REQUEST)
                                .body(Body::from(format!("Couldn't parse query parameter af-ee-data - doesn't match schema: {}", e)))
                                .expect("Unable to create Bad Request response for invalid query parameter af-ee-data")),
                        }
                        }
                        None => None,
                    };
                    let param_w_agf_info = query_params
                        .iter()
                        .filter(|e| e.0 == "w-agf-info")
                        .map(|e| e.1.to_owned())
                        .next();
                    let param_w_agf_info = match param_w_agf_info {
                        Some(param_w_agf_info) => {
                            let param_w_agf_info =
                                serde_json::from_str::<models::WAgfInfo>(&param_w_agf_info);
                            match param_w_agf_info {
                            Ok(param_w_agf_info) => Some(param_w_agf_info),
                            Err(e) => return Ok(Response::builder()
                                .status(StatusCode::BAD_REQUEST)
                                .body(Body::from(format!("Couldn't parse query parameter w-agf-info - doesn't match schema: {}", e)))
                                .expect("Unable to create Bad Request response for invalid query parameter w-agf-info")),
                        }
                        }
                        None => None,
                    };
                    let param_tngf_info = query_params
                        .iter()
                        .filter(|e| e.0 == "tngf-info")
                        .map(|e| e.1.to_owned())
                        .next();
                    let param_tngf_info = match param_tngf_info {
                        Some(param_tngf_info) => {
                            let param_tngf_info =
                                serde_json::from_str::<models::TngfInfo>(&param_tngf_info);
                            match param_tngf_info {
                            Ok(param_tngf_info) => Some(param_tngf_info),
                            Err(e) => return Ok(Response::builder()
                                .status(StatusCode::BAD_REQUEST)
                                .body(Body::from(format!("Couldn't parse query parameter tngf-info - doesn't match schema: {}", e)))
                                .expect("Unable to create Bad Request response for invalid query parameter tngf-info")),
                        }
                        }
                        None => None,
                    };
                    let param_twif_info = query_params
                        .iter()
                        .filter(|e| e.0 == "twif-info")
                        .map(|e| e.1.to_owned())
                        .next();
                    let param_twif_info = match param_twif_info {
                        Some(param_twif_info) => {
                            let param_twif_info =
                                serde_json::from_str::<models::TwifInfo>(&param_twif_info);
                            match param_twif_info {
                            Ok(param_twif_info) => Some(param_twif_info),
                            Err(e) => return Ok(Response::builder()
                                .status(StatusCode::BAD_REQUEST)
                                .body(Body::from(format!("Couldn't parse query parameter twif-info - doesn't match schema: {}", e)))
                                .expect("Unable to create Bad Request response for invalid query parameter twif-info")),
                        }
                        }
                        None => None,
                    };
                    let param_target_nf_set_id = query_params
                        .iter()
                        .filter(|e| e.0 == "target-nf-set-id")
                        .map(|e| e.1.to_owned())
                        .next();
                    let param_target_nf_set_id = match param_target_nf_set_id {
                        Some(param_target_nf_set_id) => {
                            let param_target_nf_set_id =
                                <String as std::str::FromStr>::from_str(&param_target_nf_set_id);
                            match param_target_nf_set_id {
                            Ok(param_target_nf_set_id) => Some(param_target_nf_set_id),
                            Err(e) => return Ok(Response::builder()
                                .status(StatusCode::BAD_REQUEST)
                                .body(Body::from(format!("Couldn't parse query parameter target-nf-set-id - doesn't match schema: {}", e)))
                                .expect("Unable to create Bad Request response for invalid query parameter target-nf-set-id")),
                        }
                        }
                        None => None,
                    };
                    let param_target_nf_service_set_id = query_params
                        .iter()
                        .filter(|e| e.0 == "target-nf-service-set-id")
                        .map(|e| e.1.to_owned())
                        .next();
                    let param_target_nf_service_set_id = match param_target_nf_service_set_id {
                        Some(param_target_nf_service_set_id) => {
                            let param_target_nf_service_set_id =
                                <String as std::str::FromStr>::from_str(
                                    &param_target_nf_service_set_id,
                                );
                            match param_target_nf_service_set_id {
                            Ok(param_target_nf_service_set_id) => Some(param_target_nf_service_set_id),
                            Err(e) => return Ok(Response::builder()
                                .status(StatusCode::BAD_REQUEST)
                                .body(Body::from(format!("Couldn't parse query parameter target-nf-service-set-id - doesn't match schema: {}", e)))
                                .expect("Unable to create Bad Request response for invalid query parameter target-nf-service-set-id")),
                        }
                        }
                        None => None,
                    };
                    let param_nef_id = query_params
                        .iter()
                        .filter(|e| e.0 == "nef-id")
                        .map(|e| e.1.to_owned())
                        .next();
                    let param_nef_id = match param_nef_id {
                        Some(param_nef_id) => {
                            let param_nef_id =
                                <String as std::str::FromStr>::from_str(&param_nef_id);
                            match param_nef_id {
                            Ok(param_nef_id) => Some(param_nef_id),
                            Err(e) => return Ok(Response::builder()
                                .status(StatusCode::BAD_REQUEST)
                                .body(Body::from(format!("Couldn't parse query parameter nef-id - doesn't match schema: {}", e)))
                                .expect("Unable to create Bad Request response for invalid query parameter nef-id")),
                        }
                        }
                        None => None,
                    };
                    let param_notification_type = query_params
                        .iter()
                        .filter(|e| e.0 == "notification-type")
                        .map(|e| e.1.to_owned())
                        .next();
                    let param_notification_type = match param_notification_type {
                        Some(param_notification_type) => {
                            let param_notification_type =
                                <models::NotificationType as std::str::FromStr>::from_str(
                                    &param_notification_type,
                                );
                            match param_notification_type {
                            Ok(param_notification_type) => Some(param_notification_type),
                            Err(e) => return Ok(Response::builder()
                                .status(StatusCode::BAD_REQUEST)
                                .body(Body::from(format!("Couldn't parse query parameter notification-type - doesn't match schema: {}", e)))
                                .expect("Unable to create Bad Request response for invalid query parameter notification-type")),
                        }
                        }
                        None => None,
                    };
                    let param_n1_msg_class = query_params
                        .iter()
                        .filter(|e| e.0 == "n1-msg-class")
                        .map(|e| e.1.to_owned())
                        .next();
                    let param_n1_msg_class = match param_n1_msg_class {
                        Some(param_n1_msg_class) => {
                            let param_n1_msg_class =
                                <models::N1MessageClass as std::str::FromStr>::from_str(
                                    &param_n1_msg_class,
                                );
                            match param_n1_msg_class {
                            Ok(param_n1_msg_class) => Some(param_n1_msg_class),
                            Err(e) => return Ok(Response::builder()
                                .status(StatusCode::BAD_REQUEST)
                                .body(Body::from(format!("Couldn't parse query parameter n1-msg-class - doesn't match schema: {}", e)))
                                .expect("Unable to create Bad Request response for invalid query parameter n1-msg-class")),
                        }
                        }
                        None => None,
                    };
                    let param_n2_info_class = query_params
                        .iter()
                        .filter(|e| e.0 == "n2-info-class")
                        .map(|e| e.1.to_owned())
                        .next();
                    let param_n2_info_class = match param_n2_info_class {
                        Some(param_n2_info_class) => {
                            let param_n2_info_class =
                                <models::N2InformationClass as std::str::FromStr>::from_str(
                                    &param_n2_info_class,
                                );
                            match param_n2_info_class {
                            Ok(param_n2_info_class) => Some(param_n2_info_class),
                            Err(e) => return Ok(Response::builder()
                                .status(StatusCode::BAD_REQUEST)
                                .body(Body::from(format!("Couldn't parse query parameter n2-info-class - doesn't match schema: {}", e)))
                                .expect("Unable to create Bad Request response for invalid query parameter n2-info-class")),
                        }
                        }
                        None => None,
                    };
                    let param_serving_scope = query_params
                        .iter()
                        .filter(|e| e.0 == "serving-scope")
                        .map(|e| e.1.to_owned())
                        .filter_map(|param_serving_scope| param_serving_scope.parse().ok())
                        .collect::<Vec<_>>();
                    let param_serving_scope = if !param_serving_scope.is_empty() {
                        Some(param_serving_scope)
                    } else {
                        None
                    };
                    let param_imsi = query_params
                        .iter()
                        .filter(|e| e.0 == "imsi")
                        .map(|e| e.1.to_owned())
                        .next();
                    let param_imsi = match param_imsi {
                        Some(param_imsi) => {
                            let param_imsi = <String as std::str::FromStr>::from_str(&param_imsi);
                            match param_imsi {
                            Ok(param_imsi) => Some(param_imsi),
                            Err(e) => return Ok(Response::builder()
                                .status(StatusCode::BAD_REQUEST)
                                .body(Body::from(format!("Couldn't parse query parameter imsi - doesn't match schema: {}", e)))
                                .expect("Unable to create Bad Request response for invalid query parameter imsi")),
                        }
                        }
                        None => None,
                    };
                    let param_ims_private_identity = query_params
                        .iter()
                        .filter(|e| e.0 == "ims-private-identity")
                        .map(|e| e.1.to_owned())
                        .next();
                    let param_ims_private_identity = match param_ims_private_identity {
                        Some(param_ims_private_identity) => {
                            let param_ims_private_identity =
                                <String as std::str::FromStr>::from_str(
                                    &param_ims_private_identity,
                                );
                            match param_ims_private_identity {
                            Ok(param_ims_private_identity) => Some(param_ims_private_identity),
                            Err(e) => return Ok(Response::builder()
                                .status(StatusCode::BAD_REQUEST)
                                .body(Body::from(format!("Couldn't parse query parameter ims-private-identity - doesn't match schema: {}", e)))
                                .expect("Unable to create Bad Request response for invalid query parameter ims-private-identity")),
                        }
                        }
                        None => None,
                    };
                    let param_ims_public_identity = query_params
                        .iter()
                        .filter(|e| e.0 == "ims-public-identity")
                        .map(|e| e.1.to_owned())
                        .next();
                    let param_ims_public_identity = match param_ims_public_identity {
                        Some(param_ims_public_identity) => {
                            let param_ims_public_identity =
                                <String as std::str::FromStr>::from_str(&param_ims_public_identity);
                            match param_ims_public_identity {
                            Ok(param_ims_public_identity) => Some(param_ims_public_identity),
                            Err(e) => return Ok(Response::builder()
                                .status(StatusCode::BAD_REQUEST)
                                .body(Body::from(format!("Couldn't parse query parameter ims-public-identity - doesn't match schema: {}", e)))
                                .expect("Unable to create Bad Request response for invalid query parameter ims-public-identity")),
                        }
                        }
                        None => None,
                    };
                    let param_msisdn = query_params
                        .iter()
                        .filter(|e| e.0 == "msisdn")
                        .map(|e| e.1.to_owned())
                        .next();
                    let param_msisdn = match param_msisdn {
                        Some(param_msisdn) => {
                            let param_msisdn =
                                <String as std::str::FromStr>::from_str(&param_msisdn);
                            match param_msisdn {
                            Ok(param_msisdn) => Some(param_msisdn),
                            Err(e) => return Ok(Response::builder()
                                .status(StatusCode::BAD_REQUEST)
                                .body(Body::from(format!("Couldn't parse query parameter msisdn - doesn't match schema: {}", e)))
                                .expect("Unable to create Bad Request response for invalid query parameter msisdn")),
                        }
                        }
                        None => None,
                    };
                    let param_preferred_api_versions = query_params
                        .iter()
                        .filter(|e| e.0 == "preferred-api-versions")
                        .map(|e| e.1.to_owned())
                        .next();
                    let param_preferred_api_versions = match param_preferred_api_versions {
                        Some(param_preferred_api_versions) => {
                            let param_preferred_api_versions =
                                serde_json::from_str::<std::collections::HashMap<String, String>>(
                                    &param_preferred_api_versions,
                                );
                            match param_preferred_api_versions {
                            Ok(param_preferred_api_versions) => Some(param_preferred_api_versions),
                            Err(e) => return Ok(Response::builder()
                                .status(StatusCode::BAD_REQUEST)
                                .body(Body::from(format!("Couldn't parse query parameter preferred-api-versions - doesn't match schema: {}", e)))
                                .expect("Unable to create Bad Request response for invalid query parameter preferred-api-versions")),
                        }
                        }
                        None => None,
                    };
                    let param_v2x_support_ind = query_params
                        .iter()
                        .filter(|e| e.0 == "v2x-support-ind")
                        .map(|e| e.1.to_owned())
                        .next();
                    let param_v2x_support_ind = match param_v2x_support_ind {
                        Some(param_v2x_support_ind) => {
                            let param_v2x_support_ind =
                                <bool as std::str::FromStr>::from_str(&param_v2x_support_ind);
                            match param_v2x_support_ind {
                            Ok(param_v2x_support_ind) => Some(param_v2x_support_ind),
                            Err(e) => return Ok(Response::builder()
                                .status(StatusCode::BAD_REQUEST)
                                .body(Body::from(format!("Couldn't parse query parameter v2x-support-ind - doesn't match schema: {}", e)))
                                .expect("Unable to create Bad Request response for invalid query parameter v2x-support-ind")),
                        }
                        }
                        None => None,
                    };
                    let param_redundant_gtpu = query_params
                        .iter()
                        .filter(|e| e.0 == "redundant-gtpu")
                        .map(|e| e.1.to_owned())
                        .next();
                    let param_redundant_gtpu = match param_redundant_gtpu {
                        Some(param_redundant_gtpu) => {
                            let param_redundant_gtpu =
                                <bool as std::str::FromStr>::from_str(&param_redundant_gtpu);
                            match param_redundant_gtpu {
                            Ok(param_redundant_gtpu) => Some(param_redundant_gtpu),
                            Err(e) => return Ok(Response::builder()
                                .status(StatusCode::BAD_REQUEST)
                                .body(Body::from(format!("Couldn't parse query parameter redundant-gtpu - doesn't match schema: {}", e)))
                                .expect("Unable to create Bad Request response for invalid query parameter redundant-gtpu")),
                        }
                        }
                        None => None,
                    };
                    let param_redundant_transport = query_params
                        .iter()
                        .filter(|e| e.0 == "redundant-transport")
                        .map(|e| e.1.to_owned())
                        .next();
                    let param_redundant_transport = match param_redundant_transport {
                        Some(param_redundant_transport) => {
                            let param_redundant_transport =
                                <bool as std::str::FromStr>::from_str(&param_redundant_transport);
                            match param_redundant_transport {
                            Ok(param_redundant_transport) => Some(param_redundant_transport),
                            Err(e) => return Ok(Response::builder()
                                .status(StatusCode::BAD_REQUEST)
                                .body(Body::from(format!("Couldn't parse query parameter redundant-transport - doesn't match schema: {}", e)))
                                .expect("Unable to create Bad Request response for invalid query parameter redundant-transport")),
                        }
                        }
                        None => None,
                    };
                    let param_ipups = query_params
                        .iter()
                        .filter(|e| e.0 == "ipups")
                        .map(|e| e.1.to_owned())
                        .next();
                    let param_ipups = match param_ipups {
                        Some(param_ipups) => {
                            let param_ipups = <bool as std::str::FromStr>::from_str(&param_ipups);
                            match param_ipups {
                            Ok(param_ipups) => Some(param_ipups),
                            Err(e) => return Ok(Response::builder()
                                .status(StatusCode::BAD_REQUEST)
                                .body(Body::from(format!("Couldn't parse query parameter ipups - doesn't match schema: {}", e)))
                                .expect("Unable to create Bad Request response for invalid query parameter ipups")),
                        }
                        }
                        None => None,
                    };
                    let param_scp_domain_list = query_params
                        .iter()
                        .filter(|e| e.0 == "scp-domain-list")
                        .map(|e| e.1.to_owned())
                        .filter_map(|param_scp_domain_list| param_scp_domain_list.parse().ok())
                        .collect::<Vec<_>>();
                    let param_scp_domain_list = if !param_scp_domain_list.is_empty() {
                        Some(param_scp_domain_list)
                    } else {
                        None
                    };
                    let param_address_domain = query_params
                        .iter()
                        .filter(|e| e.0 == "address-domain")
                        .map(|e| e.1.to_owned())
                        .next();
                    let param_address_domain = match param_address_domain {
                        Some(param_address_domain) => {
                            let param_address_domain =
                                <String as std::str::FromStr>::from_str(&param_address_domain);
                            match param_address_domain {
                            Ok(param_address_domain) => Some(param_address_domain),
                            Err(e) => return Ok(Response::builder()
                                .status(StatusCode::BAD_REQUEST)
                                .body(Body::from(format!("Couldn't parse query parameter address-domain - doesn't match schema: {}", e)))
                                .expect("Unable to create Bad Request response for invalid query parameter address-domain")),
                        }
                        }
                        None => None,
                    };
                    let param_ipv4_addr = query_params
                        .iter()
                        .filter(|e| e.0 == "ipv4-addr")
                        .map(|e| e.1.to_owned())
                        .next();
                    let param_ipv4_addr = match param_ipv4_addr {
                        Some(param_ipv4_addr) => {
                            let param_ipv4_addr =
                                <String as std::str::FromStr>::from_str(&param_ipv4_addr);
                            match param_ipv4_addr {
                            Ok(param_ipv4_addr) => Some(param_ipv4_addr),
                            Err(e) => return Ok(Response::builder()
                                .status(StatusCode::BAD_REQUEST)
                                .body(Body::from(format!("Couldn't parse query parameter ipv4-addr - doesn't match schema: {}", e)))
                                .expect("Unable to create Bad Request response for invalid query parameter ipv4-addr")),
                        }
                        }
                        None => None,
                    };
                    let param_ipv6_prefix = query_params
                        .iter()
                        .filter(|e| e.0 == "ipv6-prefix")
                        .map(|e| e.1.to_owned())
                        .next();
                    let param_ipv6_prefix = match param_ipv6_prefix {
                        Some(param_ipv6_prefix) => {
                            let param_ipv6_prefix =
                                <models::Ipv6Prefix as std::str::FromStr>::from_str(
                                    &param_ipv6_prefix,
                                );
                            match param_ipv6_prefix {
                            Ok(param_ipv6_prefix) => Some(param_ipv6_prefix),
                            Err(e) => return Ok(Response::builder()
                                .status(StatusCode::BAD_REQUEST)
                                .body(Body::from(format!("Couldn't parse query parameter ipv6-prefix - doesn't match schema: {}", e)))
                                .expect("Unable to create Bad Request response for invalid query parameter ipv6-prefix")),
                        }
                        }
                        None => None,
                    };
                    let param_served_nf_set_id = query_params
                        .iter()
                        .filter(|e| e.0 == "served-nf-set-id")
                        .map(|e| e.1.to_owned())
                        .next();
                    let param_served_nf_set_id = match param_served_nf_set_id {
                        Some(param_served_nf_set_id) => {
                            let param_served_nf_set_id =
                                <String as std::str::FromStr>::from_str(&param_served_nf_set_id);
                            match param_served_nf_set_id {
                            Ok(param_served_nf_set_id) => Some(param_served_nf_set_id),
                            Err(e) => return Ok(Response::builder()
                                .status(StatusCode::BAD_REQUEST)
                                .body(Body::from(format!("Couldn't parse query parameter served-nf-set-id - doesn't match schema: {}", e)))
                                .expect("Unable to create Bad Request response for invalid query parameter served-nf-set-id")),
                        }
                        }
                        None => None,
                    };
                    let param_remote_plmn_id = query_params
                        .iter()
                        .filter(|e| e.0 == "remote-plmn-id")
                        .map(|e| e.1.to_owned())
                        .next();
                    let param_remote_plmn_id = match param_remote_plmn_id {
                        Some(param_remote_plmn_id) => {
                            let param_remote_plmn_id =
                                serde_json::from_str::<models::PlmnId>(&param_remote_plmn_id);
                            match param_remote_plmn_id {
                            Ok(param_remote_plmn_id) => Some(param_remote_plmn_id),
                            Err(e) => return Ok(Response::builder()
                                .status(StatusCode::BAD_REQUEST)
                                .body(Body::from(format!("Couldn't parse query parameter remote-plmn-id - doesn't match schema: {}", e)))
                                .expect("Unable to create Bad Request response for invalid query parameter remote-plmn-id")),
                        }
                        }
                        None => None,
                    };
                    let param_remote_snpn_id = query_params
                        .iter()
                        .filter(|e| e.0 == "remote-snpn-id")
                        .map(|e| e.1.to_owned())
                        .next();
                    let param_remote_snpn_id = match param_remote_snpn_id {
                        Some(param_remote_snpn_id) => {
                            let param_remote_snpn_id =
                                serde_json::from_str::<models::PlmnIdNid>(&param_remote_snpn_id);
                            match param_remote_snpn_id {
                            Ok(param_remote_snpn_id) => Some(param_remote_snpn_id),
                            Err(e) => return Ok(Response::builder()
                                .status(StatusCode::BAD_REQUEST)
                                .body(Body::from(format!("Couldn't parse query parameter remote-snpn-id - doesn't match schema: {}", e)))
                                .expect("Unable to create Bad Request response for invalid query parameter remote-snpn-id")),
                        }
                        }
                        None => None,
                    };
                    let param_data_forwarding = query_params
                        .iter()
                        .filter(|e| e.0 == "data-forwarding")
                        .map(|e| e.1.to_owned())
                        .next();
                    let param_data_forwarding = match param_data_forwarding {
                        Some(param_data_forwarding) => {
                            let param_data_forwarding =
                                <bool as std::str::FromStr>::from_str(&param_data_forwarding);
                            match param_data_forwarding {
                            Ok(param_data_forwarding) => Some(param_data_forwarding),
                            Err(e) => return Ok(Response::builder()
                                .status(StatusCode::BAD_REQUEST)
                                .body(Body::from(format!("Couldn't parse query parameter data-forwarding - doesn't match schema: {}", e)))
                                .expect("Unable to create Bad Request response for invalid query parameter data-forwarding")),
                        }
                        }
                        None => None,
                    };
                    let param_preferred_full_plmn = query_params
                        .iter()
                        .filter(|e| e.0 == "preferred-full-plmn")
                        .map(|e| e.1.to_owned())
                        .next();
                    let param_preferred_full_plmn = match param_preferred_full_plmn {
                        Some(param_preferred_full_plmn) => {
                            let param_preferred_full_plmn =
                                <bool as std::str::FromStr>::from_str(&param_preferred_full_plmn);
                            match param_preferred_full_plmn {
                            Ok(param_preferred_full_plmn) => Some(param_preferred_full_plmn),
                            Err(e) => return Ok(Response::builder()
                                .status(StatusCode::BAD_REQUEST)
                                .body(Body::from(format!("Couldn't parse query parameter preferred-full-plmn - doesn't match schema: {}", e)))
                                .expect("Unable to create Bad Request response for invalid query parameter preferred-full-plmn")),
                        }
                        }
                        None => None,
                    };
                    let param_requester_features = query_params
                        .iter()
                        .filter(|e| e.0 == "requester-features")
                        .map(|e| e.1.to_owned())
                        .next();
                    let param_requester_features = match param_requester_features {
                        Some(param_requester_features) => {
                            let param_requester_features =
                                <String as std::str::FromStr>::from_str(&param_requester_features);
                            match param_requester_features {
                            Ok(param_requester_features) => Some(param_requester_features),
                            Err(e) => return Ok(Response::builder()
                                .status(StatusCode::BAD_REQUEST)
                                .body(Body::from(format!("Couldn't parse query parameter requester-features - doesn't match schema: {}", e)))
                                .expect("Unable to create Bad Request response for invalid query parameter requester-features")),
                        }
                        }
                        None => None,
                    };
                    let param_realm_id = query_params
                        .iter()
                        .filter(|e| e.0 == "realm-id")
                        .map(|e| e.1.to_owned())
                        .next();
                    let param_realm_id = match param_realm_id {
                        Some(param_realm_id) => {
                            let param_realm_id =
                                <String as std::str::FromStr>::from_str(&param_realm_id);
                            match param_realm_id {
                            Ok(param_realm_id) => Some(param_realm_id),
                            Err(e) => return Ok(Response::builder()
                                .status(StatusCode::BAD_REQUEST)
                                .body(Body::from(format!("Couldn't parse query parameter realm-id - doesn't match schema: {}", e)))
                                .expect("Unable to create Bad Request response for invalid query parameter realm-id")),
                        }
                        }
                        None => None,
                    };
                    let param_storage_id = query_params
                        .iter()
                        .filter(|e| e.0 == "storage-id")
                        .map(|e| e.1.to_owned())
                        .next();
                    let param_storage_id = match param_storage_id {
                        Some(param_storage_id) => {
                            let param_storage_id =
                                <String as std::str::FromStr>::from_str(&param_storage_id);
                            match param_storage_id {
                            Ok(param_storage_id) => Some(param_storage_id),
                            Err(e) => return Ok(Response::builder()
                                .status(StatusCode::BAD_REQUEST)
                                .body(Body::from(format!("Couldn't parse query parameter storage-id - doesn't match schema: {}", e)))
                                .expect("Unable to create Bad Request response for invalid query parameter storage-id")),
                        }
                        }
                        None => None,
                    };
                    let param_vsmf_support_ind = query_params
                        .iter()
                        .filter(|e| e.0 == "vsmf-support-ind")
                        .map(|e| e.1.to_owned())
                        .next();
                    let param_vsmf_support_ind = match param_vsmf_support_ind {
                        Some(param_vsmf_support_ind) => {
                            let param_vsmf_support_ind =
                                <bool as std::str::FromStr>::from_str(&param_vsmf_support_ind);
                            match param_vsmf_support_ind {
                            Ok(param_vsmf_support_ind) => Some(param_vsmf_support_ind),
                            Err(e) => return Ok(Response::builder()
                                .status(StatusCode::BAD_REQUEST)
                                .body(Body::from(format!("Couldn't parse query parameter vsmf-support-ind - doesn't match schema: {}", e)))
                                .expect("Unable to create Bad Request response for invalid query parameter vsmf-support-ind")),
                        }
                        }
                        None => None,
                    };
                    let param_ismf_support_ind = query_params
                        .iter()
                        .filter(|e| e.0 == "ismf-support-ind")
                        .map(|e| e.1.to_owned())
                        .next();
                    let param_ismf_support_ind = match param_ismf_support_ind {
                        Some(param_ismf_support_ind) => {
                            let param_ismf_support_ind =
                                <bool as std::str::FromStr>::from_str(&param_ismf_support_ind);
                            match param_ismf_support_ind {
                            Ok(param_ismf_support_ind) => Some(param_ismf_support_ind),
                            Err(e) => return Ok(Response::builder()
                                .status(StatusCode::BAD_REQUEST)
                                .body(Body::from(format!("Couldn't parse query parameter ismf-support-ind - doesn't match schema: {}", e)))
                                .expect("Unable to create Bad Request response for invalid query parameter ismf-support-ind")),
                        }
                        }
                        None => None,
                    };
                    let param_nrf_disc_uri = query_params
                        .iter()
                        .filter(|e| e.0 == "nrf-disc-uri")
                        .map(|e| e.1.to_owned())
                        .next();
                    let param_nrf_disc_uri = match param_nrf_disc_uri {
                        Some(param_nrf_disc_uri) => {
                            let param_nrf_disc_uri =
                                <String as std::str::FromStr>::from_str(&param_nrf_disc_uri);
                            match param_nrf_disc_uri {
                            Ok(param_nrf_disc_uri) => Some(param_nrf_disc_uri),
                            Err(e) => return Ok(Response::builder()
                                .status(StatusCode::BAD_REQUEST)
                                .body(Body::from(format!("Couldn't parse query parameter nrf-disc-uri - doesn't match schema: {}", e)))
                                .expect("Unable to create Bad Request response for invalid query parameter nrf-disc-uri")),
                        }
                        }
                        None => None,
                    };
                    let param_preferred_vendor_specific_features = query_params
                        .iter()
                        .filter(|e| e.0 == "preferred-vendor-specific-features")
                        .map(|e| e.1.to_owned())
                        .next();
                    let param_preferred_vendor_specific_features =
                        match param_preferred_vendor_specific_features {
                            Some(param_preferred_vendor_specific_features) => {
                                let param_preferred_vendor_specific_features = serde_json::from_str::<
                                    std::collections::HashMap<
                                        String,
                                        std::collections::HashMap<
                                            String,
                                            Vec<models::VendorSpecificFeature>,
                                        >,
                                    >,
                                >(
                                    &param_preferred_vendor_specific_features,
                                );
                                match param_preferred_vendor_specific_features {
                            Ok(param_preferred_vendor_specific_features) => Some(param_preferred_vendor_specific_features),
                            Err(e) => return Ok(Response::builder()
                                .status(StatusCode::BAD_REQUEST)
                                .body(Body::from(format!("Couldn't parse query parameter preferred-vendor-specific-features - doesn't match schema: {}", e)))
                                .expect("Unable to create Bad Request response for invalid query parameter preferred-vendor-specific-features")),
                        }
                            }
                            None => None,
                        };
                    let param_preferred_vendor_specific_nf_features = query_params
                        .iter()
                        .filter(|e| e.0 == "preferred-vendor-specific-nf-features")
                        .map(|e| e.1.to_owned())
                        .next();
                    let param_preferred_vendor_specific_nf_features =
                        match param_preferred_vendor_specific_nf_features {
                            Some(param_preferred_vendor_specific_nf_features) => {
                                let param_preferred_vendor_specific_nf_features =
                                    serde_json::from_str::<
                                        std::collections::HashMap<
                                            String,
                                            Vec<models::VendorSpecificFeature>,
                                        >,
                                    >(
                                        &param_preferred_vendor_specific_nf_features
                                    );
                                match param_preferred_vendor_specific_nf_features {
                            Ok(param_preferred_vendor_specific_nf_features) => Some(param_preferred_vendor_specific_nf_features),
                            Err(e) => return Ok(Response::builder()
                                .status(StatusCode::BAD_REQUEST)
                                .body(Body::from(format!("Couldn't parse query parameter preferred-vendor-specific-nf-features - doesn't match schema: {}", e)))
                                .expect("Unable to create Bad Request response for invalid query parameter preferred-vendor-specific-nf-features")),
                        }
                            }
                            None => None,
                        };
                    let param_required_pfcp_features = query_params
                        .iter()
                        .filter(|e| e.0 == "required-pfcp-features")
                        .map(|e| e.1.to_owned())
                        .next();
                    let param_required_pfcp_features = match param_required_pfcp_features {
                        Some(param_required_pfcp_features) => {
                            let param_required_pfcp_features =
                                <String as std::str::FromStr>::from_str(
                                    &param_required_pfcp_features,
                                );
                            match param_required_pfcp_features {
                            Ok(param_required_pfcp_features) => Some(param_required_pfcp_features),
                            Err(e) => return Ok(Response::builder()
                                .status(StatusCode::BAD_REQUEST)
                                .body(Body::from(format!("Couldn't parse query parameter required-pfcp-features - doesn't match schema: {}", e)))
                                .expect("Unable to create Bad Request response for invalid query parameter required-pfcp-features")),
                        }
                        }
                        None => None,
                    };
                    let param_home_pub_key_id = query_params
                        .iter()
                        .filter(|e| e.0 == "home-pub-key-id")
                        .map(|e| e.1.to_owned())
                        .next();
                    let param_home_pub_key_id = match param_home_pub_key_id {
                        Some(param_home_pub_key_id) => {
                            let param_home_pub_key_id =
                                <i32 as std::str::FromStr>::from_str(&param_home_pub_key_id);
                            match param_home_pub_key_id {
                            Ok(param_home_pub_key_id) => Some(param_home_pub_key_id),
                            Err(e) => return Ok(Response::builder()
                                .status(StatusCode::BAD_REQUEST)
                                .body(Body::from(format!("Couldn't parse query parameter home-pub-key-id - doesn't match schema: {}", e)))
                                .expect("Unable to create Bad Request response for invalid query parameter home-pub-key-id")),
                        }
                        }
                        None => None,
                    };
                    let param_prose_support_ind = query_params
                        .iter()
                        .filter(|e| e.0 == "prose-support-ind")
                        .map(|e| e.1.to_owned())
                        .next();
                    let param_prose_support_ind = match param_prose_support_ind {
                        Some(param_prose_support_ind) => {
                            let param_prose_support_ind =
                                <bool as std::str::FromStr>::from_str(&param_prose_support_ind);
                            match param_prose_support_ind {
                            Ok(param_prose_support_ind) => Some(param_prose_support_ind),
                            Err(e) => return Ok(Response::builder()
                                .status(StatusCode::BAD_REQUEST)
                                .body(Body::from(format!("Couldn't parse query parameter prose-support-ind - doesn't match schema: {}", e)))
                                .expect("Unable to create Bad Request response for invalid query parameter prose-support-ind")),
                        }
                        }
                        None => None,
                    };
                    let param_analytics_aggregation_ind = query_params
                        .iter()
                        .filter(|e| e.0 == "analytics-aggregation-ind")
                        .map(|e| e.1.to_owned())
                        .next();
                    let param_analytics_aggregation_ind = match param_analytics_aggregation_ind {
                        Some(param_analytics_aggregation_ind) => {
                            let param_analytics_aggregation_ind =
                                <bool as std::str::FromStr>::from_str(
                                    &param_analytics_aggregation_ind,
                                );
                            match param_analytics_aggregation_ind {
                            Ok(param_analytics_aggregation_ind) => Some(param_analytics_aggregation_ind),
                            Err(e) => return Ok(Response::builder()
                                .status(StatusCode::BAD_REQUEST)
                                .body(Body::from(format!("Couldn't parse query parameter analytics-aggregation-ind - doesn't match schema: {}", e)))
                                .expect("Unable to create Bad Request response for invalid query parameter analytics-aggregation-ind")),
                        }
                        }
                        None => None,
                    };
                    let param_serving_nf_set_id = query_params
                        .iter()
                        .filter(|e| e.0 == "serving-nf-set-id")
                        .map(|e| e.1.to_owned())
                        .next();
                    let param_serving_nf_set_id = match param_serving_nf_set_id {
                        Some(param_serving_nf_set_id) => {
                            let param_serving_nf_set_id =
                                <String as std::str::FromStr>::from_str(&param_serving_nf_set_id);
                            match param_serving_nf_set_id {
                            Ok(param_serving_nf_set_id) => Some(param_serving_nf_set_id),
                            Err(e) => return Ok(Response::builder()
                                .status(StatusCode::BAD_REQUEST)
                                .body(Body::from(format!("Couldn't parse query parameter serving-nf-set-id - doesn't match schema: {}", e)))
                                .expect("Unable to create Bad Request response for invalid query parameter serving-nf-set-id")),
                        }
                        }
                        None => None,
                    };
                    let param_serving_nf_type = query_params
                        .iter()
                        .filter(|e| e.0 == "serving-nf-type")
                        .map(|e| e.1.to_owned())
                        .next();
                    let param_serving_nf_type = match param_serving_nf_type {
                        Some(param_serving_nf_type) => {
                            let param_serving_nf_type =
                                <models::NfType as std::str::FromStr>::from_str(
                                    &param_serving_nf_type,
                                );
                            match param_serving_nf_type {
                            Ok(param_serving_nf_type) => Some(param_serving_nf_type),
                            Err(e) => return Ok(Response::builder()
                                .status(StatusCode::BAD_REQUEST)
                                .body(Body::from(format!("Couldn't parse query parameter serving-nf-type - doesn't match schema: {}", e)))
                                .expect("Unable to create Bad Request response for invalid query parameter serving-nf-type")),
                        }
                        }
                        None => None,
                    };
                    let param_ml_analytics_info_list = query_params
                        .iter()
                        .filter(|e| e.0 == "ml-analytics-info-list")
                        .map(|e| e.1.to_owned())
                        .next();
                    let param_ml_analytics_info_list = match param_ml_analytics_info_list {
                        Some(param_ml_analytics_info_list) => {
                            let param_ml_analytics_info_list =
                                serde_json::from_str::<Vec<models::MlAnalyticsInfo>>(
                                    &param_ml_analytics_info_list,
                                );
                            match param_ml_analytics_info_list {
                            Ok(param_ml_analytics_info_list) => Some(param_ml_analytics_info_list),
                            Err(e) => return Ok(Response::builder()
                                .status(StatusCode::BAD_REQUEST)
                                .body(Body::from(format!("Couldn't parse query parameter ml-analytics-info-list - doesn't match schema: {}", e)))
                                .expect("Unable to create Bad Request response for invalid query parameter ml-analytics-info-list")),
                        }
                        }
                        None => None,
                    };
                    let param_analytics_metadata_prov_ind = query_params
                        .iter()
                        .filter(|e| e.0 == "analytics-metadata-prov-ind")
                        .map(|e| e.1.to_owned())
                        .next();
                    let param_analytics_metadata_prov_ind = match param_analytics_metadata_prov_ind
                    {
                        Some(param_analytics_metadata_prov_ind) => {
                            let param_analytics_metadata_prov_ind =
                                <bool as std::str::FromStr>::from_str(
                                    &param_analytics_metadata_prov_ind,
                                );
                            match param_analytics_metadata_prov_ind {
                            Ok(param_analytics_metadata_prov_ind) => Some(param_analytics_metadata_prov_ind),
                            Err(e) => return Ok(Response::builder()
                                .status(StatusCode::BAD_REQUEST)
                                .body(Body::from(format!("Couldn't parse query parameter analytics-metadata-prov-ind - doesn't match schema: {}", e)))
                                .expect("Unable to create Bad Request response for invalid query parameter analytics-metadata-prov-ind")),
                        }
                        }
                        None => None,
                    };
                    let param_nsacf_capability = query_params
                        .iter()
                        .filter(|e| e.0 == "nsacf-capability")
                        .map(|e| e.1.to_owned())
                        .next();
                    let param_nsacf_capability = match param_nsacf_capability {
                        Some(param_nsacf_capability) => {
                            let param_nsacf_capability =
                                <models::NsacfCapability as std::str::FromStr>::from_str(
                                    &param_nsacf_capability,
                                );
                            match param_nsacf_capability {
                            Ok(param_nsacf_capability) => Some(param_nsacf_capability),
                            Err(e) => return Ok(Response::builder()
                                .status(StatusCode::BAD_REQUEST)
                                .body(Body::from(format!("Couldn't parse query parameter nsacf-capability - doesn't match schema: {}", e)))
                                .expect("Unable to create Bad Request response for invalid query parameter nsacf-capability")),
                        }
                        }
                        None => None,
                    };
                    let param_mbs_session_id_list = query_params
                        .iter()
                        .filter(|e| e.0 == "mbs-session-id-list")
                        .map(|e| e.1.to_owned())
                        .next();
                    let param_mbs_session_id_list = match param_mbs_session_id_list {
                        Some(param_mbs_session_id_list) => {
                            let param_mbs_session_id_list =
                                serde_json::from_str::<Vec<models::MbsSessionId>>(
                                    &param_mbs_session_id_list,
                                );
                            match param_mbs_session_id_list {
                            Ok(param_mbs_session_id_list) => Some(param_mbs_session_id_list),
                            Err(e) => return Ok(Response::builder()
                                .status(StatusCode::BAD_REQUEST)
                                .body(Body::from(format!("Couldn't parse query parameter mbs-session-id-list - doesn't match schema: {}", e)))
                                .expect("Unable to create Bad Request response for invalid query parameter mbs-session-id-list")),
                        }
                        }
                        None => None,
                    };
                    let param_area_session_id = query_params
                        .iter()
                        .filter(|e| e.0 == "area-session-id")
                        .map(|e| e.1.to_owned())
                        .next();
                    let param_area_session_id = match param_area_session_id {
                        Some(param_area_session_id) => {
                            let param_area_session_id =
                                <i32 as std::str::FromStr>::from_str(&param_area_session_id);
                            match param_area_session_id {
                            Ok(param_area_session_id) => Some(param_area_session_id),
                            Err(e) => return Ok(Response::builder()
                                .status(StatusCode::BAD_REQUEST)
                                .body(Body::from(format!("Couldn't parse query parameter area-session-id - doesn't match schema: {}", e)))
                                .expect("Unable to create Bad Request response for invalid query parameter area-session-id")),
                        }
                        }
                        None => None,
                    };
                    let param_gmlc_number = query_params
                        .iter()
                        .filter(|e| e.0 == "gmlc-number")
                        .map(|e| e.1.to_owned())
                        .next();
                    let param_gmlc_number = match param_gmlc_number {
                        Some(param_gmlc_number) => {
                            let param_gmlc_number =
                                <String as std::str::FromStr>::from_str(&param_gmlc_number);
                            match param_gmlc_number {
                            Ok(param_gmlc_number) => Some(param_gmlc_number),
                            Err(e) => return Ok(Response::builder()
                                .status(StatusCode::BAD_REQUEST)
                                .body(Body::from(format!("Couldn't parse query parameter gmlc-number - doesn't match schema: {}", e)))
                                .expect("Unable to create Bad Request response for invalid query parameter gmlc-number")),
                        }
                        }
                        None => None,
                    };
                    let param_upf_n6_ip = query_params
                        .iter()
                        .filter(|e| e.0 == "upf-n6-ip")
                        .map(|e| e.1.to_owned())
                        .next();
                    let param_upf_n6_ip = match param_upf_n6_ip {
                        Some(param_upf_n6_ip) => {
                            let param_upf_n6_ip =
                                serde_json::from_str::<models::IpAddr>(&param_upf_n6_ip);
                            match param_upf_n6_ip {
                            Ok(param_upf_n6_ip) => Some(param_upf_n6_ip),
                            Err(e) => return Ok(Response::builder()
                                .status(StatusCode::BAD_REQUEST)
                                .body(Body::from(format!("Couldn't parse query parameter upf-n6-ip - doesn't match schema: {}", e)))
                                .expect("Unable to create Bad Request response for invalid query parameter upf-n6-ip")),
                        }
                        }
                        None => None,
                    };
                    let param_tai_list = query_params
                        .iter()
                        .filter(|e| e.0 == "tai-list")
                        .map(|e| e.1.to_owned())
                        .next();
                    let param_tai_list = match param_tai_list {
                        Some(param_tai_list) => {
                            let param_tai_list =
                                serde_json::from_str::<Vec<models::Tai>>(&param_tai_list);
                            match param_tai_list {
                            Ok(param_tai_list) => Some(param_tai_list),
                            Err(e) => return Ok(Response::builder()
                                .status(StatusCode::BAD_REQUEST)
                                .body(Body::from(format!("Couldn't parse query parameter tai-list - doesn't match schema: {}", e)))
                                .expect("Unable to create Bad Request response for invalid query parameter tai-list")),
                        }
                        }
                        None => None,
                    };
                    let param_preferences_precedence = query_params
                        .iter()
                        .filter(|e| e.0 == "preferences-precedence")
                        .map(|e| e.1.to_owned())
                        .filter_map(|param_preferences_precedence| {
                            param_preferences_precedence.parse().ok()
                        })
                        .collect::<Vec<_>>();
                    let param_preferences_precedence = if !param_preferences_precedence.is_empty() {
                        Some(param_preferences_precedence)
                    } else {
                        None
                    };
                    let param_support_onboarding_capability = query_params
                        .iter()
                        .filter(|e| e.0 == "support-onboarding-capability")
                        .map(|e| e.1.to_owned())
                        .next();
                    let param_support_onboarding_capability =
                        match param_support_onboarding_capability {
                            Some(param_support_onboarding_capability) => {
                                let param_support_onboarding_capability =
                                    <bool as std::str::FromStr>::from_str(
                                        &param_support_onboarding_capability,
                                    );
                                match param_support_onboarding_capability {
                            Ok(param_support_onboarding_capability) => Some(param_support_onboarding_capability),
                            Err(e) => return Ok(Response::builder()
                                .status(StatusCode::BAD_REQUEST)
                                .body(Body::from(format!("Couldn't parse query parameter support-onboarding-capability - doesn't match schema: {}", e)))
                                .expect("Unable to create Bad Request response for invalid query parameter support-onboarding-capability")),
                        }
                            }
                            None => None,
                        };
                    let param_uas_nf_functionality_ind = query_params
                        .iter()
                        .filter(|e| e.0 == "uas-nf-functionality-ind")
                        .map(|e| e.1.to_owned())
                        .next();
                    let param_uas_nf_functionality_ind = match param_uas_nf_functionality_ind {
                        Some(param_uas_nf_functionality_ind) => {
                            let param_uas_nf_functionality_ind =
                                <bool as std::str::FromStr>::from_str(
                                    &param_uas_nf_functionality_ind,
                                );
                            match param_uas_nf_functionality_ind {
                            Ok(param_uas_nf_functionality_ind) => Some(param_uas_nf_functionality_ind),
                            Err(e) => return Ok(Response::builder()
                                .status(StatusCode::BAD_REQUEST)
                                .body(Body::from(format!("Couldn't parse query parameter uas-nf-functionality-ind - doesn't match schema: {}", e)))
                                .expect("Unable to create Bad Request response for invalid query parameter uas-nf-functionality-ind")),
                        }
                        }
                        None => None,
                    };
                    let param_v2x_capability = query_params
                        .iter()
                        .filter(|e| e.0 == "v2x-capability")
                        .map(|e| e.1.to_owned())
                        .next();
                    let param_v2x_capability = match param_v2x_capability {
                        Some(param_v2x_capability) => {
                            let param_v2x_capability = serde_json::from_str::<models::V2xCapability>(
                                &param_v2x_capability,
                            );
                            match param_v2x_capability {
                            Ok(param_v2x_capability) => Some(param_v2x_capability),
                            Err(e) => return Ok(Response::builder()
                                .status(StatusCode::BAD_REQUEST)
                                .body(Body::from(format!("Couldn't parse query parameter v2x-capability - doesn't match schema: {}", e)))
                                .expect("Unable to create Bad Request response for invalid query parameter v2x-capability")),
                        }
                        }
                        None => None,
                    };
                    let param_prose_capability = query_params
                        .iter()
                        .filter(|e| e.0 == "prose-capability")
                        .map(|e| e.1.to_owned())
                        .next();
                    let param_prose_capability = match param_prose_capability {
                        Some(param_prose_capability) => {
                            let param_prose_capability =
                                serde_json::from_str::<models::ProSeCapability>(
                                    &param_prose_capability,
                                );
                            match param_prose_capability {
                            Ok(param_prose_capability) => Some(param_prose_capability),
                            Err(e) => return Ok(Response::builder()
                                .status(StatusCode::BAD_REQUEST)
                                .body(Body::from(format!("Couldn't parse query parameter prose-capability - doesn't match schema: {}", e)))
                                .expect("Unable to create Bad Request response for invalid query parameter prose-capability")),
                        }
                        }
                        None => None,
                    };
                    let param_shared_data_id = query_params
                        .iter()
                        .filter(|e| e.0 == "shared-data-id")
                        .map(|e| e.1.to_owned())
                        .next();
                    let param_shared_data_id = match param_shared_data_id {
                        Some(param_shared_data_id) => {
                            let param_shared_data_id =
                                <String as std::str::FromStr>::from_str(&param_shared_data_id);
                            match param_shared_data_id {
                            Ok(param_shared_data_id) => Some(param_shared_data_id),
                            Err(e) => return Ok(Response::builder()
                                .status(StatusCode::BAD_REQUEST)
                                .body(Body::from(format!("Couldn't parse query parameter shared-data-id - doesn't match schema: {}", e)))
                                .expect("Unable to create Bad Request response for invalid query parameter shared-data-id")),
                        }
                        }
                        None => None,
                    };
                    let param_target_hni = query_params
                        .iter()
                        .filter(|e| e.0 == "target-hni")
                        .map(|e| e.1.to_owned())
                        .next();
                    let param_target_hni = match param_target_hni {
                        Some(param_target_hni) => {
                            let param_target_hni =
                                <String as std::str::FromStr>::from_str(&param_target_hni);
                            match param_target_hni {
                            Ok(param_target_hni) => Some(param_target_hni),
                            Err(e) => return Ok(Response::builder()
                                .status(StatusCode::BAD_REQUEST)
                                .body(Body::from(format!("Couldn't parse query parameter target-hni - doesn't match schema: {}", e)))
                                .expect("Unable to create Bad Request response for invalid query parameter target-hni")),
                        }
                        }
                        None => None,
                    };
                    let param_target_nw_resolution = query_params
                        .iter()
                        .filter(|e| e.0 == "target-nw-resolution")
                        .map(|e| e.1.to_owned())
                        .next();
                    let param_target_nw_resolution = match param_target_nw_resolution {
                        Some(param_target_nw_resolution) => {
                            let param_target_nw_resolution =
                                <bool as std::str::FromStr>::from_str(&param_target_nw_resolution);
                            match param_target_nw_resolution {
                            Ok(param_target_nw_resolution) => Some(param_target_nw_resolution),
                            Err(e) => return Ok(Response::builder()
                                .status(StatusCode::BAD_REQUEST)
                                .body(Body::from(format!("Couldn't parse query parameter target-nw-resolution - doesn't match schema: {}", e)))
                                .expect("Unable to create Bad Request response for invalid query parameter target-nw-resolution")),
                        }
                        }
                        None => None,
                    };
                    let param_exclude_nfinst_list = query_params
                        .iter()
                        .filter(|e| e.0 == "exclude-nfinst-list")
                        .map(|e| e.1.to_owned())
                        .filter_map(|param_exclude_nfinst_list| {
                            param_exclude_nfinst_list.parse().ok()
                        })
                        .collect::<Vec<_>>();
                    let param_exclude_nfinst_list = if !param_exclude_nfinst_list.is_empty() {
                        Some(param_exclude_nfinst_list)
                    } else {
                        None
                    };
                    let param_exclude_nfservinst_list = query_params
                        .iter()
                        .filter(|e| e.0 == "exclude-nfservinst-list")
                        .map(|e| e.1.to_owned())
                        .next();
                    let param_exclude_nfservinst_list = match param_exclude_nfservinst_list {
                        Some(param_exclude_nfservinst_list) => {
                            let param_exclude_nfservinst_list =
                                serde_json::from_str::<Vec<models::NfServiceInstance>>(
                                    &param_exclude_nfservinst_list,
                                );
                            match param_exclude_nfservinst_list {
                            Ok(param_exclude_nfservinst_list) => Some(param_exclude_nfservinst_list),
                            Err(e) => return Ok(Response::builder()
                                .status(StatusCode::BAD_REQUEST)
                                .body(Body::from(format!("Couldn't parse query parameter exclude-nfservinst-list - doesn't match schema: {}", e)))
                                .expect("Unable to create Bad Request response for invalid query parameter exclude-nfservinst-list")),
                        }
                        }
                        None => None,
                    };
                    let param_exclude_nfserviceset_list = query_params
                        .iter()
                        .filter(|e| e.0 == "exclude-nfserviceset-list")
                        .map(|e| e.1.to_owned())
                        .filter_map(|param_exclude_nfserviceset_list| {
                            param_exclude_nfserviceset_list.parse().ok()
                        })
                        .collect::<Vec<_>>();
                    let param_exclude_nfserviceset_list =
                        if !param_exclude_nfserviceset_list.is_empty() {
                            Some(param_exclude_nfserviceset_list)
                        } else {
                            None
                        };
                    let param_exclude_nfset_list = query_params
                        .iter()
                        .filter(|e| e.0 == "exclude-nfset-list")
                        .map(|e| e.1.to_owned())
                        .filter_map(|param_exclude_nfset_list| {
                            param_exclude_nfset_list.parse().ok()
                        })
                        .collect::<Vec<_>>();
                    let param_exclude_nfset_list = if !param_exclude_nfset_list.is_empty() {
                        Some(param_exclude_nfset_list)
                    } else {
                        None
                    };
                    let param_preferred_analytics_delays = query_params
                        .iter()
                        .filter(|e| e.0 == "preferred-analytics-delays")
                        .map(|e| e.1.to_owned())
                        .next();
                    let param_preferred_analytics_delays = match param_preferred_analytics_delays {
                        Some(param_preferred_analytics_delays) => {
                            let param_preferred_analytics_delays =
                                serde_json::from_str::<
                                    std::collections::HashMap<String, models::DurationSec>,
                                >(&param_preferred_analytics_delays);
                            match param_preferred_analytics_delays {
                            Ok(param_preferred_analytics_delays) => Some(param_preferred_analytics_delays),
                            Err(e) => return Ok(Response::builder()
                                .status(StatusCode::BAD_REQUEST)
                                .body(Body::from(format!("Couldn't parse query parameter preferred-analytics-delays - doesn't match schema: {}", e)))
                                .expect("Unable to create Bad Request response for invalid query parameter preferred-analytics-delays")),
                        }
                        }
                        None => None,
                    };

                    let result = api_impl
                        .search_nf_instances(
                            param_target_nf_type,
                            param_requester_nf_type,
                            param_accept_encoding,
                            param_preferred_collocated_nf_types.as_ref(),
                            param_requester_nf_instance_id,
                            param_service_names.as_ref(),
                            param_requester_nf_instance_fqdn,
                            param_target_plmn_list.as_ref(),
                            param_requester_plmn_list.as_ref(),
                            param_target_nf_instance_id,
                            param_target_nf_fqdn,
                            param_hnrf_uri,
                            param_snssais.as_ref(),
                            param_requester_snssais.as_ref(),
                            param_plmn_specific_snssai_list.as_ref(),
                            param_requester_plmn_specific_snssai_list.as_ref(),
                            param_dnn,
                            param_ipv4_index,
                            param_ipv6_index,
                            param_nsi_list.as_ref(),
                            param_smf_serving_area,
                            param_mbsmf_serving_area,
                            param_tai,
                            param_amf_region_id,
                            param_amf_set_id,
                            param_guami,
                            param_supi,
                            param_ue_ipv4_address,
                            param_ip_domain,
                            param_ue_ipv6_prefix,
                            param_pgw_ind,
                            param_preferred_pgw_ind,
                            param_pgw,
                            param_pgw_ip,
                            param_gpsi,
                            param_external_group_identity,
                            param_internal_group_identity,
                            param_pfd_data,
                            param_data_set,
                            param_routing_indicator,
                            param_group_id_list.as_ref(),
                            param_dnai_list.as_ref(),
                            param_pdu_session_types.as_ref(),
                            param_event_id_list.as_ref(),
                            param_nwdaf_event_list.as_ref(),
                            param_supported_features,
                            param_upf_iwk_eps_ind,
                            param_chf_supported_plmn,
                            param_preferred_locality,
                            param_access_type,
                            param_limit,
                            param_required_features.as_ref(),
                            param_complex_query,
                            param_max_payload_size,
                            param_max_payload_size_ext,
                            param_atsss_capability,
                            param_upf_ue_ip_addr_ind,
                            param_client_type,
                            param_lmf_id,
                            param_an_node_type,
                            param_rat_type,
                            param_preferred_tai,
                            param_preferred_nf_instances.as_ref(),
                            param_if_none_match,
                            param_target_snpn,
                            param_requester_snpn_list.as_ref(),
                            param_af_ee_data,
                            param_w_agf_info,
                            param_tngf_info,
                            param_twif_info,
                            param_target_nf_set_id,
                            param_target_nf_service_set_id,
                            param_nef_id,
                            param_notification_type,
                            param_n1_msg_class,
                            param_n2_info_class,
                            param_serving_scope.as_ref(),
                            param_imsi,
                            param_ims_private_identity,
                            param_ims_public_identity,
                            param_msisdn,
                            param_preferred_api_versions,
                            param_v2x_support_ind,
                            param_redundant_gtpu,
                            param_redundant_transport,
                            param_ipups,
                            param_scp_domain_list.as_ref(),
                            param_address_domain,
                            param_ipv4_addr,
                            param_ipv6_prefix,
                            param_served_nf_set_id,
                            param_remote_plmn_id,
                            param_remote_snpn_id,
                            param_data_forwarding,
                            param_preferred_full_plmn,
                            param_requester_features,
                            param_realm_id,
                            param_storage_id,
                            param_vsmf_support_ind,
                            param_ismf_support_ind,
                            param_nrf_disc_uri,
                            param_preferred_vendor_specific_features,
                            param_preferred_vendor_specific_nf_features,
                            param_required_pfcp_features,
                            param_home_pub_key_id,
                            param_prose_support_ind,
                            param_analytics_aggregation_ind,
                            param_serving_nf_set_id,
                            param_serving_nf_type,
                            param_ml_analytics_info_list.as_ref(),
                            param_analytics_metadata_prov_ind,
                            param_nsacf_capability,
                            param_mbs_session_id_list.as_ref(),
                            param_area_session_id,
                            param_gmlc_number,
                            param_upf_n6_ip,
                            param_tai_list.as_ref(),
                            param_preferences_precedence.as_ref(),
                            param_support_onboarding_capability,
                            param_uas_nf_functionality_ind,
                            param_v2x_capability,
                            param_prose_capability,
                            param_shared_data_id,
                            param_target_hni,
                            param_target_nw_resolution,
                            param_exclude_nfinst_list.as_ref(),
                            param_exclude_nfservinst_list.as_ref(),
                            param_exclude_nfserviceset_list.as_ref(),
                            param_exclude_nfset_list.as_ref(),
                            param_preferred_analytics_delays,
                            &context,
                        )
                        .await;
                    let mut response = Response::new(Body::empty());
                    response.headers_mut().insert(
                        HeaderName::from_static("x-span-id"),
                        HeaderValue::from_str(
                            (&context as &dyn Has<XSpanIdString>)
                                .get()
                                .0
                                .clone()
                                .as_str(),
                        )
                        .expect("Unable to create X-Span-ID header value"),
                    );

                    match result {
                        Ok(rsp) => match rsp {
                            SearchNfInstancesResponse::ExpectedResponseToAValidRequest {
                                body,
                                cache_control,
                                e_tag,
                                content_encoding,
                            } => {
                                if let Some(cache_control) = cache_control {
                                    let cache_control = match header::IntoHeaderValue(cache_control).try_into() {
                                                        Ok(val) => val,
                                                        Err(e) => {
                                                            return Ok(Response::builder()
                                                                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                                                                    .body(Body::from(format!("An internal server error occurred handling cache_control header - {}", e)))
                                                                    .expect("Unable to create Internal Server Error for invalid response header"))
                                                        }
                                                    };

                                    response.headers_mut().insert(
                                        HeaderName::from_static("cache-control"),
                                        cache_control,
                                    );
                                }
                                if let Some(e_tag) = e_tag {
                                    let e_tag = match header::IntoHeaderValue(e_tag).try_into() {
                                                        Ok(val) => val,
                                                        Err(e) => {
                                                            return Ok(Response::builder()
                                                                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                                                                    .body(Body::from(format!("An internal server error occurred handling e_tag header - {}", e)))
                                                                    .expect("Unable to create Internal Server Error for invalid response header"))
                                                        }
                                                    };

                                    response
                                        .headers_mut()
                                        .insert(HeaderName::from_static("etag"), e_tag);
                                }
                                if let Some(content_encoding) = content_encoding {
                                    let content_encoding = match header::IntoHeaderValue(content_encoding).try_into() {
                                                        Ok(val) => val,
                                                        Err(e) => {
                                                            return Ok(Response::builder()
                                                                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                                                                    .body(Body::from(format!("An internal server error occurred handling content_encoding header - {}", e)))
                                                                    .expect("Unable to create Internal Server Error for invalid response header"))
                                                        }
                                                    };

                                    response.headers_mut().insert(
                                        HeaderName::from_static("content-encoding"),
                                        content_encoding,
                                    );
                                }
                                *response.status_mut() = StatusCode::from_u16(200)
                                    .expect("Unable to turn 200 into a StatusCode");
                                response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for SEARCH_NF_INSTANCES_EXPECTED_RESPONSE_TO_A_VALID_REQUEST"));
                                let body = serde_json::to_string(&body)
                                    .expect("impossible to fail to serialize");
                                *response.body_mut() = Body::from(body);
                            }
                            SearchNfInstancesResponse::TemporaryRedirect { body, location } => {
                                let location = match header::IntoHeaderValue(location).try_into() {
                                                        Ok(val) => val,
                                                        Err(e) => {
                                                            return Ok(Response::builder()
                                                                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                                                                    .body(Body::from(format!("An internal server error occurred handling location header - {}", e)))
                                                                    .expect("Unable to create Internal Server Error for invalid response header"))
                                                        }
                                                    };

                                response
                                    .headers_mut()
                                    .insert(HeaderName::from_static("location"), location);
                                *response.status_mut() = StatusCode::from_u16(307)
                                    .expect("Unable to turn 307 into a StatusCode");
                                response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for SEARCH_NF_INSTANCES_TEMPORARY_REDIRECT"));
                                let body = serde_json::to_string(&body)
                                    .expect("impossible to fail to serialize");
                                *response.body_mut() = Body::from(body);
                            }
                            SearchNfInstancesResponse::PermanentRedirect { body, location } => {
                                let location = match header::IntoHeaderValue(location).try_into() {
                                                        Ok(val) => val,
                                                        Err(e) => {
                                                            return Ok(Response::builder()
                                                                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                                                                    .body(Body::from(format!("An internal server error occurred handling location header - {}", e)))
                                                                    .expect("Unable to create Internal Server Error for invalid response header"))
                                                        }
                                                    };

                                response
                                    .headers_mut()
                                    .insert(HeaderName::from_static("location"), location);
                                *response.status_mut() = StatusCode::from_u16(308)
                                    .expect("Unable to turn 308 into a StatusCode");
                                response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for SEARCH_NF_INSTANCES_PERMANENT_REDIRECT"));
                                let body = serde_json::to_string(&body)
                                    .expect("impossible to fail to serialize");
                                *response.body_mut() = Body::from(body);
                            }
                            SearchNfInstancesResponse::BadRequest(body) => {
                                *response.status_mut() = StatusCode::from_u16(400)
                                    .expect("Unable to turn 400 into a StatusCode");
                                response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for SEARCH_NF_INSTANCES_BAD_REQUEST"));
                                let body = serde_json::to_string(&body)
                                    .expect("impossible to fail to serialize");
                                *response.body_mut() = Body::from(body);
                            }
                            SearchNfInstancesResponse::Unauthorized(body) => {
                                *response.status_mut() = StatusCode::from_u16(401)
                                    .expect("Unable to turn 401 into a StatusCode");
                                response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for SEARCH_NF_INSTANCES_UNAUTHORIZED"));
                                let body = serde_json::to_string(&body)
                                    .expect("impossible to fail to serialize");
                                *response.body_mut() = Body::from(body);
                            }
                            SearchNfInstancesResponse::Forbidden(body) => {
                                *response.status_mut() = StatusCode::from_u16(403)
                                    .expect("Unable to turn 403 into a StatusCode");
                                response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for SEARCH_NF_INSTANCES_FORBIDDEN"));
                                let body = serde_json::to_string(&body)
                                    .expect("impossible to fail to serialize");
                                *response.body_mut() = Body::from(body);
                            }
                            SearchNfInstancesResponse::NotFound(body) => {
                                *response.status_mut() = StatusCode::from_u16(404)
                                    .expect("Unable to turn 404 into a StatusCode");
                                response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for SEARCH_NF_INSTANCES_NOT_FOUND"));
                                let body = serde_json::to_string(&body)
                                    .expect("impossible to fail to serialize");
                                *response.body_mut() = Body::from(body);
                            }
                            SearchNfInstancesResponse::Status406 => {
                                *response.status_mut() = StatusCode::from_u16(406)
                                    .expect("Unable to turn 406 into a StatusCode");
                            }
                            SearchNfInstancesResponse::LengthRequired(body) => {
                                *response.status_mut() = StatusCode::from_u16(411)
                                    .expect("Unable to turn 411 into a StatusCode");
                                response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for SEARCH_NF_INSTANCES_LENGTH_REQUIRED"));
                                let body = serde_json::to_string(&body)
                                    .expect("impossible to fail to serialize");
                                *response.body_mut() = Body::from(body);
                            }
                            SearchNfInstancesResponse::PayloadTooLarge(body) => {
                                *response.status_mut() = StatusCode::from_u16(413)
                                    .expect("Unable to turn 413 into a StatusCode");
                                response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for SEARCH_NF_INSTANCES_PAYLOAD_TOO_LARGE"));
                                let body = serde_json::to_string(&body)
                                    .expect("impossible to fail to serialize");
                                *response.body_mut() = Body::from(body);
                            }
                            SearchNfInstancesResponse::UnsupportedMediaType(body) => {
                                *response.status_mut() = StatusCode::from_u16(415)
                                    .expect("Unable to turn 415 into a StatusCode");
                                response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for SEARCH_NF_INSTANCES_UNSUPPORTED_MEDIA_TYPE"));
                                let body = serde_json::to_string(&body)
                                    .expect("impossible to fail to serialize");
                                *response.body_mut() = Body::from(body);
                            }
                            SearchNfInstancesResponse::TooManyRequests(body) => {
                                *response.status_mut() = StatusCode::from_u16(429)
                                    .expect("Unable to turn 429 into a StatusCode");
                                response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for SEARCH_NF_INSTANCES_TOO_MANY_REQUESTS"));
                                let body = serde_json::to_string(&body)
                                    .expect("impossible to fail to serialize");
                                *response.body_mut() = Body::from(body);
                            }
                            SearchNfInstancesResponse::InternalServerError(body) => {
                                *response.status_mut() = StatusCode::from_u16(500)
                                    .expect("Unable to turn 500 into a StatusCode");
                                response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for SEARCH_NF_INSTANCES_INTERNAL_SERVER_ERROR"));
                                let body = serde_json::to_string(&body)
                                    .expect("impossible to fail to serialize");
                                *response.body_mut() = Body::from(body);
                            }
                            SearchNfInstancesResponse::NotImplemented(body) => {
                                *response.status_mut() = StatusCode::from_u16(501)
                                    .expect("Unable to turn 501 into a StatusCode");
                                response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for SEARCH_NF_INSTANCES_NOT_IMPLEMENTED"));
                                let body = serde_json::to_string(&body)
                                    .expect("impossible to fail to serialize");
                                *response.body_mut() = Body::from(body);
                            }
                            SearchNfInstancesResponse::ServiceUnavailable(body) => {
                                *response.status_mut() = StatusCode::from_u16(503)
                                    .expect("Unable to turn 503 into a StatusCode");
                                response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for SEARCH_NF_INSTANCES_SERVICE_UNAVAILABLE"));
                                let body = serde_json::to_string(&body)
                                    .expect("impossible to fail to serialize");
                                *response.body_mut() = Body::from(body);
                            }
                            SearchNfInstancesResponse::GenericError => {
                                *response.status_mut() = StatusCode::from_u16(0)
                                    .expect("Unable to turn 0 into a StatusCode");
                            }
                        },
                        Err(_) => {
                            // Application code returned an error. This should not happen, as the implementation should
                            // return a valid response.
                            *response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
                            *response.body_mut() = Body::from("An internal error occurred");
                        }
                    }

                    Ok(response)
                }

                // SCpDomainRoutingInfoGet - GET /scp-domain-routing-info
                hyper::Method::GET if path.matched(paths::ID_SCP_DOMAIN_ROUTING_INFO) => {
                    {
                        let authorization = match *(&context as &dyn Has<Option<Authorization>>)
                            .get()
                        {
                            Some(ref authorization) => authorization,
                            None => {
                                return Ok(Response::builder()
                                    .status(StatusCode::FORBIDDEN)
                                    .body(Body::from("Unauthenticated"))
                                    .expect("Unable to create Authentication Forbidden response"))
                            }
                        };

                        // Authorization
                        if let Scopes::Some(ref scopes) = authorization.scopes {
                            let required_scopes: std::collections::BTreeSet<String> = vec![
                                "nnrf-disc".to_string(), // Access to the Nnrf_NFDiscovery API
                            ]
                            .into_iter()
                            .collect();

                            if !required_scopes.is_subset(scopes) {
                                let missing_scopes = required_scopes.difference(scopes);
                                return Ok(Response::builder()
                                    .status(StatusCode::FORBIDDEN)
                                    .body(Body::from(missing_scopes.fold(
                                        "Insufficient authorization, missing scopes".to_string(),
                                        |s, scope| format!("{} {}", s, scope),
                                    )))
                                    .expect(
                                        "Unable to create Authentication Insufficient response",
                                    ));
                            }
                        }
                    }

                    // Header parameters
                    let param_accept_encoding =
                        headers.get(HeaderName::from_static("accept-encoding"));

                    let param_accept_encoding = match param_accept_encoding {
                        Some(v) => {
                            match header::IntoHeaderValue::<String>::try_from((*v).clone()) {
                                Ok(result) => Some(result.0),
                                Err(err) => {
                                    return Ok(Response::builder()
                                        .status(StatusCode::BAD_REQUEST)
                                        .body(Body::from(format!("Invalid header Accept-Encoding - {}", err)))
                                        .expect("Unable to create Bad Request response for invalid header Accept-Encoding"));
                                }
                            }
                        }
                        None => None,
                    };

                    // Query parameters (note that non-required or collection query parameters will ignore garbage values, rather than causing a 400 response)
                    let query_params =
                        form_urlencoded::parse(uri.query().unwrap_or_default().as_bytes())
                            .collect::<Vec<_>>();
                    let param_local = query_params
                        .iter()
                        .filter(|e| e.0 == "local")
                        .map(|e| e.1.to_owned())
                        .next();
                    let param_local = match param_local {
                        Some(param_local) => {
                            let param_local = <bool as std::str::FromStr>::from_str(&param_local);
                            match param_local {
                            Ok(param_local) => Some(param_local),
                            Err(e) => return Ok(Response::builder()
                                .status(StatusCode::BAD_REQUEST)
                                .body(Body::from(format!("Couldn't parse query parameter local - doesn't match schema: {}", e)))
                                .expect("Unable to create Bad Request response for invalid query parameter local")),
                        }
                        }
                        None => None,
                    };

                    let result = api_impl
                        .scp_domain_routing_info_get(param_local, param_accept_encoding, &context)
                        .await;
                    let mut response = Response::new(Body::empty());
                    response.headers_mut().insert(
                        HeaderName::from_static("x-span-id"),
                        HeaderValue::from_str(
                            (&context as &dyn Has<XSpanIdString>)
                                .get()
                                .0
                                .clone()
                                .as_str(),
                        )
                        .expect("Unable to create X-Span-ID header value"),
                    );

                    match result {
                        Ok(rsp) => match rsp {
                            SCpDomainRoutingInfoGetResponse::ExpectedResponseToAValidRequest {
                                body,
                                content_encoding,
                            } => {
                                if let Some(content_encoding) = content_encoding {
                                    let content_encoding = match header::IntoHeaderValue(content_encoding).try_into() {
                                                        Ok(val) => val,
                                                        Err(e) => {
                                                            return Ok(Response::builder()
                                                                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                                                                    .body(Body::from(format!("An internal server error occurred handling content_encoding header - {}", e)))
                                                                    .expect("Unable to create Internal Server Error for invalid response header"))
                                                        }
                                                    };

                                    response.headers_mut().insert(
                                        HeaderName::from_static("content-encoding"),
                                        content_encoding,
                                    );
                                }
                                *response.status_mut() = StatusCode::from_u16(200)
                                    .expect("Unable to turn 200 into a StatusCode");
                                response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for SCP_DOMAIN_ROUTING_INFO_GET_EXPECTED_RESPONSE_TO_A_VALID_REQUEST"));
                                let body = serde_json::to_string(&body)
                                    .expect("impossible to fail to serialize");
                                *response.body_mut() = Body::from(body);
                            }
                            SCpDomainRoutingInfoGetResponse::TemporaryRedirect { location } => {
                                let location = match header::IntoHeaderValue(location).try_into() {
                                                        Ok(val) => val,
                                                        Err(e) => {
                                                            return Ok(Response::builder()
                                                                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                                                                    .body(Body::from(format!("An internal server error occurred handling location header - {}", e)))
                                                                    .expect("Unable to create Internal Server Error for invalid response header"))
                                                        }
                                                    };

                                response
                                    .headers_mut()
                                    .insert(HeaderName::from_static("location"), location);
                                *response.status_mut() = StatusCode::from_u16(307)
                                    .expect("Unable to turn 307 into a StatusCode");
                            }
                            SCpDomainRoutingInfoGetResponse::BadRequest(body) => {
                                *response.status_mut() = StatusCode::from_u16(400)
                                    .expect("Unable to turn 400 into a StatusCode");
                                response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for SCP_DOMAIN_ROUTING_INFO_GET_BAD_REQUEST"));
                                let body = serde_json::to_string(&body)
                                    .expect("impossible to fail to serialize");
                                *response.body_mut() = Body::from(body);
                            }
                            SCpDomainRoutingInfoGetResponse::Unauthorized(body) => {
                                *response.status_mut() = StatusCode::from_u16(401)
                                    .expect("Unable to turn 401 into a StatusCode");
                                response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for SCP_DOMAIN_ROUTING_INFO_GET_UNAUTHORIZED"));
                                let body = serde_json::to_string(&body)
                                    .expect("impossible to fail to serialize");
                                *response.body_mut() = Body::from(body);
                            }
                            SCpDomainRoutingInfoGetResponse::Forbidden(body) => {
                                *response.status_mut() = StatusCode::from_u16(403)
                                    .expect("Unable to turn 403 into a StatusCode");
                                response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for SCP_DOMAIN_ROUTING_INFO_GET_FORBIDDEN"));
                                let body = serde_json::to_string(&body)
                                    .expect("impossible to fail to serialize");
                                *response.body_mut() = Body::from(body);
                            }
                            SCpDomainRoutingInfoGetResponse::NotFound(body) => {
                                *response.status_mut() = StatusCode::from_u16(404)
                                    .expect("Unable to turn 404 into a StatusCode");
                                response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for SCP_DOMAIN_ROUTING_INFO_GET_NOT_FOUND"));
                                let body = serde_json::to_string(&body)
                                    .expect("impossible to fail to serialize");
                                *response.body_mut() = Body::from(body);
                            }
                            SCpDomainRoutingInfoGetResponse::Status406 => {
                                *response.status_mut() = StatusCode::from_u16(406)
                                    .expect("Unable to turn 406 into a StatusCode");
                            }
                            SCpDomainRoutingInfoGetResponse::LengthRequired(body) => {
                                *response.status_mut() = StatusCode::from_u16(411)
                                    .expect("Unable to turn 411 into a StatusCode");
                                response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for SCP_DOMAIN_ROUTING_INFO_GET_LENGTH_REQUIRED"));
                                let body = serde_json::to_string(&body)
                                    .expect("impossible to fail to serialize");
                                *response.body_mut() = Body::from(body);
                            }
                            SCpDomainRoutingInfoGetResponse::PayloadTooLarge(body) => {
                                *response.status_mut() = StatusCode::from_u16(413)
                                    .expect("Unable to turn 413 into a StatusCode");
                                response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for SCP_DOMAIN_ROUTING_INFO_GET_PAYLOAD_TOO_LARGE"));
                                let body = serde_json::to_string(&body)
                                    .expect("impossible to fail to serialize");
                                *response.body_mut() = Body::from(body);
                            }
                            SCpDomainRoutingInfoGetResponse::UnsupportedMediaType(body) => {
                                *response.status_mut() = StatusCode::from_u16(415)
                                    .expect("Unable to turn 415 into a StatusCode");
                                response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for SCP_DOMAIN_ROUTING_INFO_GET_UNSUPPORTED_MEDIA_TYPE"));
                                let body = serde_json::to_string(&body)
                                    .expect("impossible to fail to serialize");
                                *response.body_mut() = Body::from(body);
                            }
                            SCpDomainRoutingInfoGetResponse::TooManyRequests(body) => {
                                *response.status_mut() = StatusCode::from_u16(429)
                                    .expect("Unable to turn 429 into a StatusCode");
                                response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for SCP_DOMAIN_ROUTING_INFO_GET_TOO_MANY_REQUESTS"));
                                let body = serde_json::to_string(&body)
                                    .expect("impossible to fail to serialize");
                                *response.body_mut() = Body::from(body);
                            }
                            SCpDomainRoutingInfoGetResponse::InternalServerError(body) => {
                                *response.status_mut() = StatusCode::from_u16(500)
                                    .expect("Unable to turn 500 into a StatusCode");
                                response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for SCP_DOMAIN_ROUTING_INFO_GET_INTERNAL_SERVER_ERROR"));
                                let body = serde_json::to_string(&body)
                                    .expect("impossible to fail to serialize");
                                *response.body_mut() = Body::from(body);
                            }
                            SCpDomainRoutingInfoGetResponse::NotImplemented(body) => {
                                *response.status_mut() = StatusCode::from_u16(501)
                                    .expect("Unable to turn 501 into a StatusCode");
                                response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for SCP_DOMAIN_ROUTING_INFO_GET_NOT_IMPLEMENTED"));
                                let body = serde_json::to_string(&body)
                                    .expect("impossible to fail to serialize");
                                *response.body_mut() = Body::from(body);
                            }
                            SCpDomainRoutingInfoGetResponse::ServiceUnavailable(body) => {
                                *response.status_mut() = StatusCode::from_u16(503)
                                    .expect("Unable to turn 503 into a StatusCode");
                                response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for SCP_DOMAIN_ROUTING_INFO_GET_SERVICE_UNAVAILABLE"));
                                let body = serde_json::to_string(&body)
                                    .expect("impossible to fail to serialize");
                                *response.body_mut() = Body::from(body);
                            }
                            SCpDomainRoutingInfoGetResponse::GenericError => {
                                *response.status_mut() = StatusCode::from_u16(0)
                                    .expect("Unable to turn 0 into a StatusCode");
                            }
                        },
                        Err(_) => {
                            // Application code returned an error. This should not happen, as the implementation should
                            // return a valid response.
                            *response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
                            *response.body_mut() = Body::from("An internal error occurred");
                        }
                    }

                    Ok(response)
                }

                // ScpDomainRoutingInfoSubscribe - POST /scp-domain-routing-info-subs
                hyper::Method::POST if path.matched(paths::ID_SCP_DOMAIN_ROUTING_INFO_SUBS) => {
                    {
                        let authorization = match *(&context as &dyn Has<Option<Authorization>>)
                            .get()
                        {
                            Some(ref authorization) => authorization,
                            None => {
                                return Ok(Response::builder()
                                    .status(StatusCode::FORBIDDEN)
                                    .body(Body::from("Unauthenticated"))
                                    .expect("Unable to create Authentication Forbidden response"))
                            }
                        };

                        // Authorization
                        if let Scopes::Some(ref scopes) = authorization.scopes {
                            let required_scopes: std::collections::BTreeSet<String> = vec![
                                "nnrf-disc".to_string(), // Access to the Nnrf_NFDiscovery API
                            ]
                            .into_iter()
                            .collect();

                            if !required_scopes.is_subset(scopes) {
                                let missing_scopes = required_scopes.difference(scopes);
                                return Ok(Response::builder()
                                    .status(StatusCode::FORBIDDEN)
                                    .body(Body::from(missing_scopes.fold(
                                        "Insufficient authorization, missing scopes".to_string(),
                                        |s, scope| format!("{} {}", s, scope),
                                    )))
                                    .expect(
                                        "Unable to create Authentication Insufficient response",
                                    ));
                            }
                        }
                    }

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
                    let param_accept_encoding =
                        headers.get(HeaderName::from_static("accept-encoding"));

                    let param_accept_encoding = match param_accept_encoding {
                        Some(v) => {
                            match header::IntoHeaderValue::<String>::try_from((*v).clone()) {
                                Ok(result) => Some(result.0),
                                Err(err) => {
                                    return Ok(Response::builder()
                                        .status(StatusCode::BAD_REQUEST)
                                        .body(Body::from(format!("Invalid header Accept-Encoding - {}", err)))
                                        .expect("Unable to create Bad Request response for invalid header Accept-Encoding"));
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
                                let param_scp_domain_routing_info_subscription: Option<models::ScpDomainRoutingInfoSubscription> = if !body.is_empty() {
                                    let deserializer = &mut serde_json::Deserializer::from_slice(&*body);
                                    match serde_ignored::deserialize(deserializer, |path| {
                                            warn!("Ignoring unknown field in body: {}", path);
                                            unused_elements.push(path.to_string());
                                    }) {
                                        Ok(param_scp_domain_routing_info_subscription) => param_scp_domain_routing_info_subscription,
                                        Err(e) => return Ok(Response::builder()
                                                        .status(StatusCode::BAD_REQUEST)
                                                        .body(Body::from(format!("Couldn't parse body parameter ScpDomainRoutingInfoSubscription - doesn't match schema: {}", e)))
                                                        .expect("Unable to create Bad Request response for invalid body parameter ScpDomainRoutingInfoSubscription due to schema")),
                                    }
                                } else {
                                    None
                                };
                                let param_scp_domain_routing_info_subscription = match param_scp_domain_routing_info_subscription {
                                    Some(param_scp_domain_routing_info_subscription) => param_scp_domain_routing_info_subscription,
                                    None => return Ok(Response::builder()
                                                        .status(StatusCode::BAD_REQUEST)
                                                        .body(Body::from("Missing required body parameter ScpDomainRoutingInfoSubscription"))
                                                        .expect("Unable to create Bad Request response for missing body parameter ScpDomainRoutingInfoSubscription")),
                                };

                                let result = api_impl.scp_domain_routing_info_subscribe(
                                            param_scp_domain_routing_info_subscription,
                                            param_content_encoding,
                                            param_accept_encoding,
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
                                                ScpDomainRoutingInfoSubscribeResponse::ExpectedResponseToAValidRequest
                                                    {
                                                        body,
                                                        location,
                                                        accept_encoding,
                                                        content_encoding
                                                    }
                                                => {
                                                    let location = match header::IntoHeaderValue(location).try_into() {
                                                        Ok(val) => val,
                                                        Err(e) => {
                                                            return Ok(Response::builder()
                                                                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                                                                    .body(Body::from(format!("An internal server error occurred handling location header - {}", e)))
                                                                    .expect("Unable to create Internal Server Error for invalid response header"))
                                                        }
                                                    };

                                                    response.headers_mut().insert(
                                                        HeaderName::from_static("location"),
                                                        location
                                                    );
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
                                                    if let Some(content_encoding) = content_encoding {
                                                    let content_encoding = match header::IntoHeaderValue(content_encoding).try_into() {
                                                        Ok(val) => val,
                                                        Err(e) => {
                                                            return Ok(Response::builder()
                                                                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                                                                    .body(Body::from(format!("An internal server error occurred handling content_encoding header - {}", e)))
                                                                    .expect("Unable to create Internal Server Error for invalid response header"))
                                                        }
                                                    };

                                                    response.headers_mut().insert(
                                                        HeaderName::from_static("content-encoding"),
                                                        content_encoding
                                                    );
                                                    }
                                                    *response.status_mut() = StatusCode::from_u16(201).expect("Unable to turn 201 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for SCP_DOMAIN_ROUTING_INFO_SUBSCRIBE_EXPECTED_RESPONSE_TO_A_VALID_REQUEST"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                ScpDomainRoutingInfoSubscribeResponse::BadRequest
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(400).expect("Unable to turn 400 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for SCP_DOMAIN_ROUTING_INFO_SUBSCRIBE_BAD_REQUEST"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                ScpDomainRoutingInfoSubscribeResponse::Unauthorized
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(401).expect("Unable to turn 401 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for SCP_DOMAIN_ROUTING_INFO_SUBSCRIBE_UNAUTHORIZED"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                ScpDomainRoutingInfoSubscribeResponse::Forbidden
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(403).expect("Unable to turn 403 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for SCP_DOMAIN_ROUTING_INFO_SUBSCRIBE_FORBIDDEN"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                ScpDomainRoutingInfoSubscribeResponse::NotFound
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(404).expect("Unable to turn 404 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for SCP_DOMAIN_ROUTING_INFO_SUBSCRIBE_NOT_FOUND"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                ScpDomainRoutingInfoSubscribeResponse::LengthRequired
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(411).expect("Unable to turn 411 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for SCP_DOMAIN_ROUTING_INFO_SUBSCRIBE_LENGTH_REQUIRED"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                ScpDomainRoutingInfoSubscribeResponse::PayloadTooLarge
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(413).expect("Unable to turn 413 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for SCP_DOMAIN_ROUTING_INFO_SUBSCRIBE_PAYLOAD_TOO_LARGE"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                ScpDomainRoutingInfoSubscribeResponse::UnsupportedMediaType
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(415).expect("Unable to turn 415 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for SCP_DOMAIN_ROUTING_INFO_SUBSCRIBE_UNSUPPORTED_MEDIA_TYPE"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                ScpDomainRoutingInfoSubscribeResponse::TooManyRequests
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(429).expect("Unable to turn 429 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for SCP_DOMAIN_ROUTING_INFO_SUBSCRIBE_TOO_MANY_REQUESTS"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                ScpDomainRoutingInfoSubscribeResponse::InternalServerError
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(500).expect("Unable to turn 500 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for SCP_DOMAIN_ROUTING_INFO_SUBSCRIBE_INTERNAL_SERVER_ERROR"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                ScpDomainRoutingInfoSubscribeResponse::NotImplemented
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(501).expect("Unable to turn 501 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for SCP_DOMAIN_ROUTING_INFO_SUBSCRIBE_NOT_IMPLEMENTED"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                ScpDomainRoutingInfoSubscribeResponse::ServiceUnavailable
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(503).expect("Unable to turn 503 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for SCP_DOMAIN_ROUTING_INFO_SUBSCRIBE_SERVICE_UNAVAILABLE"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                ScpDomainRoutingInfoSubscribeResponse::GenericError
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
                                                .body(Body::from(format!("Couldn't read body parameter ScpDomainRoutingInfoSubscription: {}", e)))
                                                .expect("Unable to create Bad Request response due to unable to read body parameter ScpDomainRoutingInfoSubscription")),
                        }
                }

                // RetrieveStoredSearch - GET /searches/{searchId}
                hyper::Method::GET if path.matched(paths::ID_SEARCHES_SEARCHID) => {
                    {
                        let authorization = match *(&context as &dyn Has<Option<Authorization>>)
                            .get()
                        {
                            Some(ref authorization) => authorization,
                            None => {
                                return Ok(Response::builder()
                                    .status(StatusCode::FORBIDDEN)
                                    .body(Body::from("Unauthenticated"))
                                    .expect("Unable to create Authentication Forbidden response"))
                            }
                        };

                        // Authorization
                        if let Scopes::Some(ref scopes) = authorization.scopes {
                            let required_scopes: std::collections::BTreeSet<String> = vec![
                                "nnrf-disc".to_string(), // Access to the Nnrf_NFDiscovery API
                            ]
                            .into_iter()
                            .collect();

                            if !required_scopes.is_subset(scopes) {
                                let missing_scopes = required_scopes.difference(scopes);
                                return Ok(Response::builder()
                                    .status(StatusCode::FORBIDDEN)
                                    .body(Body::from(missing_scopes.fold(
                                        "Insufficient authorization, missing scopes".to_string(),
                                        |s, scope| format!("{} {}", s, scope),
                                    )))
                                    .expect(
                                        "Unable to create Authentication Insufficient response",
                                    ));
                            }
                        }
                    }

                    // Path parameters
                    let path: &str = uri.path();
                    let path_params =
                    paths::REGEX_SEARCHES_SEARCHID
                    .captures(path)
                    .unwrap_or_else(||
                        panic!("Path {} matched RE SEARCHES_SEARCHID in set but failed match against \"{}\"", path, paths::REGEX_SEARCHES_SEARCHID.as_str())
                    );

                    let param_search_id = match percent_encoding::percent_decode(path_params["searchId"].as_bytes()).decode_utf8() {
                    Ok(param_search_id) => match param_search_id.parse::<String>() {
                        Ok(param_search_id) => param_search_id,
                        Err(e) => return Ok(Response::builder()
                                        .status(StatusCode::BAD_REQUEST)
                                        .body(Body::from(format!("Couldn't parse path parameter searchId: {}", e)))
                                        .expect("Unable to create Bad Request response for invalid path parameter")),
                    },
                    Err(_) => return Ok(Response::builder()
                                        .status(StatusCode::BAD_REQUEST)
                                        .body(Body::from(format!("Couldn't percent-decode path parameter as UTF-8: {}", &path_params["searchId"])))
                                        .expect("Unable to create Bad Request response for invalid percent decode"))
                };

                    // Header parameters
                    let param_accept_encoding =
                        headers.get(HeaderName::from_static("accept-encoding"));

                    let param_accept_encoding = match param_accept_encoding {
                        Some(v) => {
                            match header::IntoHeaderValue::<String>::try_from((*v).clone()) {
                                Ok(result) => Some(result.0),
                                Err(err) => {
                                    return Ok(Response::builder()
                                        .status(StatusCode::BAD_REQUEST)
                                        .body(Body::from(format!("Invalid header Accept-Encoding - {}", err)))
                                        .expect("Unable to create Bad Request response for invalid header Accept-Encoding"));
                                }
                            }
                        }
                        None => None,
                    };

                    let result = api_impl
                        .retrieve_stored_search(param_search_id, param_accept_encoding, &context)
                        .await;
                    let mut response = Response::new(Body::empty());
                    response.headers_mut().insert(
                        HeaderName::from_static("x-span-id"),
                        HeaderValue::from_str(
                            (&context as &dyn Has<XSpanIdString>)
                                .get()
                                .0
                                .clone()
                                .as_str(),
                        )
                        .expect("Unable to create X-Span-ID header value"),
                    );

                    match result {
                        Ok(rsp) => match rsp {
                            RetrieveStoredSearchResponse::ExpectedResponseToAValidRequest {
                                body,
                                cache_control,
                                e_tag,
                                content_encoding,
                            } => {
                                if let Some(cache_control) = cache_control {
                                    let cache_control = match header::IntoHeaderValue(cache_control).try_into() {
                                                        Ok(val) => val,
                                                        Err(e) => {
                                                            return Ok(Response::builder()
                                                                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                                                                    .body(Body::from(format!("An internal server error occurred handling cache_control header - {}", e)))
                                                                    .expect("Unable to create Internal Server Error for invalid response header"))
                                                        }
                                                    };

                                    response.headers_mut().insert(
                                        HeaderName::from_static("cache-control"),
                                        cache_control,
                                    );
                                }
                                if let Some(e_tag) = e_tag {
                                    let e_tag = match header::IntoHeaderValue(e_tag).try_into() {
                                                        Ok(val) => val,
                                                        Err(e) => {
                                                            return Ok(Response::builder()
                                                                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                                                                    .body(Body::from(format!("An internal server error occurred handling e_tag header - {}", e)))
                                                                    .expect("Unable to create Internal Server Error for invalid response header"))
                                                        }
                                                    };

                                    response
                                        .headers_mut()
                                        .insert(HeaderName::from_static("etag"), e_tag);
                                }
                                if let Some(content_encoding) = content_encoding {
                                    let content_encoding = match header::IntoHeaderValue(content_encoding).try_into() {
                                                        Ok(val) => val,
                                                        Err(e) => {
                                                            return Ok(Response::builder()
                                                                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                                                                    .body(Body::from(format!("An internal server error occurred handling content_encoding header - {}", e)))
                                                                    .expect("Unable to create Internal Server Error for invalid response header"))
                                                        }
                                                    };

                                    response.headers_mut().insert(
                                        HeaderName::from_static("content-encoding"),
                                        content_encoding,
                                    );
                                }
                                *response.status_mut() = StatusCode::from_u16(200)
                                    .expect("Unable to turn 200 into a StatusCode");
                                response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for RETRIEVE_STORED_SEARCH_EXPECTED_RESPONSE_TO_A_VALID_REQUEST"));
                                let body = serde_json::to_string(&body)
                                    .expect("impossible to fail to serialize");
                                *response.body_mut() = Body::from(body);
                            }
                            RetrieveStoredSearchResponse::TemporaryRedirect { body, location } => {
                                let location = match header::IntoHeaderValue(location).try_into() {
                                                        Ok(val) => val,
                                                        Err(e) => {
                                                            return Ok(Response::builder()
                                                                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                                                                    .body(Body::from(format!("An internal server error occurred handling location header - {}", e)))
                                                                    .expect("Unable to create Internal Server Error for invalid response header"))
                                                        }
                                                    };

                                response
                                    .headers_mut()
                                    .insert(HeaderName::from_static("location"), location);
                                *response.status_mut() = StatusCode::from_u16(307)
                                    .expect("Unable to turn 307 into a StatusCode");
                                response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for RETRIEVE_STORED_SEARCH_TEMPORARY_REDIRECT"));
                                let body = serde_json::to_string(&body)
                                    .expect("impossible to fail to serialize");
                                *response.body_mut() = Body::from(body);
                            }
                            RetrieveStoredSearchResponse::PermanentRedirect { body, location } => {
                                let location = match header::IntoHeaderValue(location).try_into() {
                                                        Ok(val) => val,
                                                        Err(e) => {
                                                            return Ok(Response::builder()
                                                                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                                                                    .body(Body::from(format!("An internal server error occurred handling location header - {}", e)))
                                                                    .expect("Unable to create Internal Server Error for invalid response header"))
                                                        }
                                                    };

                                response
                                    .headers_mut()
                                    .insert(HeaderName::from_static("location"), location);
                                *response.status_mut() = StatusCode::from_u16(308)
                                    .expect("Unable to turn 308 into a StatusCode");
                                response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for RETRIEVE_STORED_SEARCH_PERMANENT_REDIRECT"));
                                let body = serde_json::to_string(&body)
                                    .expect("impossible to fail to serialize");
                                *response.body_mut() = Body::from(body);
                            }
                        },
                        Err(_) => {
                            // Application code returned an error. This should not happen, as the implementation should
                            // return a valid response.
                            *response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
                            *response.body_mut() = Body::from("An internal error occurred");
                        }
                    }

                    Ok(response)
                }

                _ if path.matched(paths::ID_NF_INSTANCES) => method_not_allowed(),
                _ if path.matched(paths::ID_SCP_DOMAIN_ROUTING_INFO) => method_not_allowed(),
                _ if path.matched(paths::ID_SCP_DOMAIN_ROUTING_INFO_SUBS) => method_not_allowed(),
                _ if path.matched(paths::ID_SCP_DOMAIN_ROUTING_INFO_SUBS_SUBSCRIPTIONID) => {
                    method_not_allowed()
                }
                _ if path.matched(paths::ID_SEARCHES_SEARCHID) => method_not_allowed(),
                _ if path.matched(paths::ID_SEARCHES_SEARCHID_COMPLETE) => method_not_allowed(),
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
            // RetrieveCompleteSearch - GET /searches/{searchId}/complete
            hyper::Method::GET if path.matched(paths::ID_SEARCHES_SEARCHID_COMPLETE) => {
                Some("RetrieveCompleteSearch")
            }
            // ScpDomainRoutingInfoUnsubscribe - DELETE /scp-domain-routing-info-subs/{subscriptionID}
            hyper::Method::DELETE
                if path.matched(paths::ID_SCP_DOMAIN_ROUTING_INFO_SUBS_SUBSCRIPTIONID) =>
            {
                Some("ScpDomainRoutingInfoUnsubscribe")
            }
            // SearchNfInstances - GET /nf-instances
            hyper::Method::GET if path.matched(paths::ID_NF_INSTANCES) => Some("SearchNfInstances"),
            // SCpDomainRoutingInfoGet - GET /scp-domain-routing-info
            hyper::Method::GET if path.matched(paths::ID_SCP_DOMAIN_ROUTING_INFO) => {
                Some("SCpDomainRoutingInfoGet")
            }
            // ScpDomainRoutingInfoSubscribe - POST /scp-domain-routing-info-subs
            hyper::Method::POST if path.matched(paths::ID_SCP_DOMAIN_ROUTING_INFO_SUBS) => {
                Some("ScpDomainRoutingInfoSubscribe")
            }
            // RetrieveStoredSearch - GET /searches/{searchId}
            hyper::Method::GET if path.matched(paths::ID_SEARCHES_SEARCHID) => {
                Some("RetrieveStoredSearch")
            }
            _ => None,
        }
    }
}
