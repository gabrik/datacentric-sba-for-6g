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

use crate::{
    Api, PostPduSessionsResponse, PostSmContextsResponse, ReleasePduSessionResponse,
    ReleaseSmContextResponse, RetrievePduSessionResponse, RetrieveSmContextResponse,
    SendMoDataResponse, TransferMoDataResponse, UpdatePduSessionResponse, UpdateSmContextResponse,
};

pub mod callbacks;

mod paths {
    use lazy_static::lazy_static;

    lazy_static! {
        pub static ref GLOBAL_REGEX_SET: regex::RegexSet = regex::RegexSet::new(vec![
            r"^/nsmf-pdusession/v1/pdu-sessions$",
            r"^/nsmf-pdusession/v1/pdu-sessions/(?P<pduSessionRef>[^/?#]*)/modify$",
            r"^/nsmf-pdusession/v1/pdu-sessions/(?P<pduSessionRef>[^/?#]*)/release$",
            r"^/nsmf-pdusession/v1/pdu-sessions/(?P<pduSessionRef>[^/?#]*)/retrieve$",
            r"^/nsmf-pdusession/v1/pdu-sessions/(?P<pduSessionRef>[^/?#]*)/transfer-mo-data$",
            r"^/nsmf-pdusession/v1/sm-contexts$",
            r"^/nsmf-pdusession/v1/sm-contexts/(?P<smContextRef>[^/?#]*)/modify$",
            r"^/nsmf-pdusession/v1/sm-contexts/(?P<smContextRef>[^/?#]*)/release$",
            r"^/nsmf-pdusession/v1/sm-contexts/(?P<smContextRef>[^/?#]*)/retrieve$",
            r"^/nsmf-pdusession/v1/sm-contexts/(?P<smContextRef>[^/?#]*)/send-mo-data$"
        ])
        .expect("Unable to create global regex set");
    }
    pub(crate) static ID_PDU_SESSIONS: usize = 0;
    pub(crate) static ID_PDU_SESSIONS_PDUSESSIONREF_MODIFY: usize = 1;
    lazy_static! {
        pub static ref REGEX_PDU_SESSIONS_PDUSESSIONREF_MODIFY: regex::Regex =
            #[allow(clippy::invalid_regex)]
            regex::Regex::new(
                r"^/nsmf-pdusession/v1/pdu-sessions/(?P<pduSessionRef>[^/?#]*)/modify$"
            )
            .expect("Unable to create regex for PDU_SESSIONS_PDUSESSIONREF_MODIFY");
    }
    pub(crate) static ID_PDU_SESSIONS_PDUSESSIONREF_RELEASE: usize = 2;
    lazy_static! {
        pub static ref REGEX_PDU_SESSIONS_PDUSESSIONREF_RELEASE: regex::Regex =
            #[allow(clippy::invalid_regex)]
            regex::Regex::new(
                r"^/nsmf-pdusession/v1/pdu-sessions/(?P<pduSessionRef>[^/?#]*)/release$"
            )
            .expect("Unable to create regex for PDU_SESSIONS_PDUSESSIONREF_RELEASE");
    }
    pub(crate) static ID_PDU_SESSIONS_PDUSESSIONREF_RETRIEVE: usize = 3;
    lazy_static! {
        pub static ref REGEX_PDU_SESSIONS_PDUSESSIONREF_RETRIEVE: regex::Regex =
            #[allow(clippy::invalid_regex)]
            regex::Regex::new(
                r"^/nsmf-pdusession/v1/pdu-sessions/(?P<pduSessionRef>[^/?#]*)/retrieve$"
            )
            .expect("Unable to create regex for PDU_SESSIONS_PDUSESSIONREF_RETRIEVE");
    }
    pub(crate) static ID_PDU_SESSIONS_PDUSESSIONREF_TRANSFER_MO_DATA: usize = 4;
    lazy_static! {
        pub static ref REGEX_PDU_SESSIONS_PDUSESSIONREF_TRANSFER_MO_DATA: regex::Regex =
            #[allow(clippy::invalid_regex)]
            regex::Regex::new(
                r"^/nsmf-pdusession/v1/pdu-sessions/(?P<pduSessionRef>[^/?#]*)/transfer-mo-data$"
            )
            .expect("Unable to create regex for PDU_SESSIONS_PDUSESSIONREF_TRANSFER_MO_DATA");
    }
    pub(crate) static ID_SM_CONTEXTS: usize = 5;
    pub(crate) static ID_SM_CONTEXTS_SMCONTEXTREF_MODIFY: usize = 6;
    lazy_static! {
        pub static ref REGEX_SM_CONTEXTS_SMCONTEXTREF_MODIFY: regex::Regex =
            #[allow(clippy::invalid_regex)]
            regex::Regex::new(
                r"^/nsmf-pdusession/v1/sm-contexts/(?P<smContextRef>[^/?#]*)/modify$"
            )
            .expect("Unable to create regex for SM_CONTEXTS_SMCONTEXTREF_MODIFY");
    }
    pub(crate) static ID_SM_CONTEXTS_SMCONTEXTREF_RELEASE: usize = 7;
    lazy_static! {
        pub static ref REGEX_SM_CONTEXTS_SMCONTEXTREF_RELEASE: regex::Regex =
            #[allow(clippy::invalid_regex)]
            regex::Regex::new(
                r"^/nsmf-pdusession/v1/sm-contexts/(?P<smContextRef>[^/?#]*)/release$"
            )
            .expect("Unable to create regex for SM_CONTEXTS_SMCONTEXTREF_RELEASE");
    }
    pub(crate) static ID_SM_CONTEXTS_SMCONTEXTREF_RETRIEVE: usize = 8;
    lazy_static! {
        pub static ref REGEX_SM_CONTEXTS_SMCONTEXTREF_RETRIEVE: regex::Regex =
            #[allow(clippy::invalid_regex)]
            regex::Regex::new(
                r"^/nsmf-pdusession/v1/sm-contexts/(?P<smContextRef>[^/?#]*)/retrieve$"
            )
            .expect("Unable to create regex for SM_CONTEXTS_SMCONTEXTREF_RETRIEVE");
    }
    pub(crate) static ID_SM_CONTEXTS_SMCONTEXTREF_SEND_MO_DATA: usize = 9;
    lazy_static! {
        pub static ref REGEX_SM_CONTEXTS_SMCONTEXTREF_SEND_MO_DATA: regex::Regex =
            #[allow(clippy::invalid_regex)]
            regex::Regex::new(
                r"^/nsmf-pdusession/v1/sm-contexts/(?P<smContextRef>[^/?#]*)/send-mo-data$"
            )
            .expect("Unable to create regex for SM_CONTEXTS_SMCONTEXTREF_SEND_MO_DATA");
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
                // ReleasePduSession - POST /pdu-sessions/{pduSessionRef}/release
                hyper::Method::POST
                    if path.matched(paths::ID_PDU_SESSIONS_PDUSESSIONREF_RELEASE) =>
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
                                "nsmf-pdusession".to_string(), // Access to the nsmf-pdusession API
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
                    paths::REGEX_PDU_SESSIONS_PDUSESSIONREF_RELEASE
                    .captures(path)
                    .unwrap_or_else(||
                        panic!("Path {} matched RE PDU_SESSIONS_PDUSESSIONREF_RELEASE in set but failed match against \"{}\"", path, paths::REGEX_PDU_SESSIONS_PDUSESSIONREF_RELEASE.as_str())
                    );

                    let param_pdu_session_ref = match percent_encoding::percent_decode(path_params["pduSessionRef"].as_bytes()).decode_utf8() {
                    Ok(param_pdu_session_ref) => match param_pdu_session_ref.parse::<String>() {
                        Ok(param_pdu_session_ref) => param_pdu_session_ref,
                        Err(e) => return Ok(Response::builder()
                                        .status(StatusCode::BAD_REQUEST)
                                        .body(Body::from(format!("Couldn't parse path parameter pduSessionRef: {}", e)))
                                        .expect("Unable to create Bad Request response for invalid path parameter")),
                    },
                    Err(_) => return Ok(Response::builder()
                                        .status(StatusCode::BAD_REQUEST)
                                        .body(Body::from(format!("Couldn't percent-decode path parameter as UTF-8: {}", &path_params["pduSessionRef"])))
                                        .expect("Unable to create Bad Request response for invalid percent decode"))
                };

                    // Body parameters (note that non-required body parameters will ignore garbage
                    // values, rather than causing a 400 response). Produce warning header and logs for
                    // any unused fields.
                    let result = body.into_raw().await;
                    match result {
                        // Ok(body) => {
                        //     let mut unused_elements = Vec::new();
                        //     let param_release_data: Option<models::ReleaseData> =
                        //         if !body.is_empty() {
                        //             let deserializer =
                        //                 &mut serde_json::Deserializer::from_slice(&*body);
                        //             match serde_ignored::deserialize(deserializer, |path| {
                        //                 warn!("Ignoring unknown field in body: {}", path);
                        //                 unused_elements.push(path.to_string());
                        //             }) {
                        //                 Ok(param_release_data) => param_release_data,
                        //                 Err(_) => None,
                        //             }
                        //         } else {
                        //             None
                        //         };

                        //     // Body parameters (note that non-required body parameters will ignore garbage
                        //     // values, rather than causing a 400 response). Produce warning header and logs for
                        //     // any unused fields.
                        //     let result = body.into_raw();
                        //     match result.await {
                            Ok(body) => {
                                let mut unused_elements: Vec<String> = vec![];
                                let param_release_data: Option<models::ReleaseData> =
                                if !body.is_empty() {
                                    let deserializer =
                                        &mut serde_json::Deserializer::from_slice(&*body);
                                    match serde_ignored::deserialize(deserializer, |path| {
                                        warn!("Ignoring unknown field in body: {}", path);
                                        unused_elements.push(path.to_string());
                                    }) {
                                        Ok(param_release_data) => param_release_data,
                                        Err(_) => None,
                                    }
                                } else {
                                    None
                                };
                                // Get multipart chunks.

                                // Extract the top-level content type header.
                                let content_type_mime = headers
                                    .get(CONTENT_TYPE)
                                    .ok_or_else(|| "Missing content-type header".to_string())
                                    .and_then(|v| v.to_str().map_err(|e| format!("Couldn't read content-type header value for ReleasePduSession: {}", e)))
                                    .and_then(|v| v.parse::<Mime2>().map_err(|_e| "Couldn't parse content-type header value for ReleasePduSession".to_string()));

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
                                                .expect("Unable to create Bad Request response due to unable to read content-type header for ReleasePduSession"));
                                    }
                                }

                                // &*body expresses the body as a byteslice, &mut provides a
                                // mutable reference to that byteslice.
                                let nodes = match read_multipart_body(&mut&*body, &multi_part_headers, false) {
                                    Ok(nodes) => nodes,
                                    Err(e) => {
                                        return Ok(Response::builder()
                                                .status(StatusCode::BAD_REQUEST)
                                                .body(Body::from(format!("Could not read multipart body for ReleasePduSession: {}", e)))
                                                .expect("Unable to create Bad Request response due to unable to read multipart body for ReleasePduSession"));
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

                                let result = api_impl.release_pdu_session(
                                            param_pdu_session_ref,
                                            param_release_data,
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
                                                ReleasePduSessionResponse::SuccessfulReleaseOfAPDUSessionWithContentInTheResponse
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(200).expect("Unable to turn 200 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for RELEASE_PDU_SESSION_SUCCESSFUL_RELEASE_OF_APDU_SESSION_WITH_CONTENT_IN_THE_RESPONSE"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                ReleasePduSessionResponse::SuccessfulReleaseOfAPDUSession
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(204).expect("Unable to turn 204 into a StatusCode");
                                                },
                                                ReleasePduSessionResponse::TemporaryRedirect
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
                                                            .expect("Unable to create Content-Type header for RELEASE_PDU_SESSION_TEMPORARY_REDIRECT"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                ReleasePduSessionResponse::PermanentRedirect
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
                                                            .expect("Unable to create Content-Type header for RELEASE_PDU_SESSION_PERMANENT_REDIRECT"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                ReleasePduSessionResponse::BadRequest
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(400).expect("Unable to turn 400 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for RELEASE_PDU_SESSION_BAD_REQUEST"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                ReleasePduSessionResponse::Forbidden
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(403).expect("Unable to turn 403 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for RELEASE_PDU_SESSION_FORBIDDEN"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                ReleasePduSessionResponse::NotFound
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(404).expect("Unable to turn 404 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for RELEASE_PDU_SESSION_NOT_FOUND"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                ReleasePduSessionResponse::LengthRequired
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(411).expect("Unable to turn 411 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for RELEASE_PDU_SESSION_LENGTH_REQUIRED"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                ReleasePduSessionResponse::PayloadTooLarge
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(413).expect("Unable to turn 413 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for RELEASE_PDU_SESSION_PAYLOAD_TOO_LARGE"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                ReleasePduSessionResponse::UnsupportedMediaType
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(415).expect("Unable to turn 415 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for RELEASE_PDU_SESSION_UNSUPPORTED_MEDIA_TYPE"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                ReleasePduSessionResponse::TooManyRequests
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(429).expect("Unable to turn 429 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for RELEASE_PDU_SESSION_TOO_MANY_REQUESTS"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                ReleasePduSessionResponse::InternalServerError
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(500).expect("Unable to turn 500 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for RELEASE_PDU_SESSION_INTERNAL_SERVER_ERROR"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                ReleasePduSessionResponse::ServiceUnavailable
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(503).expect("Unable to turn 503 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for RELEASE_PDU_SESSION_SERVICE_UNAVAILABLE"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                ReleasePduSessionResponse::GenericError
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
                                                .body(Body::from(format!("Couldn't read body parameter ReleaseData: {}", e)))
                                                .expect("Unable to create Bad Request response due to unable to read body parameter ReleaseData")),
                        }
                }

                // RetrievePduSession - POST /pdu-sessions/{pduSessionRef}/retrieve
                hyper::Method::POST
                    if path.matched(paths::ID_PDU_SESSIONS_PDUSESSIONREF_RETRIEVE) =>
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
                                "nsmf-pdusession".to_string(), // Access to the nsmf-pdusession API
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
                    paths::REGEX_PDU_SESSIONS_PDUSESSIONREF_RETRIEVE
                    .captures(path)
                    .unwrap_or_else(||
                        panic!("Path {} matched RE PDU_SESSIONS_PDUSESSIONREF_RETRIEVE in set but failed match against \"{}\"", path, paths::REGEX_PDU_SESSIONS_PDUSESSIONREF_RETRIEVE.as_str())
                    );

                    let param_pdu_session_ref = match percent_encoding::percent_decode(path_params["pduSessionRef"].as_bytes()).decode_utf8() {
                    Ok(param_pdu_session_ref) => match param_pdu_session_ref.parse::<String>() {
                        Ok(param_pdu_session_ref) => param_pdu_session_ref,
                        Err(e) => return Ok(Response::builder()
                                        .status(StatusCode::BAD_REQUEST)
                                        .body(Body::from(format!("Couldn't parse path parameter pduSessionRef: {}", e)))
                                        .expect("Unable to create Bad Request response for invalid path parameter")),
                    },
                    Err(_) => return Ok(Response::builder()
                                        .status(StatusCode::BAD_REQUEST)
                                        .body(Body::from(format!("Couldn't percent-decode path parameter as UTF-8: {}", &path_params["pduSessionRef"])))
                                        .expect("Unable to create Bad Request response for invalid percent decode"))
                };

                    // Body parameters (note that non-required body parameters will ignore garbage
                    // values, rather than causing a 400 response). Produce warning header and logs for
                    // any unused fields.
                    let result = body.into_raw().await;
                    match result {
                            Ok(body) => {
                                let mut unused_elements = Vec::new();
                                let param_retrieve_data: Option<models::RetrieveData> = if !body.is_empty() {
                                    let deserializer = &mut serde_json::Deserializer::from_slice(&*body);
                                    match serde_ignored::deserialize(deserializer, |path| {
                                            warn!("Ignoring unknown field in body: {}", path);
                                            unused_elements.push(path.to_string());
                                    }) {
                                        Ok(param_retrieve_data) => param_retrieve_data,
                                        Err(e) => return Ok(Response::builder()
                                                        .status(StatusCode::BAD_REQUEST)
                                                        .body(Body::from(format!("Couldn't parse body parameter RetrieveData - doesn't match schema: {}", e)))
                                                        .expect("Unable to create Bad Request response for invalid body parameter RetrieveData due to schema")),
                                    }
                                } else {
                                    None
                                };
                                let param_retrieve_data = match param_retrieve_data {
                                    Some(param_retrieve_data) => param_retrieve_data,
                                    None => return Ok(Response::builder()
                                                        .status(StatusCode::BAD_REQUEST)
                                                        .body(Body::from("Missing required body parameter RetrieveData"))
                                                        .expect("Unable to create Bad Request response for missing body parameter RetrieveData")),
                                };

                                let result = api_impl.retrieve_pdu_session(
                                            param_pdu_session_ref,
                                            param_retrieve_data,
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
                                                RetrievePduSessionResponse::SuccessfulInformationRetrieval
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(200).expect("Unable to turn 200 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for RETRIEVE_PDU_SESSION_SUCCESSFUL_INFORMATION_RETRIEVAL"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                RetrievePduSessionResponse::TemporaryRedirect
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
                                                            .expect("Unable to create Content-Type header for RETRIEVE_PDU_SESSION_TEMPORARY_REDIRECT"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                RetrievePduSessionResponse::PermanentRedirect
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
                                                            .expect("Unable to create Content-Type header for RETRIEVE_PDU_SESSION_PERMANENT_REDIRECT"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                RetrievePduSessionResponse::BadRequest
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(400).expect("Unable to turn 400 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for RETRIEVE_PDU_SESSION_BAD_REQUEST"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                RetrievePduSessionResponse::Forbidden
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(403).expect("Unable to turn 403 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for RETRIEVE_PDU_SESSION_FORBIDDEN"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                RetrievePduSessionResponse::NotFound
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(404).expect("Unable to turn 404 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for RETRIEVE_PDU_SESSION_NOT_FOUND"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                RetrievePduSessionResponse::LengthRequired
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(411).expect("Unable to turn 411 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for RETRIEVE_PDU_SESSION_LENGTH_REQUIRED"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                RetrievePduSessionResponse::PayloadTooLarge
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(413).expect("Unable to turn 413 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for RETRIEVE_PDU_SESSION_PAYLOAD_TOO_LARGE"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                RetrievePduSessionResponse::UnsupportedMediaType
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(415).expect("Unable to turn 415 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for RETRIEVE_PDU_SESSION_UNSUPPORTED_MEDIA_TYPE"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                RetrievePduSessionResponse::TooManyRequests
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(429).expect("Unable to turn 429 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for RETRIEVE_PDU_SESSION_TOO_MANY_REQUESTS"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                RetrievePduSessionResponse::InternalServerError
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(500).expect("Unable to turn 500 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for RETRIEVE_PDU_SESSION_INTERNAL_SERVER_ERROR"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                RetrievePduSessionResponse::ServiceUnavailable
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(503).expect("Unable to turn 503 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for RETRIEVE_PDU_SESSION_SERVICE_UNAVAILABLE"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                RetrievePduSessionResponse::GatewayTimeout
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(504).expect("Unable to turn 504 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for RETRIEVE_PDU_SESSION_GATEWAY_TIMEOUT"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                RetrievePduSessionResponse::GenericError
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
                                                .body(Body::from(format!("Couldn't read body parameter RetrieveData: {}", e)))
                                                .expect("Unable to create Bad Request response due to unable to read body parameter RetrieveData")),
                        }
                }

                // TransferMoData - POST /pdu-sessions/{pduSessionRef}/transfer-mo-data
                hyper::Method::POST
                    if path.matched(paths::ID_PDU_SESSIONS_PDUSESSIONREF_TRANSFER_MO_DATA) =>
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
                                "nsmf-pdusession".to_string(), // Access to the nsmf-pdusession API
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
                    paths::REGEX_PDU_SESSIONS_PDUSESSIONREF_TRANSFER_MO_DATA
                    .captures(path)
                    .unwrap_or_else(||
                        panic!("Path {} matched RE PDU_SESSIONS_PDUSESSIONREF_TRANSFER_MO_DATA in set but failed match against \"{}\"", path, paths::REGEX_PDU_SESSIONS_PDUSESSIONREF_TRANSFER_MO_DATA.as_str())
                    );

                    let param_pdu_session_ref = match percent_encoding::percent_decode(path_params["pduSessionRef"].as_bytes()).decode_utf8() {
                    Ok(param_pdu_session_ref) => match param_pdu_session_ref.parse::<String>() {
                        Ok(param_pdu_session_ref) => param_pdu_session_ref,
                        Err(e) => return Ok(Response::builder()
                                        .status(StatusCode::BAD_REQUEST)
                                        .body(Body::from(format!("Couldn't parse path parameter pduSessionRef: {}", e)))
                                        .expect("Unable to create Bad Request response for invalid path parameter")),
                    },
                    Err(_) => return Ok(Response::builder()
                                        .status(StatusCode::BAD_REQUEST)
                                        .body(Body::from(format!("Couldn't percent-decode path parameter as UTF-8: {}", &path_params["pduSessionRef"])))
                                        .expect("Unable to create Bad Request response for invalid percent decode"))
                };

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
                                    .and_then(|v| v.to_str().map_err(|e| format!("Couldn't read content-type header value for TransferMoData: {}", e)))
                                    .and_then(|v| v.parse::<Mime2>().map_err(|_e| "Couldn't parse content-type header value for TransferMoData".to_string()));

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
                                                .expect("Unable to create Bad Request response due to unable to read content-type header for TransferMoData"));
                                    }
                                }

                                // &*body expresses the body as a byteslice, &mut provides a
                                // mutable reference to that byteslice.
                                let nodes = match read_multipart_body(&mut&*body, &multi_part_headers, false) {
                                    Ok(nodes) => nodes,
                                    Err(e) => {
                                        return Ok(Response::builder()
                                                .status(StatusCode::BAD_REQUEST)
                                                .body(Body::from(format!("Could not read multipart body for TransferMoData: {}", e)))
                                                .expect("Unable to create Bad Request response due to unable to read multipart body for TransferMoData"));
                                    }
                                };

                                let mut param_json_data = None;
                                let mut param_binary_mo_data = None;

                                for node in nodes {
                                    if let Node::Part(part) = node {
                                        let content_type = part.content_type().map(|x| format!("{}",x));
                                        match content_type.as_deref() {
                                            Some("application/json") if param_json_data.is_none() => {
                                                // Extract JSON part.
                                                let deserializer = &mut serde_json::Deserializer::from_slice(part.body.as_slice());
                                                let json_data: models::TransferMoDataReqData = match serde_ignored::deserialize(deserializer, |path| {
                                                    warn!("Ignoring unknown field in JSON part: {}", path);
                                                    unused_elements.push(path.to_string());
                                                }) {
                                                    Ok(json_data) => json_data,
                                                    Err(e) => return Ok(Response::builder()
                                                                    .status(StatusCode::BAD_REQUEST)
                                                                    .body(Body::from(format!("Couldn't parse body parameter models::TransferMoDataReqData - doesn't match schema: {}", e)))
                                                                    .expect("Unable to create Bad Request response for invalid body parameter models::TransferMoDataReqData due to schema"))
                                                };
                                                // Push JSON part to return object.
                                                param_json_data.get_or_insert(json_data);
                                            },
                                            Some("application/vnd.3gpp.5gnas") if param_binary_mo_data.is_none() => {
                                                param_binary_mo_data.get_or_insert(swagger::ByteArray(part.body));
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

                                let result = api_impl.transfer_mo_data(
                                            param_pdu_session_ref,
                                            param_json_data,
                                            param_binary_mo_data,
                                        &context
                                    ).await;
                                let mut response = Response::new(Body::empty());
                                response.headers_mut().insert(
                                            HeaderName::from_static("x-span-id"),
                                            HeaderValue::from_str((&context as &dyn Has<XSpanIdString>).get().0.clone().as_str())
                                                .expect("Unable to create X-Span-ID header value"));

                                        match result {
                                            Ok(rsp) => match rsp {
                                                TransferMoDataResponse::SuccessfulTransferingOfMOData
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(204).expect("Unable to turn 204 into a StatusCode");
                                                },
                                                TransferMoDataResponse::TemporaryRedirect
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
                                                            .expect("Unable to create Content-Type header for TRANSFER_MO_DATA_TEMPORARY_REDIRECT"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                TransferMoDataResponse::PermanentRedirect
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
                                                            .expect("Unable to create Content-Type header for TRANSFER_MO_DATA_PERMANENT_REDIRECT"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                TransferMoDataResponse::BadRequest
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(400).expect("Unable to turn 400 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for TRANSFER_MO_DATA_BAD_REQUEST"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                TransferMoDataResponse::Unauthorized
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(401).expect("Unable to turn 401 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for TRANSFER_MO_DATA_UNAUTHORIZED"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                TransferMoDataResponse::Forbidden
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(403).expect("Unable to turn 403 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for TRANSFER_MO_DATA_FORBIDDEN"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                TransferMoDataResponse::NotFound
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(404).expect("Unable to turn 404 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for TRANSFER_MO_DATA_NOT_FOUND"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                TransferMoDataResponse::LengthRequired
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(411).expect("Unable to turn 411 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for TRANSFER_MO_DATA_LENGTH_REQUIRED"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                TransferMoDataResponse::PayloadTooLarge
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(413).expect("Unable to turn 413 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for TRANSFER_MO_DATA_PAYLOAD_TOO_LARGE"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                TransferMoDataResponse::UnsupportedMediaType
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(415).expect("Unable to turn 415 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for TRANSFER_MO_DATA_UNSUPPORTED_MEDIA_TYPE"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                TransferMoDataResponse::TooManyRequests
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(429).expect("Unable to turn 429 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for TRANSFER_MO_DATA_TOO_MANY_REQUESTS"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                TransferMoDataResponse::InternalServerError
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(500).expect("Unable to turn 500 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for TRANSFER_MO_DATA_INTERNAL_SERVER_ERROR"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                TransferMoDataResponse::ServiceUnavailable
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(503).expect("Unable to turn 503 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for TRANSFER_MO_DATA_SERVICE_UNAVAILABLE"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                TransferMoDataResponse::GenericError
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
                                                .body(Body::from(format!("Couldn't read body parameter IndividualPDUSessionHSMFOrSMF: {}", e)))
                                                .expect("Unable to create Bad Request response due to unable to read body parameter IndividualPDUSessionHSMFOrSMF")),
                        }
                }

                // UpdatePduSession - POST /pdu-sessions/{pduSessionRef}/modify
                hyper::Method::POST
                    if path.matched(paths::ID_PDU_SESSIONS_PDUSESSIONREF_MODIFY) =>
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
                                "nsmf-pdusession".to_string(), // Access to the nsmf-pdusession API
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
                    paths::REGEX_PDU_SESSIONS_PDUSESSIONREF_MODIFY
                    .captures(path)
                    .unwrap_or_else(||
                        panic!("Path {} matched RE PDU_SESSIONS_PDUSESSIONREF_MODIFY in set but failed match against \"{}\"", path, paths::REGEX_PDU_SESSIONS_PDUSESSIONREF_MODIFY.as_str())
                    );

                    let param_pdu_session_ref = match percent_encoding::percent_decode(path_params["pduSessionRef"].as_bytes()).decode_utf8() {
                    Ok(param_pdu_session_ref) => match param_pdu_session_ref.parse::<String>() {
                        Ok(param_pdu_session_ref) => param_pdu_session_ref,
                        Err(e) => return Ok(Response::builder()
                                        .status(StatusCode::BAD_REQUEST)
                                        .body(Body::from(format!("Couldn't parse path parameter pduSessionRef: {}", e)))
                                        .expect("Unable to create Bad Request response for invalid path parameter")),
                    },
                    Err(_) => return Ok(Response::builder()
                                        .status(StatusCode::BAD_REQUEST)
                                        .body(Body::from(format!("Couldn't percent-decode path parameter as UTF-8: {}", &path_params["pduSessionRef"])))
                                        .expect("Unable to create Bad Request response for invalid percent decode"))
                };

                    // Body parameters (note that non-required body parameters will ignore garbage
                    // values, rather than causing a 400 response). Produce warning header and logs for
                    // any unused fields.
                    let result = body.into_raw().await;
                    match result {
                        Ok(body) => {
                            // let mut unused_elements = Vec::new();


                            // // Body parameters (note that non-required body parameters will ignore garbage
                            // // values, rather than causing a 400 response). Produce warning header and logs for
                            // // any unused fields.
                            // let result = body.into_raw();
                            // match result.await {
                            // Ok(body) => {
                                let mut unused_elements: Vec<String> = vec![];
                                let param_hsmf_update_data: Option<models::HsmfUpdateData> = if !body
                                .is_empty()
                            {
                                let deserializer =
                                    &mut serde_json::Deserializer::from_slice(&*body);
                                match serde_ignored::deserialize(deserializer, |path| {
                                            warn!("Ignoring unknown field in body: {}", path);
                                            unused_elements.push(path.to_string());
                                    }) {
                                        Ok(param_hsmf_update_data) => param_hsmf_update_data,
                                        Err(e) => return Ok(Response::builder()
                                                        .status(StatusCode::BAD_REQUEST)
                                                        .body(Body::from(format!("Couldn't parse body parameter HsmfUpdateData - doesn't match schema: {}", e)))
                                                        .expect("Unable to create Bad Request response for invalid body parameter HsmfUpdateData due to schema")),
                                    }
                            } else {
                                None
                            };
                            let param_hsmf_update_data = match param_hsmf_update_data {
                                    Some(param_hsmf_update_data) => param_hsmf_update_data,
                                    None => return Ok(Response::builder()
                                                        .status(StatusCode::BAD_REQUEST)
                                                        .body(Body::from("Missing required body parameter HsmfUpdateData"))
                                                        .expect("Unable to create Bad Request response for missing body parameter HsmfUpdateData")),
                                };
                                // Get multipart chunks.

                                // Extract the top-level content type header.
                                let content_type_mime = headers
                                    .get(CONTENT_TYPE)
                                    .ok_or_else(|| "Missing content-type header".to_string())
                                    .and_then(|v| v.to_str().map_err(|e| format!("Couldn't read content-type header value for UpdatePduSession: {}", e)))
                                    .and_then(|v| v.parse::<Mime2>().map_err(|_e| "Couldn't parse content-type header value for UpdatePduSession".to_string()));

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
                                                .expect("Unable to create Bad Request response due to unable to read content-type header for UpdatePduSession"));
                                    }
                                }

                                // &*body expresses the body as a byteslice, &mut provides a
                                // mutable reference to that byteslice.
                                let nodes = match read_multipart_body(&mut&*body, &multi_part_headers, false) {
                                    Ok(nodes) => nodes,
                                    Err(e) => {
                                        return Ok(Response::builder()
                                                .status(StatusCode::BAD_REQUEST)
                                                .body(Body::from(format!("Could not read multipart body for UpdatePduSession: {}", e)))
                                                .expect("Unable to create Bad Request response due to unable to read multipart body for UpdatePduSession"));
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

                                let result = api_impl.update_pdu_session(
                                            param_pdu_session_ref,
                                            param_hsmf_update_data,
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
                                                UpdatePduSessionResponse::SuccessfulUpdateOfAPDUSessionWithContentInTheResponse
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(200).expect("Unable to turn 200 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for UPDATE_PDU_SESSION_SUCCESSFUL_UPDATE_OF_APDU_SESSION_WITH_CONTENT_IN_THE_RESPONSE"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                UpdatePduSessionResponse::SuccessfulUpdateOfAPDUSessionWithoutContentInTheResponse
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(204).expect("Unable to turn 204 into a StatusCode");
                                                },
                                                UpdatePduSessionResponse::TemporaryRedirect
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
                                                            .expect("Unable to create Content-Type header for UPDATE_PDU_SESSION_TEMPORARY_REDIRECT"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                UpdatePduSessionResponse::PermanentRedirect
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
                                                            .expect("Unable to create Content-Type header for UPDATE_PDU_SESSION_PERMANENT_REDIRECT"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                UpdatePduSessionResponse::UnsuccessfulUpdateOfAPDUSession
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(400).expect("Unable to turn 400 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for UPDATE_PDU_SESSION_UNSUCCESSFUL_UPDATE_OF_APDU_SESSION"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                UpdatePduSessionResponse::UnsuccessfulUpdateOfAPDUSession_2
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(403).expect("Unable to turn 403 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for UPDATE_PDU_SESSION_UNSUCCESSFUL_UPDATE_OF_APDU_SESSION_2"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                UpdatePduSessionResponse::UnsuccessfulUpdateOfAPDUSession_3
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(404).expect("Unable to turn 404 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for UPDATE_PDU_SESSION_UNSUCCESSFUL_UPDATE_OF_APDU_SESSION_3"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                UpdatePduSessionResponse::LengthRequired
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(411).expect("Unable to turn 411 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for UPDATE_PDU_SESSION_LENGTH_REQUIRED"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                UpdatePduSessionResponse::PayloadTooLarge
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(413).expect("Unable to turn 413 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for UPDATE_PDU_SESSION_PAYLOAD_TOO_LARGE"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                UpdatePduSessionResponse::UnsupportedMediaType
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(415).expect("Unable to turn 415 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for UPDATE_PDU_SESSION_UNSUPPORTED_MEDIA_TYPE"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                UpdatePduSessionResponse::TooManyRequests
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(429).expect("Unable to turn 429 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for UPDATE_PDU_SESSION_TOO_MANY_REQUESTS"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                UpdatePduSessionResponse::UnsuccessfulUpdateOfAPDUSession_4
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(500).expect("Unable to turn 500 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for UPDATE_PDU_SESSION_UNSUCCESSFUL_UPDATE_OF_APDU_SESSION_4"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                UpdatePduSessionResponse::UnsuccessfulUpdateOfAPDUSession_5
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(503).expect("Unable to turn 503 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for UPDATE_PDU_SESSION_UNSUCCESSFUL_UPDATE_OF_APDU_SESSION_5"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                UpdatePduSessionResponse::GenericError
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
                                                .body(Body::from(format!("Couldn't read body parameter HsmfUpdateData: {}", e)))
                                                .expect("Unable to create Bad Request response due to unable to read body parameter HsmfUpdateData")),
                        }
                }

                // ReleaseSmContext - POST /sm-contexts/{smContextRef}/release
                hyper::Method::POST if path.matched(paths::ID_SM_CONTEXTS_SMCONTEXTREF_RELEASE) => {
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
                                "nsmf-pdusession".to_string(), // Access to the nsmf-pdusession API
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
                    paths::REGEX_SM_CONTEXTS_SMCONTEXTREF_RELEASE
                    .captures(path)
                    .unwrap_or_else(||
                        panic!("Path {} matched RE SM_CONTEXTS_SMCONTEXTREF_RELEASE in set but failed match against \"{}\"", path, paths::REGEX_SM_CONTEXTS_SMCONTEXTREF_RELEASE.as_str())
                    );

                    let param_sm_context_ref = match percent_encoding::percent_decode(path_params["smContextRef"].as_bytes()).decode_utf8() {
                    Ok(param_sm_context_ref) => match param_sm_context_ref.parse::<String>() {
                        Ok(param_sm_context_ref) => param_sm_context_ref,
                        Err(e) => return Ok(Response::builder()
                                        .status(StatusCode::BAD_REQUEST)
                                        .body(Body::from(format!("Couldn't parse path parameter smContextRef: {}", e)))
                                        .expect("Unable to create Bad Request response for invalid path parameter")),
                    },
                    Err(_) => return Ok(Response::builder()
                                        .status(StatusCode::BAD_REQUEST)
                                        .body(Body::from(format!("Couldn't percent-decode path parameter as UTF-8: {}", &path_params["smContextRef"])))
                                        .expect("Unable to create Bad Request response for invalid percent decode"))
                };

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
                                let param_sm_context_release_data: Option<
                                        models::SmContextReleaseData,
                                    > = if !body.is_empty() {
                                        let deserializer =
                                            &mut serde_json::Deserializer::from_slice(&*body);
                                        match serde_ignored::deserialize(deserializer, |path| {
                                            warn!("Ignoring unknown field in body: {}", path);
                                            unused_elements.push(path.to_string());
                                        }) {
                                            Ok(param_sm_context_release_data) => {
                                                param_sm_context_release_data
                                            }
                                            Err(_) => None,
                                        }
                                    } else {
                                        None
                                    };
                                // Get multipart chunks.

                                // Extract the top-level content type header.
                                let content_type_mime = headers
                                    .get(CONTENT_TYPE)
                                    .ok_or_else(|| "Missing content-type header".to_string())
                                    .and_then(|v| v.to_str().map_err(|e| format!("Couldn't read content-type header value for ReleaseSmContext: {}", e)))
                                    .and_then(|v| v.parse::<Mime2>().map_err(|_e| "Couldn't parse content-type header value for ReleaseSmContext".to_string()));

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
                                                .expect("Unable to create Bad Request response due to unable to read content-type header for ReleaseSmContext"));
                                    }
                                }

                                // &*body expresses the body as a byteslice, &mut provides a
                                // mutable reference to that byteslice.
                                let nodes = match read_multipart_body(&mut&*body, &multi_part_headers, false) {
                                    Ok(nodes) => nodes,
                                    Err(e) => {
                                        return Ok(Response::builder()
                                                .status(StatusCode::BAD_REQUEST)
                                                .body(Body::from(format!("Could not read multipart body for ReleaseSmContext: {}", e)))
                                                .expect("Unable to create Bad Request response due to unable to read multipart body for ReleaseSmContext"));
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

                                let result = api_impl.release_sm_context(
                                            param_sm_context_ref,
                                            param_sm_context_release_data,
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
                                                ReleaseSmContextResponse::SuccessfulReleaseOfAPDUSessionWithContentInTheResponse
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(200).expect("Unable to turn 200 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for RELEASE_SM_CONTEXT_SUCCESSFUL_RELEASE_OF_APDU_SESSION_WITH_CONTENT_IN_THE_RESPONSE"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                ReleaseSmContextResponse::SuccessfulReleaseOfAnSMContextWithoutContentInTheResponse
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(204).expect("Unable to turn 204 into a StatusCode");
                                                },
                                                ReleaseSmContextResponse::TemporaryRedirect
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
                                                            .expect("Unable to create Content-Type header for RELEASE_SM_CONTEXT_TEMPORARY_REDIRECT"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                ReleaseSmContextResponse::PermanentRedirect
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
                                                            .expect("Unable to create Content-Type header for RELEASE_SM_CONTEXT_PERMANENT_REDIRECT"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                ReleaseSmContextResponse::BadRequest
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(400).expect("Unable to turn 400 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for RELEASE_SM_CONTEXT_BAD_REQUEST"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                ReleaseSmContextResponse::Forbidden
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(403).expect("Unable to turn 403 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for RELEASE_SM_CONTEXT_FORBIDDEN"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                ReleaseSmContextResponse::NotFound
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(404).expect("Unable to turn 404 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for RELEASE_SM_CONTEXT_NOT_FOUND"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                ReleaseSmContextResponse::LengthRequired
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(411).expect("Unable to turn 411 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for RELEASE_SM_CONTEXT_LENGTH_REQUIRED"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                ReleaseSmContextResponse::PayloadTooLarge
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(413).expect("Unable to turn 413 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for RELEASE_SM_CONTEXT_PAYLOAD_TOO_LARGE"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                ReleaseSmContextResponse::UnsupportedMediaType
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(415).expect("Unable to turn 415 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for RELEASE_SM_CONTEXT_UNSUPPORTED_MEDIA_TYPE"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                ReleaseSmContextResponse::TooManyRequests
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(429).expect("Unable to turn 429 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for RELEASE_SM_CONTEXT_TOO_MANY_REQUESTS"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                ReleaseSmContextResponse::InternalServerError
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(500).expect("Unable to turn 500 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for RELEASE_SM_CONTEXT_INTERNAL_SERVER_ERROR"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                ReleaseSmContextResponse::ServiceUnavailable
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(503).expect("Unable to turn 503 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for RELEASE_SM_CONTEXT_SERVICE_UNAVAILABLE"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                ReleaseSmContextResponse::GenericError
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
                                                .body(Body::from(format!("Couldn't read body parameter SmContextReleaseData: {}", e)))
                                                .expect("Unable to create Bad Request response due to unable to read body parameter SmContextReleaseData")),
                        }
                }

                // RetrieveSmContext - POST /sm-contexts/{smContextRef}/retrieve
                hyper::Method::POST
                    if path.matched(paths::ID_SM_CONTEXTS_SMCONTEXTREF_RETRIEVE) =>
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
                                "nsmf-pdusession".to_string(), // Access to the nsmf-pdusession API
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
                    paths::REGEX_SM_CONTEXTS_SMCONTEXTREF_RETRIEVE
                    .captures(path)
                    .unwrap_or_else(||
                        panic!("Path {} matched RE SM_CONTEXTS_SMCONTEXTREF_RETRIEVE in set but failed match against \"{}\"", path, paths::REGEX_SM_CONTEXTS_SMCONTEXTREF_RETRIEVE.as_str())
                    );

                    let param_sm_context_ref = match percent_encoding::percent_decode(path_params["smContextRef"].as_bytes()).decode_utf8() {
                    Ok(param_sm_context_ref) => match param_sm_context_ref.parse::<String>() {
                        Ok(param_sm_context_ref) => param_sm_context_ref,
                        Err(e) => return Ok(Response::builder()
                                        .status(StatusCode::BAD_REQUEST)
                                        .body(Body::from(format!("Couldn't parse path parameter smContextRef: {}", e)))
                                        .expect("Unable to create Bad Request response for invalid path parameter")),
                    },
                    Err(_) => return Ok(Response::builder()
                                        .status(StatusCode::BAD_REQUEST)
                                        .body(Body::from(format!("Couldn't percent-decode path parameter as UTF-8: {}", &path_params["smContextRef"])))
                                        .expect("Unable to create Bad Request response for invalid percent decode"))
                };

                    // Body parameters (note that non-required body parameters will ignore garbage
                    // values, rather than causing a 400 response). Produce warning header and logs for
                    // any unused fields.
                    let result = body.into_raw().await;
                    match result {
                            Ok(body) => {
                                let mut unused_elements = Vec::new();
                                let param_sm_context_retrieve_data: Option<models::SmContextRetrieveData> = if !body.is_empty() {
                                    let deserializer = &mut serde_json::Deserializer::from_slice(&*body);
                                    match serde_ignored::deserialize(deserializer, |path| {
                                            warn!("Ignoring unknown field in body: {}", path);
                                            unused_elements.push(path.to_string());
                                    }) {
                                        Ok(param_sm_context_retrieve_data) => param_sm_context_retrieve_data,
                                        Err(_) => None,
                                    }
                                } else {
                                    None
                                };

                                let result = api_impl.retrieve_sm_context(
                                            param_sm_context_ref,
                                            param_sm_context_retrieve_data,
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
                                                RetrieveSmContextResponse::SuccessfulRetrievalOfAnSMContext
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(200).expect("Unable to turn 200 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for RETRIEVE_SM_CONTEXT_SUCCESSFUL_RETRIEVAL_OF_AN_SM_CONTEXT"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                RetrieveSmContextResponse::TemporaryRedirect
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
                                                            .expect("Unable to create Content-Type header for RETRIEVE_SM_CONTEXT_TEMPORARY_REDIRECT"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                RetrieveSmContextResponse::PermanentRedirect
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
                                                            .expect("Unable to create Content-Type header for RETRIEVE_SM_CONTEXT_PERMANENT_REDIRECT"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                RetrieveSmContextResponse::BadRequest
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(400).expect("Unable to turn 400 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for RETRIEVE_SM_CONTEXT_BAD_REQUEST"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                RetrieveSmContextResponse::Forbidden
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(403).expect("Unable to turn 403 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for RETRIEVE_SM_CONTEXT_FORBIDDEN"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                RetrieveSmContextResponse::NotFound
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(404).expect("Unable to turn 404 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for RETRIEVE_SM_CONTEXT_NOT_FOUND"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                RetrieveSmContextResponse::LengthRequired
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(411).expect("Unable to turn 411 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for RETRIEVE_SM_CONTEXT_LENGTH_REQUIRED"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                RetrieveSmContextResponse::PayloadTooLarge
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(413).expect("Unable to turn 413 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for RETRIEVE_SM_CONTEXT_PAYLOAD_TOO_LARGE"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                RetrieveSmContextResponse::UnsupportedMediaType
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(415).expect("Unable to turn 415 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for RETRIEVE_SM_CONTEXT_UNSUPPORTED_MEDIA_TYPE"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                RetrieveSmContextResponse::TooManyRequests
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(429).expect("Unable to turn 429 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for RETRIEVE_SM_CONTEXT_TOO_MANY_REQUESTS"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                RetrieveSmContextResponse::InternalServerError
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(500).expect("Unable to turn 500 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for RETRIEVE_SM_CONTEXT_INTERNAL_SERVER_ERROR"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                RetrieveSmContextResponse::ServiceUnavailable
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(503).expect("Unable to turn 503 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for RETRIEVE_SM_CONTEXT_SERVICE_UNAVAILABLE"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                RetrieveSmContextResponse::GatewayTimeout
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(504).expect("Unable to turn 504 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for RETRIEVE_SM_CONTEXT_GATEWAY_TIMEOUT"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                RetrieveSmContextResponse::GenericError
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
                                                .body(Body::from(format!("Couldn't read body parameter SmContextRetrieveData: {}", e)))
                                                .expect("Unable to create Bad Request response due to unable to read body parameter SmContextRetrieveData")),
                        }
                }

                // SendMoData - POST /sm-contexts/{smContextRef}/send-mo-data
                hyper::Method::POST
                    if path.matched(paths::ID_SM_CONTEXTS_SMCONTEXTREF_SEND_MO_DATA) =>
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
                                "nsmf-pdusession".to_string(), // Access to the nsmf-pdusession API
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
                    paths::REGEX_SM_CONTEXTS_SMCONTEXTREF_SEND_MO_DATA
                    .captures(path)
                    .unwrap_or_else(||
                        panic!("Path {} matched RE SM_CONTEXTS_SMCONTEXTREF_SEND_MO_DATA in set but failed match against \"{}\"", path, paths::REGEX_SM_CONTEXTS_SMCONTEXTREF_SEND_MO_DATA.as_str())
                    );

                    let param_sm_context_ref = match percent_encoding::percent_decode(path_params["smContextRef"].as_bytes()).decode_utf8() {
                    Ok(param_sm_context_ref) => match param_sm_context_ref.parse::<String>() {
                        Ok(param_sm_context_ref) => param_sm_context_ref,
                        Err(e) => return Ok(Response::builder()
                                        .status(StatusCode::BAD_REQUEST)
                                        .body(Body::from(format!("Couldn't parse path parameter smContextRef: {}", e)))
                                        .expect("Unable to create Bad Request response for invalid path parameter")),
                    },
                    Err(_) => return Ok(Response::builder()
                                        .status(StatusCode::BAD_REQUEST)
                                        .body(Body::from(format!("Couldn't percent-decode path parameter as UTF-8: {}", &path_params["smContextRef"])))
                                        .expect("Unable to create Bad Request response for invalid percent decode"))
                };

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
                                    .and_then(|v| v.to_str().map_err(|e| format!("Couldn't read content-type header value for SendMoData: {}", e)))
                                    .and_then(|v| v.parse::<Mime2>().map_err(|_e| "Couldn't parse content-type header value for SendMoData".to_string()));

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
                                                .expect("Unable to create Bad Request response due to unable to read content-type header for SendMoData"));
                                    }
                                }

                                // &*body expresses the body as a byteslice, &mut provides a
                                // mutable reference to that byteslice.
                                let nodes = match read_multipart_body(&mut&*body, &multi_part_headers, false) {
                                    Ok(nodes) => nodes,
                                    Err(e) => {
                                        return Ok(Response::builder()
                                                .status(StatusCode::BAD_REQUEST)
                                                .body(Body::from(format!("Could not read multipart body for SendMoData: {}", e)))
                                                .expect("Unable to create Bad Request response due to unable to read multipart body for SendMoData"));
                                    }
                                };

                                let mut param_json_data = None;
                                let mut param_binary_mo_data = None;

                                for node in nodes {
                                    if let Node::Part(part) = node {
                                        let content_type = part.content_type().map(|x| format!("{}",x));
                                        match content_type.as_deref() {
                                            Some("application/json") if param_json_data.is_none() => {
                                                // Extract JSON part.
                                                let deserializer = &mut serde_json::Deserializer::from_slice(part.body.as_slice());
                                                let json_data: models::SendMoDataReqData = match serde_ignored::deserialize(deserializer, |path| {
                                                    warn!("Ignoring unknown field in JSON part: {}", path);
                                                    unused_elements.push(path.to_string());
                                                }) {
                                                    Ok(json_data) => json_data,
                                                    Err(e) => return Ok(Response::builder()
                                                                    .status(StatusCode::BAD_REQUEST)
                                                                    .body(Body::from(format!("Couldn't parse body parameter models::SendMoDataReqData - doesn't match schema: {}", e)))
                                                                    .expect("Unable to create Bad Request response for invalid body parameter models::SendMoDataReqData due to schema"))
                                                };
                                                // Push JSON part to return object.
                                                param_json_data.get_or_insert(json_data);
                                            },
                                            Some("application/vnd.3gpp.5gnas") if param_binary_mo_data.is_none() => {
                                                param_binary_mo_data.get_or_insert(swagger::ByteArray(part.body));
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

                                let result = api_impl.send_mo_data(
                                            param_sm_context_ref,
                                            param_json_data,
                                            param_binary_mo_data,
                                        &context
                                    ).await;
                                let mut response = Response::new(Body::empty());
                                response.headers_mut().insert(
                                            HeaderName::from_static("x-span-id"),
                                            HeaderValue::from_str((&context as &dyn Has<XSpanIdString>).get().0.clone().as_str())
                                                .expect("Unable to create X-Span-ID header value"));

                                        match result {
                                            Ok(rsp) => match rsp {
                                                SendMoDataResponse::SuccessfulSendingOfMOData
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(204).expect("Unable to turn 204 into a StatusCode");
                                                },
                                                SendMoDataResponse::TemporaryRedirect
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
                                                            .expect("Unable to create Content-Type header for SEND_MO_DATA_TEMPORARY_REDIRECT"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                SendMoDataResponse::PermanentRedirect
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
                                                            .expect("Unable to create Content-Type header for SEND_MO_DATA_PERMANENT_REDIRECT"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                SendMoDataResponse::BadRequest
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(400).expect("Unable to turn 400 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for SEND_MO_DATA_BAD_REQUEST"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                SendMoDataResponse::Unauthorized
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(401).expect("Unable to turn 401 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for SEND_MO_DATA_UNAUTHORIZED"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                SendMoDataResponse::Forbidden
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(403).expect("Unable to turn 403 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for SEND_MO_DATA_FORBIDDEN"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                SendMoDataResponse::NotFound
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(404).expect("Unable to turn 404 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for SEND_MO_DATA_NOT_FOUND"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                SendMoDataResponse::LengthRequired
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(411).expect("Unable to turn 411 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for SEND_MO_DATA_LENGTH_REQUIRED"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                SendMoDataResponse::PayloadTooLarge
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(413).expect("Unable to turn 413 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for SEND_MO_DATA_PAYLOAD_TOO_LARGE"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                SendMoDataResponse::UnsupportedMediaType
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(415).expect("Unable to turn 415 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for SEND_MO_DATA_UNSUPPORTED_MEDIA_TYPE"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                SendMoDataResponse::TooManyRequests
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(429).expect("Unable to turn 429 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for SEND_MO_DATA_TOO_MANY_REQUESTS"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                SendMoDataResponse::InternalServerError
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(500).expect("Unable to turn 500 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for SEND_MO_DATA_INTERNAL_SERVER_ERROR"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                SendMoDataResponse::ServiceUnavailable
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(503).expect("Unable to turn 503 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for SEND_MO_DATA_SERVICE_UNAVAILABLE"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                SendMoDataResponse::GenericError
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
                                                .body(Body::from(format!("Couldn't read body parameter IndividualSMContext: {}", e)))
                                                .expect("Unable to create Bad Request response due to unable to read body parameter IndividualSMContext")),
                        }
                }

                // UpdateSmContext - POST /sm-contexts/{smContextRef}/modify
                hyper::Method::POST if path.matched(paths::ID_SM_CONTEXTS_SMCONTEXTREF_MODIFY) => {
                    {
                        // Update SM context
                        // Request
                        // {
                        //     "n2SmInfo":	{
                        //         "contentId":	"ngap-sm"
                        //     },
                        //     "n2SmInfoType":	"PDU_RES_SETUP_RSP"
                        // }
                        // NGAP
                        // 0000   00 03 e0 ac 16 00 17 00 00 00 04 00 01

                        // Reply is 204, no data

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
                                "nsmf-pdusession".to_string(), // Access to the nsmf-pdusession API
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
                    paths::REGEX_SM_CONTEXTS_SMCONTEXTREF_MODIFY
                    .captures(path)
                    .unwrap_or_else(||
                        panic!("Path {} matched RE SM_CONTEXTS_SMCONTEXTREF_MODIFY in set but failed match against \"{}\"", path, paths::REGEX_SM_CONTEXTS_SMCONTEXTREF_MODIFY.as_str())
                    );

                    let param_sm_context_ref = match percent_encoding::percent_decode(path_params["smContextRef"].as_bytes()).decode_utf8() {
                    Ok(param_sm_context_ref) => match param_sm_context_ref.parse::<String>() {
                        Ok(param_sm_context_ref) => param_sm_context_ref,
                        Err(e) => return Ok(Response::builder()
                                        .status(StatusCode::BAD_REQUEST)
                                        .body(Body::from(format!("Couldn't parse path parameter smContextRef: {}", e)))
                                        .expect("Unable to create Bad Request response for invalid path parameter")),
                    },
                    Err(_) => return Ok(Response::builder()
                                        .status(StatusCode::BAD_REQUEST)
                                        .body(Body::from(format!("Couldn't percent-decode path parameter as UTF-8: {}", &path_params["smContextRef"])))
                                        .expect("Unable to create Bad Request response for invalid percent decode"))
                };

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
                                let param_sm_context_update_data: Option<models::SmContextUpdateData> =
                                if !body.is_empty() {
                                    let deserializer =
                                        &mut serde_json::Deserializer::from_slice(&*body);
                                    match serde_ignored::deserialize(deserializer, |path| {
                                            warn!("Ignoring unknown field in body: {}", path);
                                            unused_elements.push(path.to_string());
                                    }) {
                                        Ok(param_sm_context_update_data) => param_sm_context_update_data,
                                        Err(e) => return Ok(Response::builder()
                                                        .status(StatusCode::BAD_REQUEST)
                                                        .body(Body::from(format!("Couldn't parse body parameter SmContextUpdateData - doesn't match schema: {}", e)))
                                                        .expect("Unable to create Bad Request response for invalid body parameter SmContextUpdateData due to schema")),
                                    }
                                } else {
                                    None
                                };
                            let param_sm_context_update_data = match param_sm_context_update_data {
                                    Some(param_sm_context_update_data) => param_sm_context_update_data,
                                    None => return Ok(Response::builder()
                                                        .status(StatusCode::BAD_REQUEST)
                                                        .body(Body::from("Missing required body parameter SmContextUpdateData"))
                                                        .expect("Unable to create Bad Request response for missing body parameter SmContextUpdateData")),
                                };
                                // Get multipart chunks.

                                // Extract the top-level content type header.
                                let content_type_mime = headers
                                    .get(CONTENT_TYPE)
                                    .ok_or_else(|| "Missing content-type header".to_string())
                                    .and_then(|v| v.to_str().map_err(|e| format!("Couldn't read content-type header value for UpdateSmContext: {}", e)))
                                    .and_then(|v| v.parse::<Mime2>().map_err(|_e| "Couldn't parse content-type header value for UpdateSmContext".to_string()));

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
                                                .expect("Unable to create Bad Request response due to unable to read content-type header for UpdateSmContext"));
                                    }
                                }

                                // &*body expresses the body as a byteslice, &mut provides a
                                // mutable reference to that byteslice.
                                let nodes = match read_multipart_body(&mut&*body, &multi_part_headers, false) {
                                    Ok(nodes) => nodes,
                                    Err(e) => {
                                        return Ok(Response::builder()
                                                .status(StatusCode::BAD_REQUEST)
                                                .body(Body::from(format!("Could not read multipart body for UpdateSmContext: {}", e)))
                                                .expect("Unable to create Bad Request response due to unable to read multipart body for UpdateSmContext"));
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

                                let result = api_impl.update_sm_context(
                                            param_sm_context_ref,
                                            param_sm_context_update_data,
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
                                                UpdateSmContextResponse::SuccessfulUpdateOfAnSMContextWithContentInTheResponse
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(200).expect("Unable to turn 200 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for UPDATE_SM_CONTEXT_SUCCESSFUL_UPDATE_OF_AN_SM_CONTEXT_WITH_CONTENT_IN_THE_RESPONSE"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                UpdateSmContextResponse::SuccessfulUpdateOfAnSMContextWithoutContentInTheResponse
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(204).expect("Unable to turn 204 into a StatusCode");
                                                },
                                                UpdateSmContextResponse::TemporaryRedirect
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
                                                            .expect("Unable to create Content-Type header for UPDATE_SM_CONTEXT_TEMPORARY_REDIRECT"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                UpdateSmContextResponse::PermanentRedirect
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
                                                            .expect("Unable to create Content-Type header for UPDATE_SM_CONTEXT_PERMANENT_REDIRECT"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                UpdateSmContextResponse::UnsuccessfulUpdateOfAnSMContext
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(400).expect("Unable to turn 400 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for UPDATE_SM_CONTEXT_UNSUCCESSFUL_UPDATE_OF_AN_SM_CONTEXT"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                UpdateSmContextResponse::UnsuccessfulUpdateOfAnSMContext_2
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(403).expect("Unable to turn 403 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for UPDATE_SM_CONTEXT_UNSUCCESSFUL_UPDATE_OF_AN_SM_CONTEXT_2"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                UpdateSmContextResponse::UnsuccessfulUpdateOfAnSMContext_3
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(404).expect("Unable to turn 404 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for UPDATE_SM_CONTEXT_UNSUCCESSFUL_UPDATE_OF_AN_SM_CONTEXT_3"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                UpdateSmContextResponse::LengthRequired
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(411).expect("Unable to turn 411 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for UPDATE_SM_CONTEXT_LENGTH_REQUIRED"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                UpdateSmContextResponse::PayloadTooLarge
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(413).expect("Unable to turn 413 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for UPDATE_SM_CONTEXT_PAYLOAD_TOO_LARGE"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                UpdateSmContextResponse::UnsupportedMediaType
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(415).expect("Unable to turn 415 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for UPDATE_SM_CONTEXT_UNSUPPORTED_MEDIA_TYPE"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                UpdateSmContextResponse::TooManyRequests
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(429).expect("Unable to turn 429 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for UPDATE_SM_CONTEXT_TOO_MANY_REQUESTS"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                UpdateSmContextResponse::UnsuccessfulUpdateOfAnSMContext_4
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(500).expect("Unable to turn 500 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for UPDATE_SM_CONTEXT_UNSUCCESSFUL_UPDATE_OF_AN_SM_CONTEXT_4"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                UpdateSmContextResponse::UnsuccessfulUpdateOfAnSMContext_5
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(503).expect("Unable to turn 503 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for UPDATE_SM_CONTEXT_UNSUCCESSFUL_UPDATE_OF_AN_SM_CONTEXT_5"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                UpdateSmContextResponse::GenericError
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
                                                .body(Body::from(format!("Couldn't read body parameter SmContextUpdateData: {}", e)))
                                                .expect("Unable to create Bad Request response due to unable to read body parameter SmContextUpdateData")),
                        }
                }

                // PostPduSessions - POST /pdu-sessions
                hyper::Method::POST if path.matched(paths::ID_PDU_SESSIONS) => {
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
                                "nsmf-pdusession".to_string(), // Access to the nsmf-pdusession API
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
                                let param_pdu_session_create_data: Option<
                                models::PduSessionCreateData,
                            > = if !body.is_empty() {
                                let deserializer =
                                    &mut serde_json::Deserializer::from_slice(&*body);
                                match serde_ignored::deserialize(deserializer, |path| {
                                    warn!("Ignoring unknown field in body: {}", path);
                                    unused_elements.push(path.to_string());
                            }) {
                                Ok(param_pdu_session_create_data) => param_pdu_session_create_data,
                                Err(e) => return Ok(Response::builder()
                                                .status(StatusCode::BAD_REQUEST)
                                                .body(Body::from(format!("Couldn't parse body parameter PduSessionCreateData - doesn't match schema: {}", e)))
                                                .expect("Unable to create Bad Request response for invalid body parameter PduSessionCreateData due to schema")),
                            }
                            } else {
                                None
                            };
                            let param_pdu_session_create_data = match param_pdu_session_create_data {
                            Some(param_pdu_session_create_data) => param_pdu_session_create_data,
                            None => return Ok(Response::builder()
                                                .status(StatusCode::BAD_REQUEST)
                                                .body(Body::from("Missing required body parameter PduSessionCreateData"))
                                                .expect("Unable to create Bad Request response for missing body parameter PduSessionCreateData")),
                        };
                                // Get multipart chunks.

                                // Extract the top-level content type header.
                                let content_type_mime = headers
                                    .get(CONTENT_TYPE)
                                    .ok_or_else(|| "Missing content-type header".to_string())
                                    .and_then(|v| v.to_str().map_err(|e| format!("Couldn't read content-type header value for PostPduSessions: {}", e)))
                                    .and_then(|v| v.parse::<Mime2>().map_err(|_e| "Couldn't parse content-type header value for PostPduSessions".to_string()));

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
                                                .expect("Unable to create Bad Request response due to unable to read content-type header for PostPduSessions"));
                                    }
                                }

                                // &*body expresses the body as a byteslice, &mut provides a
                                // mutable reference to that byteslice.
                                let nodes = match read_multipart_body(&mut&*body, &multi_part_headers, false) {
                                    Ok(nodes) => nodes,
                                    Err(e) => {
                                        return Ok(Response::builder()
                                                .status(StatusCode::BAD_REQUEST)
                                                .body(Body::from(format!("Could not read multipart body for PostPduSessions: {}", e)))
                                                .expect("Unable to create Bad Request response due to unable to read multipart body for PostPduSessions"));
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

                                let result = api_impl.post_pdu_sessions(
                                            param_pdu_session_create_data,
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
                                                PostPduSessionsResponse::SuccessfulCreationOfAPDUSession
                                                    {
                                                        body,
                                                        location
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
                                                    *response.status_mut() = StatusCode::from_u16(201).expect("Unable to turn 201 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for POST_PDU_SESSIONS_SUCCESSFUL_CREATION_OF_APDU_SESSION"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                PostPduSessionsResponse::TemporaryRedirect
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
                                                            .expect("Unable to create Content-Type header for POST_PDU_SESSIONS_TEMPORARY_REDIRECT"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                PostPduSessionsResponse::PermanentRedirect
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
                                                            .expect("Unable to create Content-Type header for POST_PDU_SESSIONS_PERMANENT_REDIRECT"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                PostPduSessionsResponse::UnsuccessfulCreationOfAPDUSession
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(400).expect("Unable to turn 400 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for POST_PDU_SESSIONS_UNSUCCESSFUL_CREATION_OF_APDU_SESSION"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                PostPduSessionsResponse::UnsuccessfulCreationOfAPDUSession_2
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(403).expect("Unable to turn 403 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for POST_PDU_SESSIONS_UNSUCCESSFUL_CREATION_OF_APDU_SESSION_2"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                PostPduSessionsResponse::UnsuccessfulCreationOfAPDUSession_3
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(404).expect("Unable to turn 404 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for POST_PDU_SESSIONS_UNSUCCESSFUL_CREATION_OF_APDU_SESSION_3"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                PostPduSessionsResponse::LengthRequired
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(411).expect("Unable to turn 411 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for POST_PDU_SESSIONS_LENGTH_REQUIRED"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                PostPduSessionsResponse::PayloadTooLarge
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(413).expect("Unable to turn 413 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for POST_PDU_SESSIONS_PAYLOAD_TOO_LARGE"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                PostPduSessionsResponse::UnsupportedMediaType
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(415).expect("Unable to turn 415 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for POST_PDU_SESSIONS_UNSUPPORTED_MEDIA_TYPE"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                PostPduSessionsResponse::TooManyRequests
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(429).expect("Unable to turn 429 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for POST_PDU_SESSIONS_TOO_MANY_REQUESTS"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                PostPduSessionsResponse::UnsuccessfulCreationOfAPDUSession_4
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(500).expect("Unable to turn 500 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for POST_PDU_SESSIONS_UNSUCCESSFUL_CREATION_OF_APDU_SESSION_4"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                PostPduSessionsResponse::UnsuccessfulCreationOfAPDUSession_5
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(503).expect("Unable to turn 503 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for POST_PDU_SESSIONS_UNSUCCESSFUL_CREATION_OF_APDU_SESSION_5"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                PostPduSessionsResponse::GenericError
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
                                                .body(Body::from(format!("Couldn't read body parameter PduSessionCreateData: {}", e)))
                                                .expect("Unable to create Bad Request response due to unable to read body parameter PduSessionCreateData")),
                        }
                }

                // PostSmContexts - POST /sm-contexts
                hyper::Method::POST if path.matched(paths::ID_SM_CONTEXTS) => {
                    // Request
                    // {
                    //     "supi":	"imsi-001011234567895",
                    //     "pei":	"imeisv-4370816125816151",
                    //     "pduSessionId":	1,
                    //     "dnn":	"internet",
                    //     "sNssai":	{
                    //         "sst":	1
                    //     },
                    //     "servingNfId":	"66bf4df8-b832-41ed-aa12-4df3ea315a7c",
                    //     "guami":	{
                    //         "plmnId":	{
                    //             "mcc":	"001",
                    //             "mnc":	"01"
                    //         },
                    //         "amfId":	"020040"
                    //     },
                    //     "servingNetwork":	{
                    //         "mcc":	"001",
                    //         "mnc":	"01"
                    //     },
                    //     "n1SmMsg":	{
                    //         "contentId":	"5gnas-sm"
                    //     },
                    //     "anType":	"3GPP_ACCESS",
                    //     "ratType":	"NR",
                    //     "ueLocation":	{
                    //         "nrLocation":	{
                    //             "tai":	{
                    //                 "plmnId":	{
                    //                     "mcc":	"001",
                    //                     "mnc":	"01"
                    //                 },
                    //                 "tac":	"000001"
                    //             },
                    //             "ncgi":	{
                    //                 "plmnId":	{
                    //                     "mcc":	"001",
                    //                     "mnc":	"01"
                    //                 },
                    //                 "nrCellId":	"000000010"
                    //             },
                    //             "ueLocationTimestamp":	"2023-03-01T13:42:11.144288Z"
                    //         }
                    //     },
                    //     "ueTimeZone":	"+00:00",
                    //     "smContextStatusUri":	"http://172.22.0.10:7777/namf-callback/v1/imsi-001011234567895/sm-context-status/1",
                    //     "pcfId":	"6c05c1d4-b832-41ed-9698-8dec5d3774de"
                    // }
                    // 5GNAS
                    //
                    // 0000   2e 01 01 c1 ff ff 91 a1 28 01 00 7b 00 07 80 00
                    // 0010   0a 00 00 0d 00

                    // Reply is header with new location of the created SM session
                    // http://172.22.0.7:7777/nsmf-pdusession/v1/sm-contexts/4

                    // Async sent to AFM via an API call to POST /namf-comm/v1/ue-contexts/imsi-001011234567895/n1-n2-messages
                    // Creation of SM context
                    // {
                    //     "n1MessageContainer":	{
                    //         "n1MessageClass":	"SM",
                    //         "n1MessageContent":	{
                    //             "contentId":	"5gnas-sm"
                    //         }
                    //     },
                    //     "n2InfoContainer":	{
                    //         "n2InformationClass":	"SM",
                    //         "smInfo":	{
                    //             "pduSessionId":	1,
                    //             "n2InfoContent":	{
                    //                 "ngapIeType":	"PDU_RES_SETUP_REQ",
                    //                 "ngapData":	{
                    //                     "contentId":	"ngap-sm"
                    //                 }
                    //             }
                    //         }
                    //     },
                    //     "pduSessionId":	1
                    // }

                    // as well as 5GNAS payload and NGAP payloads
                    // NAS
                    // 0000   2e 01 01 c2 11 00 09 01 00 06 31 31 01 01 ff 01
                    // 0010   06 0b 00 01 0b 00 01 29 05 01 c0 a8 64 05 22 01
                    // 0020   01 79 00 06 01 20 41 01 01 09 7b 00 0f 80 00 0d
                    // 0030   04 08 08 08 08 00 0d 04 08 08 04 04 25 09 08 69
                    // 0040   6e 74 65 72 6e 65 74
                    //
                    // NGAP
                    // 0000   00 00 04 00 82 00 0a 0c 40 00 00 00 30 40 00 00
                    // 0010   00 00 8b 00 0a 01 f0 ac 16 00 08 00 00 00 0e 00
                    // 0020   86 00 01 00 00 88 00 07 00 01 00 00 09 1c 00

                    // with reply
                    // {
                    //	"cause":	"N1_N2_TRANSFER_INITIATED"
                    // }

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
                                "nsmf-pdusession".to_string(), // Access to the nsmf-pdusession API
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
                                    .and_then(|v| v.to_str().map_err(|e| format!("Couldn't read content-type header value for PostSmContexts: {}", e)))
                                    .and_then(|v| v.parse::<Mime2>().map_err(|_e| "Couldn't parse content-type header value for PostSmContexts".to_string()));

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
                                                .expect("Unable to create Bad Request response due to unable to read content-type header for PostSmContexts"));
                                    }
                                }

                                // &*body expresses the body as a byteslice, &mut provides a
                                // mutable reference to that byteslice.
                                let nodes = match read_multipart_body(&mut&*body, &multi_part_headers, false) {
                                    Ok(nodes) => nodes,
                                    Err(e) => {
                                        return Ok(Response::builder()
                                                .status(StatusCode::BAD_REQUEST)
                                                .body(Body::from(format!("Could not read multipart body for PostSmContexts: {}", e)))
                                                .expect("Unable to create Bad Request response due to unable to read multipart body for PostSmContexts"));
                                    }
                                };

                                let mut param_json_data = None;
                                let mut param_binary_data_n1_sm_message = None;
                                let mut param_binary_data_n2_sm_information = None;
                                let mut param_binary_data_n2_sm_information_ext1 = None;

                                for node in nodes {
                                    if let Node::Part(part) = node {
                                        let content_type = part.content_type().map(|x| format!("{}",x));
                                        match content_type.as_deref() {
                                            Some("application/json") if param_json_data.is_none() => {
                                                // Extract JSON part.
                                                let deserializer = &mut serde_json::Deserializer::from_slice(part.body.as_slice());
                                                let json_data: models::SmContextCreateData = match serde_ignored::deserialize(deserializer, |path| {
                                                    warn!("Ignoring unknown field in JSON part: {}", path);
                                                    unused_elements.push(path.to_string());
                                                }) {
                                                    Ok(json_data) => json_data,
                                                    Err(e) => return Ok(Response::builder()
                                                                    .status(StatusCode::BAD_REQUEST)
                                                                    .body(Body::from(format!("Couldn't parse body parameter models::SmContextCreateData - doesn't match schema: {}", e)))
                                                                    .expect("Unable to create Bad Request response for invalid body parameter models::SmContextCreateData due to schema"))
                                                };
                                                // Push JSON part to return object.
                                                param_json_data.get_or_insert(json_data);
                                            },
                                            Some("application/vnd.3gpp.5gnas") if param_binary_data_n1_sm_message.is_none() => {
                                                param_binary_data_n1_sm_message.get_or_insert(swagger::ByteArray(part.body));
                                            },
                                            Some("application/vnd.3gpp.ngap") if param_binary_data_n2_sm_information.is_none() => {
                                                param_binary_data_n2_sm_information.get_or_insert(swagger::ByteArray(part.body));
                                            },
                                            Some("application/vnd.3gpp.ngap") if param_binary_data_n2_sm_information_ext1.is_none() => {
                                                param_binary_data_n2_sm_information_ext1.get_or_insert(swagger::ByteArray(part.body));
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

                                let result = api_impl.post_sm_contexts(
                                            param_json_data,
                                            param_binary_data_n1_sm_message,
                                            param_binary_data_n2_sm_information,
                                            param_binary_data_n2_sm_information_ext1,
                                        &context
                                    ).await;
                                let mut response = Response::new(Body::empty());
                                response.headers_mut().insert(
                                            HeaderName::from_static("x-span-id"),
                                            HeaderValue::from_str((&context as &dyn Has<XSpanIdString>).get().0.clone().as_str())
                                                .expect("Unable to create X-Span-ID header value"));

                                        match result {
                                            Ok(rsp) => match rsp {
                                                PostSmContextsResponse::SuccessfulCreationOfAnSMContext
                                                    {
                                                        body,
                                                        location
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
                                                    *response.status_mut() = StatusCode::from_u16(201).expect("Unable to turn 201 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for POST_SM_CONTEXTS_SUCCESSFUL_CREATION_OF_AN_SM_CONTEXT"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                PostSmContextsResponse::TemporaryRedirect
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
                                                            .expect("Unable to create Content-Type header for POST_SM_CONTEXTS_TEMPORARY_REDIRECT"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                PostSmContextsResponse::PermanentRedirect
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
                                                            .expect("Unable to create Content-Type header for POST_SM_CONTEXTS_PERMANENT_REDIRECT"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                PostSmContextsResponse::UnsuccessfulCreationOfAnSMContext
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(400).expect("Unable to turn 400 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for POST_SM_CONTEXTS_UNSUCCESSFUL_CREATION_OF_AN_SM_CONTEXT"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                PostSmContextsResponse::UnsuccessfulCreationOfAnSMContext_2
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(403).expect("Unable to turn 403 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for POST_SM_CONTEXTS_UNSUCCESSFUL_CREATION_OF_AN_SM_CONTEXT_2"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                PostSmContextsResponse::UnsuccessfulCreationOfAnSMContext_3
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(404).expect("Unable to turn 404 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for POST_SM_CONTEXTS_UNSUCCESSFUL_CREATION_OF_AN_SM_CONTEXT_3"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                PostSmContextsResponse::LengthRequired
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(411).expect("Unable to turn 411 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for POST_SM_CONTEXTS_LENGTH_REQUIRED"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                PostSmContextsResponse::PayloadTooLarge
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(413).expect("Unable to turn 413 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for POST_SM_CONTEXTS_PAYLOAD_TOO_LARGE"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                PostSmContextsResponse::UnsupportedMediaType
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(415).expect("Unable to turn 415 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for POST_SM_CONTEXTS_UNSUPPORTED_MEDIA_TYPE"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                PostSmContextsResponse::TooManyRequests
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(429).expect("Unable to turn 429 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for POST_SM_CONTEXTS_TOO_MANY_REQUESTS"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                PostSmContextsResponse::UnsuccessfulCreationOfAnSMContext_4
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(500).expect("Unable to turn 500 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for POST_SM_CONTEXTS_UNSUCCESSFUL_CREATION_OF_AN_SM_CONTEXT_4"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                PostSmContextsResponse::UnsuccessfulCreationOfAnSMContext_5
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(503).expect("Unable to turn 503 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for POST_SM_CONTEXTS_UNSUCCESSFUL_CREATION_OF_AN_SM_CONTEXT_5"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                PostSmContextsResponse::UnsuccessfulCreationOfAnSMContext_6
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(504).expect("Unable to turn 504 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for POST_SM_CONTEXTS_UNSUCCESSFUL_CREATION_OF_AN_SM_CONTEXT_6"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                PostSmContextsResponse::GenericError
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
                                                .body(Body::from(format!("Couldn't read body parameter SMContextsCollection: {}", e)))
                                                .expect("Unable to create Bad Request response due to unable to read body parameter SMContextsCollection")),
                        }
                }

                _ if path.matched(paths::ID_PDU_SESSIONS) => method_not_allowed(),
                _ if path.matched(paths::ID_PDU_SESSIONS_PDUSESSIONREF_MODIFY) => {
                    method_not_allowed()
                }
                _ if path.matched(paths::ID_PDU_SESSIONS_PDUSESSIONREF_RELEASE) => {
                    method_not_allowed()
                }
                _ if path.matched(paths::ID_PDU_SESSIONS_PDUSESSIONREF_RETRIEVE) => {
                    method_not_allowed()
                }
                _ if path.matched(paths::ID_PDU_SESSIONS_PDUSESSIONREF_TRANSFER_MO_DATA) => {
                    method_not_allowed()
                }
                _ if path.matched(paths::ID_SM_CONTEXTS) => method_not_allowed(),
                _ if path.matched(paths::ID_SM_CONTEXTS_SMCONTEXTREF_MODIFY) => {
                    method_not_allowed()
                }
                _ if path.matched(paths::ID_SM_CONTEXTS_SMCONTEXTREF_RELEASE) => {
                    method_not_allowed()
                }
                _ if path.matched(paths::ID_SM_CONTEXTS_SMCONTEXTREF_RETRIEVE) => {
                    method_not_allowed()
                }
                _ if path.matched(paths::ID_SM_CONTEXTS_SMCONTEXTREF_SEND_MO_DATA) => {
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
pub struct ApiRequestParser {}
impl<T> RequestParser<T> for ApiRequestParser {
    fn parse_operation_id(request: &Request<T>) -> Option<&'static str> {
        let path = paths::GLOBAL_REGEX_SET.matches(request.uri().path());
        match *request.method() {
            // ReleasePduSession - POST /pdu-sessions/{pduSessionRef}/release
            hyper::Method::POST if path.matched(paths::ID_PDU_SESSIONS_PDUSESSIONREF_RELEASE) => {
                Some("ReleasePduSession")
            }
            // RetrievePduSession - POST /pdu-sessions/{pduSessionRef}/retrieve
            hyper::Method::POST if path.matched(paths::ID_PDU_SESSIONS_PDUSESSIONREF_RETRIEVE) => {
                Some("RetrievePduSession")
            }
            // TransferMoData - POST /pdu-sessions/{pduSessionRef}/transfer-mo-data
            hyper::Method::POST
                if path.matched(paths::ID_PDU_SESSIONS_PDUSESSIONREF_TRANSFER_MO_DATA) =>
            {
                Some("TransferMoData")
            }
            // UpdatePduSession - POST /pdu-sessions/{pduSessionRef}/modify
            hyper::Method::POST if path.matched(paths::ID_PDU_SESSIONS_PDUSESSIONREF_MODIFY) => {
                Some("UpdatePduSession")
            }
            // ReleaseSmContext - POST /sm-contexts/{smContextRef}/release
            hyper::Method::POST if path.matched(paths::ID_SM_CONTEXTS_SMCONTEXTREF_RELEASE) => {
                Some("ReleaseSmContext")
            }
            // RetrieveSmContext - POST /sm-contexts/{smContextRef}/retrieve
            hyper::Method::POST if path.matched(paths::ID_SM_CONTEXTS_SMCONTEXTREF_RETRIEVE) => {
                Some("RetrieveSmContext")
            }
            // SendMoData - POST /sm-contexts/{smContextRef}/send-mo-data
            hyper::Method::POST
                if path.matched(paths::ID_SM_CONTEXTS_SMCONTEXTREF_SEND_MO_DATA) =>
            {
                Some("SendMoData")
            }
            // UpdateSmContext - POST /sm-contexts/{smContextRef}/modify
            hyper::Method::POST if path.matched(paths::ID_SM_CONTEXTS_SMCONTEXTREF_MODIFY) => {
                Some("UpdateSmContext")
            }
            // PostPduSessions - POST /pdu-sessions
            hyper::Method::POST if path.matched(paths::ID_PDU_SESSIONS) => Some("PostPduSessions"),
            // PostSmContexts - POST /sm-contexts
            hyper::Method::POST if path.matched(paths::ID_SM_CONTEXTS) => Some("PostSmContexts"),
            _ => None,
        }
    }
}
