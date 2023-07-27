use futures::{future, future::BoxFuture, future::FutureExt, stream, stream::TryStreamExt, Stream};
use hyper::header::{HeaderName, HeaderValue, CONTENT_TYPE};
use hyper::{Body, HeaderMap, Request, Response, StatusCode};
use hyper_0_10::header::{ContentType, Headers};
use log::warn;
use mime_0_2::{Mime as Mime2, SubLevel, TopLevel};
use mime_multipart::{read_multipart_body, Node, Part};
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
use crate::ModifyPduSessionIsmfResponse;
use crate::ModifyPduSessionResponse;
use crate::NotifyStatusIsfmResponse;
use crate::NotifyStatusResponse;
use crate::SmContextStatusNotificationPostResponse;
use crate::TransferMtDataIsmfResponse;
use crate::TransferMtDataResponse;

mod paths {
    use lazy_static::lazy_static;

    lazy_static! {
        pub static ref GLOBAL_REGEX_SET: regex::RegexSet = regex::RegexSet::new(vec![
            r"^/nsmf-pdusession/v1/(?P<request_body_ismf_pdu_session_uri>.*)$",
            r"^/nsmf-pdusession/v1/(?P<request_body_ismf_pdu_session_uri>.*)/modify$",
            r"^/nsmf-pdusession/v1/(?P<request_body_ismf_pdu_session_uri>.*)/transfer-mt-data$",
            r"^/nsmf-pdusession/v1/(?P<request_body_sm_context_status_uri>.*)$",
            r"^/nsmf-pdusession/v1/(?P<request_body_vsmf_pdu_session_uri>.*)$",
            r"^/nsmf-pdusession/v1/(?P<request_body_vsmf_pdu_session_uri>.*)/modify$",
            r"^/nsmf-pdusession/v1/(?P<request_body_vsmf_pdu_session_uri>.*)/transfer-mt-data$"
        ])
        .expect("Unable to create global regex set");
    }
    pub(crate) static ID_REQUEST_BODY_ISMFPDUSESSIONURI: usize = 0;
    lazy_static! {
        pub static ref REGEX_REQUEST_BODY_ISMFPDUSESSIONURI: regex::Regex =
            #[allow(clippy::invalid_regex)]
            regex::Regex::new(r"^/nsmf-pdusession/v1/(?P<request_body_ismf_pdu_session_uri>.*)$")
                .expect("Unable to create regex for REQUEST_BODY_ISMFPDUSESSIONURI");
    }
    pub(crate) static ID_REQUEST_BODY_ISMFPDUSESSIONURI_MODIFY: usize = 1;
    lazy_static! {
        pub static ref REGEX_REQUEST_BODY_ISMFPDUSESSIONURI_MODIFY: regex::Regex =
            #[allow(clippy::invalid_regex)]
            regex::Regex::new(
                r"^/nsmf-pdusession/v1/(?P<request_body_ismf_pdu_session_uri>.*)/modify$"
            )
            .expect("Unable to create regex for REQUEST_BODY_ISMFPDUSESSIONURI_MODIFY");
    }
    pub(crate) static ID_REQUEST_BODY_ISMFPDUSESSIONURI_TRANSFER_MT_DATA: usize = 2;
    lazy_static! {
        pub static ref REGEX_REQUEST_BODY_ISMFPDUSESSIONURI_TRANSFER_MT_DATA: regex::Regex =
            #[allow(clippy::invalid_regex)]
            regex::Regex::new(
                r"^/nsmf-pdusession/v1/(?P<request_body_ismf_pdu_session_uri>.*)/transfer-mt-data$"
            )
            .expect("Unable to create regex for REQUEST_BODY_ISMFPDUSESSIONURI_TRANSFER_MT_DATA");
    }
    pub(crate) static ID_REQUEST_BODY_SMCONTEXTSTATUSURI: usize = 3;
    lazy_static! {
        pub static ref REGEX_REQUEST_BODY_SMCONTEXTSTATUSURI: regex::Regex =
            #[allow(clippy::invalid_regex)]
            regex::Regex::new(r"^/nsmf-pdusession/v1/(?P<request_body_sm_context_status_uri>.*)$")
                .expect("Unable to create regex for REQUEST_BODY_SMCONTEXTSTATUSURI");
    }
    pub(crate) static ID_REQUEST_BODY_VSMFPDUSESSIONURI: usize = 4;
    lazy_static! {
        pub static ref REGEX_REQUEST_BODY_VSMFPDUSESSIONURI: regex::Regex =
            #[allow(clippy::invalid_regex)]
            regex::Regex::new(r"^/nsmf-pdusession/v1/(?P<request_body_vsmf_pdu_session_uri>.*)$")
                .expect("Unable to create regex for REQUEST_BODY_VSMFPDUSESSIONURI");
    }
    pub(crate) static ID_REQUEST_BODY_VSMFPDUSESSIONURI_MODIFY: usize = 5;
    lazy_static! {
        pub static ref REGEX_REQUEST_BODY_VSMFPDUSESSIONURI_MODIFY: regex::Regex =
            #[allow(clippy::invalid_regex)]
            regex::Regex::new(
                r"^/nsmf-pdusession/v1/(?P<request_body_vsmf_pdu_session_uri>.*)/modify$"
            )
            .expect("Unable to create regex for REQUEST_BODY_VSMFPDUSESSIONURI_MODIFY");
    }
    pub(crate) static ID_REQUEST_BODY_VSMFPDUSESSIONURI_TRANSFER_MT_DATA: usize = 6;
    lazy_static! {
        pub static ref REGEX_REQUEST_BODY_VSMFPDUSESSIONURI_TRANSFER_MT_DATA: regex::Regex =
            #[allow(clippy::invalid_regex)]
            regex::Regex::new(
                r"^/nsmf-pdusession/v1/(?P<request_body_vsmf_pdu_session_uri>.*)/transfer-mt-data$"
            )
            .expect("Unable to create regex for REQUEST_BODY_VSMFPDUSESSIONURI_TRANSFER_MT_DATA");
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
                // NotifyStatus - POST /{$request.body#/vsmfPduSessionUri}
                hyper::Method::POST if path.matched(paths::ID_REQUEST_BODY_VSMFPDUSESSIONURI) => {
                    // Path parameters
                    let path: &str = uri.path();
                    let path_params =
                        paths::REGEX_REQUEST_BODY_VSMFPDUSESSIONURI
                        .captures(path)
                        .unwrap_or_else(||
                            panic!("Path {} matched RE REQUEST_BODY_VSMFPDUSESSIONURI in set but failed match against \"{}\"", path, paths::REGEX_REQUEST_BODY_VSMFPDUSESSIONURI.as_str())
                        );

                    let callback_request_body_vsmf_pdu_session_uri =
                        path_params["request_body_vsmf_pdu_session_uri"].to_string();
                    // Body parameters (note that non-required body parameters will ignore garbage
                    // values, rather than causing a 400 response). Produce warning header and logs for
                    // any unused fields.
                    let result = body.into_raw().await;
                    match result {
                                Ok(body) => {
                                let mut unused_elements = Vec::new();
                                let param_status_notification: Option<models::StatusNotification> = if !body.is_empty() {
                                    let deserializer = &mut serde_json::Deserializer::from_slice(&*body);
                                    match serde_ignored::deserialize(deserializer, |path| {
                                            warn!("Ignoring unknown field in body: {}", path);
                                            unused_elements.push(path.to_string());
                                    }) {
                                        Ok(param_status_notification) => param_status_notification,
                                        Err(e) => return Ok(Response::builder()
                                                        .status(StatusCode::BAD_REQUEST)
                                                        .body(Body::from(format!("Couldn't parse body parameter StatusNotification - doesn't match schema: {}", e)))
                                                        .expect("Unable to create Bad Request response for invalid body parameter StatusNotification due to schema")),
                                    }
                                } else {
                                    None
                                };
                                let param_status_notification = match param_status_notification {
                                    Some(param_status_notification) => param_status_notification,
                                    None => return Ok(Response::builder()
                                                        .status(StatusCode::BAD_REQUEST)
                                                        .body(Body::from("Missing required body parameter StatusNotification"))
                                                        .expect("Unable to create Bad Request response for missing body parameter StatusNotification")),
                                };

                                let result = api_impl.notify_status(
                                            callback_request_body_vsmf_pdu_session_uri,
                                            param_status_notification,
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
                                                NotifyStatusResponse::SuccessfulNotificationofTheStatusChange
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(204).expect("Unable to turn 204 into a StatusCode");
                                                },
                                                NotifyStatusResponse::TemporaryRedirect
                                                    {
                                                        body,
                                                        location,
                                                        param_3gpp_sbi_target_nf_id
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
                                                    if let Some(param_3gpp_sbi_target_nf_id) = param_3gpp_sbi_target_nf_id {
                                                    let param_3gpp_sbi_target_nf_id = match header::IntoHeaderValue(param_3gpp_sbi_target_nf_id).try_into() {
                                                        Ok(val) => val,
                                                        Err(e) => {
                                                            return Ok(Response::builder()
                                                                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                                                                    .body(Body::from(format!("An internal server error occurred handling param_3gpp_sbi_target_nf_id header - {}", e)))
                                                                    .expect("Unable to create Internal Server Error for invalid response header"))
                                                        }
                                                    };

                                                    response.headers_mut().insert(
                                                        HeaderName::from_static("3gpp-sbi-target-nf-id"),
                                                        param_3gpp_sbi_target_nf_id
                                                    );
                                                    }
                                                    *response.status_mut() = StatusCode::from_u16(307).expect("Unable to turn 307 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for NOTIFY_STATUS_TEMPORARY_REDIRECT"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                NotifyStatusResponse::PermanentRedirect
                                                    {
                                                        body,
                                                        location,
                                                        param_3gpp_sbi_target_nf_id
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
                                                    if let Some(param_3gpp_sbi_target_nf_id) = param_3gpp_sbi_target_nf_id {
                                                    let param_3gpp_sbi_target_nf_id = match header::IntoHeaderValue(param_3gpp_sbi_target_nf_id).try_into() {
                                                        Ok(val) => val,
                                                        Err(e) => {
                                                            return Ok(Response::builder()
                                                                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                                                                    .body(Body::from(format!("An internal server error occurred handling param_3gpp_sbi_target_nf_id header - {}", e)))
                                                                    .expect("Unable to create Internal Server Error for invalid response header"))
                                                        }
                                                    };

                                                    response.headers_mut().insert(
                                                        HeaderName::from_static("3gpp-sbi-target-nf-id"),
                                                        param_3gpp_sbi_target_nf_id
                                                    );
                                                    }
                                                    *response.status_mut() = StatusCode::from_u16(308).expect("Unable to turn 308 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for NOTIFY_STATUS_PERMANENT_REDIRECT"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                NotifyStatusResponse::BadRequest
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(400).expect("Unable to turn 400 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for NOTIFY_STATUS_BAD_REQUEST"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                NotifyStatusResponse::Forbidden
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(403).expect("Unable to turn 403 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for NOTIFY_STATUS_FORBIDDEN"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                NotifyStatusResponse::NotFound
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(404).expect("Unable to turn 404 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for NOTIFY_STATUS_NOT_FOUND"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                NotifyStatusResponse::LengthRequired
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(411).expect("Unable to turn 411 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for NOTIFY_STATUS_LENGTH_REQUIRED"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                NotifyStatusResponse::PayloadTooLarge
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(413).expect("Unable to turn 413 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for NOTIFY_STATUS_PAYLOAD_TOO_LARGE"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                NotifyStatusResponse::UnsupportedMediaType
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(415).expect("Unable to turn 415 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for NOTIFY_STATUS_UNSUPPORTED_MEDIA_TYPE"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                NotifyStatusResponse::TooManyRequests
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(429).expect("Unable to turn 429 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for NOTIFY_STATUS_TOO_MANY_REQUESTS"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                NotifyStatusResponse::InternalServerError
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(500).expect("Unable to turn 500 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for NOTIFY_STATUS_INTERNAL_SERVER_ERROR"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                NotifyStatusResponse::ServiceUnavailable
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(503).expect("Unable to turn 503 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for NOTIFY_STATUS_SERVICE_UNAVAILABLE"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                NotifyStatusResponse::GenericError
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
                                                .body(Body::from(format!("Couldn't read body parameter StatusNotification: {}", e)))
                                                .expect("Unable to create Bad Request response due to unable to read body parameter StatusNotification")),
                        }
                }

                // NotifyStatusIsfm - POST /{$request.body#/ismfPduSessionUri}
                hyper::Method::POST if path.matched(paths::ID_REQUEST_BODY_ISMFPDUSESSIONURI) => {
                    // Path parameters
                    let path: &str = uri.path();
                    let path_params =
                    paths::REGEX_REQUEST_BODY_ISMFPDUSESSIONURI
                    .captures(path)
                    .unwrap_or_else(||
                        panic!("Path {} matched RE REQUEST_BODY_ISMFPDUSESSIONURI in set but failed match against \"{}\"", path, paths::REGEX_REQUEST_BODY_ISMFPDUSESSIONURI.as_str())
                    );

                    let callback_request_body_ismf_pdu_session_uri =
                        path_params["request_body_ismf_pdu_session_uri"].to_string();
                    // Body parameters (note that non-required body parameters will ignore garbage
                    // values, rather than causing a 400 response). Produce warning header and logs for
                    // any unused fields.
                    let result = body.into_raw().await;
                    match result {
                            Ok(body) => {
                                let mut unused_elements = Vec::new();
                                let param_status_notification: Option<models::StatusNotification> = if !body.is_empty() {
                                    let deserializer = &mut serde_json::Deserializer::from_slice(&*body);
                                    match serde_ignored::deserialize(deserializer, |path| {
                                            warn!("Ignoring unknown field in body: {}", path);
                                            unused_elements.push(path.to_string());
                                    }) {
                                        Ok(param_status_notification) => param_status_notification,
                                        Err(e) => return Ok(Response::builder()
                                                        .status(StatusCode::BAD_REQUEST)
                                                        .body(Body::from(format!("Couldn't parse body parameter StatusNotification - doesn't match schema: {}", e)))
                                                        .expect("Unable to create Bad Request response for invalid body parameter StatusNotification due to schema")),
                                    }
                                } else {
                                    None
                                };
                                let param_status_notification = match param_status_notification {
                                    Some(param_status_notification) => param_status_notification,
                                    None => return Ok(Response::builder()
                                                        .status(StatusCode::BAD_REQUEST)
                                                        .body(Body::from("Missing required body parameter StatusNotification"))
                                                        .expect("Unable to create Bad Request response for missing body parameter StatusNotification")),
                                };

                                let result = api_impl.notify_status_isfm(
                                            callback_request_body_ismf_pdu_session_uri,
                                            param_status_notification,
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
                                                NotifyStatusIsfmResponse::SuccessfulNotificationofTheStatusChange
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(204).expect("Unable to turn 204 into a StatusCode");
                                                },
                                                NotifyStatusIsfmResponse::TemporaryRedirect
                                                    {
                                                        body,
                                                        location,
                                                        param_3gpp_sbi_target_nf_id
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
                                                    if let Some(param_3gpp_sbi_target_nf_id) = param_3gpp_sbi_target_nf_id {
                                                    let param_3gpp_sbi_target_nf_id = match header::IntoHeaderValue(param_3gpp_sbi_target_nf_id).try_into() {
                                                        Ok(val) => val,
                                                        Err(e) => {
                                                            return Ok(Response::builder()
                                                                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                                                                    .body(Body::from(format!("An internal server error occurred handling param_3gpp_sbi_target_nf_id header - {}", e)))
                                                                    .expect("Unable to create Internal Server Error for invalid response header"))
                                                        }
                                                    };

                                                    response.headers_mut().insert(
                                                        HeaderName::from_static("3gpp-sbi-target-nf-id"),
                                                        param_3gpp_sbi_target_nf_id
                                                    );
                                                    }
                                                    *response.status_mut() = StatusCode::from_u16(307).expect("Unable to turn 307 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for NOTIFY_STATUS_ISFM_TEMPORARY_REDIRECT"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                NotifyStatusIsfmResponse::PermanentRedirect
                                                    {
                                                        body,
                                                        location,
                                                        param_3gpp_sbi_target_nf_id
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
                                                    if let Some(param_3gpp_sbi_target_nf_id) = param_3gpp_sbi_target_nf_id {
                                                    let param_3gpp_sbi_target_nf_id = match header::IntoHeaderValue(param_3gpp_sbi_target_nf_id).try_into() {
                                                        Ok(val) => val,
                                                        Err(e) => {
                                                            return Ok(Response::builder()
                                                                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                                                                    .body(Body::from(format!("An internal server error occurred handling param_3gpp_sbi_target_nf_id header - {}", e)))
                                                                    .expect("Unable to create Internal Server Error for invalid response header"))
                                                        }
                                                    };

                                                    response.headers_mut().insert(
                                                        HeaderName::from_static("3gpp-sbi-target-nf-id"),
                                                        param_3gpp_sbi_target_nf_id
                                                    );
                                                    }
                                                    *response.status_mut() = StatusCode::from_u16(308).expect("Unable to turn 308 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for NOTIFY_STATUS_ISFM_PERMANENT_REDIRECT"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                NotifyStatusIsfmResponse::BadRequest
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(400).expect("Unable to turn 400 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for NOTIFY_STATUS_ISFM_BAD_REQUEST"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                NotifyStatusIsfmResponse::Forbidden
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(403).expect("Unable to turn 403 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for NOTIFY_STATUS_ISFM_FORBIDDEN"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                NotifyStatusIsfmResponse::NotFound
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(404).expect("Unable to turn 404 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for NOTIFY_STATUS_ISFM_NOT_FOUND"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                NotifyStatusIsfmResponse::LengthRequired
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(411).expect("Unable to turn 411 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for NOTIFY_STATUS_ISFM_LENGTH_REQUIRED"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                NotifyStatusIsfmResponse::PayloadTooLarge
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(413).expect("Unable to turn 413 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for NOTIFY_STATUS_ISFM_PAYLOAD_TOO_LARGE"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                NotifyStatusIsfmResponse::UnsupportedMediaType
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(415).expect("Unable to turn 415 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for NOTIFY_STATUS_ISFM_UNSUPPORTED_MEDIA_TYPE"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                NotifyStatusIsfmResponse::TooManyRequests
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(429).expect("Unable to turn 429 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for NOTIFY_STATUS_ISFM_TOO_MANY_REQUESTS"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                NotifyStatusIsfmResponse::InternalServerError
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(500).expect("Unable to turn 500 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for NOTIFY_STATUS_ISFM_INTERNAL_SERVER_ERROR"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                NotifyStatusIsfmResponse::ServiceUnavailable
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(503).expect("Unable to turn 503 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for NOTIFY_STATUS_ISFM_SERVICE_UNAVAILABLE"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                NotifyStatusIsfmResponse::GenericError
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
                                                .body(Body::from(format!("Couldn't read body parameter StatusNotification: {}", e)))
                                                .expect("Unable to create Bad Request response due to unable to read body parameter StatusNotification")),
                        }
                }

                // ModifyPduSession - POST /{$request.body#/vsmfPduSessionUri}/modify
                hyper::Method::POST
                    if path.matched(paths::ID_REQUEST_BODY_VSMFPDUSESSIONURI_MODIFY) =>
                {
                    // Path parameters
                    let path: &str = uri.path();
                    let path_params =
                    paths::REGEX_REQUEST_BODY_VSMFPDUSESSIONURI_MODIFY
                    .captures(path)
                    .unwrap_or_else(||
                        panic!("Path {} matched RE REQUEST_BODY_VSMFPDUSESSIONURI_MODIFY in set but failed match against \"{}\"", path, paths::REGEX_REQUEST_BODY_VSMFPDUSESSIONURI_MODIFY.as_str())
                    );

                    let callback_request_body_vsmf_pdu_session_uri =
                        path_params["request_body_vsmf_pdu_session_uri"].to_string();
                    // Body parameters (note that non-required body parameters will ignore garbage
                    // values, rather than causing a 400 response). Produce warning header and logs for
                    // any unused fields.
                    let result = body.into_raw().await;
                    match result {
                        // Ok(body) => {
                        //     let mut unused_elements = Vec::new();


                        //     // Body parameters (note that non-required body parameters will ignore garbage
                        //     // values, rather than causing a 400 response). Produce warning header and logs for
                        //     // any unused fields.
                        //     let result = body.into_raw();
                        //     match result.await {
                            Ok(body) => {
                                let mut unused_elements: Vec<String> = vec![];
                                let param_vsmf_update_data: Option<models::VsmfUpdateData> = if !body
                                .is_empty()
                            {
                                let deserializer =
                                    &mut serde_json::Deserializer::from_slice(&*body);
                                match serde_ignored::deserialize(deserializer, |path| {
                                            warn!("Ignoring unknown field in body: {}", path);
                                            unused_elements.push(path.to_string());
                                    }) {
                                        Ok(param_vsmf_update_data) => param_vsmf_update_data,
                                        Err(e) => return Ok(Response::builder()
                                                        .status(StatusCode::BAD_REQUEST)
                                                        .body(Body::from(format!("Couldn't parse body parameter VsmfUpdateData - doesn't match schema: {}", e)))
                                                        .expect("Unable to create Bad Request response for invalid body parameter VsmfUpdateData due to schema")),
                                    }
                            } else {
                                None
                            };
                            let param_vsmf_update_data = match param_vsmf_update_data {
                                    Some(param_vsmf_update_data) => param_vsmf_update_data,
                                    None => return Ok(Response::builder()
                                                        .status(StatusCode::BAD_REQUEST)
                                                        .body(Body::from("Missing required body parameter VsmfUpdateData"))
                                                        .expect("Unable to create Bad Request response for missing body parameter VsmfUpdateData")),
                                };
                                // Get multipart chunks.

                                // Extract the top-level content type header.
                                let content_type_mime = headers
                                    .get(CONTENT_TYPE)
                                    .ok_or_else(|| "Missing content-type header".to_string())
                                    .and_then(|v| v.to_str().map_err(|e| format!("Couldn't read content-type header value for ModifyPduSession: {}", e)))
                                    .and_then(|v| v.parse::<Mime2>().map_err(|_e| "Couldn't parse content-type header value for ModifyPduSession".to_string()));

                                // Insert top-level content type header into a Headers object.
                                let mut multi_part_headers = Headers::new();
                                match content_type_mime {
                                    Ok(content_type_mime) => {
                                        multi_part_headers.set(ContentType(content_type_mime));
                                    },
                                    Err(e) => {
                                        return Ok(Response::builder()
                                                .status(StatusCode::BAD_REQUEST)
                                                .body(Body::from(e))
                                                .expect("Unable to create Bad Request response due to unable to read content-type header for ModifyPduSession"));
                                    }
                                }

                                // &*body expresses the body as a byteslice, &mut provides a
                                // mutable reference to that byteslice.
                                let nodes = match read_multipart_body(&mut&*body, &multi_part_headers, false) {
                                    Ok(nodes) => nodes,
                                    Err(e) => {
                                        return Ok(Response::builder()
                                                .status(StatusCode::BAD_REQUEST)
                                                .body(Body::from(format!("Could not read multipart body for ModifyPduSession: {}", e)))
                                                .expect("Unable to create Bad Request response due to unable to read multipart body for ModifyPduSession"));
                                    }
                                };


                                for node in nodes {
                                    if let Node::Part(part) = node {
                                        let content_type = part.content_type().map(|x| format!("{}",x));
                                        match content_type.as_deref() {
                                            Some(content_type) => {
                                                warn!("Ignoring unexpected content type: {}", content_type);
                                                unused_elements.push(content_type.to_string());
                                            },
                                            None => {
                                                warn!("Missing content type");
                                            },
                                        }
                                    } else {
                                        unimplemented!("No support for handling unexpected parts");
                                        // unused_elements.push();
                                    }
                                }

                                let result = api_impl.modify_pdu_session(
                                            callback_request_body_vsmf_pdu_session_uri,
                                            param_vsmf_update_data,
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
                                                ModifyPduSessionResponse::SuccessfulUpdateOfAPDUSessionWithContentInTheResponse
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(200).expect("Unable to turn 200 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for MODIFY_PDU_SESSION_SUCCESSFUL_UPDATE_OF_APDU_SESSION_WITH_CONTENT_IN_THE_RESPONSE"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                ModifyPduSessionResponse::SuccessfulUpdateOfAPDUSessionWithoutContentInTheResponse
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(204).expect("Unable to turn 204 into a StatusCode");
                                                },
                                                ModifyPduSessionResponse::TemporaryRedirect
                                                    {
                                                        body,
                                                        location,
                                                        param_3gpp_sbi_target_nf_id
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
                                                    if let Some(param_3gpp_sbi_target_nf_id) = param_3gpp_sbi_target_nf_id {
                                                    let param_3gpp_sbi_target_nf_id = match header::IntoHeaderValue(param_3gpp_sbi_target_nf_id).try_into() {
                                                        Ok(val) => val,
                                                        Err(e) => {
                                                            return Ok(Response::builder()
                                                                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                                                                    .body(Body::from(format!("An internal server error occurred handling param_3gpp_sbi_target_nf_id header - {}", e)))
                                                                    .expect("Unable to create Internal Server Error for invalid response header"))
                                                        }
                                                    };

                                                    response.headers_mut().insert(
                                                        HeaderName::from_static("3gpp-sbi-target-nf-id"),
                                                        param_3gpp_sbi_target_nf_id
                                                    );
                                                    }
                                                    *response.status_mut() = StatusCode::from_u16(307).expect("Unable to turn 307 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for MODIFY_PDU_SESSION_TEMPORARY_REDIRECT"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                ModifyPduSessionResponse::PermanentRedirect
                                                    {
                                                        body,
                                                        location,
                                                        param_3gpp_sbi_target_nf_id
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
                                                    if let Some(param_3gpp_sbi_target_nf_id) = param_3gpp_sbi_target_nf_id {
                                                    let param_3gpp_sbi_target_nf_id = match header::IntoHeaderValue(param_3gpp_sbi_target_nf_id).try_into() {
                                                        Ok(val) => val,
                                                        Err(e) => {
                                                            return Ok(Response::builder()
                                                                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                                                                    .body(Body::from(format!("An internal server error occurred handling param_3gpp_sbi_target_nf_id header - {}", e)))
                                                                    .expect("Unable to create Internal Server Error for invalid response header"))
                                                        }
                                                    };

                                                    response.headers_mut().insert(
                                                        HeaderName::from_static("3gpp-sbi-target-nf-id"),
                                                        param_3gpp_sbi_target_nf_id
                                                    );
                                                    }
                                                    *response.status_mut() = StatusCode::from_u16(308).expect("Unable to turn 308 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for MODIFY_PDU_SESSION_PERMANENT_REDIRECT"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                ModifyPduSessionResponse::UnsuccessfulUpdateOfAPDUSession
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(400).expect("Unable to turn 400 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for MODIFY_PDU_SESSION_UNSUCCESSFUL_UPDATE_OF_APDU_SESSION"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                ModifyPduSessionResponse::UnsuccessfulUpdateOfAPDUSession_2
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(403).expect("Unable to turn 403 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for MODIFY_PDU_SESSION_UNSUCCESSFUL_UPDATE_OF_APDU_SESSION_2"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                ModifyPduSessionResponse::UnsuccessfulUpdateOfAPDUSession_3
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(404).expect("Unable to turn 404 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for MODIFY_PDU_SESSION_UNSUCCESSFUL_UPDATE_OF_APDU_SESSION_3"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                ModifyPduSessionResponse::UnsuccessfulUpdateOfAPDUSession_4
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(409).expect("Unable to turn 409 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for MODIFY_PDU_SESSION_UNSUCCESSFUL_UPDATE_OF_APDU_SESSION_4"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                ModifyPduSessionResponse::LengthRequired
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(411).expect("Unable to turn 411 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for MODIFY_PDU_SESSION_LENGTH_REQUIRED"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                ModifyPduSessionResponse::PayloadTooLarge
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(413).expect("Unable to turn 413 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for MODIFY_PDU_SESSION_PAYLOAD_TOO_LARGE"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                ModifyPduSessionResponse::UnsupportedMediaType
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(415).expect("Unable to turn 415 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for MODIFY_PDU_SESSION_UNSUPPORTED_MEDIA_TYPE"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                ModifyPduSessionResponse::TooManyRequests
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(429).expect("Unable to turn 429 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for MODIFY_PDU_SESSION_TOO_MANY_REQUESTS"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                ModifyPduSessionResponse::UnsuccessfulUpdateOfAPDUSession_5
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(500).expect("Unable to turn 500 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for MODIFY_PDU_SESSION_UNSUCCESSFUL_UPDATE_OF_APDU_SESSION_5"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                ModifyPduSessionResponse::UnsuccessfulUpdateOfAPDUSession_6
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(503).expect("Unable to turn 503 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for MODIFY_PDU_SESSION_UNSUCCESSFUL_UPDATE_OF_APDU_SESSION_6"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                ModifyPduSessionResponse::UnsuccessfulUpdateOfAPDUSession_7
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(504).expect("Unable to turn 504 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for MODIFY_PDU_SESSION_UNSUCCESSFUL_UPDATE_OF_APDU_SESSION_7"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                ModifyPduSessionResponse::GenericError
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
                                                .body(Body::from(format!("Couldn't read body parameter VsmfUpdateData: {}", e)))
                                                .expect("Unable to create Bad Request response due to unable to read body parameter VsmfUpdateData")),
                        }
                }

                // ModifyPduSessionIsmf - POST /{$request.body#/ismfPduSessionUri}/modify
                hyper::Method::POST
                    if path.matched(paths::ID_REQUEST_BODY_ISMFPDUSESSIONURI_MODIFY) =>
                {
                    // Path parameters
                    let path: &str = uri.path();
                    let path_params =
                    paths::REGEX_REQUEST_BODY_ISMFPDUSESSIONURI_MODIFY
                    .captures(path)
                    .unwrap_or_else(||
                        panic!("Path {} matched RE REQUEST_BODY_ISMFPDUSESSIONURI_MODIFY in set but failed match against \"{}\"", path, paths::REGEX_REQUEST_BODY_ISMFPDUSESSIONURI_MODIFY.as_str())
                    );

                    let callback_request_body_ismf_pdu_session_uri =
                        path_params["request_body_ismf_pdu_session_uri"].to_string();
                    // Body parameters (note that non-required body parameters will ignore garbage
                    // values, rather than causing a 400 response). Produce warning header and logs for
                    // any unused fields.
                    let result = body.into_raw().await;
                    match result {
                            Ok(body) => {
                                let mut unused_elements: Vec<String> = vec![];
                                let param_vsmf_update_data: Option<models::VsmfUpdateData> =
                                if !body.is_empty() {
                                    let deserializer =
                                        &mut serde_json::Deserializer::from_slice(&*body);
                                    match serde_ignored::deserialize(deserializer, |path| {
                                    warn!("Ignoring unknown field in body: {}", path);
                                    unused_elements.push(path.to_string());
                            }) {
                                Ok(param_vsmf_update_data) => param_vsmf_update_data,
                                Err(e) => return Ok(Response::builder()
                                                .status(StatusCode::BAD_REQUEST)
                                                .body(Body::from(format!("Couldn't parse body parameter VsmfUpdateData - doesn't match schema: {}", e)))
                                                .expect("Unable to create Bad Request response for invalid body parameter VsmfUpdateData due to schema")),
                            }
                                } else {
                                    None
                                };
                            let param_vsmf_update_data = match param_vsmf_update_data {
                            Some(param_vsmf_update_data) => param_vsmf_update_data,
                            None => return Ok(Response::builder()
                                                .status(StatusCode::BAD_REQUEST)
                                                .body(Body::from("Missing required body parameter VsmfUpdateData"))
                                                .expect("Unable to create Bad Request response for missing body parameter VsmfUpdateData")),
                        };
                                // Get multipart chunks.

                                // Extract the top-level content type header.
                                let content_type_mime = headers
                                    .get(CONTENT_TYPE)
                                    .ok_or_else(|| "Missing content-type header".to_string())
                                    .and_then(|v| v.to_str().map_err(|e| format!("Couldn't read content-type header value for ModifyPduSessionIsmf: {}", e)))
                                    .and_then(|v| v.parse::<Mime2>().map_err(|_e| "Couldn't parse content-type header value for ModifyPduSessionIsmf".to_string()));

                                // Insert top-level content type header into a Headers object.
                                let mut multi_part_headers = Headers::new();
                                match content_type_mime {
                                    Ok(content_type_mime) => {
                                        multi_part_headers.set(ContentType(content_type_mime));
                                    },
                                    Err(e) => {
                                        return Ok(Response::builder()
                                                .status(StatusCode::BAD_REQUEST)
                                                .body(Body::from(e))
                                                .expect("Unable to create Bad Request response due to unable to read content-type header for ModifyPduSessionIsmf"));
                                    }
                                }

                                // &*body expresses the body as a byteslice, &mut provides a
                                // mutable reference to that byteslice.
                                let nodes = match read_multipart_body(&mut&*body, &multi_part_headers, false) {
                                    Ok(nodes) => nodes,
                                    Err(e) => {
                                        return Ok(Response::builder()
                                                .status(StatusCode::BAD_REQUEST)
                                                .body(Body::from(format!("Could not read multipart body for ModifyPduSessionIsmf: {}", e)))
                                                .expect("Unable to create Bad Request response due to unable to read multipart body for ModifyPduSessionIsmf"));
                                    }
                                };


                                for node in nodes {
                                    if let Node::Part(part) = node {
                                        let content_type = part.content_type().map(|x| format!("{}",x));
                                        match content_type.as_deref() {
                                            Some(content_type) => {
                                                warn!("Ignoring unexpected content type: {}", content_type);
                                                unused_elements.push(content_type.to_string());
                                            },
                                            None => {
                                                warn!("Missing content type");
                                            },
                                        }
                                    } else {
                                        unimplemented!("No support for handling unexpected parts");
                                        // unused_elements.push();
                                    }
                                }

                                let result = api_impl.modify_pdu_session_ismf(
                                            callback_request_body_ismf_pdu_session_uri,
                                            param_vsmf_update_data,
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
                                                ModifyPduSessionIsmfResponse::SuccessfulUpdateOfAPDUSessionWithContentInTheResponse
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(200).expect("Unable to turn 200 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for MODIFY_PDU_SESSION_ISMF_SUCCESSFUL_UPDATE_OF_APDU_SESSION_WITH_CONTENT_IN_THE_RESPONSE"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                ModifyPduSessionIsmfResponse::SuccessfulUpdateOfAPDUSessionWithoutContentInTheResponse
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(204).expect("Unable to turn 204 into a StatusCode");
                                                },
                                                ModifyPduSessionIsmfResponse::TemporaryRedirect
                                                    {
                                                        body,
                                                        location,
                                                        param_3gpp_sbi_target_nf_id
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
                                                    if let Some(param_3gpp_sbi_target_nf_id) = param_3gpp_sbi_target_nf_id {
                                                    let param_3gpp_sbi_target_nf_id = match header::IntoHeaderValue(param_3gpp_sbi_target_nf_id).try_into() {
                                                        Ok(val) => val,
                                                        Err(e) => {
                                                            return Ok(Response::builder()
                                                                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                                                                    .body(Body::from(format!("An internal server error occurred handling param_3gpp_sbi_target_nf_id header - {}", e)))
                                                                    .expect("Unable to create Internal Server Error for invalid response header"))
                                                        }
                                                    };

                                                    response.headers_mut().insert(
                                                        HeaderName::from_static("3gpp-sbi-target-nf-id"),
                                                        param_3gpp_sbi_target_nf_id
                                                    );
                                                    }
                                                    *response.status_mut() = StatusCode::from_u16(307).expect("Unable to turn 307 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for MODIFY_PDU_SESSION_ISMF_TEMPORARY_REDIRECT"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                ModifyPduSessionIsmfResponse::PermanentRedirect
                                                    {
                                                        body,
                                                        location,
                                                        param_3gpp_sbi_target_nf_id
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
                                                    if let Some(param_3gpp_sbi_target_nf_id) = param_3gpp_sbi_target_nf_id {
                                                    let param_3gpp_sbi_target_nf_id = match header::IntoHeaderValue(param_3gpp_sbi_target_nf_id).try_into() {
                                                        Ok(val) => val,
                                                        Err(e) => {
                                                            return Ok(Response::builder()
                                                                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                                                                    .body(Body::from(format!("An internal server error occurred handling param_3gpp_sbi_target_nf_id header - {}", e)))
                                                                    .expect("Unable to create Internal Server Error for invalid response header"))
                                                        }
                                                    };

                                                    response.headers_mut().insert(
                                                        HeaderName::from_static("3gpp-sbi-target-nf-id"),
                                                        param_3gpp_sbi_target_nf_id
                                                    );
                                                    }
                                                    *response.status_mut() = StatusCode::from_u16(308).expect("Unable to turn 308 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for MODIFY_PDU_SESSION_ISMF_PERMANENT_REDIRECT"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                ModifyPduSessionIsmfResponse::UnsuccessfulUpdateOfAPDUSession
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(400).expect("Unable to turn 400 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for MODIFY_PDU_SESSION_ISMF_UNSUCCESSFUL_UPDATE_OF_APDU_SESSION"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                ModifyPduSessionIsmfResponse::UnsuccessfulUpdateOfAPDUSession_2
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(403).expect("Unable to turn 403 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for MODIFY_PDU_SESSION_ISMF_UNSUCCESSFUL_UPDATE_OF_APDU_SESSION_2"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                ModifyPduSessionIsmfResponse::UnsuccessfulUpdateOfAPDUSession_3
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(404).expect("Unable to turn 404 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for MODIFY_PDU_SESSION_ISMF_UNSUCCESSFUL_UPDATE_OF_APDU_SESSION_3"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                ModifyPduSessionIsmfResponse::UnsuccessfulUpdateOfAPDUSession_4
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(409).expect("Unable to turn 409 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for MODIFY_PDU_SESSION_ISMF_UNSUCCESSFUL_UPDATE_OF_APDU_SESSION_4"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                ModifyPduSessionIsmfResponse::LengthRequired
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(411).expect("Unable to turn 411 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for MODIFY_PDU_SESSION_ISMF_LENGTH_REQUIRED"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                ModifyPduSessionIsmfResponse::PayloadTooLarge
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(413).expect("Unable to turn 413 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for MODIFY_PDU_SESSION_ISMF_PAYLOAD_TOO_LARGE"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                ModifyPduSessionIsmfResponse::UnsupportedMediaType
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(415).expect("Unable to turn 415 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for MODIFY_PDU_SESSION_ISMF_UNSUPPORTED_MEDIA_TYPE"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                ModifyPduSessionIsmfResponse::TooManyRequests
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(429).expect("Unable to turn 429 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for MODIFY_PDU_SESSION_ISMF_TOO_MANY_REQUESTS"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                ModifyPduSessionIsmfResponse::UnsuccessfulUpdateOfAPDUSession_5
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(500).expect("Unable to turn 500 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for MODIFY_PDU_SESSION_ISMF_UNSUCCESSFUL_UPDATE_OF_APDU_SESSION_5"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                ModifyPduSessionIsmfResponse::UnsuccessfulUpdateOfAPDUSession_6
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(503).expect("Unable to turn 503 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for MODIFY_PDU_SESSION_ISMF_UNSUCCESSFUL_UPDATE_OF_APDU_SESSION_6"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                ModifyPduSessionIsmfResponse::UnsuccessfulUpdateOfAPDUSession_7
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(504).expect("Unable to turn 504 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for MODIFY_PDU_SESSION_ISMF_UNSUCCESSFUL_UPDATE_OF_APDU_SESSION_7"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                ModifyPduSessionIsmfResponse::GenericError
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
                                                .body(Body::from(format!("Couldn't read body parameter VsmfUpdateData: {}", e)))
                                                .expect("Unable to create Bad Request response due to unable to read body parameter VsmfUpdateData")),
                        }
                }

                // TransferMtData - POST /{$request.body#/vsmfPduSessionUri}/transfer-mt-data
                hyper::Method::POST
                    if path.matched(paths::ID_REQUEST_BODY_VSMFPDUSESSIONURI_TRANSFER_MT_DATA) =>
                {
                    // Path parameters
                    let path: &str = uri.path();
                    let path_params =
                    paths::REGEX_REQUEST_BODY_VSMFPDUSESSIONURI_TRANSFER_MT_DATA
                    .captures(path)
                    .unwrap_or_else(||
                        panic!("Path {} matched RE REQUEST_BODY_VSMFPDUSESSIONURI_TRANSFER_MT_DATA in set but failed match against \"{}\"", path, paths::REGEX_REQUEST_BODY_VSMFPDUSESSIONURI_TRANSFER_MT_DATA.as_str())
                    );

                    let callback_request_body_vsmf_pdu_session_uri =
                        path_params["request_body_vsmf_pdu_session_uri"].to_string();
                    // Body parameters (note that non-required body parameters will ignore garbage
                    // values, rather than causing a 400 response). Produce warning header and logs for
                    // any unused fields.
                    let result = body.into_raw();
                    match result.await {
                            Ok(body) => {
                                let mut unused_elements: Vec<String> = vec![];

                                // Get multipart chunks.

                                // Extract the top-level content type header.
                                let content_type_mime = headers
                                    .get(CONTENT_TYPE)
                                    .ok_or_else(|| "Missing content-type header".to_string())
                                    .and_then(|v| v.to_str().map_err(|e| format!("Couldn't read content-type header value for TransferMtData: {}", e)))
                                    .and_then(|v| v.parse::<Mime2>().map_err(|_e| "Couldn't parse content-type header value for TransferMtData".to_string()));

                                // Insert top-level content type header into a Headers object.
                                let mut multi_part_headers = Headers::new();
                                match content_type_mime {
                                    Ok(content_type_mime) => {
                                        multi_part_headers.set(ContentType(content_type_mime));
                                    },
                                    Err(e) => {
                                        return Ok(Response::builder()
                                                .status(StatusCode::BAD_REQUEST)
                                                .body(Body::from(e))
                                                .expect("Unable to create Bad Request response due to unable to read content-type header for TransferMtData"));
                                    }
                                }

                                // &*body expresses the body as a byteslice, &mut provides a
                                // mutable reference to that byteslice.
                                let nodes = match read_multipart_body(&mut&*body, &multi_part_headers, false) {
                                    Ok(nodes) => nodes,
                                    Err(e) => {
                                        return Ok(Response::builder()
                                                .status(StatusCode::BAD_REQUEST)
                                                .body(Body::from(format!("Could not read multipart body for TransferMtData: {}", e)))
                                                .expect("Unable to create Bad Request response due to unable to read multipart body for TransferMtData"));
                                    }
                                };

                                let mut param_json_data = None;
                                let mut param_binary_mt_data = None;

                                for node in nodes {
                                    if let Node::Part(part) = node {
                                        let content_type = part.content_type().map(|x| format!("{}",x));
                                        match content_type.as_deref() {
                                            Some("application/json") if param_json_data.is_none() => {
                                                // Extract JSON part.
                                                let deserializer = &mut serde_json::Deserializer::from_slice(part.body.as_slice());
                                                let json_data: models::TransferMtDataReqData = match serde_ignored::deserialize(deserializer, |path| {
                                                    warn!("Ignoring unknown field in JSON part: {}", path);
                                                    unused_elements.push(path.to_string());
                                                }) {
                                                    Ok(json_data) => json_data,
                                                    Err(e) => return Ok(Response::builder()
                                                                    .status(StatusCode::BAD_REQUEST)
                                                                    .body(Body::from(format!("Couldn't parse body parameter models::TransferMtDataReqData - doesn't match schema: {}", e)))
                                                                    .expect("Unable to create Bad Request response for invalid body parameter models::TransferMtDataReqData due to schema"))
                                                };
                                                // Push JSON part to return object.
                                                param_json_data.get_or_insert(json_data);
                                            },
                                            Some("application/vnd.3gpp.5gnas") if param_binary_mt_data.is_none() => {
                                                param_binary_mt_data.get_or_insert(swagger::ByteArray(part.body));
                                            },
                                            Some(content_type) => {
                                                warn!("Ignoring unexpected content type: {}", content_type);
                                                unused_elements.push(content_type.to_string());
                                            },
                                            None => {
                                                warn!("Missing content type");
                                            },
                                        }
                                    } else {
                                        unimplemented!("No support for handling unexpected parts");
                                        // unused_elements.push();
                                    }
                                }

                                // Check that the required multipart chunks are present.

                                let result = api_impl.transfer_mt_data(
                                            callback_request_body_vsmf_pdu_session_uri,
                                            param_json_data,
                                            param_binary_mt_data,
                                        &context
                                    ).await;
                                let mut response = Response::new(Body::empty());
                                response.headers_mut().insert(
                                            HeaderName::from_static("x-span-id"),
                                            HeaderValue::from_str((&context as &dyn Has<XSpanIdString>).get().0.clone().as_str())
                                                .expect("Unable to create X-Span-ID header value"));

                                        match result {
                                            Ok(rsp) => match rsp {
                                                TransferMtDataResponse::SuccessfulTransferingOfMTData
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(204).expect("Unable to turn 204 into a StatusCode");
                                                },
                                                TransferMtDataResponse::TemporaryRedirect
                                                    {
                                                        body,
                                                        location,
                                                        param_3gpp_sbi_target_nf_id
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
                                                    if let Some(param_3gpp_sbi_target_nf_id) = param_3gpp_sbi_target_nf_id {
                                                    let param_3gpp_sbi_target_nf_id = match header::IntoHeaderValue(param_3gpp_sbi_target_nf_id).try_into() {
                                                        Ok(val) => val,
                                                        Err(e) => {
                                                            return Ok(Response::builder()
                                                                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                                                                    .body(Body::from(format!("An internal server error occurred handling param_3gpp_sbi_target_nf_id header - {}", e)))
                                                                    .expect("Unable to create Internal Server Error for invalid response header"))
                                                        }
                                                    };

                                                    response.headers_mut().insert(
                                                        HeaderName::from_static("3gpp-sbi-target-nf-id"),
                                                        param_3gpp_sbi_target_nf_id
                                                    );
                                                    }
                                                    *response.status_mut() = StatusCode::from_u16(307).expect("Unable to turn 307 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for TRANSFER_MT_DATA_TEMPORARY_REDIRECT"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                TransferMtDataResponse::PermanentRedirect
                                                    {
                                                        body,
                                                        location,
                                                        param_3gpp_sbi_target_nf_id
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
                                                    if let Some(param_3gpp_sbi_target_nf_id) = param_3gpp_sbi_target_nf_id {
                                                    let param_3gpp_sbi_target_nf_id = match header::IntoHeaderValue(param_3gpp_sbi_target_nf_id).try_into() {
                                                        Ok(val) => val,
                                                        Err(e) => {
                                                            return Ok(Response::builder()
                                                                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                                                                    .body(Body::from(format!("An internal server error occurred handling param_3gpp_sbi_target_nf_id header - {}", e)))
                                                                    .expect("Unable to create Internal Server Error for invalid response header"))
                                                        }
                                                    };

                                                    response.headers_mut().insert(
                                                        HeaderName::from_static("3gpp-sbi-target-nf-id"),
                                                        param_3gpp_sbi_target_nf_id
                                                    );
                                                    }
                                                    *response.status_mut() = StatusCode::from_u16(308).expect("Unable to turn 308 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for TRANSFER_MT_DATA_PERMANENT_REDIRECT"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                TransferMtDataResponse::BadRequest
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(400).expect("Unable to turn 400 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for TRANSFER_MT_DATA_BAD_REQUEST"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                TransferMtDataResponse::Unauthorized
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(401).expect("Unable to turn 401 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for TRANSFER_MT_DATA_UNAUTHORIZED"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                TransferMtDataResponse::Forbidden
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(403).expect("Unable to turn 403 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for TRANSFER_MT_DATA_FORBIDDEN"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                TransferMtDataResponse::NotFound
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(404).expect("Unable to turn 404 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for TRANSFER_MT_DATA_NOT_FOUND"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                TransferMtDataResponse::LengthRequired
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(411).expect("Unable to turn 411 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for TRANSFER_MT_DATA_LENGTH_REQUIRED"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                TransferMtDataResponse::PayloadTooLarge
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(413).expect("Unable to turn 413 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for TRANSFER_MT_DATA_PAYLOAD_TOO_LARGE"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                TransferMtDataResponse::UnsupportedMediaType
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(415).expect("Unable to turn 415 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for TRANSFER_MT_DATA_UNSUPPORTED_MEDIA_TYPE"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                TransferMtDataResponse::TooManyRequests
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(429).expect("Unable to turn 429 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for TRANSFER_MT_DATA_TOO_MANY_REQUESTS"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                TransferMtDataResponse::InternalServerError
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(500).expect("Unable to turn 500 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for TRANSFER_MT_DATA_INTERNAL_SERVER_ERROR"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                TransferMtDataResponse::ServiceUnavailable
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(503).expect("Unable to turn 503 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for TRANSFER_MT_DATA_SERVICE_UNAVAILABLE"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                TransferMtDataResponse::UnsuccessfulDeliveryOfMobileTerminatedData
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(504).expect("Unable to turn 504 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for TRANSFER_MT_DATA_UNSUCCESSFUL_DELIVERY_OF_MOBILE_TERMINATED_DATA"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                TransferMtDataResponse::GenericError
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
                                                .body(Body::from(format!("Couldn't read body parameter : {}", e)))
                                                .expect("Unable to create Bad Request response due to unable to read body parameter ")),
                        }
                }

                // TransferMtDataIsmf - POST /{$request.body#/ismfPduSessionUri}/transfer-mt-data
                hyper::Method::POST
                    if path.matched(paths::ID_REQUEST_BODY_ISMFPDUSESSIONURI_TRANSFER_MT_DATA) =>
                {
                    // Path parameters
                    let path: &str = uri.path();
                    let path_params =
                    paths::REGEX_REQUEST_BODY_ISMFPDUSESSIONURI_TRANSFER_MT_DATA
                    .captures(path)
                    .unwrap_or_else(||
                        panic!("Path {} matched RE REQUEST_BODY_ISMFPDUSESSIONURI_TRANSFER_MT_DATA in set but failed match against \"{}\"", path, paths::REGEX_REQUEST_BODY_ISMFPDUSESSIONURI_TRANSFER_MT_DATA.as_str())
                    );

                    let callback_request_body_ismf_pdu_session_uri =
                        path_params["request_body_ismf_pdu_session_uri"].to_string();
                    // Body parameters (note that non-required body parameters will ignore garbage
                    // values, rather than causing a 400 response). Produce warning header and logs for
                    // any unused fields.
                    let result = body.into_raw();
                    match result.await {
                            Ok(body) => {
                                let mut unused_elements: Vec<String> = vec![];

                                // Get multipart chunks.

                                // Extract the top-level content type header.
                                let content_type_mime = headers
                                    .get(CONTENT_TYPE)
                                    .ok_or_else(|| "Missing content-type header".to_string())
                                    .and_then(|v| v.to_str().map_err(|e| format!("Couldn't read content-type header value for TransferMtDataIsmf: {}", e)))
                                    .and_then(|v| v.parse::<Mime2>().map_err(|_e| "Couldn't parse content-type header value for TransferMtDataIsmf".to_string()));

                                // Insert top-level content type header into a Headers object.
                                let mut multi_part_headers = Headers::new();
                                match content_type_mime {
                                    Ok(content_type_mime) => {
                                        multi_part_headers.set(ContentType(content_type_mime));
                                    },
                                    Err(e) => {
                                        return Ok(Response::builder()
                                                .status(StatusCode::BAD_REQUEST)
                                                .body(Body::from(e))
                                                .expect("Unable to create Bad Request response due to unable to read content-type header for TransferMtDataIsmf"));
                                    }
                                }

                                // &*body expresses the body as a byteslice, &mut provides a
                                // mutable reference to that byteslice.
                                let nodes = match read_multipart_body(&mut&*body, &multi_part_headers, false) {
                                    Ok(nodes) => nodes,
                                    Err(e) => {
                                        return Ok(Response::builder()
                                                .status(StatusCode::BAD_REQUEST)
                                                .body(Body::from(format!("Could not read multipart body for TransferMtDataIsmf: {}", e)))
                                                .expect("Unable to create Bad Request response due to unable to read multipart body for TransferMtDataIsmf"));
                                    }
                                };

                                let mut param_json_data = None;
                                let mut param_binary_mt_data = None;

                                for node in nodes {
                                    if let Node::Part(part) = node {
                                        let content_type = part.content_type().map(|x| format!("{}",x));
                                        match content_type.as_deref() {
                                            Some("application/json") if param_json_data.is_none() => {
                                                // Extract JSON part.
                                                let deserializer = &mut serde_json::Deserializer::from_slice(part.body.as_slice());
                                                let json_data: models::TransferMtDataReqData = match serde_ignored::deserialize(deserializer, |path| {
                                                    warn!("Ignoring unknown field in JSON part: {}", path);
                                                    unused_elements.push(path.to_string());
                                                }) {
                                                    Ok(json_data) => json_data,
                                                    Err(e) => return Ok(Response::builder()
                                                                    .status(StatusCode::BAD_REQUEST)
                                                                    .body(Body::from(format!("Couldn't parse body parameter models::TransferMtDataReqData - doesn't match schema: {}", e)))
                                                                    .expect("Unable to create Bad Request response for invalid body parameter models::TransferMtDataReqData due to schema"))
                                                };
                                                // Push JSON part to return object.
                                                param_json_data.get_or_insert(json_data);
                                            },
                                            Some("application/vnd.3gpp.5gnas") if param_binary_mt_data.is_none() => {
                                                param_binary_mt_data.get_or_insert(swagger::ByteArray(part.body));
                                            },
                                            Some(content_type) => {
                                                warn!("Ignoring unexpected content type: {}", content_type);
                                                unused_elements.push(content_type.to_string());
                                            },
                                            None => {
                                                warn!("Missing content type");
                                            },
                                        }
                                    } else {
                                        unimplemented!("No support for handling unexpected parts");
                                        // unused_elements.push();
                                    }
                                }

                                // Check that the required multipart chunks are present.

                                let result = api_impl.transfer_mt_data_ismf(
                                            callback_request_body_ismf_pdu_session_uri,
                                            param_json_data,
                                            param_binary_mt_data,
                                        &context
                                    ).await;
                                let mut response = Response::new(Body::empty());
                                response.headers_mut().insert(
                                            HeaderName::from_static("x-span-id"),
                                            HeaderValue::from_str((&context as &dyn Has<XSpanIdString>).get().0.clone().as_str())
                                                .expect("Unable to create X-Span-ID header value"));

                                        match result {
                                            Ok(rsp) => match rsp {
                                                TransferMtDataIsmfResponse::SuccessfulTransferingOfMTData
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(204).expect("Unable to turn 204 into a StatusCode");
                                                },
                                                TransferMtDataIsmfResponse::TemporaryRedirect
                                                    {
                                                        body,
                                                        location,
                                                        param_3gpp_sbi_target_nf_id
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
                                                    if let Some(param_3gpp_sbi_target_nf_id) = param_3gpp_sbi_target_nf_id {
                                                    let param_3gpp_sbi_target_nf_id = match header::IntoHeaderValue(param_3gpp_sbi_target_nf_id).try_into() {
                                                        Ok(val) => val,
                                                        Err(e) => {
                                                            return Ok(Response::builder()
                                                                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                                                                    .body(Body::from(format!("An internal server error occurred handling param_3gpp_sbi_target_nf_id header - {}", e)))
                                                                    .expect("Unable to create Internal Server Error for invalid response header"))
                                                        }
                                                    };

                                                    response.headers_mut().insert(
                                                        HeaderName::from_static("3gpp-sbi-target-nf-id"),
                                                        param_3gpp_sbi_target_nf_id
                                                    );
                                                    }
                                                    *response.status_mut() = StatusCode::from_u16(307).expect("Unable to turn 307 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for TRANSFER_MT_DATA_ISMF_TEMPORARY_REDIRECT"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                TransferMtDataIsmfResponse::PermanentRedirect
                                                    {
                                                        body,
                                                        location,
                                                        param_3gpp_sbi_target_nf_id
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
                                                    if let Some(param_3gpp_sbi_target_nf_id) = param_3gpp_sbi_target_nf_id {
                                                    let param_3gpp_sbi_target_nf_id = match header::IntoHeaderValue(param_3gpp_sbi_target_nf_id).try_into() {
                                                        Ok(val) => val,
                                                        Err(e) => {
                                                            return Ok(Response::builder()
                                                                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                                                                    .body(Body::from(format!("An internal server error occurred handling param_3gpp_sbi_target_nf_id header - {}", e)))
                                                                    .expect("Unable to create Internal Server Error for invalid response header"))
                                                        }
                                                    };

                                                    response.headers_mut().insert(
                                                        HeaderName::from_static("3gpp-sbi-target-nf-id"),
                                                        param_3gpp_sbi_target_nf_id
                                                    );
                                                    }
                                                    *response.status_mut() = StatusCode::from_u16(308).expect("Unable to turn 308 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for TRANSFER_MT_DATA_ISMF_PERMANENT_REDIRECT"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                TransferMtDataIsmfResponse::BadRequest
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(400).expect("Unable to turn 400 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for TRANSFER_MT_DATA_ISMF_BAD_REQUEST"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                TransferMtDataIsmfResponse::Unauthorized
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(401).expect("Unable to turn 401 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for TRANSFER_MT_DATA_ISMF_UNAUTHORIZED"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                TransferMtDataIsmfResponse::Forbidden
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(403).expect("Unable to turn 403 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for TRANSFER_MT_DATA_ISMF_FORBIDDEN"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                TransferMtDataIsmfResponse::NotFound
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(404).expect("Unable to turn 404 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for TRANSFER_MT_DATA_ISMF_NOT_FOUND"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                TransferMtDataIsmfResponse::LengthRequired
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(411).expect("Unable to turn 411 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for TRANSFER_MT_DATA_ISMF_LENGTH_REQUIRED"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                TransferMtDataIsmfResponse::PayloadTooLarge
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(413).expect("Unable to turn 413 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for TRANSFER_MT_DATA_ISMF_PAYLOAD_TOO_LARGE"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                TransferMtDataIsmfResponse::UnsupportedMediaType
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(415).expect("Unable to turn 415 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for TRANSFER_MT_DATA_ISMF_UNSUPPORTED_MEDIA_TYPE"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                TransferMtDataIsmfResponse::TooManyRequests
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(429).expect("Unable to turn 429 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for TRANSFER_MT_DATA_ISMF_TOO_MANY_REQUESTS"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                TransferMtDataIsmfResponse::InternalServerError
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(500).expect("Unable to turn 500 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for TRANSFER_MT_DATA_ISMF_INTERNAL_SERVER_ERROR"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                TransferMtDataIsmfResponse::ServiceUnavailable
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(503).expect("Unable to turn 503 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for TRANSFER_MT_DATA_ISMF_SERVICE_UNAVAILABLE"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                TransferMtDataIsmfResponse::UnsuccessfulDeliveryOfMobileTerminatedData
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(504).expect("Unable to turn 504 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for TRANSFER_MT_DATA_ISMF_UNSUCCESSFUL_DELIVERY_OF_MOBILE_TERMINATED_DATA"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                TransferMtDataIsmfResponse::GenericError
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
                                                .body(Body::from(format!("Couldn't read body parameter : {}", e)))
                                                .expect("Unable to create Bad Request response due to unable to read body parameter ")),
                        }
                }

                // SmContextStatusNotificationPost - POST /{$request.body#/smContextStatusUri}
                hyper::Method::POST if path.matched(paths::ID_REQUEST_BODY_SMCONTEXTSTATUSURI) => {
                    // Path parameters
                    let path: &str = uri.path();
                    let path_params =
                    paths::REGEX_REQUEST_BODY_SMCONTEXTSTATUSURI
                    .captures(path)
                    .unwrap_or_else(||
                        panic!("Path {} matched RE REQUEST_BODY_SMCONTEXTSTATUSURI in set but failed match against \"{}\"", path, paths::REGEX_REQUEST_BODY_SMCONTEXTSTATUSURI.as_str())
                    );

                    let callback_request_body_sm_context_status_uri =
                        path_params["request_body_sm_context_status_uri"].to_string();
                    // Body parameters (note that non-required body parameters will ignore garbage
                    // values, rather than causing a 400 response). Produce warning header and logs for
                    // any unused fields.
                    let result = body.into_raw().await;
                    match result {
                            Ok(body) => {
                                let mut unused_elements = Vec::new();
                                let param_sm_context_status_notification: Option<models::SmContextStatusNotification> = if !body.is_empty() {
                                    let deserializer = &mut serde_json::Deserializer::from_slice(&*body);
                                    match serde_ignored::deserialize(deserializer, |path| {
                                            warn!("Ignoring unknown field in body: {}", path);
                                            unused_elements.push(path.to_string());
                                    }) {
                                        Ok(param_sm_context_status_notification) => param_sm_context_status_notification,
                                        Err(e) => return Ok(Response::builder()
                                                        .status(StatusCode::BAD_REQUEST)
                                                        .body(Body::from(format!("Couldn't parse body parameter SmContextStatusNotification - doesn't match schema: {}", e)))
                                                        .expect("Unable to create Bad Request response for invalid body parameter SmContextStatusNotification due to schema")),
                                    }
                                } else {
                                    None
                                };
                                let param_sm_context_status_notification = match param_sm_context_status_notification {
                                    Some(param_sm_context_status_notification) => param_sm_context_status_notification,
                                    None => return Ok(Response::builder()
                                                        .status(StatusCode::BAD_REQUEST)
                                                        .body(Body::from("Missing required body parameter SmContextStatusNotification"))
                                                        .expect("Unable to create Bad Request response for missing body parameter SmContextStatusNotification")),
                                };

                                let result = api_impl.sm_context_status_notification_post(
                                            callback_request_body_sm_context_status_uri,
                                            param_sm_context_status_notification,
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
                                                SmContextStatusNotificationPostResponse::SuccessfulNotification
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(204).expect("Unable to turn 204 into a StatusCode");
                                                },
                                                SmContextStatusNotificationPostResponse::TemporaryRedirect
                                                    {
                                                        body,
                                                        location,
                                                        param_3gpp_sbi_target_nf_id
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
                                                    if let Some(param_3gpp_sbi_target_nf_id) = param_3gpp_sbi_target_nf_id {
                                                    let param_3gpp_sbi_target_nf_id = match header::IntoHeaderValue(param_3gpp_sbi_target_nf_id).try_into() {
                                                        Ok(val) => val,
                                                        Err(e) => {
                                                            return Ok(Response::builder()
                                                                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                                                                    .body(Body::from(format!("An internal server error occurred handling param_3gpp_sbi_target_nf_id header - {}", e)))
                                                                    .expect("Unable to create Internal Server Error for invalid response header"))
                                                        }
                                                    };

                                                    response.headers_mut().insert(
                                                        HeaderName::from_static("3gpp-sbi-target-nf-id"),
                                                        param_3gpp_sbi_target_nf_id
                                                    );
                                                    }
                                                    *response.status_mut() = StatusCode::from_u16(307).expect("Unable to turn 307 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for SM_CONTEXT_STATUS_NOTIFICATION_POST_TEMPORARY_REDIRECT"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                SmContextStatusNotificationPostResponse::PermanentRedirect
                                                    {
                                                        body,
                                                        location,
                                                        param_3gpp_sbi_target_nf_id
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
                                                    if let Some(param_3gpp_sbi_target_nf_id) = param_3gpp_sbi_target_nf_id {
                                                    let param_3gpp_sbi_target_nf_id = match header::IntoHeaderValue(param_3gpp_sbi_target_nf_id).try_into() {
                                                        Ok(val) => val,
                                                        Err(e) => {
                                                            return Ok(Response::builder()
                                                                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                                                                    .body(Body::from(format!("An internal server error occurred handling param_3gpp_sbi_target_nf_id header - {}", e)))
                                                                    .expect("Unable to create Internal Server Error for invalid response header"))
                                                        }
                                                    };

                                                    response.headers_mut().insert(
                                                        HeaderName::from_static("3gpp-sbi-target-nf-id"),
                                                        param_3gpp_sbi_target_nf_id
                                                    );
                                                    }
                                                    *response.status_mut() = StatusCode::from_u16(308).expect("Unable to turn 308 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for SM_CONTEXT_STATUS_NOTIFICATION_POST_PERMANENT_REDIRECT"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                SmContextStatusNotificationPostResponse::BadRequest
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(400).expect("Unable to turn 400 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for SM_CONTEXT_STATUS_NOTIFICATION_POST_BAD_REQUEST"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                SmContextStatusNotificationPostResponse::Forbidden
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(403).expect("Unable to turn 403 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for SM_CONTEXT_STATUS_NOTIFICATION_POST_FORBIDDEN"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                SmContextStatusNotificationPostResponse::NotFound
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(404).expect("Unable to turn 404 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for SM_CONTEXT_STATUS_NOTIFICATION_POST_NOT_FOUND"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                SmContextStatusNotificationPostResponse::LengthRequired
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(411).expect("Unable to turn 411 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for SM_CONTEXT_STATUS_NOTIFICATION_POST_LENGTH_REQUIRED"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                SmContextStatusNotificationPostResponse::PayloadTooLarge
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(413).expect("Unable to turn 413 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for SM_CONTEXT_STATUS_NOTIFICATION_POST_PAYLOAD_TOO_LARGE"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                SmContextStatusNotificationPostResponse::UnsupportedMediaType
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(415).expect("Unable to turn 415 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for SM_CONTEXT_STATUS_NOTIFICATION_POST_UNSUPPORTED_MEDIA_TYPE"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                SmContextStatusNotificationPostResponse::TooManyRequests
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(429).expect("Unable to turn 429 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for SM_CONTEXT_STATUS_NOTIFICATION_POST_TOO_MANY_REQUESTS"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                SmContextStatusNotificationPostResponse::InternalServerError
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(500).expect("Unable to turn 500 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for SM_CONTEXT_STATUS_NOTIFICATION_POST_INTERNAL_SERVER_ERROR"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                SmContextStatusNotificationPostResponse::ServiceUnavailable
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(503).expect("Unable to turn 503 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for SM_CONTEXT_STATUS_NOTIFICATION_POST_SERVICE_UNAVAILABLE"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
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
                                                .body(Body::from(format!("Couldn't read body parameter SmContextStatusNotification: {}", e)))
                                                .expect("Unable to create Bad Request response due to unable to read body parameter SmContextStatusNotification")),
                        }
                }

                _ if path.matched(paths::ID_REQUEST_BODY_ISMFPDUSESSIONURI) => method_not_allowed(),
                _ if path.matched(paths::ID_REQUEST_BODY_ISMFPDUSESSIONURI_MODIFY) => {
                    method_not_allowed()
                }
                _ if path.matched(paths::ID_REQUEST_BODY_ISMFPDUSESSIONURI_TRANSFER_MT_DATA) => {
                    method_not_allowed()
                }
                _ if path.matched(paths::ID_REQUEST_BODY_SMCONTEXTSTATUSURI) => {
                    method_not_allowed()
                }
                _ if path.matched(paths::ID_REQUEST_BODY_VSMFPDUSESSIONURI) => method_not_allowed(),
                _ if path.matched(paths::ID_REQUEST_BODY_VSMFPDUSESSIONURI_MODIFY) => {
                    method_not_allowed()
                }
                _ if path.matched(paths::ID_REQUEST_BODY_VSMFPDUSESSIONURI_TRANSFER_MT_DATA) => {
                    method_not_allowed()
                }
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
            // NotifyStatus - POST /{$request.body#/vsmfPduSessionUri}
            hyper::Method::POST if path.matched(paths::ID_REQUEST_BODY_VSMFPDUSESSIONURI) => {
                Some("NotifyStatus")
            }
            // NotifyStatusIsfm - POST /{$request.body#/ismfPduSessionUri}
            hyper::Method::POST if path.matched(paths::ID_REQUEST_BODY_ISMFPDUSESSIONURI) => {
                Some("NotifyStatusIsfm")
            }
            // ModifyPduSession - POST /{$request.body#/vsmfPduSessionUri}/modify
            hyper::Method::POST
                if path.matched(paths::ID_REQUEST_BODY_VSMFPDUSESSIONURI_MODIFY) =>
            {
                Some("ModifyPduSession")
            }
            // ModifyPduSessionIsmf - POST /{$request.body#/ismfPduSessionUri}/modify
            hyper::Method::POST
                if path.matched(paths::ID_REQUEST_BODY_ISMFPDUSESSIONURI_MODIFY) =>
            {
                Some("ModifyPduSessionIsmf")
            }
            // TransferMtData - POST /{$request.body#/vsmfPduSessionUri}/transfer-mt-data
            hyper::Method::POST
                if path.matched(paths::ID_REQUEST_BODY_VSMFPDUSESSIONURI_TRANSFER_MT_DATA) =>
            {
                Some("TransferMtData")
            }
            // TransferMtDataIsmf - POST /{$request.body#/ismfPduSessionUri}/transfer-mt-data
            hyper::Method::POST
                if path.matched(paths::ID_REQUEST_BODY_ISMFPDUSESSIONURI_TRANSFER_MT_DATA) =>
            {
                Some("TransferMtDataIsmf")
            }
            // SmContextStatusNotificationPost - POST /{$request.body#/smContextStatusUri}
            hyper::Method::POST if path.matched(paths::ID_REQUEST_BODY_SMCONTEXTSTATUSURI) => {
                Some("SmContextStatusNotificationPost")
            }
            _ => None,
        }
    }
}
