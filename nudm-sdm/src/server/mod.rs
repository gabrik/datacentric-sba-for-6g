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

mod paths {
    use lazy_static::lazy_static;

    lazy_static! {
        pub static ref GLOBAL_REGEX_SET: regex::RegexSet = regex::RegexSet::new(vec![
            r"^/nudm-sdm/v2/group-data/group-identifiers$",
            r"^/nudm-sdm/v2/multiple-identifiers$",
            r"^/nudm-sdm/v2/shared-data$",
            r"^/nudm-sdm/v2/shared-data-subscriptions$",
            r"^/nudm-sdm/v2/shared-data-subscriptions/(?P<subscriptionId>[^/?#]*)$",
            r"^/nudm-sdm/v2/shared-data/(?P<sharedDataId>[^/?#]*)$",
            r"^/nudm-sdm/v2/(?P<supi>[^/?#]*)$",
            r"^/nudm-sdm/v2/(?P<supi>[^/?#]*)/5mbs-data$",
            r"^/nudm-sdm/v2/(?P<supi>[^/?#]*)/am-data$",
            r"^/nudm-sdm/v2/(?P<supi>[^/?#]*)/am-data/cag-ack$",
            r"^/nudm-sdm/v2/(?P<supi>[^/?#]*)/am-data/ecr-data$",
            r"^/nudm-sdm/v2/(?P<supi>[^/?#]*)/am-data/sor-ack$",
            r"^/nudm-sdm/v2/(?P<supi>[^/?#]*)/am-data/subscribed-snssais-ack$",
            r"^/nudm-sdm/v2/(?P<supi>[^/?#]*)/am-data/update-sor$",
            r"^/nudm-sdm/v2/(?P<supi>[^/?#]*)/am-data/upu-ack$",
            r"^/nudm-sdm/v2/(?P<supi>[^/?#]*)/lcs-bca-data$",
            r"^/nudm-sdm/v2/(?P<supi>[^/?#]*)/lcs-mo-data$",
            r"^/nudm-sdm/v2/(?P<supi>[^/?#]*)/nssai$",
            r"^/nudm-sdm/v2/(?P<supi>[^/?#]*)/prose-data$",
            r"^/nudm-sdm/v2/(?P<supi>[^/?#]*)/sm-data$",
            r"^/nudm-sdm/v2/(?P<supi>[^/?#]*)/smf-select-data$",
            r"^/nudm-sdm/v2/(?P<supi>[^/?#]*)/sms-data$",
            r"^/nudm-sdm/v2/(?P<supi>[^/?#]*)/sms-mng-data$",
            r"^/nudm-sdm/v2/(?P<supi>[^/?#]*)/trace-data$",
            r"^/nudm-sdm/v2/(?P<supi>[^/?#]*)/uc-data$",
            r"^/nudm-sdm/v2/(?P<supi>[^/?#]*)/ue-context-in-amf-data$",
            r"^/nudm-sdm/v2/(?P<supi>[^/?#]*)/ue-context-in-smf-data$",
            r"^/nudm-sdm/v2/(?P<supi>[^/?#]*)/ue-context-in-smsf-data$",
            r"^/nudm-sdm/v2/(?P<supi>[^/?#]*)/v2x-data$",
            r"^/nudm-sdm/v2/(?P<ueId>[^/?#]*)/id-translation-result$",
            r"^/nudm-sdm/v2/(?P<ueId>[^/?#]*)/lcs-privacy-data$",
            r"^/nudm-sdm/v2/(?P<ueId>[^/?#]*)/sdm-subscriptions$",
            r"^/nudm-sdm/v2/(?P<ueId>[^/?#]*)/sdm-subscriptions/(?P<subscriptionId>[^/?#]*)$"
        ])
        .expect("Unable to create global regex set");
    }
    pub(crate) static ID_GROUP_DATA_GROUP_IDENTIFIERS: usize = 0;
    pub(crate) static ID_MULTIPLE_IDENTIFIERS: usize = 1;
    pub(crate) static ID_SHARED_DATA: usize = 2;
    pub(crate) static ID_SHARED_DATA_SUBSCRIPTIONS: usize = 3;
    pub(crate) static ID_SHARED_DATA_SUBSCRIPTIONS_SUBSCRIPTIONID: usize = 4;
    lazy_static! {
        pub static ref REGEX_SHARED_DATA_SUBSCRIPTIONS_SUBSCRIPTIONID: regex::Regex =
            #[allow(clippy::invalid_regex)]
            regex::Regex::new(
                r"^/nudm-sdm/v2/shared-data-subscriptions/(?P<subscriptionId>[^/?#]*)$"
            )
            .expect("Unable to create regex for SHARED_DATA_SUBSCRIPTIONS_SUBSCRIPTIONID");
    }
    pub(crate) static ID_SHARED_DATA_SHAREDDATAID: usize = 5;
    lazy_static! {
        pub static ref REGEX_SHARED_DATA_SHAREDDATAID: regex::Regex =
            #[allow(clippy::invalid_regex)]
            regex::Regex::new(r"^/nudm-sdm/v2/shared-data/(?P<sharedDataId>[^/?#]*)$")
                .expect("Unable to create regex for SHARED_DATA_SHAREDDATAID");
    }
    pub(crate) static ID_SUPI: usize = 6;
    lazy_static! {
        pub static ref REGEX_SUPI: regex::Regex =
            #[allow(clippy::invalid_regex)]
            regex::Regex::new(r"^/nudm-sdm/v2/(?P<supi>[^/?#]*)$")
                .expect("Unable to create regex for SUPI");
    }
    pub(crate) static ID_SUPI_5MBS_DATA: usize = 7;
    lazy_static! {
        pub static ref REGEX_SUPI_5MBS_DATA: regex::Regex =
            #[allow(clippy::invalid_regex)]
            regex::Regex::new(r"^/nudm-sdm/v2/(?P<supi>[^/?#]*)/5mbs-data$")
                .expect("Unable to create regex for SUPI_5MBS_DATA");
    }
    pub(crate) static ID_SUPI_AM_DATA: usize = 8;
    lazy_static! {
        pub static ref REGEX_SUPI_AM_DATA: regex::Regex =
            #[allow(clippy::invalid_regex)]
            regex::Regex::new(r"^/nudm-sdm/v2/(?P<supi>[^/?#]*)/am-data$")
                .expect("Unable to create regex for SUPI_AM_DATA");
    }
    pub(crate) static ID_SUPI_AM_DATA_CAG_ACK: usize = 9;
    lazy_static! {
        pub static ref REGEX_SUPI_AM_DATA_CAG_ACK: regex::Regex =
            #[allow(clippy::invalid_regex)]
            regex::Regex::new(r"^/nudm-sdm/v2/(?P<supi>[^/?#]*)/am-data/cag-ack$")
                .expect("Unable to create regex for SUPI_AM_DATA_CAG_ACK");
    }
    pub(crate) static ID_SUPI_AM_DATA_ECR_DATA: usize = 10;
    lazy_static! {
        pub static ref REGEX_SUPI_AM_DATA_ECR_DATA: regex::Regex =
            #[allow(clippy::invalid_regex)]
            regex::Regex::new(r"^/nudm-sdm/v2/(?P<supi>[^/?#]*)/am-data/ecr-data$")
                .expect("Unable to create regex for SUPI_AM_DATA_ECR_DATA");
    }
    pub(crate) static ID_SUPI_AM_DATA_SOR_ACK: usize = 11;
    lazy_static! {
        pub static ref REGEX_SUPI_AM_DATA_SOR_ACK: regex::Regex =
            #[allow(clippy::invalid_regex)]
            regex::Regex::new(r"^/nudm-sdm/v2/(?P<supi>[^/?#]*)/am-data/sor-ack$")
                .expect("Unable to create regex for SUPI_AM_DATA_SOR_ACK");
    }
    pub(crate) static ID_SUPI_AM_DATA_SUBSCRIBED_SNSSAIS_ACK: usize = 12;
    lazy_static! {
        pub static ref REGEX_SUPI_AM_DATA_SUBSCRIBED_SNSSAIS_ACK: regex::Regex =
            #[allow(clippy::invalid_regex)]
            regex::Regex::new(r"^/nudm-sdm/v2/(?P<supi>[^/?#]*)/am-data/subscribed-snssais-ack$")
                .expect("Unable to create regex for SUPI_AM_DATA_SUBSCRIBED_SNSSAIS_ACK");
    }
    pub(crate) static ID_SUPI_AM_DATA_UPDATE_SOR: usize = 13;
    lazy_static! {
        pub static ref REGEX_SUPI_AM_DATA_UPDATE_SOR: regex::Regex =
            #[allow(clippy::invalid_regex)]
            regex::Regex::new(r"^/nudm-sdm/v2/(?P<supi>[^/?#]*)/am-data/update-sor$")
                .expect("Unable to create regex for SUPI_AM_DATA_UPDATE_SOR");
    }
    pub(crate) static ID_SUPI_AM_DATA_UPU_ACK: usize = 14;
    lazy_static! {
        pub static ref REGEX_SUPI_AM_DATA_UPU_ACK: regex::Regex =
            #[allow(clippy::invalid_regex)]
            regex::Regex::new(r"^/nudm-sdm/v2/(?P<supi>[^/?#]*)/am-data/upu-ack$")
                .expect("Unable to create regex for SUPI_AM_DATA_UPU_ACK");
    }
    pub(crate) static ID_SUPI_LCS_BCA_DATA: usize = 15;
    lazy_static! {
        pub static ref REGEX_SUPI_LCS_BCA_DATA: regex::Regex =
            #[allow(clippy::invalid_regex)]
            regex::Regex::new(r"^/nudm-sdm/v2/(?P<supi>[^/?#]*)/lcs-bca-data$")
                .expect("Unable to create regex for SUPI_LCS_BCA_DATA");
    }
    pub(crate) static ID_SUPI_LCS_MO_DATA: usize = 16;
    lazy_static! {
        pub static ref REGEX_SUPI_LCS_MO_DATA: regex::Regex =
            #[allow(clippy::invalid_regex)]
            regex::Regex::new(r"^/nudm-sdm/v2/(?P<supi>[^/?#]*)/lcs-mo-data$")
                .expect("Unable to create regex for SUPI_LCS_MO_DATA");
    }
    pub(crate) static ID_SUPI_NSSAI: usize = 17;
    lazy_static! {
        pub static ref REGEX_SUPI_NSSAI: regex::Regex =
            #[allow(clippy::invalid_regex)]
            regex::Regex::new(r"^/nudm-sdm/v2/(?P<supi>[^/?#]*)/nssai$")
                .expect("Unable to create regex for SUPI_NSSAI");
    }
    pub(crate) static ID_SUPI_PROSE_DATA: usize = 18;
    lazy_static! {
        pub static ref REGEX_SUPI_PROSE_DATA: regex::Regex =
            #[allow(clippy::invalid_regex)]
            regex::Regex::new(r"^/nudm-sdm/v2/(?P<supi>[^/?#]*)/prose-data$")
                .expect("Unable to create regex for SUPI_PROSE_DATA");
    }
    pub(crate) static ID_SUPI_SM_DATA: usize = 19;
    lazy_static! {
        pub static ref REGEX_SUPI_SM_DATA: regex::Regex =
            #[allow(clippy::invalid_regex)]
            regex::Regex::new(r"^/nudm-sdm/v2/(?P<supi>[^/?#]*)/sm-data$")
                .expect("Unable to create regex for SUPI_SM_DATA");
    }
    pub(crate) static ID_SUPI_SMF_SELECT_DATA: usize = 20;
    lazy_static! {
        pub static ref REGEX_SUPI_SMF_SELECT_DATA: regex::Regex =
            #[allow(clippy::invalid_regex)]
            regex::Regex::new(r"^/nudm-sdm/v2/(?P<supi>[^/?#]*)/smf-select-data$")
                .expect("Unable to create regex for SUPI_SMF_SELECT_DATA");
    }
    pub(crate) static ID_SUPI_SMS_DATA: usize = 21;
    lazy_static! {
        pub static ref REGEX_SUPI_SMS_DATA: regex::Regex =
            #[allow(clippy::invalid_regex)]
            regex::Regex::new(r"^/nudm-sdm/v2/(?P<supi>[^/?#]*)/sms-data$")
                .expect("Unable to create regex for SUPI_SMS_DATA");
    }
    pub(crate) static ID_SUPI_SMS_MNG_DATA: usize = 22;
    lazy_static! {
        pub static ref REGEX_SUPI_SMS_MNG_DATA: regex::Regex =
            #[allow(clippy::invalid_regex)]
            regex::Regex::new(r"^/nudm-sdm/v2/(?P<supi>[^/?#]*)/sms-mng-data$")
                .expect("Unable to create regex for SUPI_SMS_MNG_DATA");
    }
    pub(crate) static ID_SUPI_TRACE_DATA: usize = 23;
    lazy_static! {
        pub static ref REGEX_SUPI_TRACE_DATA: regex::Regex =
            #[allow(clippy::invalid_regex)]
            regex::Regex::new(r"^/nudm-sdm/v2/(?P<supi>[^/?#]*)/trace-data$")
                .expect("Unable to create regex for SUPI_TRACE_DATA");
    }
    pub(crate) static ID_SUPI_UC_DATA: usize = 24;
    lazy_static! {
        pub static ref REGEX_SUPI_UC_DATA: regex::Regex =
            #[allow(clippy::invalid_regex)]
            regex::Regex::new(r"^/nudm-sdm/v2/(?P<supi>[^/?#]*)/uc-data$")
                .expect("Unable to create regex for SUPI_UC_DATA");
    }
    pub(crate) static ID_SUPI_UE_CONTEXT_IN_AMF_DATA: usize = 25;
    lazy_static! {
        pub static ref REGEX_SUPI_UE_CONTEXT_IN_AMF_DATA: regex::Regex =
            #[allow(clippy::invalid_regex)]
            regex::Regex::new(r"^/nudm-sdm/v2/(?P<supi>[^/?#]*)/ue-context-in-amf-data$")
                .expect("Unable to create regex for SUPI_UE_CONTEXT_IN_AMF_DATA");
    }
    pub(crate) static ID_SUPI_UE_CONTEXT_IN_SMF_DATA: usize = 26;
    lazy_static! {
        pub static ref REGEX_SUPI_UE_CONTEXT_IN_SMF_DATA: regex::Regex =
            #[allow(clippy::invalid_regex)]
            regex::Regex::new(r"^/nudm-sdm/v2/(?P<supi>[^/?#]*)/ue-context-in-smf-data$")
                .expect("Unable to create regex for SUPI_UE_CONTEXT_IN_SMF_DATA");
    }
    pub(crate) static ID_SUPI_UE_CONTEXT_IN_SMSF_DATA: usize = 27;
    lazy_static! {
        pub static ref REGEX_SUPI_UE_CONTEXT_IN_SMSF_DATA: regex::Regex =
            #[allow(clippy::invalid_regex)]
            regex::Regex::new(r"^/nudm-sdm/v2/(?P<supi>[^/?#]*)/ue-context-in-smsf-data$")
                .expect("Unable to create regex for SUPI_UE_CONTEXT_IN_SMSF_DATA");
    }
    pub(crate) static ID_SUPI_V2X_DATA: usize = 28;
    lazy_static! {
        pub static ref REGEX_SUPI_V2X_DATA: regex::Regex =
            #[allow(clippy::invalid_regex)]
            regex::Regex::new(r"^/nudm-sdm/v2/(?P<supi>[^/?#]*)/v2x-data$")
                .expect("Unable to create regex for SUPI_V2X_DATA");
    }
    pub(crate) static ID_UEID_ID_TRANSLATION_RESULT: usize = 29;
    lazy_static! {
        pub static ref REGEX_UEID_ID_TRANSLATION_RESULT: regex::Regex =
            #[allow(clippy::invalid_regex)]
            regex::Regex::new(r"^/nudm-sdm/v2/(?P<ueId>[^/?#]*)/id-translation-result$")
                .expect("Unable to create regex for UEID_ID_TRANSLATION_RESULT");
    }
    pub(crate) static ID_UEID_LCS_PRIVACY_DATA: usize = 30;
    lazy_static! {
        pub static ref REGEX_UEID_LCS_PRIVACY_DATA: regex::Regex =
            #[allow(clippy::invalid_regex)]
            regex::Regex::new(r"^/nudm-sdm/v2/(?P<ueId>[^/?#]*)/lcs-privacy-data$")
                .expect("Unable to create regex for UEID_LCS_PRIVACY_DATA");
    }
    pub(crate) static ID_UEID_SDM_SUBSCRIPTIONS: usize = 31;
    lazy_static! {
        pub static ref REGEX_UEID_SDM_SUBSCRIPTIONS: regex::Regex =
            #[allow(clippy::invalid_regex)]
            regex::Regex::new(r"^/nudm-sdm/v2/(?P<ueId>[^/?#]*)/sdm-subscriptions$")
                .expect("Unable to create regex for UEID_SDM_SUBSCRIPTIONS");
    }
    pub(crate) static ID_UEID_SDM_SUBSCRIPTIONS_SUBSCRIPTIONID: usize = 32;
    lazy_static! {
        pub static ref REGEX_UEID_SDM_SUBSCRIPTIONS_SUBSCRIPTIONID: regex::Regex =
            #[allow(clippy::invalid_regex)]
            regex::Regex::new(
                r"^/nudm-sdm/v2/(?P<ueId>[^/?#]*)/sdm-subscriptions/(?P<subscriptionId>[^/?#]*)$"
            )
            .expect("Unable to create regex for UEID_SDM_SUBSCRIPTIONS_SUBSCRIPTIONID");
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
                // GetAmData - GET /{supi}/am-data
                hyper::Method::GET if path.matched(paths::ID_SUPI_AM_DATA) => {
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
                                "nudm-sdm".to_string(), // Access to the nudm-sdm API
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
                    paths::REGEX_SUPI_AM_DATA
                    .captures(path)
                    .unwrap_or_else(||
                        panic!("Path {} matched RE SUPI_AM_DATA in set but failed match against \"{}\"", path, paths::REGEX_SUPI_AM_DATA.as_str())
                    );

                    let param_supi = match percent_encoding::percent_decode(path_params["supi"].as_bytes()).decode_utf8() {
                    Ok(param_supi) => match param_supi.parse::<String>() {
                        Ok(param_supi) => param_supi,
                        Err(e) => return Ok(Response::builder()
                                        .status(StatusCode::BAD_REQUEST)
                                        .body(Body::from(format!("Couldn't parse path parameter supi: {}", e)))
                                        .expect("Unable to create Bad Request response for invalid path parameter")),
                    },
                    Err(_) => return Ok(Response::builder()
                                        .status(StatusCode::BAD_REQUEST)
                                        .body(Body::from(format!("Couldn't percent-decode path parameter as UTF-8: {}", &path_params["supi"])))
                                        .expect("Unable to create Bad Request response for invalid percent decode"))
                };

                    // Header parameters
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
                    let param_if_modified_since =
                        headers.get(HeaderName::from_static("if-modified-since"));

                    let param_if_modified_since = match param_if_modified_since {
                        Some(v) => {
                            match header::IntoHeaderValue::<String>::try_from((*v).clone()) {
                                Ok(result) => Some(result.0),
                                Err(err) => {
                                    return Ok(Response::builder()
                                        .status(StatusCode::BAD_REQUEST)
                                        .body(Body::from(format!("Invalid header If-Modified-Since - {}", err)))
                                        .expect("Unable to create Bad Request response for invalid header If-Modified-Since"));
                                }
                            }
                        }
                        None => None,
                    };

                    // Query parameters (note that non-required or collection query parameters will ignore garbage values, rather than causing a 400 response)
                    let query_params =
                        form_urlencoded::parse(uri.query().unwrap_or_default().as_bytes())
                            .collect::<Vec<_>>();
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
                    let param_plmn_id = query_params
                        .iter()
                        .filter(|e| e.0 == "plmn-id")
                        .map(|e| e.1.to_owned())
                        .next();
                    let param_plmn_id = match param_plmn_id {
                        Some(param_plmn_id) => {
                            let param_plmn_id =
                                serde_json::from_str::<models::PlmnIdNid>(&param_plmn_id);
                            match param_plmn_id {
                            Ok(param_plmn_id) => Some(param_plmn_id),
                            Err(e) => return Ok(Response::builder()
                                .status(StatusCode::BAD_REQUEST)
                                .body(Body::from(format!("Couldn't parse query parameter plmn-id - doesn't match schema: {}", e)))
                                .expect("Unable to create Bad Request response for invalid query parameter plmn-id")),
                        }
                        }
                        None => None,
                    };
                    let param_adjacent_plmns = query_params
                        .iter()
                        .filter(|e| e.0 == "adjacent-plmns")
                        .map(|e| e.1.to_owned())
                        .filter_map(|param_adjacent_plmns| param_adjacent_plmns.parse().ok())
                        .collect::<Vec<_>>();
                    let param_adjacent_plmns = if !param_adjacent_plmns.is_empty() {
                        Some(param_adjacent_plmns)
                    } else {
                        None
                    };
                    let param_disaster_roaming_ind = query_params
                        .iter()
                        .filter(|e| e.0 == "disaster-roaming-ind")
                        .map(|e| e.1.to_owned())
                        .next();
                    let param_disaster_roaming_ind = match param_disaster_roaming_ind {
                        Some(param_disaster_roaming_ind) => {
                            let param_disaster_roaming_ind =
                                <bool as std::str::FromStr>::from_str(&param_disaster_roaming_ind);
                            match param_disaster_roaming_ind {
                            Ok(param_disaster_roaming_ind) => Some(param_disaster_roaming_ind),
                            Err(e) => return Ok(Response::builder()
                                .status(StatusCode::BAD_REQUEST)
                                .body(Body::from(format!("Couldn't parse query parameter disaster-roaming-ind - doesn't match schema: {}", e)))
                                .expect("Unable to create Bad Request response for invalid query parameter disaster-roaming-ind")),
                        }
                        }
                        None => None,
                    };

                    let result = api_impl
                        .get_am_data(
                            param_supi,
                            param_supported_features,
                            param_plmn_id,
                            param_adjacent_plmns.as_ref(),
                            param_disaster_roaming_ind,
                            param_if_none_match,
                            param_if_modified_since,
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
                            GetAmDataResponse::ExpectedResponseToAValidRequest {
                                body,
                                cache_control,
                                e_tag,
                                last_modified,
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
                                if let Some(last_modified) = last_modified {
                                    let last_modified = match header::IntoHeaderValue(last_modified).try_into() {
                                                        Ok(val) => val,
                                                        Err(e) => {
                                                            return Ok(Response::builder()
                                                                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                                                                    .body(Body::from(format!("An internal server error occurred handling last_modified header - {}", e)))
                                                                    .expect("Unable to create Internal Server Error for invalid response header"))
                                                        }
                                                    };

                                    response.headers_mut().insert(
                                        HeaderName::from_static("last-modified"),
                                        last_modified,
                                    );
                                }
                                *response.status_mut() = StatusCode::from_u16(200)
                                    .expect("Unable to turn 200 into a StatusCode");
                                response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for GET_AM_DATA_EXPECTED_RESPONSE_TO_A_VALID_REQUEST"));
                                let body = serde_json::to_string(&body)
                                    .expect("impossible to fail to serialize");
                                *response.body_mut() = Body::from(body);
                            }
                            GetAmDataResponse::BadRequest(body) => {
                                *response.status_mut() = StatusCode::from_u16(400)
                                    .expect("Unable to turn 400 into a StatusCode");
                                response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for GET_AM_DATA_BAD_REQUEST"));
                                let body = serde_json::to_string(&body)
                                    .expect("impossible to fail to serialize");
                                *response.body_mut() = Body::from(body);
                            }
                            GetAmDataResponse::NotFound(body) => {
                                *response.status_mut() = StatusCode::from_u16(404)
                                    .expect("Unable to turn 404 into a StatusCode");
                                response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for GET_AM_DATA_NOT_FOUND"));
                                let body = serde_json::to_string(&body)
                                    .expect("impossible to fail to serialize");
                                *response.body_mut() = Body::from(body);
                            }
                            GetAmDataResponse::InternalServerError(body) => {
                                *response.status_mut() = StatusCode::from_u16(500)
                                    .expect("Unable to turn 500 into a StatusCode");
                                response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for GET_AM_DATA_INTERNAL_SERVER_ERROR"));
                                let body = serde_json::to_string(&body)
                                    .expect("impossible to fail to serialize");
                                *response.body_mut() = Body::from(body);
                            }
                            GetAmDataResponse::ServiceUnavailable(body) => {
                                *response.status_mut() = StatusCode::from_u16(503)
                                    .expect("Unable to turn 503 into a StatusCode");
                                response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for GET_AM_DATA_SERVICE_UNAVAILABLE"));
                                let body = serde_json::to_string(&body)
                                    .expect("impossible to fail to serialize");
                                *response.body_mut() = Body::from(body);
                            }
                            GetAmDataResponse::UnexpectedError => {
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

                // GetMbsData - GET /{supi}/5mbs-data
                hyper::Method::GET if path.matched(paths::ID_SUPI_5MBS_DATA) => {
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
                                "nudm-sdm".to_string(), // Access to the nudm-sdm API
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
                    paths::REGEX_SUPI_5MBS_DATA
                    .captures(path)
                    .unwrap_or_else(||
                        panic!("Path {} matched RE SUPI_5MBS_DATA in set but failed match against \"{}\"", path, paths::REGEX_SUPI_5MBS_DATA.as_str())
                    );

                    let param_supi = match percent_encoding::percent_decode(path_params["supi"].as_bytes()).decode_utf8() {
                    Ok(param_supi) => match param_supi.parse::<String>() {
                        Ok(param_supi) => param_supi,
                        Err(e) => return Ok(Response::builder()
                                        .status(StatusCode::BAD_REQUEST)
                                        .body(Body::from(format!("Couldn't parse path parameter supi: {}", e)))
                                        .expect("Unable to create Bad Request response for invalid path parameter")),
                    },
                    Err(_) => return Ok(Response::builder()
                                        .status(StatusCode::BAD_REQUEST)
                                        .body(Body::from(format!("Couldn't percent-decode path parameter as UTF-8: {}", &path_params["supi"])))
                                        .expect("Unable to create Bad Request response for invalid percent decode"))
                };

                    // Header parameters
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
                    let param_if_modified_since =
                        headers.get(HeaderName::from_static("if-modified-since"));

                    let param_if_modified_since = match param_if_modified_since {
                        Some(v) => {
                            match header::IntoHeaderValue::<String>::try_from((*v).clone()) {
                                Ok(result) => Some(result.0),
                                Err(err) => {
                                    return Ok(Response::builder()
                                        .status(StatusCode::BAD_REQUEST)
                                        .body(Body::from(format!("Invalid header If-Modified-Since - {}", err)))
                                        .expect("Unable to create Bad Request response for invalid header If-Modified-Since"));
                                }
                            }
                        }
                        None => None,
                    };

                    // Query parameters (note that non-required or collection query parameters will ignore garbage values, rather than causing a 400 response)
                    let query_params =
                        form_urlencoded::parse(uri.query().unwrap_or_default().as_bytes())
                            .collect::<Vec<_>>();
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

                    let result = api_impl
                        .get_mbs_data(
                            param_supi,
                            param_supported_features,
                            param_if_none_match,
                            param_if_modified_since,
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
                            GetMbsDataResponse::ExpectedResponseToAValidRequest {
                                body,
                                cache_control,
                                e_tag,
                                last_modified,
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
                                if let Some(last_modified) = last_modified {
                                    let last_modified = match header::IntoHeaderValue(last_modified).try_into() {
                                                        Ok(val) => val,
                                                        Err(e) => {
                                                            return Ok(Response::builder()
                                                                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                                                                    .body(Body::from(format!("An internal server error occurred handling last_modified header - {}", e)))
                                                                    .expect("Unable to create Internal Server Error for invalid response header"))
                                                        }
                                                    };

                                    response.headers_mut().insert(
                                        HeaderName::from_static("last-modified"),
                                        last_modified,
                                    );
                                }
                                *response.status_mut() = StatusCode::from_u16(200)
                                    .expect("Unable to turn 200 into a StatusCode");
                                response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for GET_MBS_DATA_EXPECTED_RESPONSE_TO_A_VALID_REQUEST"));
                                let body = serde_json::to_string(&body)
                                    .expect("impossible to fail to serialize");
                                *response.body_mut() = Body::from(body);
                            }
                            GetMbsDataResponse::BadRequest(body) => {
                                *response.status_mut() = StatusCode::from_u16(400)
                                    .expect("Unable to turn 400 into a StatusCode");
                                response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for GET_MBS_DATA_BAD_REQUEST"));
                                let body = serde_json::to_string(&body)
                                    .expect("impossible to fail to serialize");
                                *response.body_mut() = Body::from(body);
                            }
                            GetMbsDataResponse::NotFound(body) => {
                                *response.status_mut() = StatusCode::from_u16(404)
                                    .expect("Unable to turn 404 into a StatusCode");
                                response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for GET_MBS_DATA_NOT_FOUND"));
                                let body = serde_json::to_string(&body)
                                    .expect("impossible to fail to serialize");
                                *response.body_mut() = Body::from(body);
                            }
                            GetMbsDataResponse::InternalServerError(body) => {
                                *response.status_mut() = StatusCode::from_u16(500)
                                    .expect("Unable to turn 500 into a StatusCode");
                                response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for GET_MBS_DATA_INTERNAL_SERVER_ERROR"));
                                let body = serde_json::to_string(&body)
                                    .expect("impossible to fail to serialize");
                                *response.body_mut() = Body::from(body);
                            }
                            GetMbsDataResponse::ServiceUnavailable(body) => {
                                *response.status_mut() = StatusCode::from_u16(503)
                                    .expect("Unable to turn 503 into a StatusCode");
                                response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for GET_MBS_DATA_SERVICE_UNAVAILABLE"));
                                let body = serde_json::to_string(&body)
                                    .expect("impossible to fail to serialize");
                                *response.body_mut() = Body::from(body);
                            }
                            GetMbsDataResponse::UnexpectedError => {
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

                // GetEcrData - GET /{supi}/am-data/ecr-data
                hyper::Method::GET if path.matched(paths::ID_SUPI_AM_DATA_ECR_DATA) => {
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
                                "nudm-sdm".to_string(), // Access to the nudm-sdm API
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
                    paths::REGEX_SUPI_AM_DATA_ECR_DATA
                    .captures(path)
                    .unwrap_or_else(||
                        panic!("Path {} matched RE SUPI_AM_DATA_ECR_DATA in set but failed match against \"{}\"", path, paths::REGEX_SUPI_AM_DATA_ECR_DATA.as_str())
                    );

                    let param_supi = match percent_encoding::percent_decode(path_params["supi"].as_bytes()).decode_utf8() {
                    Ok(param_supi) => match param_supi.parse::<String>() {
                        Ok(param_supi) => param_supi,
                        Err(e) => return Ok(Response::builder()
                                        .status(StatusCode::BAD_REQUEST)
                                        .body(Body::from(format!("Couldn't parse path parameter supi: {}", e)))
                                        .expect("Unable to create Bad Request response for invalid path parameter")),
                    },
                    Err(_) => return Ok(Response::builder()
                                        .status(StatusCode::BAD_REQUEST)
                                        .body(Body::from(format!("Couldn't percent-decode path parameter as UTF-8: {}", &path_params["supi"])))
                                        .expect("Unable to create Bad Request response for invalid percent decode"))
                };

                    // Header parameters
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
                    let param_if_modified_since =
                        headers.get(HeaderName::from_static("if-modified-since"));

                    let param_if_modified_since = match param_if_modified_since {
                        Some(v) => {
                            match header::IntoHeaderValue::<String>::try_from((*v).clone()) {
                                Ok(result) => Some(result.0),
                                Err(err) => {
                                    return Ok(Response::builder()
                                        .status(StatusCode::BAD_REQUEST)
                                        .body(Body::from(format!("Invalid header If-Modified-Since - {}", err)))
                                        .expect("Unable to create Bad Request response for invalid header If-Modified-Since"));
                                }
                            }
                        }
                        None => None,
                    };

                    // Query parameters (note that non-required or collection query parameters will ignore garbage values, rather than causing a 400 response)
                    let query_params =
                        form_urlencoded::parse(uri.query().unwrap_or_default().as_bytes())
                            .collect::<Vec<_>>();
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

                    let result = api_impl
                        .get_ecr_data(
                            param_supi,
                            param_supported_features,
                            param_if_none_match,
                            param_if_modified_since,
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
                            GetEcrDataResponse::ExpectedResponseToAValidRequest {
                                body,
                                cache_control,
                                e_tag,
                                last_modified,
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
                                if let Some(last_modified) = last_modified {
                                    let last_modified = match header::IntoHeaderValue(last_modified).try_into() {
                                                        Ok(val) => val,
                                                        Err(e) => {
                                                            return Ok(Response::builder()
                                                                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                                                                    .body(Body::from(format!("An internal server error occurred handling last_modified header - {}", e)))
                                                                    .expect("Unable to create Internal Server Error for invalid response header"))
                                                        }
                                                    };

                                    response.headers_mut().insert(
                                        HeaderName::from_static("last-modified"),
                                        last_modified,
                                    );
                                }
                                *response.status_mut() = StatusCode::from_u16(200)
                                    .expect("Unable to turn 200 into a StatusCode");
                                response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for GET_ECR_DATA_EXPECTED_RESPONSE_TO_A_VALID_REQUEST"));
                                let body = serde_json::to_string(&body)
                                    .expect("impossible to fail to serialize");
                                *response.body_mut() = Body::from(body);
                            }
                            GetEcrDataResponse::BadRequest(body) => {
                                *response.status_mut() = StatusCode::from_u16(400)
                                    .expect("Unable to turn 400 into a StatusCode");
                                response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for GET_ECR_DATA_BAD_REQUEST"));
                                let body = serde_json::to_string(&body)
                                    .expect("impossible to fail to serialize");
                                *response.body_mut() = Body::from(body);
                            }
                            GetEcrDataResponse::NotFound(body) => {
                                *response.status_mut() = StatusCode::from_u16(404)
                                    .expect("Unable to turn 404 into a StatusCode");
                                response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for GET_ECR_DATA_NOT_FOUND"));
                                let body = serde_json::to_string(&body)
                                    .expect("impossible to fail to serialize");
                                *response.body_mut() = Body::from(body);
                            }
                            GetEcrDataResponse::InternalServerError(body) => {
                                *response.status_mut() = StatusCode::from_u16(500)
                                    .expect("Unable to turn 500 into a StatusCode");
                                response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for GET_ECR_DATA_INTERNAL_SERVER_ERROR"));
                                let body = serde_json::to_string(&body)
                                    .expect("impossible to fail to serialize");
                                *response.body_mut() = Body::from(body);
                            }
                            GetEcrDataResponse::ServiceUnavailable(body) => {
                                *response.status_mut() = StatusCode::from_u16(503)
                                    .expect("Unable to turn 503 into a StatusCode");
                                response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for GET_ECR_DATA_SERVICE_UNAVAILABLE"));
                                let body = serde_json::to_string(&body)
                                    .expect("impossible to fail to serialize");
                                *response.body_mut() = Body::from(body);
                            }
                            GetEcrDataResponse::UnexpectedError => {
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

                // GetSupiOrGpsi - GET /{ueId}/id-translation-result
                hyper::Method::GET if path.matched(paths::ID_UEID_ID_TRANSLATION_RESULT) => {
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
                                "nudm-sdm".to_string(), // Access to the nudm-sdm API
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
                    paths::REGEX_UEID_ID_TRANSLATION_RESULT
                    .captures(path)
                    .unwrap_or_else(||
                        panic!("Path {} matched RE UEID_ID_TRANSLATION_RESULT in set but failed match against \"{}\"", path, paths::REGEX_UEID_ID_TRANSLATION_RESULT.as_str())
                    );

                    let param_ue_id = match percent_encoding::percent_decode(path_params["ueId"].as_bytes()).decode_utf8() {
                    Ok(param_ue_id) => match param_ue_id.parse::<String>() {
                        Ok(param_ue_id) => param_ue_id,
                        Err(e) => return Ok(Response::builder()
                                        .status(StatusCode::BAD_REQUEST)
                                        .body(Body::from(format!("Couldn't parse path parameter ueId: {}", e)))
                                        .expect("Unable to create Bad Request response for invalid path parameter")),
                    },
                    Err(_) => return Ok(Response::builder()
                                        .status(StatusCode::BAD_REQUEST)
                                        .body(Body::from(format!("Couldn't percent-decode path parameter as UTF-8: {}", &path_params["ueId"])))
                                        .expect("Unable to create Bad Request response for invalid percent decode"))
                };

                    // Header parameters
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
                    let param_if_modified_since =
                        headers.get(HeaderName::from_static("if-modified-since"));

                    let param_if_modified_since = match param_if_modified_since {
                        Some(v) => {
                            match header::IntoHeaderValue::<String>::try_from((*v).clone()) {
                                Ok(result) => Some(result.0),
                                Err(err) => {
                                    return Ok(Response::builder()
                                        .status(StatusCode::BAD_REQUEST)
                                        .body(Body::from(format!("Invalid header If-Modified-Since - {}", err)))
                                        .expect("Unable to create Bad Request response for invalid header If-Modified-Since"));
                                }
                            }
                        }
                        None => None,
                    };

                    // Query parameters (note that non-required or collection query parameters will ignore garbage values, rather than causing a 400 response)
                    let query_params =
                        form_urlencoded::parse(uri.query().unwrap_or_default().as_bytes())
                            .collect::<Vec<_>>();
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
                    let param_af_id = query_params
                        .iter()
                        .filter(|e| e.0 == "af-id")
                        .map(|e| e.1.to_owned())
                        .next();
                    let param_af_id = match param_af_id {
                        Some(param_af_id) => {
                            let param_af_id = <String as std::str::FromStr>::from_str(&param_af_id);
                            match param_af_id {
                            Ok(param_af_id) => Some(param_af_id),
                            Err(e) => return Ok(Response::builder()
                                .status(StatusCode::BAD_REQUEST)
                                .body(Body::from(format!("Couldn't parse query parameter af-id - doesn't match schema: {}", e)))
                                .expect("Unable to create Bad Request response for invalid query parameter af-id")),
                        }
                        }
                        None => None,
                    };
                    let param_app_port_id = query_params
                        .iter()
                        .filter(|e| e.0 == "app-port-id")
                        .map(|e| e.1.to_owned())
                        .next();
                    let param_app_port_id = match param_app_port_id {
                        Some(param_app_port_id) => {
                            let param_app_port_id =
                                serde_json::from_str::<models::AppPortId>(&param_app_port_id);
                            match param_app_port_id {
                            Ok(param_app_port_id) => Some(param_app_port_id),
                            Err(e) => return Ok(Response::builder()
                                .status(StatusCode::BAD_REQUEST)
                                .body(Body::from(format!("Couldn't parse query parameter app-port-id - doesn't match schema: {}", e)))
                                .expect("Unable to create Bad Request response for invalid query parameter app-port-id")),
                        }
                        }
                        None => None,
                    };
                    let param_af_service_id = query_params
                        .iter()
                        .filter(|e| e.0 == "af-service-id")
                        .map(|e| e.1.to_owned())
                        .next();
                    let param_af_service_id = match param_af_service_id {
                        Some(param_af_service_id) => {
                            let param_af_service_id =
                                <String as std::str::FromStr>::from_str(&param_af_service_id);
                            match param_af_service_id {
                            Ok(param_af_service_id) => Some(param_af_service_id),
                            Err(e) => return Ok(Response::builder()
                                .status(StatusCode::BAD_REQUEST)
                                .body(Body::from(format!("Couldn't parse query parameter af-service-id - doesn't match schema: {}", e)))
                                .expect("Unable to create Bad Request response for invalid query parameter af-service-id")),
                        }
                        }
                        None => None,
                    };
                    let param_mtc_provider_info = query_params
                        .iter()
                        .filter(|e| e.0 == "mtc-provider-info")
                        .map(|e| e.1.to_owned())
                        .next();
                    let param_mtc_provider_info = match param_mtc_provider_info {
                        Some(param_mtc_provider_info) => {
                            let param_mtc_provider_info =
                                <String as std::str::FromStr>::from_str(&param_mtc_provider_info);
                            match param_mtc_provider_info {
                            Ok(param_mtc_provider_info) => Some(param_mtc_provider_info),
                            Err(e) => return Ok(Response::builder()
                                .status(StatusCode::BAD_REQUEST)
                                .body(Body::from(format!("Couldn't parse query parameter mtc-provider-info - doesn't match schema: {}", e)))
                                .expect("Unable to create Bad Request response for invalid query parameter mtc-provider-info")),
                        }
                        }
                        None => None,
                    };
                    let param_requested_gpsi_type = query_params
                        .iter()
                        .filter(|e| e.0 == "requested-gpsi-type")
                        .map(|e| e.1.to_owned())
                        .next();
                    let param_requested_gpsi_type = match param_requested_gpsi_type {
                        Some(param_requested_gpsi_type) => {
                            let param_requested_gpsi_type =
                                <models::GpsiType as std::str::FromStr>::from_str(
                                    &param_requested_gpsi_type,
                                );
                            match param_requested_gpsi_type {
                            Ok(param_requested_gpsi_type) => Some(param_requested_gpsi_type),
                            Err(e) => return Ok(Response::builder()
                                .status(StatusCode::BAD_REQUEST)
                                .body(Body::from(format!("Couldn't parse query parameter requested-gpsi-type - doesn't match schema: {}", e)))
                                .expect("Unable to create Bad Request response for invalid query parameter requested-gpsi-type")),
                        }
                        }
                        None => None,
                    };

                    let result = api_impl
                        .get_supi_or_gpsi(
                            param_ue_id,
                            param_supported_features,
                            param_af_id,
                            param_app_port_id,
                            param_af_service_id,
                            param_mtc_provider_info,
                            param_requested_gpsi_type,
                            param_if_none_match,
                            param_if_modified_since,
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
                            GetSupiOrGpsiResponse::ExpectedResponseToAValidRequest {
                                body,
                                cache_control,
                                e_tag,
                                last_modified,
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
                                if let Some(last_modified) = last_modified {
                                    let last_modified = match header::IntoHeaderValue(last_modified).try_into() {
                                                        Ok(val) => val,
                                                        Err(e) => {
                                                            return Ok(Response::builder()
                                                                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                                                                    .body(Body::from(format!("An internal server error occurred handling last_modified header - {}", e)))
                                                                    .expect("Unable to create Internal Server Error for invalid response header"))
                                                        }
                                                    };

                                    response.headers_mut().insert(
                                        HeaderName::from_static("last-modified"),
                                        last_modified,
                                    );
                                }
                                *response.status_mut() = StatusCode::from_u16(200)
                                    .expect("Unable to turn 200 into a StatusCode");
                                response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for GET_SUPI_OR_GPSI_EXPECTED_RESPONSE_TO_A_VALID_REQUEST"));
                                let body = serde_json::to_string(&body)
                                    .expect("impossible to fail to serialize");
                                *response.body_mut() = Body::from(body);
                            }
                            GetSupiOrGpsiResponse::BadRequest(body) => {
                                *response.status_mut() = StatusCode::from_u16(400)
                                    .expect("Unable to turn 400 into a StatusCode");
                                response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for GET_SUPI_OR_GPSI_BAD_REQUEST"));
                                let body = serde_json::to_string(&body)
                                    .expect("impossible to fail to serialize");
                                *response.body_mut() = Body::from(body);
                            }
                            GetSupiOrGpsiResponse::Forbidden(body) => {
                                *response.status_mut() = StatusCode::from_u16(403)
                                    .expect("Unable to turn 403 into a StatusCode");
                                response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for GET_SUPI_OR_GPSI_FORBIDDEN"));
                                let body = serde_json::to_string(&body)
                                    .expect("impossible to fail to serialize");
                                *response.body_mut() = Body::from(body);
                            }
                            GetSupiOrGpsiResponse::NotFound(body) => {
                                *response.status_mut() = StatusCode::from_u16(404)
                                    .expect("Unable to turn 404 into a StatusCode");
                                response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for GET_SUPI_OR_GPSI_NOT_FOUND"));
                                let body = serde_json::to_string(&body)
                                    .expect("impossible to fail to serialize");
                                *response.body_mut() = Body::from(body);
                            }
                            GetSupiOrGpsiResponse::InternalServerError(body) => {
                                *response.status_mut() = StatusCode::from_u16(500)
                                    .expect("Unable to turn 500 into a StatusCode");
                                response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for GET_SUPI_OR_GPSI_INTERNAL_SERVER_ERROR"));
                                let body = serde_json::to_string(&body)
                                    .expect("impossible to fail to serialize");
                                *response.body_mut() = Body::from(body);
                            }
                            GetSupiOrGpsiResponse::ServiceUnavailable(body) => {
                                *response.status_mut() = StatusCode::from_u16(503)
                                    .expect("Unable to turn 503 into a StatusCode");
                                response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for GET_SUPI_OR_GPSI_SERVICE_UNAVAILABLE"));
                                let body = serde_json::to_string(&body)
                                    .expect("impossible to fail to serialize");
                                *response.body_mut() = Body::from(body);
                            }
                            GetSupiOrGpsiResponse::UnexpectedError => {
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

                // GetGroupIdentifiers - GET /group-data/group-identifiers
                hyper::Method::GET if path.matched(paths::ID_GROUP_DATA_GROUP_IDENTIFIERS) => {
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
                                "nudm-sdm".to_string(), // Access to the nudm-sdm API
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
                    let param_if_modified_since =
                        headers.get(HeaderName::from_static("if-modified-since"));

                    let param_if_modified_since = match param_if_modified_since {
                        Some(v) => {
                            match header::IntoHeaderValue::<String>::try_from((*v).clone()) {
                                Ok(result) => Some(result.0),
                                Err(err) => {
                                    return Ok(Response::builder()
                                        .status(StatusCode::BAD_REQUEST)
                                        .body(Body::from(format!("Invalid header If-Modified-Since - {}", err)))
                                        .expect("Unable to create Bad Request response for invalid header If-Modified-Since"));
                                }
                            }
                        }
                        None => None,
                    };

                    // Query parameters (note that non-required or collection query parameters will ignore garbage values, rather than causing a 400 response)
                    let query_params =
                        form_urlencoded::parse(uri.query().unwrap_or_default().as_bytes())
                            .collect::<Vec<_>>();
                    let param_ext_group_id = query_params
                        .iter()
                        .filter(|e| e.0 == "ext-group-id")
                        .map(|e| e.1.to_owned())
                        .next();
                    let param_ext_group_id = match param_ext_group_id {
                        Some(param_ext_group_id) => {
                            let param_ext_group_id =
                                <String as std::str::FromStr>::from_str(&param_ext_group_id);
                            match param_ext_group_id {
                            Ok(param_ext_group_id) => Some(param_ext_group_id),
                            Err(e) => return Ok(Response::builder()
                                .status(StatusCode::BAD_REQUEST)
                                .body(Body::from(format!("Couldn't parse query parameter ext-group-id - doesn't match schema: {}", e)))
                                .expect("Unable to create Bad Request response for invalid query parameter ext-group-id")),
                        }
                        }
                        None => None,
                    };
                    let param_int_group_id = query_params
                        .iter()
                        .filter(|e| e.0 == "int-group-id")
                        .map(|e| e.1.to_owned())
                        .next();
                    let param_int_group_id = match param_int_group_id {
                        Some(param_int_group_id) => {
                            let param_int_group_id =
                                <String as std::str::FromStr>::from_str(&param_int_group_id);
                            match param_int_group_id {
                            Ok(param_int_group_id) => Some(param_int_group_id),
                            Err(e) => return Ok(Response::builder()
                                .status(StatusCode::BAD_REQUEST)
                                .body(Body::from(format!("Couldn't parse query parameter int-group-id - doesn't match schema: {}", e)))
                                .expect("Unable to create Bad Request response for invalid query parameter int-group-id")),
                        }
                        }
                        None => None,
                    };
                    let param_ue_id_ind = query_params
                        .iter()
                        .filter(|e| e.0 == "ue-id-ind")
                        .map(|e| e.1.to_owned())
                        .next();
                    let param_ue_id_ind = match param_ue_id_ind {
                        Some(param_ue_id_ind) => {
                            let param_ue_id_ind =
                                <bool as std::str::FromStr>::from_str(&param_ue_id_ind);
                            match param_ue_id_ind {
                            Ok(param_ue_id_ind) => Some(param_ue_id_ind),
                            Err(e) => return Ok(Response::builder()
                                .status(StatusCode::BAD_REQUEST)
                                .body(Body::from(format!("Couldn't parse query parameter ue-id-ind - doesn't match schema: {}", e)))
                                .expect("Unable to create Bad Request response for invalid query parameter ue-id-ind")),
                        }
                        }
                        None => None,
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
                    let param_af_id = query_params
                        .iter()
                        .filter(|e| e.0 == "af-id")
                        .map(|e| e.1.to_owned())
                        .next();
                    let param_af_id = match param_af_id {
                        Some(param_af_id) => {
                            let param_af_id = <String as std::str::FromStr>::from_str(&param_af_id);
                            match param_af_id {
                            Ok(param_af_id) => Some(param_af_id),
                            Err(e) => return Ok(Response::builder()
                                .status(StatusCode::BAD_REQUEST)
                                .body(Body::from(format!("Couldn't parse query parameter af-id - doesn't match schema: {}", e)))
                                .expect("Unable to create Bad Request response for invalid query parameter af-id")),
                        }
                        }
                        None => None,
                    };

                    let result = api_impl
                        .get_group_identifiers(
                            param_ext_group_id,
                            param_int_group_id,
                            param_ue_id_ind,
                            param_supported_features,
                            param_af_id,
                            param_if_none_match,
                            param_if_modified_since,
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
                            GetGroupIdentifiersResponse::ExpectedResponseToAValidRequest {
                                body,
                                cache_control,
                                e_tag,
                                last_modified,
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
                                if let Some(last_modified) = last_modified {
                                    let last_modified = match header::IntoHeaderValue(last_modified).try_into() {
                                                        Ok(val) => val,
                                                        Err(e) => {
                                                            return Ok(Response::builder()
                                                                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                                                                    .body(Body::from(format!("An internal server error occurred handling last_modified header - {}", e)))
                                                                    .expect("Unable to create Internal Server Error for invalid response header"))
                                                        }
                                                    };

                                    response.headers_mut().insert(
                                        HeaderName::from_static("last-modified"),
                                        last_modified,
                                    );
                                }
                                *response.status_mut() = StatusCode::from_u16(200)
                                    .expect("Unable to turn 200 into a StatusCode");
                                response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for GET_GROUP_IDENTIFIERS_EXPECTED_RESPONSE_TO_A_VALID_REQUEST"));
                                let body = serde_json::to_string(&body)
                                    .expect("impossible to fail to serialize");
                                *response.body_mut() = Body::from(body);
                            }
                            GetGroupIdentifiersResponse::BadRequest(body) => {
                                *response.status_mut() = StatusCode::from_u16(400)
                                    .expect("Unable to turn 400 into a StatusCode");
                                response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for GET_GROUP_IDENTIFIERS_BAD_REQUEST"));
                                let body = serde_json::to_string(&body)
                                    .expect("impossible to fail to serialize");
                                *response.body_mut() = Body::from(body);
                            }
                            GetGroupIdentifiersResponse::Forbidden(body) => {
                                *response.status_mut() = StatusCode::from_u16(403)
                                    .expect("Unable to turn 403 into a StatusCode");
                                response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for GET_GROUP_IDENTIFIERS_FORBIDDEN"));
                                let body = serde_json::to_string(&body)
                                    .expect("impossible to fail to serialize");
                                *response.body_mut() = Body::from(body);
                            }
                            GetGroupIdentifiersResponse::NotFound(body) => {
                                *response.status_mut() = StatusCode::from_u16(404)
                                    .expect("Unable to turn 404 into a StatusCode");
                                response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for GET_GROUP_IDENTIFIERS_NOT_FOUND"));
                                let body = serde_json::to_string(&body)
                                    .expect("impossible to fail to serialize");
                                *response.body_mut() = Body::from(body);
                            }
                            GetGroupIdentifiersResponse::InternalServerError(body) => {
                                *response.status_mut() = StatusCode::from_u16(500)
                                    .expect("Unable to turn 500 into a StatusCode");
                                response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for GET_GROUP_IDENTIFIERS_INTERNAL_SERVER_ERROR"));
                                let body = serde_json::to_string(&body)
                                    .expect("impossible to fail to serialize");
                                *response.body_mut() = Body::from(body);
                            }
                            GetGroupIdentifiersResponse::ServiceUnavailable(body) => {
                                *response.status_mut() = StatusCode::from_u16(503)
                                    .expect("Unable to turn 503 into a StatusCode");
                                response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for GET_GROUP_IDENTIFIERS_SERVICE_UNAVAILABLE"));
                                let body = serde_json::to_string(&body)
                                    .expect("impossible to fail to serialize");
                                *response.body_mut() = Body::from(body);
                            }
                            GetGroupIdentifiersResponse::UnexpectedError => {
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

                // GetLcsBcaData - GET /{supi}/lcs-bca-data
                hyper::Method::GET if path.matched(paths::ID_SUPI_LCS_BCA_DATA) => {
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
                                "nudm-sdm".to_string(), // Access to the nudm-sdm API
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
                    paths::REGEX_SUPI_LCS_BCA_DATA
                    .captures(path)
                    .unwrap_or_else(||
                        panic!("Path {} matched RE SUPI_LCS_BCA_DATA in set but failed match against \"{}\"", path, paths::REGEX_SUPI_LCS_BCA_DATA.as_str())
                    );

                    let param_supi = match percent_encoding::percent_decode(path_params["supi"].as_bytes()).decode_utf8() {
                    Ok(param_supi) => match param_supi.parse::<String>() {
                        Ok(param_supi) => param_supi,
                        Err(e) => return Ok(Response::builder()
                                        .status(StatusCode::BAD_REQUEST)
                                        .body(Body::from(format!("Couldn't parse path parameter supi: {}", e)))
                                        .expect("Unable to create Bad Request response for invalid path parameter")),
                    },
                    Err(_) => return Ok(Response::builder()
                                        .status(StatusCode::BAD_REQUEST)
                                        .body(Body::from(format!("Couldn't percent-decode path parameter as UTF-8: {}", &path_params["supi"])))
                                        .expect("Unable to create Bad Request response for invalid percent decode"))
                };

                    // Header parameters
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
                    let param_if_modified_since =
                        headers.get(HeaderName::from_static("if-modified-since"));

                    let param_if_modified_since = match param_if_modified_since {
                        Some(v) => {
                            match header::IntoHeaderValue::<String>::try_from((*v).clone()) {
                                Ok(result) => Some(result.0),
                                Err(err) => {
                                    return Ok(Response::builder()
                                        .status(StatusCode::BAD_REQUEST)
                                        .body(Body::from(format!("Invalid header If-Modified-Since - {}", err)))
                                        .expect("Unable to create Bad Request response for invalid header If-Modified-Since"));
                                }
                            }
                        }
                        None => None,
                    };

                    // Query parameters (note that non-required or collection query parameters will ignore garbage values, rather than causing a 400 response)
                    let query_params =
                        form_urlencoded::parse(uri.query().unwrap_or_default().as_bytes())
                            .collect::<Vec<_>>();
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
                    let param_plmn_id = query_params
                        .iter()
                        .filter(|e| e.0 == "plmn-id")
                        .map(|e| e.1.to_owned())
                        .next();
                    let param_plmn_id = match param_plmn_id {
                        Some(param_plmn_id) => {
                            let param_plmn_id =
                                serde_json::from_str::<models::PlmnId>(&param_plmn_id);
                            match param_plmn_id {
                            Ok(param_plmn_id) => Some(param_plmn_id),
                            Err(e) => return Ok(Response::builder()
                                .status(StatusCode::BAD_REQUEST)
                                .body(Body::from(format!("Couldn't parse query parameter plmn-id - doesn't match schema: {}", e)))
                                .expect("Unable to create Bad Request response for invalid query parameter plmn-id")),
                        }
                        }
                        None => None,
                    };

                    let result = api_impl
                        .get_lcs_bca_data(
                            param_supi,
                            param_supported_features,
                            param_plmn_id,
                            param_if_none_match,
                            param_if_modified_since,
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
                            GetLcsBcaDataResponse::ExpectedResponseToAValidRequest {
                                body,
                                cache_control,
                                e_tag,
                                last_modified,
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
                                if let Some(last_modified) = last_modified {
                                    let last_modified = match header::IntoHeaderValue(last_modified).try_into() {
                                                        Ok(val) => val,
                                                        Err(e) => {
                                                            return Ok(Response::builder()
                                                                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                                                                    .body(Body::from(format!("An internal server error occurred handling last_modified header - {}", e)))
                                                                    .expect("Unable to create Internal Server Error for invalid response header"))
                                                        }
                                                    };

                                    response.headers_mut().insert(
                                        HeaderName::from_static("last-modified"),
                                        last_modified,
                                    );
                                }
                                *response.status_mut() = StatusCode::from_u16(200)
                                    .expect("Unable to turn 200 into a StatusCode");
                                response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for GET_LCS_BCA_DATA_EXPECTED_RESPONSE_TO_A_VALID_REQUEST"));
                                let body = serde_json::to_string(&body)
                                    .expect("impossible to fail to serialize");
                                *response.body_mut() = Body::from(body);
                            }
                            GetLcsBcaDataResponse::BadRequest(body) => {
                                *response.status_mut() = StatusCode::from_u16(400)
                                    .expect("Unable to turn 400 into a StatusCode");
                                response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for GET_LCS_BCA_DATA_BAD_REQUEST"));
                                let body = serde_json::to_string(&body)
                                    .expect("impossible to fail to serialize");
                                *response.body_mut() = Body::from(body);
                            }
                            GetLcsBcaDataResponse::NotFound(body) => {
                                *response.status_mut() = StatusCode::from_u16(404)
                                    .expect("Unable to turn 404 into a StatusCode");
                                response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for GET_LCS_BCA_DATA_NOT_FOUND"));
                                let body = serde_json::to_string(&body)
                                    .expect("impossible to fail to serialize");
                                *response.body_mut() = Body::from(body);
                            }
                            GetLcsBcaDataResponse::InternalServerError(body) => {
                                *response.status_mut() = StatusCode::from_u16(500)
                                    .expect("Unable to turn 500 into a StatusCode");
                                response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for GET_LCS_BCA_DATA_INTERNAL_SERVER_ERROR"));
                                let body = serde_json::to_string(&body)
                                    .expect("impossible to fail to serialize");
                                *response.body_mut() = Body::from(body);
                            }
                            GetLcsBcaDataResponse::ServiceUnavailable(body) => {
                                *response.status_mut() = StatusCode::from_u16(503)
                                    .expect("Unable to turn 503 into a StatusCode");
                                response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for GET_LCS_BCA_DATA_SERVICE_UNAVAILABLE"));
                                let body = serde_json::to_string(&body)
                                    .expect("impossible to fail to serialize");
                                *response.body_mut() = Body::from(body);
                            }
                            GetLcsBcaDataResponse::UnexpectedError => {
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

                // GetLcsMoData - GET /{supi}/lcs-mo-data
                hyper::Method::GET if path.matched(paths::ID_SUPI_LCS_MO_DATA) => {
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
                                "nudm-sdm".to_string(), // Access to the nudm-sdm API
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
                    paths::REGEX_SUPI_LCS_MO_DATA
                    .captures(path)
                    .unwrap_or_else(||
                        panic!("Path {} matched RE SUPI_LCS_MO_DATA in set but failed match against \"{}\"", path, paths::REGEX_SUPI_LCS_MO_DATA.as_str())
                    );

                    let param_supi = match percent_encoding::percent_decode(path_params["supi"].as_bytes()).decode_utf8() {
                    Ok(param_supi) => match param_supi.parse::<String>() {
                        Ok(param_supi) => param_supi,
                        Err(e) => return Ok(Response::builder()
                                        .status(StatusCode::BAD_REQUEST)
                                        .body(Body::from(format!("Couldn't parse path parameter supi: {}", e)))
                                        .expect("Unable to create Bad Request response for invalid path parameter")),
                    },
                    Err(_) => return Ok(Response::builder()
                                        .status(StatusCode::BAD_REQUEST)
                                        .body(Body::from(format!("Couldn't percent-decode path parameter as UTF-8: {}", &path_params["supi"])))
                                        .expect("Unable to create Bad Request response for invalid percent decode"))
                };

                    // Header parameters
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
                    let param_if_modified_since =
                        headers.get(HeaderName::from_static("if-modified-since"));

                    let param_if_modified_since = match param_if_modified_since {
                        Some(v) => {
                            match header::IntoHeaderValue::<String>::try_from((*v).clone()) {
                                Ok(result) => Some(result.0),
                                Err(err) => {
                                    return Ok(Response::builder()
                                        .status(StatusCode::BAD_REQUEST)
                                        .body(Body::from(format!("Invalid header If-Modified-Since - {}", err)))
                                        .expect("Unable to create Bad Request response for invalid header If-Modified-Since"));
                                }
                            }
                        }
                        None => None,
                    };

                    // Query parameters (note that non-required or collection query parameters will ignore garbage values, rather than causing a 400 response)
                    let query_params =
                        form_urlencoded::parse(uri.query().unwrap_or_default().as_bytes())
                            .collect::<Vec<_>>();
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

                    let result = api_impl
                        .get_lcs_mo_data(
                            param_supi,
                            param_supported_features,
                            param_if_none_match,
                            param_if_modified_since,
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
                            GetLcsMoDataResponse::ExpectedResponseToAValidRequest {
                                body,
                                cache_control,
                                e_tag,
                                last_modified,
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
                                if let Some(last_modified) = last_modified {
                                    let last_modified = match header::IntoHeaderValue(last_modified).try_into() {
                                                        Ok(val) => val,
                                                        Err(e) => {
                                                            return Ok(Response::builder()
                                                                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                                                                    .body(Body::from(format!("An internal server error occurred handling last_modified header - {}", e)))
                                                                    .expect("Unable to create Internal Server Error for invalid response header"))
                                                        }
                                                    };

                                    response.headers_mut().insert(
                                        HeaderName::from_static("last-modified"),
                                        last_modified,
                                    );
                                }
                                *response.status_mut() = StatusCode::from_u16(200)
                                    .expect("Unable to turn 200 into a StatusCode");
                                response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for GET_LCS_MO_DATA_EXPECTED_RESPONSE_TO_A_VALID_REQUEST"));
                                let body = serde_json::to_string(&body)
                                    .expect("impossible to fail to serialize");
                                *response.body_mut() = Body::from(body);
                            }
                            GetLcsMoDataResponse::BadRequest(body) => {
                                *response.status_mut() = StatusCode::from_u16(400)
                                    .expect("Unable to turn 400 into a StatusCode");
                                response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for GET_LCS_MO_DATA_BAD_REQUEST"));
                                let body = serde_json::to_string(&body)
                                    .expect("impossible to fail to serialize");
                                *response.body_mut() = Body::from(body);
                            }
                            GetLcsMoDataResponse::NotFound(body) => {
                                *response.status_mut() = StatusCode::from_u16(404)
                                    .expect("Unable to turn 404 into a StatusCode");
                                response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for GET_LCS_MO_DATA_NOT_FOUND"));
                                let body = serde_json::to_string(&body)
                                    .expect("impossible to fail to serialize");
                                *response.body_mut() = Body::from(body);
                            }
                            GetLcsMoDataResponse::InternalServerError(body) => {
                                *response.status_mut() = StatusCode::from_u16(500)
                                    .expect("Unable to turn 500 into a StatusCode");
                                response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for GET_LCS_MO_DATA_INTERNAL_SERVER_ERROR"));
                                let body = serde_json::to_string(&body)
                                    .expect("impossible to fail to serialize");
                                *response.body_mut() = Body::from(body);
                            }
                            GetLcsMoDataResponse::ServiceUnavailable(body) => {
                                *response.status_mut() = StatusCode::from_u16(503)
                                    .expect("Unable to turn 503 into a StatusCode");
                                response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for GET_LCS_MO_DATA_SERVICE_UNAVAILABLE"));
                                let body = serde_json::to_string(&body)
                                    .expect("impossible to fail to serialize");
                                *response.body_mut() = Body::from(body);
                            }
                            GetLcsMoDataResponse::UnexpectedError => {
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

                // GetLcsPrivacyData - GET /{ueId}/lcs-privacy-data
                hyper::Method::GET if path.matched(paths::ID_UEID_LCS_PRIVACY_DATA) => {
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
                                "nudm-sdm".to_string(), // Access to the nudm-sdm API
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
                    paths::REGEX_UEID_LCS_PRIVACY_DATA
                    .captures(path)
                    .unwrap_or_else(||
                        panic!("Path {} matched RE UEID_LCS_PRIVACY_DATA in set but failed match against \"{}\"", path, paths::REGEX_UEID_LCS_PRIVACY_DATA.as_str())
                    );

                    let param_ue_id = match percent_encoding::percent_decode(path_params["ueId"].as_bytes()).decode_utf8() {
                    Ok(param_ue_id) => match param_ue_id.parse::<String>() {
                        Ok(param_ue_id) => param_ue_id,
                        Err(e) => return Ok(Response::builder()
                                        .status(StatusCode::BAD_REQUEST)
                                        .body(Body::from(format!("Couldn't parse path parameter ueId: {}", e)))
                                        .expect("Unable to create Bad Request response for invalid path parameter")),
                    },
                    Err(_) => return Ok(Response::builder()
                                        .status(StatusCode::BAD_REQUEST)
                                        .body(Body::from(format!("Couldn't percent-decode path parameter as UTF-8: {}", &path_params["ueId"])))
                                        .expect("Unable to create Bad Request response for invalid percent decode"))
                };

                    // Header parameters
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
                    let param_if_modified_since =
                        headers.get(HeaderName::from_static("if-modified-since"));

                    let param_if_modified_since = match param_if_modified_since {
                        Some(v) => {
                            match header::IntoHeaderValue::<String>::try_from((*v).clone()) {
                                Ok(result) => Some(result.0),
                                Err(err) => {
                                    return Ok(Response::builder()
                                        .status(StatusCode::BAD_REQUEST)
                                        .body(Body::from(format!("Invalid header If-Modified-Since - {}", err)))
                                        .expect("Unable to create Bad Request response for invalid header If-Modified-Since"));
                                }
                            }
                        }
                        None => None,
                    };

                    // Query parameters (note that non-required or collection query parameters will ignore garbage values, rather than causing a 400 response)
                    let query_params =
                        form_urlencoded::parse(uri.query().unwrap_or_default().as_bytes())
                            .collect::<Vec<_>>();
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

                    let result = api_impl
                        .get_lcs_privacy_data(
                            param_ue_id,
                            param_supported_features,
                            param_if_none_match,
                            param_if_modified_since,
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
                            GetLcsPrivacyDataResponse::ExpectedResponseToAValidRequest {
                                body,
                                cache_control,
                                e_tag,
                                last_modified,
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
                                if let Some(last_modified) = last_modified {
                                    let last_modified = match header::IntoHeaderValue(last_modified).try_into() {
                                                        Ok(val) => val,
                                                        Err(e) => {
                                                            return Ok(Response::builder()
                                                                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                                                                    .body(Body::from(format!("An internal server error occurred handling last_modified header - {}", e)))
                                                                    .expect("Unable to create Internal Server Error for invalid response header"))
                                                        }
                                                    };

                                    response.headers_mut().insert(
                                        HeaderName::from_static("last-modified"),
                                        last_modified,
                                    );
                                }
                                *response.status_mut() = StatusCode::from_u16(200)
                                    .expect("Unable to turn 200 into a StatusCode");
                                response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for GET_LCS_PRIVACY_DATA_EXPECTED_RESPONSE_TO_A_VALID_REQUEST"));
                                let body = serde_json::to_string(&body)
                                    .expect("impossible to fail to serialize");
                                *response.body_mut() = Body::from(body);
                            }
                            GetLcsPrivacyDataResponse::BadRequest(body) => {
                                *response.status_mut() = StatusCode::from_u16(400)
                                    .expect("Unable to turn 400 into a StatusCode");
                                response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for GET_LCS_PRIVACY_DATA_BAD_REQUEST"));
                                let body = serde_json::to_string(&body)
                                    .expect("impossible to fail to serialize");
                                *response.body_mut() = Body::from(body);
                            }
                            GetLcsPrivacyDataResponse::NotFound(body) => {
                                *response.status_mut() = StatusCode::from_u16(404)
                                    .expect("Unable to turn 404 into a StatusCode");
                                response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for GET_LCS_PRIVACY_DATA_NOT_FOUND"));
                                let body = serde_json::to_string(&body)
                                    .expect("impossible to fail to serialize");
                                *response.body_mut() = Body::from(body);
                            }
                            GetLcsPrivacyDataResponse::InternalServerError(body) => {
                                *response.status_mut() = StatusCode::from_u16(500)
                                    .expect("Unable to turn 500 into a StatusCode");
                                response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for GET_LCS_PRIVACY_DATA_INTERNAL_SERVER_ERROR"));
                                let body = serde_json::to_string(&body)
                                    .expect("impossible to fail to serialize");
                                *response.body_mut() = Body::from(body);
                            }
                            GetLcsPrivacyDataResponse::ServiceUnavailable(body) => {
                                *response.status_mut() = StatusCode::from_u16(503)
                                    .expect("Unable to turn 503 into a StatusCode");
                                response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for GET_LCS_PRIVACY_DATA_SERVICE_UNAVAILABLE"));
                                let body = serde_json::to_string(&body)
                                    .expect("impossible to fail to serialize");
                                *response.body_mut() = Body::from(body);
                            }
                            GetLcsPrivacyDataResponse::UnexpectedError => {
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

                // GetMultipleIdentifiers - GET /multiple-identifiers
                hyper::Method::GET if path.matched(paths::ID_MULTIPLE_IDENTIFIERS) => {
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
                                "nudm-sdm".to_string(), // Access to the nudm-sdm API
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

                    // Query parameters (note that non-required or collection query parameters will ignore garbage values, rather than causing a 400 response)
                    let query_params =
                        form_urlencoded::parse(uri.query().unwrap_or_default().as_bytes())
                            .collect::<Vec<_>>();
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
                    let param_gpsi_list = query_params
                        .iter()
                        .filter(|e| e.0 == "gpsi-list")
                        .map(|e| e.1.to_owned())
                        .filter_map(|param_gpsi_list| param_gpsi_list.parse().ok())
                        .collect::<Vec<_>>();

                    let result = api_impl
                        .get_multiple_identifiers(
                            param_gpsi_list.as_ref(),
                            param_supported_features,
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
                            GetMultipleIdentifiersResponse::ExpectedResponseToAValidRequest {
                                body,
                                cache_control,
                                e_tag,
                                last_modified,
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
                                if let Some(last_modified) = last_modified {
                                    let last_modified = match header::IntoHeaderValue(last_modified).try_into() {
                                                        Ok(val) => val,
                                                        Err(e) => {
                                                            return Ok(Response::builder()
                                                                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                                                                    .body(Body::from(format!("An internal server error occurred handling last_modified header - {}", e)))
                                                                    .expect("Unable to create Internal Server Error for invalid response header"))
                                                        }
                                                    };

                                    response.headers_mut().insert(
                                        HeaderName::from_static("last-modified"),
                                        last_modified,
                                    );
                                }
                                *response.status_mut() = StatusCode::from_u16(200)
                                    .expect("Unable to turn 200 into a StatusCode");
                                response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for GET_MULTIPLE_IDENTIFIERS_EXPECTED_RESPONSE_TO_A_VALID_REQUEST"));
                                let body = serde_json::to_string(&body)
                                    .expect("impossible to fail to serialize");
                                *response.body_mut() = Body::from(body);
                            }
                            GetMultipleIdentifiersResponse::BadRequest(body) => {
                                *response.status_mut() = StatusCode::from_u16(400)
                                    .expect("Unable to turn 400 into a StatusCode");
                                response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for GET_MULTIPLE_IDENTIFIERS_BAD_REQUEST"));
                                let body = serde_json::to_string(&body)
                                    .expect("impossible to fail to serialize");
                                *response.body_mut() = Body::from(body);
                            }
                            GetMultipleIdentifiersResponse::Unauthorized(body) => {
                                *response.status_mut() = StatusCode::from_u16(401)
                                    .expect("Unable to turn 401 into a StatusCode");
                                response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for GET_MULTIPLE_IDENTIFIERS_UNAUTHORIZED"));
                                let body = serde_json::to_string(&body)
                                    .expect("impossible to fail to serialize");
                                *response.body_mut() = Body::from(body);
                            }
                            GetMultipleIdentifiersResponse::Forbidden(body) => {
                                *response.status_mut() = StatusCode::from_u16(403)
                                    .expect("Unable to turn 403 into a StatusCode");
                                response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for GET_MULTIPLE_IDENTIFIERS_FORBIDDEN"));
                                let body = serde_json::to_string(&body)
                                    .expect("impossible to fail to serialize");
                                *response.body_mut() = Body::from(body);
                            }
                            GetMultipleIdentifiersResponse::NotFound(body) => {
                                *response.status_mut() = StatusCode::from_u16(404)
                                    .expect("Unable to turn 404 into a StatusCode");
                                response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for GET_MULTIPLE_IDENTIFIERS_NOT_FOUND"));
                                let body = serde_json::to_string(&body)
                                    .expect("impossible to fail to serialize");
                                *response.body_mut() = Body::from(body);
                            }
                            GetMultipleIdentifiersResponse::Status406 => {
                                *response.status_mut() = StatusCode::from_u16(406)
                                    .expect("Unable to turn 406 into a StatusCode");
                            }
                            GetMultipleIdentifiersResponse::TooManyRequests(body) => {
                                *response.status_mut() = StatusCode::from_u16(429)
                                    .expect("Unable to turn 429 into a StatusCode");
                                response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for GET_MULTIPLE_IDENTIFIERS_TOO_MANY_REQUESTS"));
                                let body = serde_json::to_string(&body)
                                    .expect("impossible to fail to serialize");
                                *response.body_mut() = Body::from(body);
                            }
                            GetMultipleIdentifiersResponse::InternalServerError(body) => {
                                *response.status_mut() = StatusCode::from_u16(500)
                                    .expect("Unable to turn 500 into a StatusCode");
                                response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for GET_MULTIPLE_IDENTIFIERS_INTERNAL_SERVER_ERROR"));
                                let body = serde_json::to_string(&body)
                                    .expect("impossible to fail to serialize");
                                *response.body_mut() = Body::from(body);
                            }
                            GetMultipleIdentifiersResponse::BadGateway(body) => {
                                *response.status_mut() = StatusCode::from_u16(502)
                                    .expect("Unable to turn 502 into a StatusCode");
                                response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for GET_MULTIPLE_IDENTIFIERS_BAD_GATEWAY"));
                                let body = serde_json::to_string(&body)
                                    .expect("impossible to fail to serialize");
                                *response.body_mut() = Body::from(body);
                            }
                            GetMultipleIdentifiersResponse::ServiceUnavailable(body) => {
                                *response.status_mut() = StatusCode::from_u16(503)
                                    .expect("Unable to turn 503 into a StatusCode");
                                response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for GET_MULTIPLE_IDENTIFIERS_SERVICE_UNAVAILABLE"));
                                let body = serde_json::to_string(&body)
                                    .expect("impossible to fail to serialize");
                                *response.body_mut() = Body::from(body);
                            }
                            GetMultipleIdentifiersResponse::UnexpectedError => {
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

                // GetProseData - GET /{supi}/prose-data
                hyper::Method::GET if path.matched(paths::ID_SUPI_PROSE_DATA) => {
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
                                "nudm-sdm".to_string(), // Access to the nudm-sdm API
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
                    paths::REGEX_SUPI_PROSE_DATA
                    .captures(path)
                    .unwrap_or_else(||
                        panic!("Path {} matched RE SUPI_PROSE_DATA in set but failed match against \"{}\"", path, paths::REGEX_SUPI_PROSE_DATA.as_str())
                    );

                    let param_supi = match percent_encoding::percent_decode(path_params["supi"].as_bytes()).decode_utf8() {
                    Ok(param_supi) => match param_supi.parse::<String>() {
                        Ok(param_supi) => param_supi,
                        Err(e) => return Ok(Response::builder()
                                        .status(StatusCode::BAD_REQUEST)
                                        .body(Body::from(format!("Couldn't parse path parameter supi: {}", e)))
                                        .expect("Unable to create Bad Request response for invalid path parameter")),
                    },
                    Err(_) => return Ok(Response::builder()
                                        .status(StatusCode::BAD_REQUEST)
                                        .body(Body::from(format!("Couldn't percent-decode path parameter as UTF-8: {}", &path_params["supi"])))
                                        .expect("Unable to create Bad Request response for invalid percent decode"))
                };

                    // Header parameters
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
                    let param_if_modified_since =
                        headers.get(HeaderName::from_static("if-modified-since"));

                    let param_if_modified_since = match param_if_modified_since {
                        Some(v) => {
                            match header::IntoHeaderValue::<String>::try_from((*v).clone()) {
                                Ok(result) => Some(result.0),
                                Err(err) => {
                                    return Ok(Response::builder()
                                        .status(StatusCode::BAD_REQUEST)
                                        .body(Body::from(format!("Invalid header If-Modified-Since - {}", err)))
                                        .expect("Unable to create Bad Request response for invalid header If-Modified-Since"));
                                }
                            }
                        }
                        None => None,
                    };

                    // Query parameters (note that non-required or collection query parameters will ignore garbage values, rather than causing a 400 response)
                    let query_params =
                        form_urlencoded::parse(uri.query().unwrap_or_default().as_bytes())
                            .collect::<Vec<_>>();
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

                    let result = api_impl
                        .get_prose_data(
                            param_supi,
                            param_supported_features,
                            param_if_none_match,
                            param_if_modified_since,
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
                            GetProseDataResponse::ExpectedResponseToAValidRequest {
                                body,
                                cache_control,
                                e_tag,
                                last_modified,
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
                                if let Some(last_modified) = last_modified {
                                    let last_modified = match header::IntoHeaderValue(last_modified).try_into() {
                                                        Ok(val) => val,
                                                        Err(e) => {
                                                            return Ok(Response::builder()
                                                                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                                                                    .body(Body::from(format!("An internal server error occurred handling last_modified header - {}", e)))
                                                                    .expect("Unable to create Internal Server Error for invalid response header"))
                                                        }
                                                    };

                                    response.headers_mut().insert(
                                        HeaderName::from_static("last-modified"),
                                        last_modified,
                                    );
                                }
                                *response.status_mut() = StatusCode::from_u16(200)
                                    .expect("Unable to turn 200 into a StatusCode");
                                response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for GET_PROSE_DATA_EXPECTED_RESPONSE_TO_A_VALID_REQUEST"));
                                let body = serde_json::to_string(&body)
                                    .expect("impossible to fail to serialize");
                                *response.body_mut() = Body::from(body);
                            }
                            GetProseDataResponse::BadRequest(body) => {
                                *response.status_mut() = StatusCode::from_u16(400)
                                    .expect("Unable to turn 400 into a StatusCode");
                                response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for GET_PROSE_DATA_BAD_REQUEST"));
                                let body = serde_json::to_string(&body)
                                    .expect("impossible to fail to serialize");
                                *response.body_mut() = Body::from(body);
                            }
                            GetProseDataResponse::NotFound(body) => {
                                *response.status_mut() = StatusCode::from_u16(404)
                                    .expect("Unable to turn 404 into a StatusCode");
                                response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for GET_PROSE_DATA_NOT_FOUND"));
                                let body = serde_json::to_string(&body)
                                    .expect("impossible to fail to serialize");
                                *response.body_mut() = Body::from(body);
                            }
                            GetProseDataResponse::InternalServerError(body) => {
                                *response.status_mut() = StatusCode::from_u16(500)
                                    .expect("Unable to turn 500 into a StatusCode");
                                response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for GET_PROSE_DATA_INTERNAL_SERVER_ERROR"));
                                let body = serde_json::to_string(&body)
                                    .expect("impossible to fail to serialize");
                                *response.body_mut() = Body::from(body);
                            }
                            GetProseDataResponse::ServiceUnavailable(body) => {
                                *response.status_mut() = StatusCode::from_u16(503)
                                    .expect("Unable to turn 503 into a StatusCode");
                                response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for GET_PROSE_DATA_SERVICE_UNAVAILABLE"));
                                let body = serde_json::to_string(&body)
                                    .expect("impossible to fail to serialize");
                                *response.body_mut() = Body::from(body);
                            }
                            GetProseDataResponse::UnexpectedError => {
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

                // CAgAck - PUT /{supi}/am-data/cag-ack
                hyper::Method::PUT if path.matched(paths::ID_SUPI_AM_DATA_CAG_ACK) => {
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
                                "nudm-sdm".to_string(), // Access to the nudm-sdm API
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
                    paths::REGEX_SUPI_AM_DATA_CAG_ACK
                    .captures(path)
                    .unwrap_or_else(||
                        panic!("Path {} matched RE SUPI_AM_DATA_CAG_ACK in set but failed match against \"{}\"", path, paths::REGEX_SUPI_AM_DATA_CAG_ACK.as_str())
                    );

                    let param_supi = match percent_encoding::percent_decode(path_params["supi"].as_bytes()).decode_utf8() {
                    Ok(param_supi) => match param_supi.parse::<String>() {
                        Ok(param_supi) => param_supi,
                        Err(e) => return Ok(Response::builder()
                                        .status(StatusCode::BAD_REQUEST)
                                        .body(Body::from(format!("Couldn't parse path parameter supi: {}", e)))
                                        .expect("Unable to create Bad Request response for invalid path parameter")),
                    },
                    Err(_) => return Ok(Response::builder()
                                        .status(StatusCode::BAD_REQUEST)
                                        .body(Body::from(format!("Couldn't percent-decode path parameter as UTF-8: {}", &path_params["supi"])))
                                        .expect("Unable to create Bad Request response for invalid percent decode"))
                };

                    // Body parameters (note that non-required body parameters will ignore garbage
                    // values, rather than causing a 400 response). Produce warning header and logs for
                    // any unused fields.
                    let result = body.into_raw().await;
                    match result {
                            Ok(body) => {
                                let mut unused_elements = Vec::new();
                                let param_acknowledge_info: Option<models::AcknowledgeInfo> = if !body.is_empty() {
                                    let deserializer = &mut serde_json::Deserializer::from_slice(&*body);
                                    match serde_ignored::deserialize(deserializer, |path| {
                                            warn!("Ignoring unknown field in body: {}", path);
                                            unused_elements.push(path.to_string());
                                    }) {
                                        Ok(param_acknowledge_info) => param_acknowledge_info,
                                        Err(_) => None,
                                    }
                                } else {
                                    None
                                };

                                let result = api_impl.cag_ack(
                                            param_supi,
                                            param_acknowledge_info,
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
                                                CAgAckResponse::SuccessfulAcknowledgement
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(204).expect("Unable to turn 204 into a StatusCode");
                                                },
                                                CAgAckResponse::BadRequest
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(400).expect("Unable to turn 400 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for CAG_ACK_BAD_REQUEST"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                CAgAckResponse::InternalServerError
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(500).expect("Unable to turn 500 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for CAG_ACK_INTERNAL_SERVER_ERROR"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                CAgAckResponse::ServiceUnavailable
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(503).expect("Unable to turn 503 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for CAG_ACK_SERVICE_UNAVAILABLE"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                CAgAckResponse::UnexpectedError
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
                                                .body(Body::from(format!("Couldn't read body parameter AcknowledgeInfo: {}", e)))
                                                .expect("Unable to create Bad Request response due to unable to read body parameter AcknowledgeInfo")),
                        }
                }

                // SNssaisAck - PUT /{supi}/am-data/subscribed-snssais-ack
                hyper::Method::PUT
                    if path.matched(paths::ID_SUPI_AM_DATA_SUBSCRIBED_SNSSAIS_ACK) =>
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
                                "nudm-sdm".to_string(), // Access to the nudm-sdm API
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
                    paths::REGEX_SUPI_AM_DATA_SUBSCRIBED_SNSSAIS_ACK
                    .captures(path)
                    .unwrap_or_else(||
                        panic!("Path {} matched RE SUPI_AM_DATA_SUBSCRIBED_SNSSAIS_ACK in set but failed match against \"{}\"", path, paths::REGEX_SUPI_AM_DATA_SUBSCRIBED_SNSSAIS_ACK.as_str())
                    );

                    let param_supi = match percent_encoding::percent_decode(path_params["supi"].as_bytes()).decode_utf8() {
                    Ok(param_supi) => match param_supi.parse::<String>() {
                        Ok(param_supi) => param_supi,
                        Err(e) => return Ok(Response::builder()
                                        .status(StatusCode::BAD_REQUEST)
                                        .body(Body::from(format!("Couldn't parse path parameter supi: {}", e)))
                                        .expect("Unable to create Bad Request response for invalid path parameter")),
                    },
                    Err(_) => return Ok(Response::builder()
                                        .status(StatusCode::BAD_REQUEST)
                                        .body(Body::from(format!("Couldn't percent-decode path parameter as UTF-8: {}", &path_params["supi"])))
                                        .expect("Unable to create Bad Request response for invalid percent decode"))
                };

                    // Body parameters (note that non-required body parameters will ignore garbage
                    // values, rather than causing a 400 response). Produce warning header and logs for
                    // any unused fields.
                    let result = body.into_raw().await;
                    match result {
                            Ok(body) => {
                                let mut unused_elements = Vec::new();
                                let param_acknowledge_info: Option<models::AcknowledgeInfo> = if !body.is_empty() {
                                    let deserializer = &mut serde_json::Deserializer::from_slice(&*body);
                                    match serde_ignored::deserialize(deserializer, |path| {
                                            warn!("Ignoring unknown field in body: {}", path);
                                            unused_elements.push(path.to_string());
                                    }) {
                                        Ok(param_acknowledge_info) => param_acknowledge_info,
                                        Err(_) => None,
                                    }
                                } else {
                                    None
                                };

                                let result = api_impl.s_nssais_ack(
                                            param_supi,
                                            param_acknowledge_info,
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
                                                SNssaisAckResponse::SuccessfulAcknowledgement
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(204).expect("Unable to turn 204 into a StatusCode");
                                                },
                                                SNssaisAckResponse::BadRequest
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(400).expect("Unable to turn 400 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for S_NSSAIS_ACK_BAD_REQUEST"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                SNssaisAckResponse::InternalServerError
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(500).expect("Unable to turn 500 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for S_NSSAIS_ACK_INTERNAL_SERVER_ERROR"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                SNssaisAckResponse::ServiceUnavailable
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(503).expect("Unable to turn 503 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for S_NSSAIS_ACK_SERVICE_UNAVAILABLE"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                SNssaisAckResponse::UnexpectedError
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
                                                .body(Body::from(format!("Couldn't read body parameter AcknowledgeInfo: {}", e)))
                                                .expect("Unable to create Bad Request response due to unable to read body parameter AcknowledgeInfo")),
                        }
                }

                // SorAckInfo - PUT /{supi}/am-data/sor-ack
                hyper::Method::PUT if path.matched(paths::ID_SUPI_AM_DATA_SOR_ACK) => {
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
                                "nudm-sdm".to_string(), // Access to the nudm-sdm API
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
                    paths::REGEX_SUPI_AM_DATA_SOR_ACK
                    .captures(path)
                    .unwrap_or_else(||
                        panic!("Path {} matched RE SUPI_AM_DATA_SOR_ACK in set but failed match against \"{}\"", path, paths::REGEX_SUPI_AM_DATA_SOR_ACK.as_str())
                    );

                    let param_supi = match percent_encoding::percent_decode(path_params["supi"].as_bytes()).decode_utf8() {
                    Ok(param_supi) => match param_supi.parse::<String>() {
                        Ok(param_supi) => param_supi,
                        Err(e) => return Ok(Response::builder()
                                        .status(StatusCode::BAD_REQUEST)
                                        .body(Body::from(format!("Couldn't parse path parameter supi: {}", e)))
                                        .expect("Unable to create Bad Request response for invalid path parameter")),
                    },
                    Err(_) => return Ok(Response::builder()
                                        .status(StatusCode::BAD_REQUEST)
                                        .body(Body::from(format!("Couldn't percent-decode path parameter as UTF-8: {}", &path_params["supi"])))
                                        .expect("Unable to create Bad Request response for invalid percent decode"))
                };

                    // Body parameters (note that non-required body parameters will ignore garbage
                    // values, rather than causing a 400 response). Produce warning header and logs for
                    // any unused fields.
                    let result = body.into_raw().await;
                    match result {
                            Ok(body) => {
                                let mut unused_elements = Vec::new();
                                let param_acknowledge_info: Option<models::AcknowledgeInfo> = if !body.is_empty() {
                                    let deserializer = &mut serde_json::Deserializer::from_slice(&*body);
                                    match serde_ignored::deserialize(deserializer, |path| {
                                            warn!("Ignoring unknown field in body: {}", path);
                                            unused_elements.push(path.to_string());
                                    }) {
                                        Ok(param_acknowledge_info) => param_acknowledge_info,
                                        Err(_) => None,
                                    }
                                } else {
                                    None
                                };

                                let result = api_impl.sor_ack_info(
                                            param_supi,
                                            param_acknowledge_info,
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
                                                SorAckInfoResponse::SuccessfulAcknowledgement
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(204).expect("Unable to turn 204 into a StatusCode");
                                                },
                                                SorAckInfoResponse::BadRequest
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(400).expect("Unable to turn 400 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for SOR_ACK_INFO_BAD_REQUEST"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                SorAckInfoResponse::InternalServerError
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(500).expect("Unable to turn 500 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for SOR_ACK_INFO_INTERNAL_SERVER_ERROR"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                SorAckInfoResponse::ServiceUnavailable
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(503).expect("Unable to turn 503 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for SOR_ACK_INFO_SERVICE_UNAVAILABLE"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                SorAckInfoResponse::UnexpectedError
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
                                                .body(Body::from(format!("Couldn't read body parameter AcknowledgeInfo: {}", e)))
                                                .expect("Unable to create Bad Request response due to unable to read body parameter AcknowledgeInfo")),
                        }
                }

                // UpuAck - PUT /{supi}/am-data/upu-ack
                hyper::Method::PUT if path.matched(paths::ID_SUPI_AM_DATA_UPU_ACK) => {
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
                                "nudm-sdm".to_string(), // Access to the nudm-sdm API
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
                    paths::REGEX_SUPI_AM_DATA_UPU_ACK
                    .captures(path)
                    .unwrap_or_else(||
                        panic!("Path {} matched RE SUPI_AM_DATA_UPU_ACK in set but failed match against \"{}\"", path, paths::REGEX_SUPI_AM_DATA_UPU_ACK.as_str())
                    );

                    let param_supi = match percent_encoding::percent_decode(path_params["supi"].as_bytes()).decode_utf8() {
                    Ok(param_supi) => match param_supi.parse::<String>() {
                        Ok(param_supi) => param_supi,
                        Err(e) => return Ok(Response::builder()
                                        .status(StatusCode::BAD_REQUEST)
                                        .body(Body::from(format!("Couldn't parse path parameter supi: {}", e)))
                                        .expect("Unable to create Bad Request response for invalid path parameter")),
                    },
                    Err(_) => return Ok(Response::builder()
                                        .status(StatusCode::BAD_REQUEST)
                                        .body(Body::from(format!("Couldn't percent-decode path parameter as UTF-8: {}", &path_params["supi"])))
                                        .expect("Unable to create Bad Request response for invalid percent decode"))
                };

                    // Body parameters (note that non-required body parameters will ignore garbage
                    // values, rather than causing a 400 response). Produce warning header and logs for
                    // any unused fields.
                    let result = body.into_raw().await;
                    match result {
                            Ok(body) => {
                                let mut unused_elements = Vec::new();
                                let param_acknowledge_info: Option<models::AcknowledgeInfo> = if !body.is_empty() {
                                    let deserializer = &mut serde_json::Deserializer::from_slice(&*body);
                                    match serde_ignored::deserialize(deserializer, |path| {
                                            warn!("Ignoring unknown field in body: {}", path);
                                            unused_elements.push(path.to_string());
                                    }) {
                                        Ok(param_acknowledge_info) => param_acknowledge_info,
                                        Err(_) => None,
                                    }
                                } else {
                                    None
                                };

                                let result = api_impl.upu_ack(
                                            param_supi,
                                            param_acknowledge_info,
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
                                                UpuAckResponse::SuccessfulAcknowledgement
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(204).expect("Unable to turn 204 into a StatusCode");
                                                },
                                                UpuAckResponse::BadRequest
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(400).expect("Unable to turn 400 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for UPU_ACK_BAD_REQUEST"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                UpuAckResponse::InternalServerError
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(500).expect("Unable to turn 500 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for UPU_ACK_INTERNAL_SERVER_ERROR"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                UpuAckResponse::ServiceUnavailable
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(503).expect("Unable to turn 503 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for UPU_ACK_SERVICE_UNAVAILABLE"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                UpuAckResponse::UnexpectedError
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
                                                .body(Body::from(format!("Couldn't read body parameter AcknowledgeInfo: {}", e)))
                                                .expect("Unable to create Bad Request response due to unable to read body parameter AcknowledgeInfo")),
                        }
                }

                // GetDataSets - GET /{supi}
                hyper::Method::GET if path.matched(paths::ID_SUPI) => {
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
                                "nudm-sdm".to_string(), // Access to the nudm-sdm API
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
                    let path_params = paths::REGEX_SUPI.captures(path).unwrap_or_else(|| {
                        panic!(
                            "Path {} matched RE SUPI in set but failed match against \"{}\"",
                            path,
                            paths::REGEX_SUPI.as_str()
                        )
                    });

                    let param_supi = match percent_encoding::percent_decode(path_params["supi"].as_bytes()).decode_utf8() {
                    Ok(param_supi) => match param_supi.parse::<String>() {
                        Ok(param_supi) => param_supi,
                        Err(e) => return Ok(Response::builder()
                                        .status(StatusCode::BAD_REQUEST)
                                        .body(Body::from(format!("Couldn't parse path parameter supi: {}", e)))
                                        .expect("Unable to create Bad Request response for invalid path parameter")),
                    },
                    Err(_) => return Ok(Response::builder()
                                        .status(StatusCode::BAD_REQUEST)
                                        .body(Body::from(format!("Couldn't percent-decode path parameter as UTF-8: {}", &path_params["supi"])))
                                        .expect("Unable to create Bad Request response for invalid percent decode"))
                };

                    // Header parameters
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
                    let param_if_modified_since =
                        headers.get(HeaderName::from_static("if-modified-since"));

                    let param_if_modified_since = match param_if_modified_since {
                        Some(v) => {
                            match header::IntoHeaderValue::<String>::try_from((*v).clone()) {
                                Ok(result) => Some(result.0),
                                Err(err) => {
                                    return Ok(Response::builder()
                                        .status(StatusCode::BAD_REQUEST)
                                        .body(Body::from(format!("Invalid header If-Modified-Since - {}", err)))
                                        .expect("Unable to create Bad Request response for invalid header If-Modified-Since"));
                                }
                            }
                        }
                        None => None,
                    };

                    // Query parameters (note that non-required or collection query parameters will ignore garbage values, rather than causing a 400 response)
                    let query_params =
                        form_urlencoded::parse(uri.query().unwrap_or_default().as_bytes())
                            .collect::<Vec<_>>();
                    let param_dataset_names = query_params
                        .iter()
                        .filter(|e| e.0 == "dataset-names")
                        .map(|e| e.1.to_owned())
                        .filter_map(|param_dataset_names| param_dataset_names.parse().ok())
                        .collect::<Vec<_>>();
                    let param_plmn_id = query_params
                        .iter()
                        .filter(|e| e.0 == "plmn-id")
                        .map(|e| e.1.to_owned())
                        .next();
                    let param_plmn_id = match param_plmn_id {
                        Some(param_plmn_id) => {
                            let param_plmn_id =
                                serde_json::from_str::<models::PlmnIdNid>(&param_plmn_id);
                            match param_plmn_id {
                            Ok(param_plmn_id) => Some(param_plmn_id),
                            Err(e) => return Ok(Response::builder()
                                .status(StatusCode::BAD_REQUEST)
                                .body(Body::from(format!("Couldn't parse query parameter plmn-id - doesn't match schema: {}", e)))
                                .expect("Unable to create Bad Request response for invalid query parameter plmn-id")),
                        }
                        }
                        None => None,
                    };
                    let param_disaster_roaming_ind = query_params
                        .iter()
                        .filter(|e| e.0 == "disaster-roaming-ind")
                        .map(|e| e.1.to_owned())
                        .next();
                    let param_disaster_roaming_ind = match param_disaster_roaming_ind {
                        Some(param_disaster_roaming_ind) => {
                            let param_disaster_roaming_ind =
                                <bool as std::str::FromStr>::from_str(&param_disaster_roaming_ind);
                            match param_disaster_roaming_ind {
                            Ok(param_disaster_roaming_ind) => Some(param_disaster_roaming_ind),
                            Err(e) => return Ok(Response::builder()
                                .status(StatusCode::BAD_REQUEST)
                                .body(Body::from(format!("Couldn't parse query parameter disaster-roaming-ind - doesn't match schema: {}", e)))
                                .expect("Unable to create Bad Request response for invalid query parameter disaster-roaming-ind")),
                        }
                        }
                        None => None,
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

                    let result = api_impl
                        .get_data_sets(
                            param_supi,
                            param_dataset_names.as_ref(),
                            param_plmn_id,
                            param_disaster_roaming_ind,
                            param_supported_features,
                            param_if_none_match,
                            param_if_modified_since,
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
                            GetDataSetsResponse::ExpectedResponseToAValidRequest {
                                body,
                                cache_control,
                                e_tag,
                                last_modified,
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
                                if let Some(last_modified) = last_modified {
                                    let last_modified = match header::IntoHeaderValue(last_modified).try_into() {
                                                        Ok(val) => val,
                                                        Err(e) => {
                                                            return Ok(Response::builder()
                                                                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                                                                    .body(Body::from(format!("An internal server error occurred handling last_modified header - {}", e)))
                                                                    .expect("Unable to create Internal Server Error for invalid response header"))
                                                        }
                                                    };

                                    response.headers_mut().insert(
                                        HeaderName::from_static("last-modified"),
                                        last_modified,
                                    );
                                }
                                *response.status_mut() = StatusCode::from_u16(200)
                                    .expect("Unable to turn 200 into a StatusCode");
                                response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for GET_DATA_SETS_EXPECTED_RESPONSE_TO_A_VALID_REQUEST"));
                                let body = serde_json::to_string(&body)
                                    .expect("impossible to fail to serialize");
                                *response.body_mut() = Body::from(body);
                            }
                            GetDataSetsResponse::BadRequest(body) => {
                                *response.status_mut() = StatusCode::from_u16(400)
                                    .expect("Unable to turn 400 into a StatusCode");
                                response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for GET_DATA_SETS_BAD_REQUEST"));
                                let body = serde_json::to_string(&body)
                                    .expect("impossible to fail to serialize");
                                *response.body_mut() = Body::from(body);
                            }
                            GetDataSetsResponse::NotFound(body) => {
                                *response.status_mut() = StatusCode::from_u16(404)
                                    .expect("Unable to turn 404 into a StatusCode");
                                response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for GET_DATA_SETS_NOT_FOUND"));
                                let body = serde_json::to_string(&body)
                                    .expect("impossible to fail to serialize");
                                *response.body_mut() = Body::from(body);
                            }
                            GetDataSetsResponse::InternalServerError(body) => {
                                *response.status_mut() = StatusCode::from_u16(500)
                                    .expect("Unable to turn 500 into a StatusCode");
                                response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for GET_DATA_SETS_INTERNAL_SERVER_ERROR"));
                                let body = serde_json::to_string(&body)
                                    .expect("impossible to fail to serialize");
                                *response.body_mut() = Body::from(body);
                            }
                            GetDataSetsResponse::ServiceUnavailable(body) => {
                                *response.status_mut() = StatusCode::from_u16(503)
                                    .expect("Unable to turn 503 into a StatusCode");
                                response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for GET_DATA_SETS_SERVICE_UNAVAILABLE"));
                                let body = serde_json::to_string(&body)
                                    .expect("impossible to fail to serialize");
                                *response.body_mut() = Body::from(body);
                            }
                            GetDataSetsResponse::UnexpectedError => {
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

                // GetSharedData - GET /shared-data
                hyper::Method::GET if path.matched(paths::ID_SHARED_DATA) => {
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
                                "nudm-sdm".to_string(), // Access to the nudm-sdm API
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
                    let param_if_modified_since =
                        headers.get(HeaderName::from_static("if-modified-since"));

                    let param_if_modified_since = match param_if_modified_since {
                        Some(v) => {
                            match header::IntoHeaderValue::<String>::try_from((*v).clone()) {
                                Ok(result) => Some(result.0),
                                Err(err) => {
                                    return Ok(Response::builder()
                                        .status(StatusCode::BAD_REQUEST)
                                        .body(Body::from(format!("Invalid header If-Modified-Since - {}", err)))
                                        .expect("Unable to create Bad Request response for invalid header If-Modified-Since"));
                                }
                            }
                        }
                        None => None,
                    };

                    // Query parameters (note that non-required or collection query parameters will ignore garbage values, rather than causing a 400 response)
                    let query_params =
                        form_urlencoded::parse(uri.query().unwrap_or_default().as_bytes())
                            .collect::<Vec<_>>();
                    let param_shared_data_ids = query_params
                        .iter()
                        .filter(|e| e.0 == "shared-data-ids")
                        .map(|e| e.1.to_owned())
                        .filter_map(|param_shared_data_ids| param_shared_data_ids.parse().ok())
                        .collect::<Vec<_>>();
                    let param_supported_features = query_params
                        .iter()
                        .filter(|e| e.0 == "supportedFeatures")
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
                                .body(Body::from(format!("Couldn't parse query parameter supportedFeatures - doesn't match schema: {}", e)))
                                .expect("Unable to create Bad Request response for invalid query parameter supportedFeatures")),
                        }
                        }
                        None => None,
                    };
                    let param_supported_features2 = query_params
                        .iter()
                        .filter(|e| e.0 == "supported-features")
                        .map(|e| e.1.to_owned())
                        .next();
                    let param_supported_features2 = match param_supported_features2 {
                        Some(param_supported_features2) => {
                            let param_supported_features2 =
                                <String as std::str::FromStr>::from_str(&param_supported_features2);
                            match param_supported_features2 {
                            Ok(param_supported_features2) => Some(param_supported_features2),
                            Err(e) => return Ok(Response::builder()
                                .status(StatusCode::BAD_REQUEST)
                                .body(Body::from(format!("Couldn't parse query parameter supported-features - doesn't match schema: {}", e)))
                                .expect("Unable to create Bad Request response for invalid query parameter supported-features")),
                        }
                        }
                        None => None,
                    };

                    let result = api_impl
                        .get_shared_data(
                            param_shared_data_ids.as_ref(),
                            param_supported_features,
                            param_supported_features2,
                            param_if_none_match,
                            param_if_modified_since,
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
                            GetSharedDataResponse::ExpectedResponseToAValidRequest {
                                body,
                                cache_control,
                                e_tag,
                                last_modified,
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
                                if let Some(last_modified) = last_modified {
                                    let last_modified = match header::IntoHeaderValue(last_modified).try_into() {
                                                        Ok(val) => val,
                                                        Err(e) => {
                                                            return Ok(Response::builder()
                                                                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                                                                    .body(Body::from(format!("An internal server error occurred handling last_modified header - {}", e)))
                                                                    .expect("Unable to create Internal Server Error for invalid response header"))
                                                        }
                                                    };

                                    response.headers_mut().insert(
                                        HeaderName::from_static("last-modified"),
                                        last_modified,
                                    );
                                }
                                *response.status_mut() = StatusCode::from_u16(200)
                                    .expect("Unable to turn 200 into a StatusCode");
                                response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for GET_SHARED_DATA_EXPECTED_RESPONSE_TO_A_VALID_REQUEST"));
                                let body = serde_json::to_string(&body)
                                    .expect("impossible to fail to serialize");
                                *response.body_mut() = Body::from(body);
                            }
                            GetSharedDataResponse::BadRequest(body) => {
                                *response.status_mut() = StatusCode::from_u16(400)
                                    .expect("Unable to turn 400 into a StatusCode");
                                response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for GET_SHARED_DATA_BAD_REQUEST"));
                                let body = serde_json::to_string(&body)
                                    .expect("impossible to fail to serialize");
                                *response.body_mut() = Body::from(body);
                            }
                            GetSharedDataResponse::NotFound(body) => {
                                *response.status_mut() = StatusCode::from_u16(404)
                                    .expect("Unable to turn 404 into a StatusCode");
                                response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for GET_SHARED_DATA_NOT_FOUND"));
                                let body = serde_json::to_string(&body)
                                    .expect("impossible to fail to serialize");
                                *response.body_mut() = Body::from(body);
                            }
                            GetSharedDataResponse::InternalServerError(body) => {
                                *response.status_mut() = StatusCode::from_u16(500)
                                    .expect("Unable to turn 500 into a StatusCode");
                                response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for GET_SHARED_DATA_INTERNAL_SERVER_ERROR"));
                                let body = serde_json::to_string(&body)
                                    .expect("impossible to fail to serialize");
                                *response.body_mut() = Body::from(body);
                            }
                            GetSharedDataResponse::ServiceUnavailable(body) => {
                                *response.status_mut() = StatusCode::from_u16(503)
                                    .expect("Unable to turn 503 into a StatusCode");
                                response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for GET_SHARED_DATA_SERVICE_UNAVAILABLE"));
                                let body = serde_json::to_string(&body)
                                    .expect("impossible to fail to serialize");
                                *response.body_mut() = Body::from(body);
                            }
                            GetSharedDataResponse::UnexpectedError => {
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

                // GetIndividualSharedData - GET /shared-data/{sharedDataId}
                hyper::Method::GET if path.matched(paths::ID_SHARED_DATA_SHAREDDATAID) => {
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
                                "nudm-sdm".to_string(), // Access to the nudm-sdm API
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
                    paths::REGEX_SHARED_DATA_SHAREDDATAID
                    .captures(path)
                    .unwrap_or_else(||
                        panic!("Path {} matched RE SHARED_DATA_SHAREDDATAID in set but failed match against \"{}\"", path, paths::REGEX_SHARED_DATA_SHAREDDATAID.as_str())
                    );

                    let param_shared_data_id = match percent_encoding::percent_decode(path_params["sharedDataId"].as_bytes()).decode_utf8() {
                    Ok(param_shared_data_id) => match models::SharedDataId::from_vec_str(&param_shared_data_id) { //match param_shared_data_id.parse::<Vec<models::SharedDataId>>() {
                        Ok(param_shared_data_id) => param_shared_data_id,
                        Err(e) => return Ok(Response::builder()
                                        .status(StatusCode::BAD_REQUEST)
                                        .body(Body::from(format!("Couldn't parse path parameter sharedDataId: {}", e)))
                                        .expect("Unable to create Bad Request response for invalid path parameter")),
                    },
                    Err(_) => return Ok(Response::builder()
                                        .status(StatusCode::BAD_REQUEST)
                                        .body(Body::from(format!("Couldn't percent-decode path parameter as UTF-8: {}", &path_params["sharedDataId"])))
                                        .expect("Unable to create Bad Request response for invalid percent decode"))
                };

                    // Header parameters
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
                    let param_if_modified_since =
                        headers.get(HeaderName::from_static("if-modified-since"));

                    let param_if_modified_since = match param_if_modified_since {
                        Some(v) => {
                            match header::IntoHeaderValue::<String>::try_from((*v).clone()) {
                                Ok(result) => Some(result.0),
                                Err(err) => {
                                    return Ok(Response::builder()
                                        .status(StatusCode::BAD_REQUEST)
                                        .body(Body::from(format!("Invalid header If-Modified-Since - {}", err)))
                                        .expect("Unable to create Bad Request response for invalid header If-Modified-Since"));
                                }
                            }
                        }
                        None => None,
                    };

                    // Query parameters (note that non-required or collection query parameters will ignore garbage values, rather than causing a 400 response)
                    let query_params =
                        form_urlencoded::parse(uri.query().unwrap_or_default().as_bytes())
                            .collect::<Vec<_>>();
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

                    let result = api_impl
                        .get_individual_shared_data(
                            param_shared_data_id.as_ref(),
                            param_supported_features,
                            param_if_none_match,
                            param_if_modified_since,
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
                            GetIndividualSharedDataResponse::ExpectedResponseToAValidRequest {
                                body,
                                cache_control,
                                e_tag,
                                last_modified,
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
                                if let Some(last_modified) = last_modified {
                                    let last_modified = match header::IntoHeaderValue(last_modified).try_into() {
                                                        Ok(val) => val,
                                                        Err(e) => {
                                                            return Ok(Response::builder()
                                                                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                                                                    .body(Body::from(format!("An internal server error occurred handling last_modified header - {}", e)))
                                                                    .expect("Unable to create Internal Server Error for invalid response header"))
                                                        }
                                                    };

                                    response.headers_mut().insert(
                                        HeaderName::from_static("last-modified"),
                                        last_modified,
                                    );
                                }
                                *response.status_mut() = StatusCode::from_u16(200)
                                    .expect("Unable to turn 200 into a StatusCode");
                                response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for GET_INDIVIDUAL_SHARED_DATA_EXPECTED_RESPONSE_TO_A_VALID_REQUEST"));
                                let body = serde_json::to_string(&body)
                                    .expect("impossible to fail to serialize");
                                *response.body_mut() = Body::from(body);
                            }
                            GetIndividualSharedDataResponse::BadRequest(body) => {
                                *response.status_mut() = StatusCode::from_u16(400)
                                    .expect("Unable to turn 400 into a StatusCode");
                                response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for GET_INDIVIDUAL_SHARED_DATA_BAD_REQUEST"));
                                let body = serde_json::to_string(&body)
                                    .expect("impossible to fail to serialize");
                                *response.body_mut() = Body::from(body);
                            }
                            GetIndividualSharedDataResponse::NotFound(body) => {
                                *response.status_mut() = StatusCode::from_u16(404)
                                    .expect("Unable to turn 404 into a StatusCode");
                                response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for GET_INDIVIDUAL_SHARED_DATA_NOT_FOUND"));
                                let body = serde_json::to_string(&body)
                                    .expect("impossible to fail to serialize");
                                *response.body_mut() = Body::from(body);
                            }
                            GetIndividualSharedDataResponse::InternalServerError(body) => {
                                *response.status_mut() = StatusCode::from_u16(500)
                                    .expect("Unable to turn 500 into a StatusCode");
                                response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for GET_INDIVIDUAL_SHARED_DATA_INTERNAL_SERVER_ERROR"));
                                let body = serde_json::to_string(&body)
                                    .expect("impossible to fail to serialize");
                                *response.body_mut() = Body::from(body);
                            }
                            GetIndividualSharedDataResponse::ServiceUnavailable(body) => {
                                *response.status_mut() = StatusCode::from_u16(503)
                                    .expect("Unable to turn 503 into a StatusCode");
                                response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for GET_INDIVIDUAL_SHARED_DATA_SERVICE_UNAVAILABLE"));
                                let body = serde_json::to_string(&body)
                                    .expect("impossible to fail to serialize");
                                *response.body_mut() = Body::from(body);
                            }
                            GetIndividualSharedDataResponse::UnexpectedError => {
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

                // GetSmfSelData - GET /{supi}/smf-select-data
                hyper::Method::GET if path.matched(paths::ID_SUPI_SMF_SELECT_DATA) => {
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
                                "nudm-sdm".to_string(), // Access to the nudm-sdm API
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
                    paths::REGEX_SUPI_SMF_SELECT_DATA
                    .captures(path)
                    .unwrap_or_else(||
                        panic!("Path {} matched RE SUPI_SMF_SELECT_DATA in set but failed match against \"{}\"", path, paths::REGEX_SUPI_SMF_SELECT_DATA.as_str())
                    );

                    let param_supi = match percent_encoding::percent_decode(path_params["supi"].as_bytes()).decode_utf8() {
                    Ok(param_supi) => match param_supi.parse::<String>() {
                        Ok(param_supi) => param_supi,
                        Err(e) => return Ok(Response::builder()
                                        .status(StatusCode::BAD_REQUEST)
                                        .body(Body::from(format!("Couldn't parse path parameter supi: {}", e)))
                                        .expect("Unable to create Bad Request response for invalid path parameter")),
                    },
                    Err(_) => return Ok(Response::builder()
                                        .status(StatusCode::BAD_REQUEST)
                                        .body(Body::from(format!("Couldn't percent-decode path parameter as UTF-8: {}", &path_params["supi"])))
                                        .expect("Unable to create Bad Request response for invalid percent decode"))
                };

                    // Header parameters
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
                    let param_if_modified_since =
                        headers.get(HeaderName::from_static("if-modified-since"));

                    let param_if_modified_since = match param_if_modified_since {
                        Some(v) => {
                            match header::IntoHeaderValue::<String>::try_from((*v).clone()) {
                                Ok(result) => Some(result.0),
                                Err(err) => {
                                    return Ok(Response::builder()
                                        .status(StatusCode::BAD_REQUEST)
                                        .body(Body::from(format!("Invalid header If-Modified-Since - {}", err)))
                                        .expect("Unable to create Bad Request response for invalid header If-Modified-Since"));
                                }
                            }
                        }
                        None => None,
                    };

                    // Query parameters (note that non-required or collection query parameters will ignore garbage values, rather than causing a 400 response)
                    let query_params =
                        form_urlencoded::parse(uri.query().unwrap_or_default().as_bytes())
                            .collect::<Vec<_>>();
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
                    let param_plmn_id = query_params
                        .iter()
                        .filter(|e| e.0 == "plmn-id")
                        .map(|e| e.1.to_owned())
                        .next();
                    let param_plmn_id = match param_plmn_id {
                        Some(param_plmn_id) => {
                            let param_plmn_id =
                                serde_json::from_str::<models::PlmnId>(&param_plmn_id);
                            match param_plmn_id {
                            Ok(param_plmn_id) => Some(param_plmn_id),
                            Err(e) => return Ok(Response::builder()
                                .status(StatusCode::BAD_REQUEST)
                                .body(Body::from(format!("Couldn't parse query parameter plmn-id - doesn't match schema: {}", e)))
                                .expect("Unable to create Bad Request response for invalid query parameter plmn-id")),
                        }
                        }
                        None => None,
                    };
                    let param_disaster_roaming_ind = query_params
                        .iter()
                        .filter(|e| e.0 == "disaster-roaming-ind")
                        .map(|e| e.1.to_owned())
                        .next();
                    let param_disaster_roaming_ind = match param_disaster_roaming_ind {
                        Some(param_disaster_roaming_ind) => {
                            let param_disaster_roaming_ind =
                                <bool as std::str::FromStr>::from_str(&param_disaster_roaming_ind);
                            match param_disaster_roaming_ind {
                            Ok(param_disaster_roaming_ind) => Some(param_disaster_roaming_ind),
                            Err(e) => return Ok(Response::builder()
                                .status(StatusCode::BAD_REQUEST)
                                .body(Body::from(format!("Couldn't parse query parameter disaster-roaming-ind - doesn't match schema: {}", e)))
                                .expect("Unable to create Bad Request response for invalid query parameter disaster-roaming-ind")),
                        }
                        }
                        None => None,
                    };

                    let result = api_impl
                        .get_smf_sel_data(
                            param_supi,
                            param_supported_features,
                            param_plmn_id,
                            param_disaster_roaming_ind,
                            param_if_none_match,
                            param_if_modified_since,
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
                            GetSmfSelDataResponse::ExpectedResponseToAValidRequest {
                                body,
                                cache_control,
                                e_tag,
                                last_modified,
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
                                if let Some(last_modified) = last_modified {
                                    let last_modified = match header::IntoHeaderValue(last_modified).try_into() {
                                                        Ok(val) => val,
                                                        Err(e) => {
                                                            return Ok(Response::builder()
                                                                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                                                                    .body(Body::from(format!("An internal server error occurred handling last_modified header - {}", e)))
                                                                    .expect("Unable to create Internal Server Error for invalid response header"))
                                                        }
                                                    };

                                    response.headers_mut().insert(
                                        HeaderName::from_static("last-modified"),
                                        last_modified,
                                    );
                                }
                                *response.status_mut() = StatusCode::from_u16(200)
                                    .expect("Unable to turn 200 into a StatusCode");
                                response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for GET_SMF_SEL_DATA_EXPECTED_RESPONSE_TO_A_VALID_REQUEST"));
                                let body = serde_json::to_string(&body)
                                    .expect("impossible to fail to serialize");
                                *response.body_mut() = Body::from(body);
                            }
                            GetSmfSelDataResponse::BadRequest(body) => {
                                *response.status_mut() = StatusCode::from_u16(400)
                                    .expect("Unable to turn 400 into a StatusCode");
                                response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for GET_SMF_SEL_DATA_BAD_REQUEST"));
                                let body = serde_json::to_string(&body)
                                    .expect("impossible to fail to serialize");
                                *response.body_mut() = Body::from(body);
                            }
                            GetSmfSelDataResponse::NotFound(body) => {
                                *response.status_mut() = StatusCode::from_u16(404)
                                    .expect("Unable to turn 404 into a StatusCode");
                                response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for GET_SMF_SEL_DATA_NOT_FOUND"));
                                let body = serde_json::to_string(&body)
                                    .expect("impossible to fail to serialize");
                                *response.body_mut() = Body::from(body);
                            }
                            GetSmfSelDataResponse::InternalServerError(body) => {
                                *response.status_mut() = StatusCode::from_u16(500)
                                    .expect("Unable to turn 500 into a StatusCode");
                                response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for GET_SMF_SEL_DATA_INTERNAL_SERVER_ERROR"));
                                let body = serde_json::to_string(&body)
                                    .expect("impossible to fail to serialize");
                                *response.body_mut() = Body::from(body);
                            }
                            GetSmfSelDataResponse::ServiceUnavailable(body) => {
                                *response.status_mut() = StatusCode::from_u16(503)
                                    .expect("Unable to turn 503 into a StatusCode");
                                response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for GET_SMF_SEL_DATA_SERVICE_UNAVAILABLE"));
                                let body = serde_json::to_string(&body)
                                    .expect("impossible to fail to serialize");
                                *response.body_mut() = Body::from(body);
                            }
                            GetSmfSelDataResponse::UnexpectedError => {
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

                // GetSmsMngtData - GET /{supi}/sms-mng-data
                hyper::Method::GET if path.matched(paths::ID_SUPI_SMS_MNG_DATA) => {
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
                                "nudm-sdm".to_string(), // Access to the nudm-sdm API
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
                    paths::REGEX_SUPI_SMS_MNG_DATA
                    .captures(path)
                    .unwrap_or_else(||
                        panic!("Path {} matched RE SUPI_SMS_MNG_DATA in set but failed match against \"{}\"", path, paths::REGEX_SUPI_SMS_MNG_DATA.as_str())
                    );

                    let param_supi = match percent_encoding::percent_decode(path_params["supi"].as_bytes()).decode_utf8() {
                    Ok(param_supi) => match param_supi.parse::<String>() {
                        Ok(param_supi) => param_supi,
                        Err(e) => return Ok(Response::builder()
                                        .status(StatusCode::BAD_REQUEST)
                                        .body(Body::from(format!("Couldn't parse path parameter supi: {}", e)))
                                        .expect("Unable to create Bad Request response for invalid path parameter")),
                    },
                    Err(_) => return Ok(Response::builder()
                                        .status(StatusCode::BAD_REQUEST)
                                        .body(Body::from(format!("Couldn't percent-decode path parameter as UTF-8: {}", &path_params["supi"])))
                                        .expect("Unable to create Bad Request response for invalid percent decode"))
                };

                    // Header parameters
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
                    let param_if_modified_since =
                        headers.get(HeaderName::from_static("if-modified-since"));

                    let param_if_modified_since = match param_if_modified_since {
                        Some(v) => {
                            match header::IntoHeaderValue::<String>::try_from((*v).clone()) {
                                Ok(result) => Some(result.0),
                                Err(err) => {
                                    return Ok(Response::builder()
                                        .status(StatusCode::BAD_REQUEST)
                                        .body(Body::from(format!("Invalid header If-Modified-Since - {}", err)))
                                        .expect("Unable to create Bad Request response for invalid header If-Modified-Since"));
                                }
                            }
                        }
                        None => None,
                    };

                    // Query parameters (note that non-required or collection query parameters will ignore garbage values, rather than causing a 400 response)
                    let query_params =
                        form_urlencoded::parse(uri.query().unwrap_or_default().as_bytes())
                            .collect::<Vec<_>>();
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
                    let param_plmn_id = query_params
                        .iter()
                        .filter(|e| e.0 == "plmn-id")
                        .map(|e| e.1.to_owned())
                        .next();
                    let param_plmn_id = match param_plmn_id {
                        Some(param_plmn_id) => {
                            let param_plmn_id =
                                serde_json::from_str::<models::PlmnId>(&param_plmn_id);
                            match param_plmn_id {
                            Ok(param_plmn_id) => Some(param_plmn_id),
                            Err(e) => return Ok(Response::builder()
                                .status(StatusCode::BAD_REQUEST)
                                .body(Body::from(format!("Couldn't parse query parameter plmn-id - doesn't match schema: {}", e)))
                                .expect("Unable to create Bad Request response for invalid query parameter plmn-id")),
                        }
                        }
                        None => None,
                    };

                    let result = api_impl
                        .get_sms_mngt_data(
                            param_supi,
                            param_supported_features,
                            param_plmn_id,
                            param_if_none_match,
                            param_if_modified_since,
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
                            GetSmsMngtDataResponse::ExpectedResponseToAValidRequest {
                                body,
                                cache_control,
                                e_tag,
                                last_modified,
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
                                if let Some(last_modified) = last_modified {
                                    let last_modified = match header::IntoHeaderValue(last_modified).try_into() {
                                                        Ok(val) => val,
                                                        Err(e) => {
                                                            return Ok(Response::builder()
                                                                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                                                                    .body(Body::from(format!("An internal server error occurred handling last_modified header - {}", e)))
                                                                    .expect("Unable to create Internal Server Error for invalid response header"))
                                                        }
                                                    };

                                    response.headers_mut().insert(
                                        HeaderName::from_static("last-modified"),
                                        last_modified,
                                    );
                                }
                                *response.status_mut() = StatusCode::from_u16(200)
                                    .expect("Unable to turn 200 into a StatusCode");
                                response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for GET_SMS_MNGT_DATA_EXPECTED_RESPONSE_TO_A_VALID_REQUEST"));
                                let body = serde_json::to_string(&body)
                                    .expect("impossible to fail to serialize");
                                *response.body_mut() = Body::from(body);
                            }
                            GetSmsMngtDataResponse::BadRequest(body) => {
                                *response.status_mut() = StatusCode::from_u16(400)
                                    .expect("Unable to turn 400 into a StatusCode");
                                response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for GET_SMS_MNGT_DATA_BAD_REQUEST"));
                                let body = serde_json::to_string(&body)
                                    .expect("impossible to fail to serialize");
                                *response.body_mut() = Body::from(body);
                            }
                            GetSmsMngtDataResponse::NotFound(body) => {
                                *response.status_mut() = StatusCode::from_u16(404)
                                    .expect("Unable to turn 404 into a StatusCode");
                                response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for GET_SMS_MNGT_DATA_NOT_FOUND"));
                                let body = serde_json::to_string(&body)
                                    .expect("impossible to fail to serialize");
                                *response.body_mut() = Body::from(body);
                            }
                            GetSmsMngtDataResponse::InternalServerError(body) => {
                                *response.status_mut() = StatusCode::from_u16(500)
                                    .expect("Unable to turn 500 into a StatusCode");
                                response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for GET_SMS_MNGT_DATA_INTERNAL_SERVER_ERROR"));
                                let body = serde_json::to_string(&body)
                                    .expect("impossible to fail to serialize");
                                *response.body_mut() = Body::from(body);
                            }
                            GetSmsMngtDataResponse::ServiceUnavailable(body) => {
                                *response.status_mut() = StatusCode::from_u16(503)
                                    .expect("Unable to turn 503 into a StatusCode");
                                response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for GET_SMS_MNGT_DATA_SERVICE_UNAVAILABLE"));
                                let body = serde_json::to_string(&body)
                                    .expect("impossible to fail to serialize");
                                *response.body_mut() = Body::from(body);
                            }
                            GetSmsMngtDataResponse::UnexpectedError => {
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

                // GetSmsData - GET /{supi}/sms-data
                hyper::Method::GET if path.matched(paths::ID_SUPI_SMS_DATA) => {
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
                                "nudm-sdm".to_string(), // Access to the nudm-sdm API
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
                    paths::REGEX_SUPI_SMS_DATA
                    .captures(path)
                    .unwrap_or_else(||
                        panic!("Path {} matched RE SUPI_SMS_DATA in set but failed match against \"{}\"", path, paths::REGEX_SUPI_SMS_DATA.as_str())
                    );

                    let param_supi = match percent_encoding::percent_decode(path_params["supi"].as_bytes()).decode_utf8() {
                    Ok(param_supi) => match param_supi.parse::<String>() {
                        Ok(param_supi) => param_supi,
                        Err(e) => return Ok(Response::builder()
                                        .status(StatusCode::BAD_REQUEST)
                                        .body(Body::from(format!("Couldn't parse path parameter supi: {}", e)))
                                        .expect("Unable to create Bad Request response for invalid path parameter")),
                    },
                    Err(_) => return Ok(Response::builder()
                                        .status(StatusCode::BAD_REQUEST)
                                        .body(Body::from(format!("Couldn't percent-decode path parameter as UTF-8: {}", &path_params["supi"])))
                                        .expect("Unable to create Bad Request response for invalid percent decode"))
                };

                    // Header parameters
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
                    let param_if_modified_since =
                        headers.get(HeaderName::from_static("if-modified-since"));

                    let param_if_modified_since = match param_if_modified_since {
                        Some(v) => {
                            match header::IntoHeaderValue::<String>::try_from((*v).clone()) {
                                Ok(result) => Some(result.0),
                                Err(err) => {
                                    return Ok(Response::builder()
                                        .status(StatusCode::BAD_REQUEST)
                                        .body(Body::from(format!("Invalid header If-Modified-Since - {}", err)))
                                        .expect("Unable to create Bad Request response for invalid header If-Modified-Since"));
                                }
                            }
                        }
                        None => None,
                    };

                    // Query parameters (note that non-required or collection query parameters will ignore garbage values, rather than causing a 400 response)
                    let query_params =
                        form_urlencoded::parse(uri.query().unwrap_or_default().as_bytes())
                            .collect::<Vec<_>>();
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
                    let param_plmn_id = query_params
                        .iter()
                        .filter(|e| e.0 == "plmn-id")
                        .map(|e| e.1.to_owned())
                        .next();
                    let param_plmn_id = match param_plmn_id {
                        Some(param_plmn_id) => {
                            let param_plmn_id =
                                serde_json::from_str::<models::PlmnId>(&param_plmn_id);
                            match param_plmn_id {
                            Ok(param_plmn_id) => Some(param_plmn_id),
                            Err(e) => return Ok(Response::builder()
                                .status(StatusCode::BAD_REQUEST)
                                .body(Body::from(format!("Couldn't parse query parameter plmn-id - doesn't match schema: {}", e)))
                                .expect("Unable to create Bad Request response for invalid query parameter plmn-id")),
                        }
                        }
                        None => None,
                    };

                    let result = api_impl
                        .get_sms_data(
                            param_supi,
                            param_supported_features,
                            param_plmn_id,
                            param_if_none_match,
                            param_if_modified_since,
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
                            GetSmsDataResponse::ExpectedResponseToAValidRequest {
                                body,
                                cache_control,
                                e_tag,
                                last_modified,
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
                                if let Some(last_modified) = last_modified {
                                    let last_modified = match header::IntoHeaderValue(last_modified).try_into() {
                                                        Ok(val) => val,
                                                        Err(e) => {
                                                            return Ok(Response::builder()
                                                                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                                                                    .body(Body::from(format!("An internal server error occurred handling last_modified header - {}", e)))
                                                                    .expect("Unable to create Internal Server Error for invalid response header"))
                                                        }
                                                    };

                                    response.headers_mut().insert(
                                        HeaderName::from_static("last-modified"),
                                        last_modified,
                                    );
                                }
                                *response.status_mut() = StatusCode::from_u16(200)
                                    .expect("Unable to turn 200 into a StatusCode");
                                response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for GET_SMS_DATA_EXPECTED_RESPONSE_TO_A_VALID_REQUEST"));
                                let body = serde_json::to_string(&body)
                                    .expect("impossible to fail to serialize");
                                *response.body_mut() = Body::from(body);
                            }
                            GetSmsDataResponse::BadRequest(body) => {
                                *response.status_mut() = StatusCode::from_u16(400)
                                    .expect("Unable to turn 400 into a StatusCode");
                                response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for GET_SMS_DATA_BAD_REQUEST"));
                                let body = serde_json::to_string(&body)
                                    .expect("impossible to fail to serialize");
                                *response.body_mut() = Body::from(body);
                            }
                            GetSmsDataResponse::NotFound(body) => {
                                *response.status_mut() = StatusCode::from_u16(404)
                                    .expect("Unable to turn 404 into a StatusCode");
                                response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for GET_SMS_DATA_NOT_FOUND"));
                                let body = serde_json::to_string(&body)
                                    .expect("impossible to fail to serialize");
                                *response.body_mut() = Body::from(body);
                            }
                            GetSmsDataResponse::InternalServerError(body) => {
                                *response.status_mut() = StatusCode::from_u16(500)
                                    .expect("Unable to turn 500 into a StatusCode");
                                response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for GET_SMS_DATA_INTERNAL_SERVER_ERROR"));
                                let body = serde_json::to_string(&body)
                                    .expect("impossible to fail to serialize");
                                *response.body_mut() = Body::from(body);
                            }
                            GetSmsDataResponse::ServiceUnavailable(body) => {
                                *response.status_mut() = StatusCode::from_u16(503)
                                    .expect("Unable to turn 503 into a StatusCode");
                                response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for GET_SMS_DATA_SERVICE_UNAVAILABLE"));
                                let body = serde_json::to_string(&body)
                                    .expect("impossible to fail to serialize");
                                *response.body_mut() = Body::from(body);
                            }
                            GetSmsDataResponse::UnexpectedError => {
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

                // GetSmData - GET /{supi}/sm-data
                hyper::Method::GET if path.matched(paths::ID_SUPI_SM_DATA) => {
                    // This is the method called during PDU session establishment

                    // Request path:
                    // /nudm-sdm/v2/imsi-001011234567895/sm-data?single-nssai=%7B%0A%09%22sst%22%3A%091%0A%7D&dnn=internet

                    // Reply from open5gs
                    // [{
                    //     "singleNssai":	{
                    //         "sst":	1
                    //     },
                    //     "dnnConfigurations":	{
                    //         "internet":	{
                    //             "pduSessionTypes":	{
                    //                 "defaultSessionType":	"IPV4",
                    //                 "allowedSessionTypes":	["IPV4"]
                    //             },
                    //             "sscModes":	{
                    //                 "defaultSscMode":	"SSC_MODE_1",
                    //                 "allowedSscModes":	["SSC_MODE_1", "SSC_MODE_2", "SSC_MODE_3"]
                    //             },
                    //             "5gQosProfile":	{
                    //                 "5qi":	9,
                    //                 "arp":	{
                    //                     "priorityLevel":	8,
                    //                     "preemptCap":	"NOT_PREEMPT",
                    //                     "preemptVuln":	"NOT_PREEMPTABLE"
                    //                 },
                    //                 "priorityLevel":	8
                    //             },
                    //             "sessionAmbr":	{
                    //                 "uplink":	"1048576 Kbps",
                    //                 "downlink":	"1048576 Kbps"
                    //             }
                    //         }
                    //     }
                    // }]

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
                                "nudm-sdm".to_string(), // Access to the nudm-sdm API
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
                    paths::REGEX_SUPI_SM_DATA
                    .captures(path)
                    .unwrap_or_else(||
                        panic!("Path {} matched RE SUPI_SM_DATA in set but failed match against \"{}\"", path, paths::REGEX_SUPI_SM_DATA.as_str())
                    );

                    let param_supi = match percent_encoding::percent_decode(path_params["supi"].as_bytes()).decode_utf8() {
                    Ok(param_supi) => match param_supi.parse::<String>() {
                        Ok(param_supi) => param_supi,
                        Err(e) => return Ok(Response::builder()
                                        .status(StatusCode::BAD_REQUEST)
                                        .body(Body::from(format!("Couldn't parse path parameter supi: {}", e)))
                                        .expect("Unable to create Bad Request response for invalid path parameter")),
                    },
                    Err(_) => return Ok(Response::builder()
                                        .status(StatusCode::BAD_REQUEST)
                                        .body(Body::from(format!("Couldn't percent-decode path parameter as UTF-8: {}", &path_params["supi"])))
                                        .expect("Unable to create Bad Request response for invalid percent decode"))
                };

                    // Header parameters
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
                    let param_if_modified_since =
                        headers.get(HeaderName::from_static("if-modified-since"));

                    let param_if_modified_since = match param_if_modified_since {
                        Some(v) => {
                            match header::IntoHeaderValue::<String>::try_from((*v).clone()) {
                                Ok(result) => Some(result.0),
                                Err(err) => {
                                    return Ok(Response::builder()
                                        .status(StatusCode::BAD_REQUEST)
                                        .body(Body::from(format!("Invalid header If-Modified-Since - {}", err)))
                                        .expect("Unable to create Bad Request response for invalid header If-Modified-Since"));
                                }
                            }
                        }
                        None => None,
                    };

                    // Query parameters (note that non-required or collection query parameters will ignore garbage values, rather than causing a 400 response)
                    let query_params =
                        form_urlencoded::parse(uri.query().unwrap_or_default().as_bytes())
                            .collect::<Vec<_>>();
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
                    let param_single_nssai = query_params
                        .iter()
                        .filter(|e| e.0 == "single-nssai")
                        .map(|e| e.1.to_owned())
                        .next();
                    let param_single_nssai = match param_single_nssai {
                        Some(param_single_nssai) => {
                            let param_single_nssai =
                                serde_json::from_str::<models::Snssai>(&param_single_nssai);
                            match param_single_nssai {
                            Ok(param_single_nssai) => Some(param_single_nssai),
                            Err(e) => return Ok(Response::builder()
                                .status(StatusCode::BAD_REQUEST)
                                .body(Body::from(format!("Couldn't parse query parameter single-nssai - doesn't match schema: {}", e)))
                                .expect("Unable to create Bad Request response for invalid query parameter single-nssai")),
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
                    let param_plmn_id = query_params
                        .iter()
                        .filter(|e| e.0 == "plmn-id")
                        .map(|e| e.1.to_owned())
                        .next();
                    let param_plmn_id = match param_plmn_id {
                        Some(param_plmn_id) => {
                            let param_plmn_id =
                                serde_json::from_str::<models::PlmnId>(&param_plmn_id);
                            match param_plmn_id {
                            Ok(param_plmn_id) => Some(param_plmn_id),
                            Err(e) => return Ok(Response::builder()
                                .status(StatusCode::BAD_REQUEST)
                                .body(Body::from(format!("Couldn't parse query parameter plmn-id - doesn't match schema: {}", e)))
                                .expect("Unable to create Bad Request response for invalid query parameter plmn-id")),
                        }
                        }
                        None => None,
                    };

                    let result = api_impl
                        .get_sm_data(
                            param_supi,
                            param_supported_features,
                            param_single_nssai,
                            param_dnn,
                            param_plmn_id,
                            param_if_none_match,
                            param_if_modified_since,
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
                            GetSmDataResponse::ExpectedResponseToAValidRequest {
                                body,
                                cache_control,
                                e_tag,
                                last_modified,
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
                                if let Some(last_modified) = last_modified {
                                    let last_modified = match header::IntoHeaderValue(last_modified).try_into() {
                                                        Ok(val) => val,
                                                        Err(e) => {
                                                            return Ok(Response::builder()
                                                                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                                                                    .body(Body::from(format!("An internal server error occurred handling last_modified header - {}", e)))
                                                                    .expect("Unable to create Internal Server Error for invalid response header"))
                                                        }
                                                    };

                                    response.headers_mut().insert(
                                        HeaderName::from_static("last-modified"),
                                        last_modified,
                                    );
                                }
                                *response.status_mut() = StatusCode::from_u16(200)
                                    .expect("Unable to turn 200 into a StatusCode");
                                response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for GET_SM_DATA_EXPECTED_RESPONSE_TO_A_VALID_REQUEST"));
                                let body = serde_json::to_string(&body)
                                    .expect("impossible to fail to serialize");
                                *response.body_mut() = Body::from(body);
                            }
                            GetSmDataResponse::BadRequest(body) => {
                                *response.status_mut() = StatusCode::from_u16(400)
                                    .expect("Unable to turn 400 into a StatusCode");
                                response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for GET_SM_DATA_BAD_REQUEST"));
                                let body = serde_json::to_string(&body)
                                    .expect("impossible to fail to serialize");
                                *response.body_mut() = Body::from(body);
                            }
                            GetSmDataResponse::NotFound(body) => {
                                *response.status_mut() = StatusCode::from_u16(404)
                                    .expect("Unable to turn 404 into a StatusCode");
                                response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for GET_SM_DATA_NOT_FOUND"));
                                let body = serde_json::to_string(&body)
                                    .expect("impossible to fail to serialize");
                                *response.body_mut() = Body::from(body);
                            }
                            GetSmDataResponse::InternalServerError(body) => {
                                *response.status_mut() = StatusCode::from_u16(500)
                                    .expect("Unable to turn 500 into a StatusCode");
                                response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for GET_SM_DATA_INTERNAL_SERVER_ERROR"));
                                let body = serde_json::to_string(&body)
                                    .expect("impossible to fail to serialize");
                                *response.body_mut() = Body::from(body);
                            }
                            GetSmDataResponse::ServiceUnavailable(body) => {
                                *response.status_mut() = StatusCode::from_u16(503)
                                    .expect("Unable to turn 503 into a StatusCode");
                                response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for GET_SM_DATA_SERVICE_UNAVAILABLE"));
                                let body = serde_json::to_string(&body)
                                    .expect("impossible to fail to serialize");
                                *response.body_mut() = Body::from(body);
                            }
                            GetSmDataResponse::UnexpectedError => {
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

                // GetNssai - GET /{supi}/nssai
                hyper::Method::GET if path.matched(paths::ID_SUPI_NSSAI) => {
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
                                "nudm-sdm".to_string(), // Access to the nudm-sdm API
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
                    let path_params = paths::REGEX_SUPI_NSSAI.captures(path).unwrap_or_else(|| {
                        panic!(
                            "Path {} matched RE SUPI_NSSAI in set but failed match against \"{}\"",
                            path,
                            paths::REGEX_SUPI_NSSAI.as_str()
                        )
                    });

                    let param_supi = match percent_encoding::percent_decode(path_params["supi"].as_bytes()).decode_utf8() {
                    Ok(param_supi) => match param_supi.parse::<String>() {
                        Ok(param_supi) => param_supi,
                        Err(e) => return Ok(Response::builder()
                                        .status(StatusCode::BAD_REQUEST)
                                        .body(Body::from(format!("Couldn't parse path parameter supi: {}", e)))
                                        .expect("Unable to create Bad Request response for invalid path parameter")),
                    },
                    Err(_) => return Ok(Response::builder()
                                        .status(StatusCode::BAD_REQUEST)
                                        .body(Body::from(format!("Couldn't percent-decode path parameter as UTF-8: {}", &path_params["supi"])))
                                        .expect("Unable to create Bad Request response for invalid percent decode"))
                };

                    // Header parameters
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
                    let param_if_modified_since =
                        headers.get(HeaderName::from_static("if-modified-since"));

                    let param_if_modified_since = match param_if_modified_since {
                        Some(v) => {
                            match header::IntoHeaderValue::<String>::try_from((*v).clone()) {
                                Ok(result) => Some(result.0),
                                Err(err) => {
                                    return Ok(Response::builder()
                                        .status(StatusCode::BAD_REQUEST)
                                        .body(Body::from(format!("Invalid header If-Modified-Since - {}", err)))
                                        .expect("Unable to create Bad Request response for invalid header If-Modified-Since"));
                                }
                            }
                        }
                        None => None,
                    };

                    // Query parameters (note that non-required or collection query parameters will ignore garbage values, rather than causing a 400 response)
                    let query_params =
                        form_urlencoded::parse(uri.query().unwrap_or_default().as_bytes())
                            .collect::<Vec<_>>();
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
                    let param_plmn_id = query_params
                        .iter()
                        .filter(|e| e.0 == "plmn-id")
                        .map(|e| e.1.to_owned())
                        .next();
                    let param_plmn_id = match param_plmn_id {
                        Some(param_plmn_id) => {
                            let param_plmn_id =
                                serde_json::from_str::<models::PlmnId>(&param_plmn_id);
                            match param_plmn_id {
                            Ok(param_plmn_id) => Some(param_plmn_id),
                            Err(e) => return Ok(Response::builder()
                                .status(StatusCode::BAD_REQUEST)
                                .body(Body::from(format!("Couldn't parse query parameter plmn-id - doesn't match schema: {}", e)))
                                .expect("Unable to create Bad Request response for invalid query parameter plmn-id")),
                        }
                        }
                        None => None,
                    };
                    let param_disaster_roaming_ind = query_params
                        .iter()
                        .filter(|e| e.0 == "disaster-roaming-ind")
                        .map(|e| e.1.to_owned())
                        .next();
                    let param_disaster_roaming_ind = match param_disaster_roaming_ind {
                        Some(param_disaster_roaming_ind) => {
                            let param_disaster_roaming_ind =
                                <bool as std::str::FromStr>::from_str(&param_disaster_roaming_ind);
                            match param_disaster_roaming_ind {
                            Ok(param_disaster_roaming_ind) => Some(param_disaster_roaming_ind),
                            Err(e) => return Ok(Response::builder()
                                .status(StatusCode::BAD_REQUEST)
                                .body(Body::from(format!("Couldn't parse query parameter disaster-roaming-ind - doesn't match schema: {}", e)))
                                .expect("Unable to create Bad Request response for invalid query parameter disaster-roaming-ind")),
                        }
                        }
                        None => None,
                    };

                    let result = api_impl
                        .get_nssai(
                            param_supi,
                            param_supported_features,
                            param_plmn_id,
                            param_disaster_roaming_ind,
                            param_if_none_match,
                            param_if_modified_since,
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
                            GetNssaiResponse::ExpectedResponseToAValidRequest {
                                body,
                                cache_control,
                                e_tag,
                                last_modified,
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
                                if let Some(last_modified) = last_modified {
                                    let last_modified = match header::IntoHeaderValue(last_modified).try_into() {
                                                        Ok(val) => val,
                                                        Err(e) => {
                                                            return Ok(Response::builder()
                                                                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                                                                    .body(Body::from(format!("An internal server error occurred handling last_modified header - {}", e)))
                                                                    .expect("Unable to create Internal Server Error for invalid response header"))
                                                        }
                                                    };

                                    response.headers_mut().insert(
                                        HeaderName::from_static("last-modified"),
                                        last_modified,
                                    );
                                }
                                *response.status_mut() = StatusCode::from_u16(200)
                                    .expect("Unable to turn 200 into a StatusCode");
                                response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for GET_NSSAI_EXPECTED_RESPONSE_TO_A_VALID_REQUEST"));
                                let body = serde_json::to_string(&body)
                                    .expect("impossible to fail to serialize");
                                *response.body_mut() = Body::from(body);
                            }
                            GetNssaiResponse::BadRequest(body) => {
                                *response.status_mut() = StatusCode::from_u16(400)
                                    .expect("Unable to turn 400 into a StatusCode");
                                response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for GET_NSSAI_BAD_REQUEST"));
                                let body = serde_json::to_string(&body)
                                    .expect("impossible to fail to serialize");
                                *response.body_mut() = Body::from(body);
                            }
                            GetNssaiResponse::NotFound(body) => {
                                *response.status_mut() = StatusCode::from_u16(404)
                                    .expect("Unable to turn 404 into a StatusCode");
                                response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for GET_NSSAI_NOT_FOUND"));
                                let body = serde_json::to_string(&body)
                                    .expect("impossible to fail to serialize");
                                *response.body_mut() = Body::from(body);
                            }
                            GetNssaiResponse::InternalServerError(body) => {
                                *response.status_mut() = StatusCode::from_u16(500)
                                    .expect("Unable to turn 500 into a StatusCode");
                                response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for GET_NSSAI_INTERNAL_SERVER_ERROR"));
                                let body = serde_json::to_string(&body)
                                    .expect("impossible to fail to serialize");
                                *response.body_mut() = Body::from(body);
                            }
                            GetNssaiResponse::ServiceUnavailable(body) => {
                                *response.status_mut() = StatusCode::from_u16(503)
                                    .expect("Unable to turn 503 into a StatusCode");
                                response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for GET_NSSAI_SERVICE_UNAVAILABLE"));
                                let body = serde_json::to_string(&body)
                                    .expect("impossible to fail to serialize");
                                *response.body_mut() = Body::from(body);
                            }
                            GetNssaiResponse::UnexpectedError => {
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

                // Subscribe - POST /{ueId}/sdm-subscriptions
                hyper::Method::POST if path.matched(paths::ID_UEID_SDM_SUBSCRIPTIONS) => {
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
                                "nudm-sdm".to_string(), // Access to the nudm-sdm API
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
                    paths::REGEX_UEID_SDM_SUBSCRIPTIONS
                    .captures(path)
                    .unwrap_or_else(||
                        panic!("Path {} matched RE UEID_SDM_SUBSCRIPTIONS in set but failed match against \"{}\"", path, paths::REGEX_UEID_SDM_SUBSCRIPTIONS.as_str())
                    );

                    let param_ue_id = match percent_encoding::percent_decode(path_params["ueId"].as_bytes()).decode_utf8() {
                    Ok(param_ue_id) => match param_ue_id.parse::<String>() {
                        Ok(param_ue_id) => param_ue_id,
                        Err(e) => return Ok(Response::builder()
                                        .status(StatusCode::BAD_REQUEST)
                                        .body(Body::from(format!("Couldn't parse path parameter ueId: {}", e)))
                                        .expect("Unable to create Bad Request response for invalid path parameter")),
                    },
                    Err(_) => return Ok(Response::builder()
                                        .status(StatusCode::BAD_REQUEST)
                                        .body(Body::from(format!("Couldn't percent-decode path parameter as UTF-8: {}", &path_params["ueId"])))
                                        .expect("Unable to create Bad Request response for invalid percent decode"))
                };

                    // Body parameters (note that non-required body parameters will ignore garbage
                    // values, rather than causing a 400 response). Produce warning header and logs for
                    // any unused fields.
                    let result = body.into_raw().await;
                    match result {
                            Ok(body) => {
                                let mut unused_elements = Vec::new();
                                let param_sdm_subscription: Option<models::SdmSubscription> = if !body.is_empty() {
                                    let deserializer = &mut serde_json::Deserializer::from_slice(&*body);
                                    match serde_ignored::deserialize(deserializer, |path| {
                                            warn!("Ignoring unknown field in body: {}", path);
                                            unused_elements.push(path.to_string());
                                    }) {
                                        Ok(param_sdm_subscription) => param_sdm_subscription,
                                        Err(e) => return Ok(Response::builder()
                                                        .status(StatusCode::BAD_REQUEST)
                                                        .body(Body::from(format!("Couldn't parse body parameter SdmSubscription - doesn't match schema: {}", e)))
                                                        .expect("Unable to create Bad Request response for invalid body parameter SdmSubscription due to schema")),
                                    }
                                } else {
                                    None
                                };
                                let param_sdm_subscription = match param_sdm_subscription {
                                    Some(param_sdm_subscription) => param_sdm_subscription,
                                    None => return Ok(Response::builder()
                                                        .status(StatusCode::BAD_REQUEST)
                                                        .body(Body::from("Missing required body parameter SdmSubscription"))
                                                        .expect("Unable to create Bad Request response for missing body parameter SdmSubscription")),
                                };

                                let result = api_impl.subscribe(
                                            param_ue_id,
                                            param_sdm_subscription,
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
                                                SubscribeResponse::ExpectedResponseToAValidRequest
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
                                                            .expect("Unable to create Content-Type header for SUBSCRIBE_EXPECTED_RESPONSE_TO_A_VALID_REQUEST"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                SubscribeResponse::BadRequest
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(400).expect("Unable to turn 400 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for SUBSCRIBE_BAD_REQUEST"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                SubscribeResponse::NotFound
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(404).expect("Unable to turn 404 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for SUBSCRIBE_NOT_FOUND"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                SubscribeResponse::InternalServerError
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(500).expect("Unable to turn 500 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for SUBSCRIBE_INTERNAL_SERVER_ERROR"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                SubscribeResponse::NotImplemented
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(501).expect("Unable to turn 501 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for SUBSCRIBE_NOT_IMPLEMENTED"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                SubscribeResponse::ServiceUnavailable
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(503).expect("Unable to turn 503 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for SUBSCRIBE_SERVICE_UNAVAILABLE"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                SubscribeResponse::UnexpectedError
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
                                                .body(Body::from(format!("Couldn't read body parameter SdmSubscription: {}", e)))
                                                .expect("Unable to create Bad Request response due to unable to read body parameter SdmSubscription")),
                        }
                }

                // SubscribeToSharedData - POST /shared-data-subscriptions
                hyper::Method::POST if path.matched(paths::ID_SHARED_DATA_SUBSCRIPTIONS) => {
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
                                "nudm-sdm".to_string(), // Access to the nudm-sdm API
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
                            Ok(body) => {
                                let mut unused_elements = Vec::new();
                                let param_sdm_subscription: Option<models::SdmSubscription> = if !body.is_empty() {
                                    let deserializer = &mut serde_json::Deserializer::from_slice(&*body);
                                    match serde_ignored::deserialize(deserializer, |path| {
                                            warn!("Ignoring unknown field in body: {}", path);
                                            unused_elements.push(path.to_string());
                                    }) {
                                        Ok(param_sdm_subscription) => param_sdm_subscription,
                                        Err(e) => return Ok(Response::builder()
                                                        .status(StatusCode::BAD_REQUEST)
                                                        .body(Body::from(format!("Couldn't parse body parameter SdmSubscription - doesn't match schema: {}", e)))
                                                        .expect("Unable to create Bad Request response for invalid body parameter SdmSubscription due to schema")),
                                    }
                                } else {
                                    None
                                };
                                let param_sdm_subscription = match param_sdm_subscription {
                                    Some(param_sdm_subscription) => param_sdm_subscription,
                                    None => return Ok(Response::builder()
                                                        .status(StatusCode::BAD_REQUEST)
                                                        .body(Body::from("Missing required body parameter SdmSubscription"))
                                                        .expect("Unable to create Bad Request response for missing body parameter SdmSubscription")),
                                };

                                let result = api_impl.subscribe_to_shared_data(
                                            param_sdm_subscription,
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
                                                SubscribeToSharedDataResponse::ExpectedResponseToAValidRequest
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
                                                            .expect("Unable to create Content-Type header for SUBSCRIBE_TO_SHARED_DATA_EXPECTED_RESPONSE_TO_A_VALID_REQUEST"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                SubscribeToSharedDataResponse::BadRequest
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(400).expect("Unable to turn 400 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for SUBSCRIBE_TO_SHARED_DATA_BAD_REQUEST"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                SubscribeToSharedDataResponse::NotFound
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(404).expect("Unable to turn 404 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for SUBSCRIBE_TO_SHARED_DATA_NOT_FOUND"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                SubscribeToSharedDataResponse::UnexpectedError
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
                                                .body(Body::from(format!("Couldn't read body parameter SdmSubscription: {}", e)))
                                                .expect("Unable to create Bad Request response due to unable to read body parameter SdmSubscription")),
                        }
                }

                // Unsubscribe - DELETE /{ueId}/sdm-subscriptions/{subscriptionId}
                hyper::Method::DELETE
                    if path.matched(paths::ID_UEID_SDM_SUBSCRIPTIONS_SUBSCRIPTIONID) =>
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
                                "nudm-sdm".to_string(), // Access to the nudm-sdm API
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
                    paths::REGEX_UEID_SDM_SUBSCRIPTIONS_SUBSCRIPTIONID
                    .captures(path)
                    .unwrap_or_else(||
                        panic!("Path {} matched RE UEID_SDM_SUBSCRIPTIONS_SUBSCRIPTIONID in set but failed match against \"{}\"", path, paths::REGEX_UEID_SDM_SUBSCRIPTIONS_SUBSCRIPTIONID.as_str())
                    );

                    let param_ue_id = match percent_encoding::percent_decode(path_params["ueId"].as_bytes()).decode_utf8() {
                    Ok(param_ue_id) => match param_ue_id.parse::<String>() {
                        Ok(param_ue_id) => param_ue_id,
                        Err(e) => return Ok(Response::builder()
                                        .status(StatusCode::BAD_REQUEST)
                                        .body(Body::from(format!("Couldn't parse path parameter ueId: {}", e)))
                                        .expect("Unable to create Bad Request response for invalid path parameter")),
                    },
                    Err(_) => return Ok(Response::builder()
                                        .status(StatusCode::BAD_REQUEST)
                                        .body(Body::from(format!("Couldn't percent-decode path parameter as UTF-8: {}", &path_params["ueId"])))
                                        .expect("Unable to create Bad Request response for invalid percent decode"))
                };

                    let param_subscription_id = match percent_encoding::percent_decode(path_params["subscriptionId"].as_bytes()).decode_utf8() {
                    Ok(param_subscription_id) => match param_subscription_id.parse::<String>() {
                        Ok(param_subscription_id) => param_subscription_id,
                        Err(e) => return Ok(Response::builder()
                                        .status(StatusCode::BAD_REQUEST)
                                        .body(Body::from(format!("Couldn't parse path parameter subscriptionId: {}", e)))
                                        .expect("Unable to create Bad Request response for invalid path parameter")),
                    },
                    Err(_) => return Ok(Response::builder()
                                        .status(StatusCode::BAD_REQUEST)
                                        .body(Body::from(format!("Couldn't percent-decode path parameter as UTF-8: {}", &path_params["subscriptionId"])))
                                        .expect("Unable to create Bad Request response for invalid percent decode"))
                };

                    let result = api_impl
                        .unsubscribe(param_ue_id, param_subscription_id, &context)
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
                            UnsubscribeResponse::SuccessfulResponse => {
                                *response.status_mut() = StatusCode::from_u16(204)
                                    .expect("Unable to turn 204 into a StatusCode");
                            }
                            UnsubscribeResponse::BadRequest(body) => {
                                *response.status_mut() = StatusCode::from_u16(400)
                                    .expect("Unable to turn 400 into a StatusCode");
                                response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for UNSUBSCRIBE_BAD_REQUEST"));
                                let body = serde_json::to_string(&body)
                                    .expect("impossible to fail to serialize");
                                *response.body_mut() = Body::from(body);
                            }
                            UnsubscribeResponse::NotFound(body) => {
                                *response.status_mut() = StatusCode::from_u16(404)
                                    .expect("Unable to turn 404 into a StatusCode");
                                response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for UNSUBSCRIBE_NOT_FOUND"));
                                let body = serde_json::to_string(&body)
                                    .expect("impossible to fail to serialize");
                                *response.body_mut() = Body::from(body);
                            }
                            UnsubscribeResponse::InternalServerError(body) => {
                                *response.status_mut() = StatusCode::from_u16(500)
                                    .expect("Unable to turn 500 into a StatusCode");
                                response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for UNSUBSCRIBE_INTERNAL_SERVER_ERROR"));
                                let body = serde_json::to_string(&body)
                                    .expect("impossible to fail to serialize");
                                *response.body_mut() = Body::from(body);
                            }
                            UnsubscribeResponse::ServiceUnavailable(body) => {
                                *response.status_mut() = StatusCode::from_u16(503)
                                    .expect("Unable to turn 503 into a StatusCode");
                                response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for UNSUBSCRIBE_SERVICE_UNAVAILABLE"));
                                let body = serde_json::to_string(&body)
                                    .expect("impossible to fail to serialize");
                                *response.body_mut() = Body::from(body);
                            }
                            UnsubscribeResponse::UnexpectedError => {
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

                // UnsubscribeForSharedData - DELETE /shared-data-subscriptions/{subscriptionId}
                hyper::Method::DELETE
                    if path.matched(paths::ID_SHARED_DATA_SUBSCRIPTIONS_SUBSCRIPTIONID) =>
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
                                "nudm-sdm".to_string(), // Access to the nudm-sdm API
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
                    paths::REGEX_SHARED_DATA_SUBSCRIPTIONS_SUBSCRIPTIONID
                    .captures(path)
                    .unwrap_or_else(||
                        panic!("Path {} matched RE SHARED_DATA_SUBSCRIPTIONS_SUBSCRIPTIONID in set but failed match against \"{}\"", path, paths::REGEX_SHARED_DATA_SUBSCRIPTIONS_SUBSCRIPTIONID.as_str())
                    );

                    let param_subscription_id = match percent_encoding::percent_decode(path_params["subscriptionId"].as_bytes()).decode_utf8() {
                    Ok(param_subscription_id) => match param_subscription_id.parse::<String>() {
                        Ok(param_subscription_id) => param_subscription_id,
                        Err(e) => return Ok(Response::builder()
                                        .status(StatusCode::BAD_REQUEST)
                                        .body(Body::from(format!("Couldn't parse path parameter subscriptionId: {}", e)))
                                        .expect("Unable to create Bad Request response for invalid path parameter")),
                    },
                    Err(_) => return Ok(Response::builder()
                                        .status(StatusCode::BAD_REQUEST)
                                        .body(Body::from(format!("Couldn't percent-decode path parameter as UTF-8: {}", &path_params["subscriptionId"])))
                                        .expect("Unable to create Bad Request response for invalid percent decode"))
                };

                    let result = api_impl
                        .unsubscribe_for_shared_data(param_subscription_id, &context)
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
                            UnsubscribeForSharedDataResponse::SuccessfulResponse => {
                                *response.status_mut() = StatusCode::from_u16(204)
                                    .expect("Unable to turn 204 into a StatusCode");
                            }
                            UnsubscribeForSharedDataResponse::BadRequest(body) => {
                                *response.status_mut() = StatusCode::from_u16(400)
                                    .expect("Unable to turn 400 into a StatusCode");
                                response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for UNSUBSCRIBE_FOR_SHARED_DATA_BAD_REQUEST"));
                                let body = serde_json::to_string(&body)
                                    .expect("impossible to fail to serialize");
                                *response.body_mut() = Body::from(body);
                            }
                            UnsubscribeForSharedDataResponse::NotFound(body) => {
                                *response.status_mut() = StatusCode::from_u16(404)
                                    .expect("Unable to turn 404 into a StatusCode");
                                response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for UNSUBSCRIBE_FOR_SHARED_DATA_NOT_FOUND"));
                                let body = serde_json::to_string(&body)
                                    .expect("impossible to fail to serialize");
                                *response.body_mut() = Body::from(body);
                            }
                            UnsubscribeForSharedDataResponse::InternalServerError(body) => {
                                *response.status_mut() = StatusCode::from_u16(500)
                                    .expect("Unable to turn 500 into a StatusCode");
                                response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for UNSUBSCRIBE_FOR_SHARED_DATA_INTERNAL_SERVER_ERROR"));
                                let body = serde_json::to_string(&body)
                                    .expect("impossible to fail to serialize");
                                *response.body_mut() = Body::from(body);
                            }
                            UnsubscribeForSharedDataResponse::ServiceUnavailable(body) => {
                                *response.status_mut() = StatusCode::from_u16(503)
                                    .expect("Unable to turn 503 into a StatusCode");
                                response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for UNSUBSCRIBE_FOR_SHARED_DATA_SERVICE_UNAVAILABLE"));
                                let body = serde_json::to_string(&body)
                                    .expect("impossible to fail to serialize");
                                *response.body_mut() = Body::from(body);
                            }
                            UnsubscribeForSharedDataResponse::UnexpectedError => {
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

                // Modify - PATCH /{ueId}/sdm-subscriptions/{subscriptionId}
                hyper::Method::PATCH
                    if path.matched(paths::ID_UEID_SDM_SUBSCRIPTIONS_SUBSCRIPTIONID) =>
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
                                "nudm-sdm".to_string(), // Access to the nudm-sdm API
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
                    paths::REGEX_UEID_SDM_SUBSCRIPTIONS_SUBSCRIPTIONID
                    .captures(path)
                    .unwrap_or_else(||
                        panic!("Path {} matched RE UEID_SDM_SUBSCRIPTIONS_SUBSCRIPTIONID in set but failed match against \"{}\"", path, paths::REGEX_UEID_SDM_SUBSCRIPTIONS_SUBSCRIPTIONID.as_str())
                    );

                    let param_ue_id = match percent_encoding::percent_decode(path_params["ueId"].as_bytes()).decode_utf8() {
                    Ok(param_ue_id) => match param_ue_id.parse::<String>() {
                        Ok(param_ue_id) => param_ue_id,
                        Err(e) => return Ok(Response::builder()
                                        .status(StatusCode::BAD_REQUEST)
                                        .body(Body::from(format!("Couldn't parse path parameter ueId: {}", e)))
                                        .expect("Unable to create Bad Request response for invalid path parameter")),
                    },
                    Err(_) => return Ok(Response::builder()
                                        .status(StatusCode::BAD_REQUEST)
                                        .body(Body::from(format!("Couldn't percent-decode path parameter as UTF-8: {}", &path_params["ueId"])))
                                        .expect("Unable to create Bad Request response for invalid percent decode"))
                };

                    let param_subscription_id = match percent_encoding::percent_decode(path_params["subscriptionId"].as_bytes()).decode_utf8() {
                    Ok(param_subscription_id) => match param_subscription_id.parse::<String>() {
                        Ok(param_subscription_id) => param_subscription_id,
                        Err(e) => return Ok(Response::builder()
                                        .status(StatusCode::BAD_REQUEST)
                                        .body(Body::from(format!("Couldn't parse path parameter subscriptionId: {}", e)))
                                        .expect("Unable to create Bad Request response for invalid path parameter")),
                    },
                    Err(_) => return Ok(Response::builder()
                                        .status(StatusCode::BAD_REQUEST)
                                        .body(Body::from(format!("Couldn't percent-decode path parameter as UTF-8: {}", &path_params["subscriptionId"])))
                                        .expect("Unable to create Bad Request response for invalid percent decode"))
                };

                    // Query parameters (note that non-required or collection query parameters will ignore garbage values, rather than causing a 400 response)
                    let query_params =
                        form_urlencoded::parse(uri.query().unwrap_or_default().as_bytes())
                            .collect::<Vec<_>>();
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

                    // Body parameters (note that non-required body parameters will ignore garbage
                    // values, rather than causing a 400 response). Produce warning header and logs for
                    // any unused fields.
                    let result = body.into_raw().await;
                    match result {
                            Ok(body) => {
                                let mut unused_elements = Vec::new();
                                let param_sdm_subs_modification: Option<models::SdmSubsModification> = if !body.is_empty() {
                                    let deserializer = &mut serde_json::Deserializer::from_slice(&*body);
                                    match serde_ignored::deserialize(deserializer, |path| {
                                            warn!("Ignoring unknown field in body: {}", path);
                                            unused_elements.push(path.to_string());
                                    }) {
                                        Ok(param_sdm_subs_modification) => param_sdm_subs_modification,
                                        Err(e) => return Ok(Response::builder()
                                                        .status(StatusCode::BAD_REQUEST)
                                                        .body(Body::from(format!("Couldn't parse body parameter SdmSubsModification - doesn't match schema: {}", e)))
                                                        .expect("Unable to create Bad Request response for invalid body parameter SdmSubsModification due to schema")),
                                    }
                                } else {
                                    None
                                };
                                let param_sdm_subs_modification = match param_sdm_subs_modification {
                                    Some(param_sdm_subs_modification) => param_sdm_subs_modification,
                                    None => return Ok(Response::builder()
                                                        .status(StatusCode::BAD_REQUEST)
                                                        .body(Body::from("Missing required body parameter SdmSubsModification"))
                                                        .expect("Unable to create Bad Request response for missing body parameter SdmSubsModification")),
                                };

                                let result = api_impl.modify(
                                            param_ue_id,
                                            param_subscription_id,
                                            param_sdm_subs_modification,
                                            param_supported_features,
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
                                                ModifyResponse::ExpectedResponseToAValidRequest
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(200).expect("Unable to turn 200 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for MODIFY_EXPECTED_RESPONSE_TO_A_VALID_REQUEST"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                ModifyResponse::BadRequest
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(400).expect("Unable to turn 400 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for MODIFY_BAD_REQUEST"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                ModifyResponse::Forbidden
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(403).expect("Unable to turn 403 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for MODIFY_FORBIDDEN"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                ModifyResponse::NotFound
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(404).expect("Unable to turn 404 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for MODIFY_NOT_FOUND"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                ModifyResponse::InternalServerError
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(500).expect("Unable to turn 500 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for MODIFY_INTERNAL_SERVER_ERROR"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                ModifyResponse::ServiceUnavailable
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(503).expect("Unable to turn 503 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for MODIFY_SERVICE_UNAVAILABLE"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                ModifyResponse::UnexpectedError
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
                                                .body(Body::from(format!("Couldn't read body parameter SdmSubsModification: {}", e)))
                                                .expect("Unable to create Bad Request response due to unable to read body parameter SdmSubsModification")),
                        }
                }

                // ModifySharedDataSubs - PATCH /shared-data-subscriptions/{subscriptionId}
                hyper::Method::PATCH
                    if path.matched(paths::ID_SHARED_DATA_SUBSCRIPTIONS_SUBSCRIPTIONID) =>
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
                                "nudm-sdm".to_string(), // Access to the nudm-sdm API
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
                    paths::REGEX_SHARED_DATA_SUBSCRIPTIONS_SUBSCRIPTIONID
                    .captures(path)
                    .unwrap_or_else(||
                        panic!("Path {} matched RE SHARED_DATA_SUBSCRIPTIONS_SUBSCRIPTIONID in set but failed match against \"{}\"", path, paths::REGEX_SHARED_DATA_SUBSCRIPTIONS_SUBSCRIPTIONID.as_str())
                    );

                    let param_subscription_id = match percent_encoding::percent_decode(path_params["subscriptionId"].as_bytes()).decode_utf8() {
                    Ok(param_subscription_id) => match param_subscription_id.parse::<String>() {
                        Ok(param_subscription_id) => param_subscription_id,
                        Err(e) => return Ok(Response::builder()
                                        .status(StatusCode::BAD_REQUEST)
                                        .body(Body::from(format!("Couldn't parse path parameter subscriptionId: {}", e)))
                                        .expect("Unable to create Bad Request response for invalid path parameter")),
                    },
                    Err(_) => return Ok(Response::builder()
                                        .status(StatusCode::BAD_REQUEST)
                                        .body(Body::from(format!("Couldn't percent-decode path parameter as UTF-8: {}", &path_params["subscriptionId"])))
                                        .expect("Unable to create Bad Request response for invalid percent decode"))
                };

                    // Query parameters (note that non-required or collection query parameters will ignore garbage values, rather than causing a 400 response)
                    let query_params =
                        form_urlencoded::parse(uri.query().unwrap_or_default().as_bytes())
                            .collect::<Vec<_>>();
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

                    // Body parameters (note that non-required body parameters will ignore garbage
                    // values, rather than causing a 400 response). Produce warning header and logs for
                    // any unused fields.
                    let result = body.into_raw().await;
                    match result {
                            Ok(body) => {
                                let mut unused_elements = Vec::new();
                                let param_sdm_subs_modification: Option<models::SdmSubsModification> = if !body.is_empty() {
                                    let deserializer = &mut serde_json::Deserializer::from_slice(&*body);
                                    match serde_ignored::deserialize(deserializer, |path| {
                                            warn!("Ignoring unknown field in body: {}", path);
                                            unused_elements.push(path.to_string());
                                    }) {
                                        Ok(param_sdm_subs_modification) => param_sdm_subs_modification,
                                        Err(e) => return Ok(Response::builder()
                                                        .status(StatusCode::BAD_REQUEST)
                                                        .body(Body::from(format!("Couldn't parse body parameter SdmSubsModification - doesn't match schema: {}", e)))
                                                        .expect("Unable to create Bad Request response for invalid body parameter SdmSubsModification due to schema")),
                                    }
                                } else {
                                    None
                                };
                                let param_sdm_subs_modification = match param_sdm_subs_modification {
                                    Some(param_sdm_subs_modification) => param_sdm_subs_modification,
                                    None => return Ok(Response::builder()
                                                        .status(StatusCode::BAD_REQUEST)
                                                        .body(Body::from("Missing required body parameter SdmSubsModification"))
                                                        .expect("Unable to create Bad Request response for missing body parameter SdmSubsModification")),
                                };

                                let result = api_impl.modify_shared_data_subs(
                                            param_subscription_id,
                                            param_sdm_subs_modification,
                                            param_supported_features,
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
                                                ModifySharedDataSubsResponse::ExpectedResponseToAValidRequest
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(200).expect("Unable to turn 200 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for MODIFY_SHARED_DATA_SUBS_EXPECTED_RESPONSE_TO_A_VALID_REQUEST"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                ModifySharedDataSubsResponse::BadRequest
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(400).expect("Unable to turn 400 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for MODIFY_SHARED_DATA_SUBS_BAD_REQUEST"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                ModifySharedDataSubsResponse::Forbidden
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(403).expect("Unable to turn 403 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for MODIFY_SHARED_DATA_SUBS_FORBIDDEN"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                ModifySharedDataSubsResponse::NotFound
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(404).expect("Unable to turn 404 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for MODIFY_SHARED_DATA_SUBS_NOT_FOUND"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                ModifySharedDataSubsResponse::InternalServerError
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(500).expect("Unable to turn 500 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for MODIFY_SHARED_DATA_SUBS_INTERNAL_SERVER_ERROR"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                ModifySharedDataSubsResponse::ServiceUnavailable
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(503).expect("Unable to turn 503 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for MODIFY_SHARED_DATA_SUBS_SERVICE_UNAVAILABLE"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                ModifySharedDataSubsResponse::UnexpectedError
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
                                                .body(Body::from(format!("Couldn't read body parameter SdmSubsModification: {}", e)))
                                                .expect("Unable to create Bad Request response due to unable to read body parameter SdmSubsModification")),
                        }
                }

                // GetTraceConfigData - GET /{supi}/trace-data
                hyper::Method::GET if path.matched(paths::ID_SUPI_TRACE_DATA) => {
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
                                "nudm-sdm".to_string(), // Access to the nudm-sdm API
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
                    paths::REGEX_SUPI_TRACE_DATA
                    .captures(path)
                    .unwrap_or_else(||
                        panic!("Path {} matched RE SUPI_TRACE_DATA in set but failed match against \"{}\"", path, paths::REGEX_SUPI_TRACE_DATA.as_str())
                    );

                    let param_supi = match percent_encoding::percent_decode(path_params["supi"].as_bytes()).decode_utf8() {
                    Ok(param_supi) => match param_supi.parse::<String>() {
                        Ok(param_supi) => param_supi,
                        Err(e) => return Ok(Response::builder()
                                        .status(StatusCode::BAD_REQUEST)
                                        .body(Body::from(format!("Couldn't parse path parameter supi: {}", e)))
                                        .expect("Unable to create Bad Request response for invalid path parameter")),
                    },
                    Err(_) => return Ok(Response::builder()
                                        .status(StatusCode::BAD_REQUEST)
                                        .body(Body::from(format!("Couldn't percent-decode path parameter as UTF-8: {}", &path_params["supi"])))
                                        .expect("Unable to create Bad Request response for invalid percent decode"))
                };

                    // Header parameters
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
                    let param_if_modified_since =
                        headers.get(HeaderName::from_static("if-modified-since"));

                    let param_if_modified_since = match param_if_modified_since {
                        Some(v) => {
                            match header::IntoHeaderValue::<String>::try_from((*v).clone()) {
                                Ok(result) => Some(result.0),
                                Err(err) => {
                                    return Ok(Response::builder()
                                        .status(StatusCode::BAD_REQUEST)
                                        .body(Body::from(format!("Invalid header If-Modified-Since - {}", err)))
                                        .expect("Unable to create Bad Request response for invalid header If-Modified-Since"));
                                }
                            }
                        }
                        None => None,
                    };

                    // Query parameters (note that non-required or collection query parameters will ignore garbage values, rather than causing a 400 response)
                    let query_params =
                        form_urlencoded::parse(uri.query().unwrap_or_default().as_bytes())
                            .collect::<Vec<_>>();
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
                    let param_plmn_id = query_params
                        .iter()
                        .filter(|e| e.0 == "plmn-id")
                        .map(|e| e.1.to_owned())
                        .next();
                    let param_plmn_id = match param_plmn_id {
                        Some(param_plmn_id) => {
                            let param_plmn_id =
                                serde_json::from_str::<models::PlmnId>(&param_plmn_id);
                            match param_plmn_id {
                            Ok(param_plmn_id) => Some(param_plmn_id),
                            Err(e) => return Ok(Response::builder()
                                .status(StatusCode::BAD_REQUEST)
                                .body(Body::from(format!("Couldn't parse query parameter plmn-id - doesn't match schema: {}", e)))
                                .expect("Unable to create Bad Request response for invalid query parameter plmn-id")),
                        }
                        }
                        None => None,
                    };

                    let result = api_impl
                        .get_trace_config_data(
                            param_supi,
                            param_supported_features,
                            param_plmn_id,
                            param_if_none_match,
                            param_if_modified_since,
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
                            GetTraceConfigDataResponse::ExpectedResponseToAValidRequest {
                                body,
                                cache_control,
                                e_tag,
                                last_modified,
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
                                if let Some(last_modified) = last_modified {
                                    let last_modified = match header::IntoHeaderValue(last_modified).try_into() {
                                                        Ok(val) => val,
                                                        Err(e) => {
                                                            return Ok(Response::builder()
                                                                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                                                                    .body(Body::from(format!("An internal server error occurred handling last_modified header - {}", e)))
                                                                    .expect("Unable to create Internal Server Error for invalid response header"))
                                                        }
                                                    };

                                    response.headers_mut().insert(
                                        HeaderName::from_static("last-modified"),
                                        last_modified,
                                    );
                                }
                                *response.status_mut() = StatusCode::from_u16(200)
                                    .expect("Unable to turn 200 into a StatusCode");
                                response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for GET_TRACE_CONFIG_DATA_EXPECTED_RESPONSE_TO_A_VALID_REQUEST"));
                                let body = serde_json::to_string(&body)
                                    .expect("impossible to fail to serialize");
                                *response.body_mut() = Body::from(body);
                            }
                            GetTraceConfigDataResponse::BadRequest(body) => {
                                *response.status_mut() = StatusCode::from_u16(400)
                                    .expect("Unable to turn 400 into a StatusCode");
                                response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for GET_TRACE_CONFIG_DATA_BAD_REQUEST"));
                                let body = serde_json::to_string(&body)
                                    .expect("impossible to fail to serialize");
                                *response.body_mut() = Body::from(body);
                            }
                            GetTraceConfigDataResponse::NotFound(body) => {
                                *response.status_mut() = StatusCode::from_u16(404)
                                    .expect("Unable to turn 404 into a StatusCode");
                                response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for GET_TRACE_CONFIG_DATA_NOT_FOUND"));
                                let body = serde_json::to_string(&body)
                                    .expect("impossible to fail to serialize");
                                *response.body_mut() = Body::from(body);
                            }
                            GetTraceConfigDataResponse::InternalServerError(body) => {
                                *response.status_mut() = StatusCode::from_u16(500)
                                    .expect("Unable to turn 500 into a StatusCode");
                                response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for GET_TRACE_CONFIG_DATA_INTERNAL_SERVER_ERROR"));
                                let body = serde_json::to_string(&body)
                                    .expect("impossible to fail to serialize");
                                *response.body_mut() = Body::from(body);
                            }
                            GetTraceConfigDataResponse::ServiceUnavailable(body) => {
                                *response.status_mut() = StatusCode::from_u16(503)
                                    .expect("Unable to turn 503 into a StatusCode");
                                response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for GET_TRACE_CONFIG_DATA_SERVICE_UNAVAILABLE"));
                                let body = serde_json::to_string(&body)
                                    .expect("impossible to fail to serialize");
                                *response.body_mut() = Body::from(body);
                            }
                            GetTraceConfigDataResponse::UnexpectedError => {
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

                // UpdateSorInfo - POST /{supi}/am-data/update-sor
                hyper::Method::POST if path.matched(paths::ID_SUPI_AM_DATA_UPDATE_SOR) => {
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
                                "nudm-sdm".to_string(), // Access to the nudm-sdm API
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
                    paths::REGEX_SUPI_AM_DATA_UPDATE_SOR
                    .captures(path)
                    .unwrap_or_else(||
                        panic!("Path {} matched RE SUPI_AM_DATA_UPDATE_SOR in set but failed match against \"{}\"", path, paths::REGEX_SUPI_AM_DATA_UPDATE_SOR.as_str())
                    );

                    let param_supi = match percent_encoding::percent_decode(path_params["supi"].as_bytes()).decode_utf8() {
                    Ok(param_supi) => match param_supi.parse::<String>() {
                        Ok(param_supi) => param_supi,
                        Err(e) => return Ok(Response::builder()
                                        .status(StatusCode::BAD_REQUEST)
                                        .body(Body::from(format!("Couldn't parse path parameter supi: {}", e)))
                                        .expect("Unable to create Bad Request response for invalid path parameter")),
                    },
                    Err(_) => return Ok(Response::builder()
                                        .status(StatusCode::BAD_REQUEST)
                                        .body(Body::from(format!("Couldn't percent-decode path parameter as UTF-8: {}", &path_params["supi"])))
                                        .expect("Unable to create Bad Request response for invalid percent decode"))
                };

                    // Body parameters (note that non-required body parameters will ignore garbage
                    // values, rather than causing a 400 response). Produce warning header and logs for
                    // any unused fields.
                    let result = body.into_raw().await;
                    match result {
                            Ok(body) => {
                                let mut unused_elements = Vec::new();
                                let param_sor_update_info: Option<models::SorUpdateInfo> = if !body.is_empty() {
                                    let deserializer = &mut serde_json::Deserializer::from_slice(&*body);
                                    match serde_ignored::deserialize(deserializer, |path| {
                                            warn!("Ignoring unknown field in body: {}", path);
                                            unused_elements.push(path.to_string());
                                    }) {
                                        Ok(param_sor_update_info) => param_sor_update_info,
                                        Err(_) => None,
                                    }
                                } else {
                                    None
                                };

                                let result = api_impl.update_sor_info(
                                            param_supi,
                                            param_sor_update_info,
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
                                                UpdateSorInfoResponse::ExpectedResponseToAValidRequest
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(200).expect("Unable to turn 200 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for UPDATE_SOR_INFO_EXPECTED_RESPONSE_TO_A_VALID_REQUEST"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                UpdateSorInfoResponse::BadRequest
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(400).expect("Unable to turn 400 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for UPDATE_SOR_INFO_BAD_REQUEST"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                UpdateSorInfoResponse::NotFound
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(404).expect("Unable to turn 404 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for UPDATE_SOR_INFO_NOT_FOUND"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                UpdateSorInfoResponse::InternalServerError
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(500).expect("Unable to turn 500 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for UPDATE_SOR_INFO_INTERNAL_SERVER_ERROR"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                UpdateSorInfoResponse::ServiceUnavailable
                                                    (body)
                                                => {
                                                    *response.status_mut() = StatusCode::from_u16(503).expect("Unable to turn 503 into a StatusCode");
                                                    response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for UPDATE_SOR_INFO_SERVICE_UNAVAILABLE"));
                                                    let body = serde_json::to_string(&body).expect("impossible to fail to serialize");
                                                    *response.body_mut() = Body::from(body);
                                                },
                                                UpdateSorInfoResponse::UnexpectedError
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
                                                .body(Body::from(format!("Couldn't read body parameter SorUpdateInfo: {}", e)))
                                                .expect("Unable to create Bad Request response due to unable to read body parameter SorUpdateInfo")),
                        }
                }

                // GetUeCtxInAmfData - GET /{supi}/ue-context-in-amf-data
                hyper::Method::GET if path.matched(paths::ID_SUPI_UE_CONTEXT_IN_AMF_DATA) => {
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
                                "nudm-sdm".to_string(), // Access to the nudm-sdm API
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
                    paths::REGEX_SUPI_UE_CONTEXT_IN_AMF_DATA
                    .captures(path)
                    .unwrap_or_else(||
                        panic!("Path {} matched RE SUPI_UE_CONTEXT_IN_AMF_DATA in set but failed match against \"{}\"", path, paths::REGEX_SUPI_UE_CONTEXT_IN_AMF_DATA.as_str())
                    );

                    let param_supi = match percent_encoding::percent_decode(path_params["supi"].as_bytes()).decode_utf8() {
                    Ok(param_supi) => match param_supi.parse::<String>() {
                        Ok(param_supi) => param_supi,
                        Err(e) => return Ok(Response::builder()
                                        .status(StatusCode::BAD_REQUEST)
                                        .body(Body::from(format!("Couldn't parse path parameter supi: {}", e)))
                                        .expect("Unable to create Bad Request response for invalid path parameter")),
                    },
                    Err(_) => return Ok(Response::builder()
                                        .status(StatusCode::BAD_REQUEST)
                                        .body(Body::from(format!("Couldn't percent-decode path parameter as UTF-8: {}", &path_params["supi"])))
                                        .expect("Unable to create Bad Request response for invalid percent decode"))
                };

                    // Query parameters (note that non-required or collection query parameters will ignore garbage values, rather than causing a 400 response)
                    let query_params =
                        form_urlencoded::parse(uri.query().unwrap_or_default().as_bytes())
                            .collect::<Vec<_>>();
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

                    let result = api_impl
                        .get_ue_ctx_in_amf_data(param_supi, param_supported_features, &context)
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
                            GetUeCtxInAmfDataResponse::ExpectedResponseToAValidRequest(body) => {
                                *response.status_mut() = StatusCode::from_u16(200)
                                    .expect("Unable to turn 200 into a StatusCode");
                                response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for GET_UE_CTX_IN_AMF_DATA_EXPECTED_RESPONSE_TO_A_VALID_REQUEST"));
                                let body = serde_json::to_string(&body)
                                    .expect("impossible to fail to serialize");
                                *response.body_mut() = Body::from(body);
                            }
                            GetUeCtxInAmfDataResponse::BadRequest(body) => {
                                *response.status_mut() = StatusCode::from_u16(400)
                                    .expect("Unable to turn 400 into a StatusCode");
                                response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for GET_UE_CTX_IN_AMF_DATA_BAD_REQUEST"));
                                let body = serde_json::to_string(&body)
                                    .expect("impossible to fail to serialize");
                                *response.body_mut() = Body::from(body);
                            }
                            GetUeCtxInAmfDataResponse::NotFound(body) => {
                                *response.status_mut() = StatusCode::from_u16(404)
                                    .expect("Unable to turn 404 into a StatusCode");
                                response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for GET_UE_CTX_IN_AMF_DATA_NOT_FOUND"));
                                let body = serde_json::to_string(&body)
                                    .expect("impossible to fail to serialize");
                                *response.body_mut() = Body::from(body);
                            }
                            GetUeCtxInAmfDataResponse::InternalServerError(body) => {
                                *response.status_mut() = StatusCode::from_u16(500)
                                    .expect("Unable to turn 500 into a StatusCode");
                                response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for GET_UE_CTX_IN_AMF_DATA_INTERNAL_SERVER_ERROR"));
                                let body = serde_json::to_string(&body)
                                    .expect("impossible to fail to serialize");
                                *response.body_mut() = Body::from(body);
                            }
                            GetUeCtxInAmfDataResponse::ServiceUnavailable(body) => {
                                *response.status_mut() = StatusCode::from_u16(503)
                                    .expect("Unable to turn 503 into a StatusCode");
                                response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for GET_UE_CTX_IN_AMF_DATA_SERVICE_UNAVAILABLE"));
                                let body = serde_json::to_string(&body)
                                    .expect("impossible to fail to serialize");
                                *response.body_mut() = Body::from(body);
                            }
                            GetUeCtxInAmfDataResponse::UnexpectedError => {
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

                // GetUeCtxInSmfData - GET /{supi}/ue-context-in-smf-data
                hyper::Method::GET if path.matched(paths::ID_SUPI_UE_CONTEXT_IN_SMF_DATA) => {
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
                                "nudm-sdm".to_string(), // Access to the nudm-sdm API
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
                    paths::REGEX_SUPI_UE_CONTEXT_IN_SMF_DATA
                    .captures(path)
                    .unwrap_or_else(||
                        panic!("Path {} matched RE SUPI_UE_CONTEXT_IN_SMF_DATA in set but failed match against \"{}\"", path, paths::REGEX_SUPI_UE_CONTEXT_IN_SMF_DATA.as_str())
                    );

                    let param_supi = match percent_encoding::percent_decode(path_params["supi"].as_bytes()).decode_utf8() {
                    Ok(param_supi) => match param_supi.parse::<String>() {
                        Ok(param_supi) => param_supi,
                        Err(e) => return Ok(Response::builder()
                                        .status(StatusCode::BAD_REQUEST)
                                        .body(Body::from(format!("Couldn't parse path parameter supi: {}", e)))
                                        .expect("Unable to create Bad Request response for invalid path parameter")),
                    },
                    Err(_) => return Ok(Response::builder()
                                        .status(StatusCode::BAD_REQUEST)
                                        .body(Body::from(format!("Couldn't percent-decode path parameter as UTF-8: {}", &path_params["supi"])))
                                        .expect("Unable to create Bad Request response for invalid percent decode"))
                };

                    // Query parameters (note that non-required or collection query parameters will ignore garbage values, rather than causing a 400 response)
                    let query_params =
                        form_urlencoded::parse(uri.query().unwrap_or_default().as_bytes())
                            .collect::<Vec<_>>();
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

                    let result = api_impl
                        .get_ue_ctx_in_smf_data(param_supi, param_supported_features, &context)
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
                            GetUeCtxInSmfDataResponse::ExpectedResponseToAValidRequest(body) => {
                                *response.status_mut() = StatusCode::from_u16(200)
                                    .expect("Unable to turn 200 into a StatusCode");
                                response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for GET_UE_CTX_IN_SMF_DATA_EXPECTED_RESPONSE_TO_A_VALID_REQUEST"));
                                let body = serde_json::to_string(&body)
                                    .expect("impossible to fail to serialize");
                                *response.body_mut() = Body::from(body);
                            }
                            GetUeCtxInSmfDataResponse::BadRequest(body) => {
                                *response.status_mut() = StatusCode::from_u16(400)
                                    .expect("Unable to turn 400 into a StatusCode");
                                response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for GET_UE_CTX_IN_SMF_DATA_BAD_REQUEST"));
                                let body = serde_json::to_string(&body)
                                    .expect("impossible to fail to serialize");
                                *response.body_mut() = Body::from(body);
                            }
                            GetUeCtxInSmfDataResponse::NotFound(body) => {
                                *response.status_mut() = StatusCode::from_u16(404)
                                    .expect("Unable to turn 404 into a StatusCode");
                                response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for GET_UE_CTX_IN_SMF_DATA_NOT_FOUND"));
                                let body = serde_json::to_string(&body)
                                    .expect("impossible to fail to serialize");
                                *response.body_mut() = Body::from(body);
                            }
                            GetUeCtxInSmfDataResponse::InternalServerError(body) => {
                                *response.status_mut() = StatusCode::from_u16(500)
                                    .expect("Unable to turn 500 into a StatusCode");
                                response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for GET_UE_CTX_IN_SMF_DATA_INTERNAL_SERVER_ERROR"));
                                let body = serde_json::to_string(&body)
                                    .expect("impossible to fail to serialize");
                                *response.body_mut() = Body::from(body);
                            }
                            GetUeCtxInSmfDataResponse::ServiceUnavailable(body) => {
                                *response.status_mut() = StatusCode::from_u16(503)
                                    .expect("Unable to turn 503 into a StatusCode");
                                response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for GET_UE_CTX_IN_SMF_DATA_SERVICE_UNAVAILABLE"));
                                let body = serde_json::to_string(&body)
                                    .expect("impossible to fail to serialize");
                                *response.body_mut() = Body::from(body);
                            }
                            GetUeCtxInSmfDataResponse::UnexpectedError => {
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

                // GetUeCtxInSmsfData - GET /{supi}/ue-context-in-smsf-data
                hyper::Method::GET if path.matched(paths::ID_SUPI_UE_CONTEXT_IN_SMSF_DATA) => {
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
                                "nudm-sdm".to_string(), // Access to the nudm-sdm API
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
                    paths::REGEX_SUPI_UE_CONTEXT_IN_SMSF_DATA
                    .captures(path)
                    .unwrap_or_else(||
                        panic!("Path {} matched RE SUPI_UE_CONTEXT_IN_SMSF_DATA in set but failed match against \"{}\"", path, paths::REGEX_SUPI_UE_CONTEXT_IN_SMSF_DATA.as_str())
                    );

                    let param_supi = match percent_encoding::percent_decode(path_params["supi"].as_bytes()).decode_utf8() {
                    Ok(param_supi) => match param_supi.parse::<String>() {
                        Ok(param_supi) => param_supi,
                        Err(e) => return Ok(Response::builder()
                                        .status(StatusCode::BAD_REQUEST)
                                        .body(Body::from(format!("Couldn't parse path parameter supi: {}", e)))
                                        .expect("Unable to create Bad Request response for invalid path parameter")),
                    },
                    Err(_) => return Ok(Response::builder()
                                        .status(StatusCode::BAD_REQUEST)
                                        .body(Body::from(format!("Couldn't percent-decode path parameter as UTF-8: {}", &path_params["supi"])))
                                        .expect("Unable to create Bad Request response for invalid percent decode"))
                };

                    // Query parameters (note that non-required or collection query parameters will ignore garbage values, rather than causing a 400 response)
                    let query_params =
                        form_urlencoded::parse(uri.query().unwrap_or_default().as_bytes())
                            .collect::<Vec<_>>();
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

                    let result = api_impl
                        .get_ue_ctx_in_smsf_data(param_supi, param_supported_features, &context)
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
                            GetUeCtxInSmsfDataResponse::ExpectedResponseToAValidRequest(body) => {
                                *response.status_mut() = StatusCode::from_u16(200)
                                    .expect("Unable to turn 200 into a StatusCode");
                                response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for GET_UE_CTX_IN_SMSF_DATA_EXPECTED_RESPONSE_TO_A_VALID_REQUEST"));
                                let body = serde_json::to_string(&body)
                                    .expect("impossible to fail to serialize");
                                *response.body_mut() = Body::from(body);
                            }
                            GetUeCtxInSmsfDataResponse::BadRequest(body) => {
                                *response.status_mut() = StatusCode::from_u16(400)
                                    .expect("Unable to turn 400 into a StatusCode");
                                response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for GET_UE_CTX_IN_SMSF_DATA_BAD_REQUEST"));
                                let body = serde_json::to_string(&body)
                                    .expect("impossible to fail to serialize");
                                *response.body_mut() = Body::from(body);
                            }
                            GetUeCtxInSmsfDataResponse::NotFound(body) => {
                                *response.status_mut() = StatusCode::from_u16(404)
                                    .expect("Unable to turn 404 into a StatusCode");
                                response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for GET_UE_CTX_IN_SMSF_DATA_NOT_FOUND"));
                                let body = serde_json::to_string(&body)
                                    .expect("impossible to fail to serialize");
                                *response.body_mut() = Body::from(body);
                            }
                            GetUeCtxInSmsfDataResponse::InternalServerError(body) => {
                                *response.status_mut() = StatusCode::from_u16(500)
                                    .expect("Unable to turn 500 into a StatusCode");
                                response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for GET_UE_CTX_IN_SMSF_DATA_INTERNAL_SERVER_ERROR"));
                                let body = serde_json::to_string(&body)
                                    .expect("impossible to fail to serialize");
                                *response.body_mut() = Body::from(body);
                            }
                            GetUeCtxInSmsfDataResponse::ServiceUnavailable(body) => {
                                *response.status_mut() = StatusCode::from_u16(503)
                                    .expect("Unable to turn 503 into a StatusCode");
                                response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for GET_UE_CTX_IN_SMSF_DATA_SERVICE_UNAVAILABLE"));
                                let body = serde_json::to_string(&body)
                                    .expect("impossible to fail to serialize");
                                *response.body_mut() = Body::from(body);
                            }
                            GetUeCtxInSmsfDataResponse::UnexpectedError => {
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

                // GetUcData - GET /{supi}/uc-data
                hyper::Method::GET if path.matched(paths::ID_SUPI_UC_DATA) => {
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
                                "nudm-sdm".to_string(), // Access to the nudm-sdm API
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
                    paths::REGEX_SUPI_UC_DATA
                    .captures(path)
                    .unwrap_or_else(||
                        panic!("Path {} matched RE SUPI_UC_DATA in set but failed match against \"{}\"", path, paths::REGEX_SUPI_UC_DATA.as_str())
                    );

                    let param_supi = match percent_encoding::percent_decode(path_params["supi"].as_bytes()).decode_utf8() {
                    Ok(param_supi) => match param_supi.parse::<String>() {
                        Ok(param_supi) => param_supi,
                        Err(e) => return Ok(Response::builder()
                                        .status(StatusCode::BAD_REQUEST)
                                        .body(Body::from(format!("Couldn't parse path parameter supi: {}", e)))
                                        .expect("Unable to create Bad Request response for invalid path parameter")),
                    },
                    Err(_) => return Ok(Response::builder()
                                        .status(StatusCode::BAD_REQUEST)
                                        .body(Body::from(format!("Couldn't percent-decode path parameter as UTF-8: {}", &path_params["supi"])))
                                        .expect("Unable to create Bad Request response for invalid percent decode"))
                };

                    // Header parameters
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
                    let param_if_modified_since =
                        headers.get(HeaderName::from_static("if-modified-since"));

                    let param_if_modified_since = match param_if_modified_since {
                        Some(v) => {
                            match header::IntoHeaderValue::<String>::try_from((*v).clone()) {
                                Ok(result) => Some(result.0),
                                Err(err) => {
                                    return Ok(Response::builder()
                                        .status(StatusCode::BAD_REQUEST)
                                        .body(Body::from(format!("Invalid header If-Modified-Since - {}", err)))
                                        .expect("Unable to create Bad Request response for invalid header If-Modified-Since"));
                                }
                            }
                        }
                        None => None,
                    };

                    // Query parameters (note that non-required or collection query parameters will ignore garbage values, rather than causing a 400 response)
                    let query_params =
                        form_urlencoded::parse(uri.query().unwrap_or_default().as_bytes())
                            .collect::<Vec<_>>();
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
                    let param_uc_purpose = query_params
                        .iter()
                        .filter(|e| e.0 == "uc-purpose")
                        .map(|e| e.1.to_owned())
                        .next();
                    let param_uc_purpose = match param_uc_purpose {
                        Some(param_uc_purpose) => {
                            let param_uc_purpose =
                                <models::UcPurpose as std::str::FromStr>::from_str(
                                    &param_uc_purpose,
                                );
                            match param_uc_purpose {
                            Ok(param_uc_purpose) => Some(param_uc_purpose),
                            Err(e) => return Ok(Response::builder()
                                .status(StatusCode::BAD_REQUEST)
                                .body(Body::from(format!("Couldn't parse query parameter uc-purpose - doesn't match schema: {}", e)))
                                .expect("Unable to create Bad Request response for invalid query parameter uc-purpose")),
                        }
                        }
                        None => None,
                    };

                    let result = api_impl
                        .get_uc_data(
                            param_supi,
                            param_supported_features,
                            param_uc_purpose,
                            param_if_none_match,
                            param_if_modified_since,
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
                            GetUcDataResponse::ExpectedResponseToAValidRequest {
                                body,
                                cache_control,
                                e_tag,
                                last_modified,
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
                                if let Some(last_modified) = last_modified {
                                    let last_modified = match header::IntoHeaderValue(last_modified).try_into() {
                                                        Ok(val) => val,
                                                        Err(e) => {
                                                            return Ok(Response::builder()
                                                                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                                                                    .body(Body::from(format!("An internal server error occurred handling last_modified header - {}", e)))
                                                                    .expect("Unable to create Internal Server Error for invalid response header"))
                                                        }
                                                    };

                                    response.headers_mut().insert(
                                        HeaderName::from_static("last-modified"),
                                        last_modified,
                                    );
                                }
                                *response.status_mut() = StatusCode::from_u16(200)
                                    .expect("Unable to turn 200 into a StatusCode");
                                response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for GET_UC_DATA_EXPECTED_RESPONSE_TO_A_VALID_REQUEST"));
                                let body = serde_json::to_string(&body)
                                    .expect("impossible to fail to serialize");
                                *response.body_mut() = Body::from(body);
                            }
                            GetUcDataResponse::BadRequest(body) => {
                                *response.status_mut() = StatusCode::from_u16(400)
                                    .expect("Unable to turn 400 into a StatusCode");
                                response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for GET_UC_DATA_BAD_REQUEST"));
                                let body = serde_json::to_string(&body)
                                    .expect("impossible to fail to serialize");
                                *response.body_mut() = Body::from(body);
                            }
                            GetUcDataResponse::NotFound(body) => {
                                *response.status_mut() = StatusCode::from_u16(404)
                                    .expect("Unable to turn 404 into a StatusCode");
                                response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for GET_UC_DATA_NOT_FOUND"));
                                let body = serde_json::to_string(&body)
                                    .expect("impossible to fail to serialize");
                                *response.body_mut() = Body::from(body);
                            }
                            GetUcDataResponse::InternalServerError(body) => {
                                *response.status_mut() = StatusCode::from_u16(500)
                                    .expect("Unable to turn 500 into a StatusCode");
                                response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for GET_UC_DATA_INTERNAL_SERVER_ERROR"));
                                let body = serde_json::to_string(&body)
                                    .expect("impossible to fail to serialize");
                                *response.body_mut() = Body::from(body);
                            }
                            GetUcDataResponse::ServiceUnavailable(body) => {
                                *response.status_mut() = StatusCode::from_u16(503)
                                    .expect("Unable to turn 503 into a StatusCode");
                                response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for GET_UC_DATA_SERVICE_UNAVAILABLE"));
                                let body = serde_json::to_string(&body)
                                    .expect("impossible to fail to serialize");
                                *response.body_mut() = Body::from(body);
                            }
                            GetUcDataResponse::UnexpectedError => {
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

                // GetV2xData - GET /{supi}/v2x-data
                hyper::Method::GET if path.matched(paths::ID_SUPI_V2X_DATA) => {
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
                                "nudm-sdm".to_string(), // Access to the nudm-sdm API
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
                    paths::REGEX_SUPI_V2X_DATA
                    .captures(path)
                    .unwrap_or_else(||
                        panic!("Path {} matched RE SUPI_V2X_DATA in set but failed match against \"{}\"", path, paths::REGEX_SUPI_V2X_DATA.as_str())
                    );

                    let param_supi = match percent_encoding::percent_decode(path_params["supi"].as_bytes()).decode_utf8() {
                    Ok(param_supi) => match param_supi.parse::<String>() {
                        Ok(param_supi) => param_supi,
                        Err(e) => return Ok(Response::builder()
                                        .status(StatusCode::BAD_REQUEST)
                                        .body(Body::from(format!("Couldn't parse path parameter supi: {}", e)))
                                        .expect("Unable to create Bad Request response for invalid path parameter")),
                    },
                    Err(_) => return Ok(Response::builder()
                                        .status(StatusCode::BAD_REQUEST)
                                        .body(Body::from(format!("Couldn't percent-decode path parameter as UTF-8: {}", &path_params["supi"])))
                                        .expect("Unable to create Bad Request response for invalid percent decode"))
                };

                    // Header parameters
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
                    let param_if_modified_since =
                        headers.get(HeaderName::from_static("if-modified-since"));

                    let param_if_modified_since = match param_if_modified_since {
                        Some(v) => {
                            match header::IntoHeaderValue::<String>::try_from((*v).clone()) {
                                Ok(result) => Some(result.0),
                                Err(err) => {
                                    return Ok(Response::builder()
                                        .status(StatusCode::BAD_REQUEST)
                                        .body(Body::from(format!("Invalid header If-Modified-Since - {}", err)))
                                        .expect("Unable to create Bad Request response for invalid header If-Modified-Since"));
                                }
                            }
                        }
                        None => None,
                    };

                    // Query parameters (note that non-required or collection query parameters will ignore garbage values, rather than causing a 400 response)
                    let query_params =
                        form_urlencoded::parse(uri.query().unwrap_or_default().as_bytes())
                            .collect::<Vec<_>>();
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

                    let result = api_impl
                        .get_v2x_data(
                            param_supi,
                            param_supported_features,
                            param_if_none_match,
                            param_if_modified_since,
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
                            GetV2xDataResponse::ExpectedResponseToAValidRequest {
                                body,
                                cache_control,
                                e_tag,
                                last_modified,
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
                                if let Some(last_modified) = last_modified {
                                    let last_modified = match header::IntoHeaderValue(last_modified).try_into() {
                                                        Ok(val) => val,
                                                        Err(e) => {
                                                            return Ok(Response::builder()
                                                                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                                                                    .body(Body::from(format!("An internal server error occurred handling last_modified header - {}", e)))
                                                                    .expect("Unable to create Internal Server Error for invalid response header"))
                                                        }
                                                    };

                                    response.headers_mut().insert(
                                        HeaderName::from_static("last-modified"),
                                        last_modified,
                                    );
                                }
                                *response.status_mut() = StatusCode::from_u16(200)
                                    .expect("Unable to turn 200 into a StatusCode");
                                response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/json")
                                                            .expect("Unable to create Content-Type header for GET_V2X_DATA_EXPECTED_RESPONSE_TO_A_VALID_REQUEST"));
                                let body = serde_json::to_string(&body)
                                    .expect("impossible to fail to serialize");
                                *response.body_mut() = Body::from(body);
                            }
                            GetV2xDataResponse::BadRequest(body) => {
                                *response.status_mut() = StatusCode::from_u16(400)
                                    .expect("Unable to turn 400 into a StatusCode");
                                response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for GET_V2X_DATA_BAD_REQUEST"));
                                let body = serde_json::to_string(&body)
                                    .expect("impossible to fail to serialize");
                                *response.body_mut() = Body::from(body);
                            }
                            GetV2xDataResponse::NotFound(body) => {
                                *response.status_mut() = StatusCode::from_u16(404)
                                    .expect("Unable to turn 404 into a StatusCode");
                                response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for GET_V2X_DATA_NOT_FOUND"));
                                let body = serde_json::to_string(&body)
                                    .expect("impossible to fail to serialize");
                                *response.body_mut() = Body::from(body);
                            }
                            GetV2xDataResponse::InternalServerError(body) => {
                                *response.status_mut() = StatusCode::from_u16(500)
                                    .expect("Unable to turn 500 into a StatusCode");
                                response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for GET_V2X_DATA_INTERNAL_SERVER_ERROR"));
                                let body = serde_json::to_string(&body)
                                    .expect("impossible to fail to serialize");
                                *response.body_mut() = Body::from(body);
                            }
                            GetV2xDataResponse::ServiceUnavailable(body) => {
                                *response.status_mut() = StatusCode::from_u16(503)
                                    .expect("Unable to turn 503 into a StatusCode");
                                response.headers_mut().insert(
                                                        CONTENT_TYPE,
                                                        HeaderValue::from_str("application/problem+json")
                                                            .expect("Unable to create Content-Type header for GET_V2X_DATA_SERVICE_UNAVAILABLE"));
                                let body = serde_json::to_string(&body)
                                    .expect("impossible to fail to serialize");
                                *response.body_mut() = Body::from(body);
                            }
                            GetV2xDataResponse::UnexpectedError => {
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

                _ if path.matched(paths::ID_GROUP_DATA_GROUP_IDENTIFIERS) => method_not_allowed(),
                _ if path.matched(paths::ID_MULTIPLE_IDENTIFIERS) => method_not_allowed(),
                _ if path.matched(paths::ID_SHARED_DATA) => method_not_allowed(),
                _ if path.matched(paths::ID_SHARED_DATA_SUBSCRIPTIONS) => method_not_allowed(),
                _ if path.matched(paths::ID_SHARED_DATA_SUBSCRIPTIONS_SUBSCRIPTIONID) => {
                    method_not_allowed()
                }
                _ if path.matched(paths::ID_SHARED_DATA_SHAREDDATAID) => method_not_allowed(),
                _ if path.matched(paths::ID_SUPI) => method_not_allowed(),
                _ if path.matched(paths::ID_SUPI_5MBS_DATA) => method_not_allowed(),
                _ if path.matched(paths::ID_SUPI_AM_DATA) => method_not_allowed(),
                _ if path.matched(paths::ID_SUPI_AM_DATA_CAG_ACK) => method_not_allowed(),
                _ if path.matched(paths::ID_SUPI_AM_DATA_ECR_DATA) => method_not_allowed(),
                _ if path.matched(paths::ID_SUPI_AM_DATA_SOR_ACK) => method_not_allowed(),
                _ if path.matched(paths::ID_SUPI_AM_DATA_SUBSCRIBED_SNSSAIS_ACK) => {
                    method_not_allowed()
                }
                _ if path.matched(paths::ID_SUPI_AM_DATA_UPDATE_SOR) => method_not_allowed(),
                _ if path.matched(paths::ID_SUPI_AM_DATA_UPU_ACK) => method_not_allowed(),
                _ if path.matched(paths::ID_SUPI_LCS_BCA_DATA) => method_not_allowed(),
                _ if path.matched(paths::ID_SUPI_LCS_MO_DATA) => method_not_allowed(),
                _ if path.matched(paths::ID_SUPI_NSSAI) => method_not_allowed(),
                _ if path.matched(paths::ID_SUPI_PROSE_DATA) => method_not_allowed(),
                _ if path.matched(paths::ID_SUPI_SM_DATA) => method_not_allowed(),
                _ if path.matched(paths::ID_SUPI_SMF_SELECT_DATA) => method_not_allowed(),
                _ if path.matched(paths::ID_SUPI_SMS_DATA) => method_not_allowed(),
                _ if path.matched(paths::ID_SUPI_SMS_MNG_DATA) => method_not_allowed(),
                _ if path.matched(paths::ID_SUPI_TRACE_DATA) => method_not_allowed(),
                _ if path.matched(paths::ID_SUPI_UC_DATA) => method_not_allowed(),
                _ if path.matched(paths::ID_SUPI_UE_CONTEXT_IN_AMF_DATA) => method_not_allowed(),
                _ if path.matched(paths::ID_SUPI_UE_CONTEXT_IN_SMF_DATA) => method_not_allowed(),
                _ if path.matched(paths::ID_SUPI_UE_CONTEXT_IN_SMSF_DATA) => method_not_allowed(),
                _ if path.matched(paths::ID_SUPI_V2X_DATA) => method_not_allowed(),
                _ if path.matched(paths::ID_UEID_ID_TRANSLATION_RESULT) => method_not_allowed(),
                _ if path.matched(paths::ID_UEID_LCS_PRIVACY_DATA) => method_not_allowed(),
                _ if path.matched(paths::ID_UEID_SDM_SUBSCRIPTIONS) => method_not_allowed(),
                _ if path.matched(paths::ID_UEID_SDM_SUBSCRIPTIONS_SUBSCRIPTIONID) => {
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
            // GetAmData - GET /{supi}/am-data
            hyper::Method::GET if path.matched(paths::ID_SUPI_AM_DATA) => Some("GetAmData"),
            // GetMbsData - GET /{supi}/5mbs-data
            hyper::Method::GET if path.matched(paths::ID_SUPI_5MBS_DATA) => Some("GetMbsData"),
            // GetEcrData - GET /{supi}/am-data/ecr-data
            hyper::Method::GET if path.matched(paths::ID_SUPI_AM_DATA_ECR_DATA) => {
                Some("GetEcrData")
            }
            // GetSupiOrGpsi - GET /{ueId}/id-translation-result
            hyper::Method::GET if path.matched(paths::ID_UEID_ID_TRANSLATION_RESULT) => {
                Some("GetSupiOrGpsi")
            }
            // GetGroupIdentifiers - GET /group-data/group-identifiers
            hyper::Method::GET if path.matched(paths::ID_GROUP_DATA_GROUP_IDENTIFIERS) => {
                Some("GetGroupIdentifiers")
            }
            // GetLcsBcaData - GET /{supi}/lcs-bca-data
            hyper::Method::GET if path.matched(paths::ID_SUPI_LCS_BCA_DATA) => {
                Some("GetLcsBcaData")
            }
            // GetLcsMoData - GET /{supi}/lcs-mo-data
            hyper::Method::GET if path.matched(paths::ID_SUPI_LCS_MO_DATA) => Some("GetLcsMoData"),
            // GetLcsPrivacyData - GET /{ueId}/lcs-privacy-data
            hyper::Method::GET if path.matched(paths::ID_UEID_LCS_PRIVACY_DATA) => {
                Some("GetLcsPrivacyData")
            }
            // GetMultipleIdentifiers - GET /multiple-identifiers
            hyper::Method::GET if path.matched(paths::ID_MULTIPLE_IDENTIFIERS) => {
                Some("GetMultipleIdentifiers")
            }
            // GetProseData - GET /{supi}/prose-data
            hyper::Method::GET if path.matched(paths::ID_SUPI_PROSE_DATA) => Some("GetProseData"),
            // CAgAck - PUT /{supi}/am-data/cag-ack
            hyper::Method::PUT if path.matched(paths::ID_SUPI_AM_DATA_CAG_ACK) => Some("CAgAck"),
            // SNssaisAck - PUT /{supi}/am-data/subscribed-snssais-ack
            hyper::Method::PUT if path.matched(paths::ID_SUPI_AM_DATA_SUBSCRIBED_SNSSAIS_ACK) => {
                Some("SNssaisAck")
            }
            // SorAckInfo - PUT /{supi}/am-data/sor-ack
            hyper::Method::PUT if path.matched(paths::ID_SUPI_AM_DATA_SOR_ACK) => {
                Some("SorAckInfo")
            }
            // UpuAck - PUT /{supi}/am-data/upu-ack
            hyper::Method::PUT if path.matched(paths::ID_SUPI_AM_DATA_UPU_ACK) => Some("UpuAck"),
            // GetDataSets - GET /{supi}
            hyper::Method::GET if path.matched(paths::ID_SUPI) => Some("GetDataSets"),
            // GetSharedData - GET /shared-data
            hyper::Method::GET if path.matched(paths::ID_SHARED_DATA) => Some("GetSharedData"),
            // GetIndividualSharedData - GET /shared-data/{sharedDataId}
            hyper::Method::GET if path.matched(paths::ID_SHARED_DATA_SHAREDDATAID) => {
                Some("GetIndividualSharedData")
            }
            // GetSmfSelData - GET /{supi}/smf-select-data
            hyper::Method::GET if path.matched(paths::ID_SUPI_SMF_SELECT_DATA) => {
                Some("GetSmfSelData")
            }
            // GetSmsMngtData - GET /{supi}/sms-mng-data
            hyper::Method::GET if path.matched(paths::ID_SUPI_SMS_MNG_DATA) => {
                Some("GetSmsMngtData")
            }
            // GetSmsData - GET /{supi}/sms-data
            hyper::Method::GET if path.matched(paths::ID_SUPI_SMS_DATA) => Some("GetSmsData"),
            // GetSmData - GET /{supi}/sm-data
            hyper::Method::GET if path.matched(paths::ID_SUPI_SM_DATA) => Some("GetSmData"),
            // GetNssai - GET /{supi}/nssai
            hyper::Method::GET if path.matched(paths::ID_SUPI_NSSAI) => Some("GetNssai"),
            // Subscribe - POST /{ueId}/sdm-subscriptions
            hyper::Method::POST if path.matched(paths::ID_UEID_SDM_SUBSCRIPTIONS) => {
                Some("Subscribe")
            }
            // SubscribeToSharedData - POST /shared-data-subscriptions
            hyper::Method::POST if path.matched(paths::ID_SHARED_DATA_SUBSCRIPTIONS) => {
                Some("SubscribeToSharedData")
            }
            // Unsubscribe - DELETE /{ueId}/sdm-subscriptions/{subscriptionId}
            hyper::Method::DELETE
                if path.matched(paths::ID_UEID_SDM_SUBSCRIPTIONS_SUBSCRIPTIONID) =>
            {
                Some("Unsubscribe")
            }
            // UnsubscribeForSharedData - DELETE /shared-data-subscriptions/{subscriptionId}
            hyper::Method::DELETE
                if path.matched(paths::ID_SHARED_DATA_SUBSCRIPTIONS_SUBSCRIPTIONID) =>
            {
                Some("UnsubscribeForSharedData")
            }
            // Modify - PATCH /{ueId}/sdm-subscriptions/{subscriptionId}
            hyper::Method::PATCH
                if path.matched(paths::ID_UEID_SDM_SUBSCRIPTIONS_SUBSCRIPTIONID) =>
            {
                Some("Modify")
            }
            // ModifySharedDataSubs - PATCH /shared-data-subscriptions/{subscriptionId}
            hyper::Method::PATCH
                if path.matched(paths::ID_SHARED_DATA_SUBSCRIPTIONS_SUBSCRIPTIONID) =>
            {
                Some("ModifySharedDataSubs")
            }
            // GetTraceConfigData - GET /{supi}/trace-data
            hyper::Method::GET if path.matched(paths::ID_SUPI_TRACE_DATA) => {
                Some("GetTraceConfigData")
            }
            // UpdateSorInfo - POST /{supi}/am-data/update-sor
            hyper::Method::POST if path.matched(paths::ID_SUPI_AM_DATA_UPDATE_SOR) => {
                Some("UpdateSorInfo")
            }
            // GetUeCtxInAmfData - GET /{supi}/ue-context-in-amf-data
            hyper::Method::GET if path.matched(paths::ID_SUPI_UE_CONTEXT_IN_AMF_DATA) => {
                Some("GetUeCtxInAmfData")
            }
            // GetUeCtxInSmfData - GET /{supi}/ue-context-in-smf-data
            hyper::Method::GET if path.matched(paths::ID_SUPI_UE_CONTEXT_IN_SMF_DATA) => {
                Some("GetUeCtxInSmfData")
            }
            // GetUeCtxInSmsfData - GET /{supi}/ue-context-in-smsf-data
            hyper::Method::GET if path.matched(paths::ID_SUPI_UE_CONTEXT_IN_SMSF_DATA) => {
                Some("GetUeCtxInSmsfData")
            }
            // GetUcData - GET /{supi}/uc-data
            hyper::Method::GET if path.matched(paths::ID_SUPI_UC_DATA) => Some("GetUcData"),
            // GetV2xData - GET /{supi}/v2x-data
            hyper::Method::GET if path.matched(paths::ID_SUPI_V2X_DATA) => Some("GetV2xData"),
            _ => None,
        }
    }
}
