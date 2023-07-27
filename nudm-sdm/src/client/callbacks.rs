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
use crate::DatachangeNotificationRequestBodyCallbackReferencePostResponse;
// use crate::DatachangeNotificationRequestBodyCallbackReferencePostResponse;

mod paths {
    use lazy_static::lazy_static;

    lazy_static! {
        pub static ref GLOBAL_REGEX_SET: regex::RegexSet =
            regex::RegexSet::new(vec![r"^/nudm-sdm/v2/{request.body#/callbackReference}$"])
                .expect("Unable to create global regex set");
    }
    pub(crate) static ID_REQUEST_BODY_CALLBACKREFERENCE: usize = 0;
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
                // DatachangeNotificationRequestBodyCallbackReferencePost - POST /{request.body#/callbackReference}
                hyper::Method::POST if path.matched(paths::ID_REQUEST_BODY_CALLBACKREFERENCE) => {
                    // Body parameters (note that non-required body parameters will ignore garbage
                    // values, rather than causing a 400 response). Produce warning header and logs for
                    // any unused fields.
                    let result = body.into_raw().await;
                    match result {
                            Ok(body) => {
                                let mut unused_elements = Vec::new();
                                let param_modification_notification: Option<models::ModificationNotification> = if !body.is_empty() {
                                    let deserializer = &mut serde_json::Deserializer::from_slice(&*body);
                                    match serde_ignored::deserialize(deserializer, |path| {
                                            warn!("Ignoring unknown field in body: {}", path);
                                            unused_elements.push(path.to_string());
                                    }) {
                                        Ok(param_modification_notification) => param_modification_notification,
                                        Err(e) => return Ok(Response::builder()
                                                        .status(StatusCode::BAD_REQUEST)
                                                        .body(Body::from(format!("Couldn't parse body parameter ModificationNotification - doesn't match schema: {}", e)))
                                                        .expect("Unable to create Bad Request response for invalid body parameter ModificationNotification due to schema")),
                                    }
                                } else {
                                    None
                                };
                                let param_modification_notification = match param_modification_notification {
                                    Some(param_modification_notification) => param_modification_notification,
                                    None => return Ok(Response::builder()
                                                        .status(StatusCode::BAD_REQUEST)
                                                        .body(Body::from("Missing required body parameter ModificationNotification"))
                                                        .expect("Unable to create Bad Request response for missing body parameter ModificationNotification")),
                                };

                                let result = api_impl.datachange_notification_request_body_callback_reference_post(
                                            param_modification_notification,
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
                                                DatachangeNotificationRequestBodyCallbackReferencePostResponse::SuccessfulNotificationResponse
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(204).expect("Unable to turn 204 into a StatusCode");
                                                },
                                                DatachangeNotificationRequestBodyCallbackReferencePostResponse::TemporaryRedirect
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
                                                            .expect("Unable to create Content-Type header for DATACHANGE_NOTIFICATION_REQUEST_BODY_CALLBACK_REFERENCE_POST_TEMPORARY_REDIRECT"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                DatachangeNotificationRequestBodyCallbackReferencePostResponse::PermanentRedirect
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
                                                            .expect("Unable to create Content-Type header for DATACHANGE_NOTIFICATION_REQUEST_BODY_CALLBACK_REFERENCE_POST_PERMANENT_REDIRECT"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                DatachangeNotificationRequestBodyCallbackReferencePostResponse::BadRequest
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(400).expect("Unable to turn 400 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for DATACHANGE_NOTIFICATION_REQUEST_BODY_CALLBACK_REFERENCE_POST_BAD_REQUEST"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                DatachangeNotificationRequestBodyCallbackReferencePostResponse::NotFound
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(404).expect("Unable to turn 404 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for DATACHANGE_NOTIFICATION_REQUEST_BODY_CALLBACK_REFERENCE_POST_NOT_FOUND"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                DatachangeNotificationRequestBodyCallbackReferencePostResponse::InternalServerError
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(500).expect("Unable to turn 500 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for DATACHANGE_NOTIFICATION_REQUEST_BODY_CALLBACK_REFERENCE_POST_INTERNAL_SERVER_ERROR"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                DatachangeNotificationRequestBodyCallbackReferencePostResponse::ServiceUnavailable
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(503).expect("Unable to turn 503 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for DATACHANGE_NOTIFICATION_REQUEST_BODY_CALLBACK_REFERENCE_POST_SERVICE_UNAVAILABLE"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                DatachangeNotificationRequestBodyCallbackReferencePostResponse::UnexpectedError
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
                                                .body(Body::from(format!("Couldn't read body parameter ModificationNotification: {}", e)))
                                                .expect("Unable to create Bad Request response due to unable to read body parameter ModificationNotification")),
                        }
                }

                // DatachangeNotificationRequestBodyCallbackReferencePost - POST /{request.body#/callbackReference}
                hyper::Method::POST if path.matched(paths::ID_REQUEST_BODY_CALLBACKREFERENCE) => {
                    // Body parameters (note that non-required body parameters will ignore garbage
                    // values, rather than causing a 400 response). Produce warning header and logs for
                    // any unused fields.
                    let result = body.into_raw().await;
                    match result {
                            Ok(body) => {
                                let mut unused_elements = Vec::new();
                                let param_modification_notification: Option<models::ModificationNotification> = if !body.is_empty() {
                                    let deserializer = &mut serde_json::Deserializer::from_slice(&*body);
                                    match serde_ignored::deserialize(deserializer, |path| {
                                            warn!("Ignoring unknown field in body: {}", path);
                                            unused_elements.push(path.to_string());
                                    }) {
                                        Ok(param_modification_notification) => param_modification_notification,
                                        Err(e) => return Ok(Response::builder()
                                                        .status(StatusCode::BAD_REQUEST)
                                                        .body(Body::from(format!("Couldn't parse body parameter ModificationNotification - doesn't match schema: {}", e)))
                                                        .expect("Unable to create Bad Request response for invalid body parameter ModificationNotification due to schema")),
                                    }
                                } else {
                                    None
                                };
                                let param_modification_notification = match param_modification_notification {
                                    Some(param_modification_notification) => param_modification_notification,
                                    None => return Ok(Response::builder()
                                                        .status(StatusCode::BAD_REQUEST)
                                                        .body(Body::from("Missing required body parameter ModificationNotification"))
                                                        .expect("Unable to create Bad Request response for missing body parameter ModificationNotification")),
                                };

                                let result = api_impl.datachange_notification_request_body_callback_reference_post(
                                            param_modification_notification,
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
                                                DatachangeNotificationRequestBodyCallbackReferencePostResponse::SuccessfulNotificationResponse
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(204).expect("Unable to turn 204 into a StatusCode");
                                                },
                                                DatachangeNotificationRequestBodyCallbackReferencePostResponse::TemporaryRedirect
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
                                                            .expect("Unable to create Content-Type header for DATACHANGE_NOTIFICATION_REQUEST_BODY_CALLBACK_REFERENCE_POST_TEMPORARY_REDIRECT"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                DatachangeNotificationRequestBodyCallbackReferencePostResponse::PermanentRedirect
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
                                                            .expect("Unable to create Content-Type header for DATACHANGE_NOTIFICATION_REQUEST_BODY_CALLBACK_REFERENCE_POST_PERMANENT_REDIRECT"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                DatachangeNotificationRequestBodyCallbackReferencePostResponse::BadRequest
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(400).expect("Unable to turn 400 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for DATACHANGE_NOTIFICATION_REQUEST_BODY_CALLBACK_REFERENCE_POST_BAD_REQUEST"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                DatachangeNotificationRequestBodyCallbackReferencePostResponse::NotFound
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(404).expect("Unable to turn 404 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for DATACHANGE_NOTIFICATION_REQUEST_BODY_CALLBACK_REFERENCE_POST_NOT_FOUND"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                DatachangeNotificationRequestBodyCallbackReferencePostResponse::InternalServerError
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(500).expect("Unable to turn 500 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for DATACHANGE_NOTIFICATION_REQUEST_BODY_CALLBACK_REFERENCE_POST_INTERNAL_SERVER_ERROR"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                DatachangeNotificationRequestBodyCallbackReferencePostResponse::ServiceUnavailable
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(503).expect("Unable to turn 503 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for DATACHANGE_NOTIFICATION_REQUEST_BODY_CALLBACK_REFERENCE_POST_SERVICE_UNAVAILABLE"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                DatachangeNotificationRequestBodyCallbackReferencePostResponse::UnexpectedError
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
                                                .body(Body::from(format!("Couldn't read body parameter ModificationNotification: {}", e)))
                                                .expect("Unable to create Bad Request response due to unable to read body parameter ModificationNotification")),
                        }
                }

                _ if path.matched(paths::ID_REQUEST_BODY_CALLBACKREFERENCE) => method_not_allowed(),
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
            // DatachangeNotificationRequestBodyCallbackReferencePost - POST /{request.body#/callbackReference}
            hyper::Method::POST if path.matched(paths::ID_REQUEST_BODY_CALLBACKREFERENCE) => {
                Some("DatachangeNotificationRequestBodyCallbackReferencePost")
            }
            // DatachangeNotificationRequestBodyCallbackReferencePost - POST /{request.body#/callbackReference}
            hyper::Method::POST if path.matched(paths::ID_REQUEST_BODY_CALLBACKREFERENCE) => {
                Some("DatachangeNotificationRequestBodyCallbackReferencePost")
            }
            _ => None,
        }
    }
}
