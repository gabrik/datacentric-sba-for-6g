#![allow(
    missing_docs,
    trivial_casts,
    unused_variables,
    unused_mut,
    unused_imports,
    unused_extern_crates,
    non_camel_case_types
)]
#![allow(unused_imports, unused_attributes)]
#![allow(clippy::derive_partial_eq_without_eq, clippy::blacklisted_name)]

use async_trait::async_trait;
use futures::Stream;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::task::{Context, Poll};
use swagger::{ApiError, ContextWrapper};

type ServiceError = Box<dyn Error + Send + Sync + 'static>;

pub const BASE_PATH: &str = "/nnrf-disc/v1";
pub const API_VERSION: &str = "1.2.2";

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[must_use]
pub enum RetrieveCompleteSearchResponse {
    /// Expected response to a valid request
    ExpectedResponseToAValidRequest {
        body: models::StoredSearchResult,
        cache_control: Option<String>,
        e_tag: Option<String>,
        content_encoding: Option<String>,
    },
    /// Temporary Redirect
    TemporaryRedirect {
        body: models::RedirectResponse,
        location: String,
    },
    /// Permanent Redirect
    PermanentRedirect {
        body: models::RedirectResponse,
        location: String,
    },
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[must_use]
pub enum ScpDomainRoutingInfoUnsubscribeResponse {
    /// Expected response to a successful subscription removal
    ExpectedResponseToASuccessfulSubscriptionRemoval,
    /// Bad request
    BadRequest(models::ProblemDetails),
    /// Unauthorized
    Unauthorized(models::ProblemDetails),
    /// Forbidden
    Forbidden(models::ProblemDetails),
    /// Not Found
    NotFound(models::ProblemDetails),
    /// Length Required
    LengthRequired(models::ProblemDetails),
    /// Payload Too Large
    PayloadTooLarge(models::ProblemDetails),
    /// Unsupported Media Type
    UnsupportedMediaType(models::ProblemDetails),
    /// Too Many Requests
    TooManyRequests(models::ProblemDetails),
    /// Internal Server Error
    InternalServerError(models::ProblemDetails),
    /// Not Implemented
    NotImplemented(models::ProblemDetails),
    /// Service Unavailable
    ServiceUnavailable(models::ProblemDetails),
    /// Generic Error
    GenericError,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[must_use]
pub enum SearchNfInstancesResponse {
    /// Expected response to a valid request
    ExpectedResponseToAValidRequest {
        body: models::SearchResult,
        cache_control: Option<String>,
        e_tag: Option<String>,
        content_encoding: Option<String>,
    },
    /// Temporary Redirect
    TemporaryRedirect {
        body: models::RedirectResponse,
        location: String,
    },
    /// Permanent Redirect
    PermanentRedirect {
        body: models::RedirectResponse,
        location: String,
    },
    /// Bad request
    BadRequest(models::ProblemDetails),
    /// Unauthorized
    Unauthorized(models::ProblemDetails),
    /// Forbidden
    Forbidden(models::ProblemDetails),
    /// Not Found
    NotFound(models::ProblemDetails),
    /// 406 Not Acceptable
    Status406,
    /// Length Required
    LengthRequired(models::ProblemDetails),
    /// Payload Too Large
    PayloadTooLarge(models::ProblemDetails),
    /// Unsupported Media Type
    UnsupportedMediaType(models::ProblemDetails),
    /// Too Many Requests
    TooManyRequests(models::ProblemDetails),
    /// Internal Server Error
    InternalServerError(models::ProblemDetails),
    /// Not Implemented
    NotImplemented(models::ProblemDetails),
    /// Service Unavailable
    ServiceUnavailable(models::ProblemDetails),
    /// Generic Error
    GenericError,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[must_use]
pub enum SCpDomainRoutingInfoGetResponse {
    /// Expected response to a valid request
    ExpectedResponseToAValidRequest {
        body: models::ScpDomainRoutingInformation,
        content_encoding: Option<String>,
    },
    /// Temporary Redirect
    TemporaryRedirect { location: String },
    /// Bad request
    BadRequest(models::ProblemDetails),
    /// Unauthorized
    Unauthorized(models::ProblemDetails),
    /// Forbidden
    Forbidden(models::ProblemDetails),
    /// Not Found
    NotFound(models::ProblemDetails),
    /// 406 Not Acceptable
    Status406,
    /// Length Required
    LengthRequired(models::ProblemDetails),
    /// Payload Too Large
    PayloadTooLarge(models::ProblemDetails),
    /// Unsupported Media Type
    UnsupportedMediaType(models::ProblemDetails),
    /// Too Many Requests
    TooManyRequests(models::ProblemDetails),
    /// Internal Server Error
    InternalServerError(models::ProblemDetails),
    /// Not Implemented
    NotImplemented(models::ProblemDetails),
    /// Service Unavailable
    ServiceUnavailable(models::ProblemDetails),
    /// Generic Error
    GenericError,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[must_use]
pub enum ScpDomainRoutingInfoSubscribeResponse {
    /// Expected response to a valid request
    ExpectedResponseToAValidRequest {
        body: models::ScpDomainRoutingInfoSubscription,
        location: String,
        accept_encoding: Option<String>,
        content_encoding: Option<String>,
    },
    /// Bad request
    BadRequest(models::ProblemDetails),
    /// Unauthorized
    Unauthorized(models::ProblemDetails),
    /// Forbidden
    Forbidden(models::ProblemDetails),
    /// Not Found
    NotFound(models::ProblemDetails),
    /// Length Required
    LengthRequired(models::ProblemDetails),
    /// Payload Too Large
    PayloadTooLarge(models::ProblemDetails),
    /// Unsupported Media Type
    UnsupportedMediaType(models::ProblemDetails),
    /// Too Many Requests
    TooManyRequests(models::ProblemDetails),
    /// Internal Server Error
    InternalServerError(models::ProblemDetails),
    /// Not Implemented
    NotImplemented(models::ProblemDetails),
    /// Service Unavailable
    ServiceUnavailable(models::ProblemDetails),
    /// Generic Error
    GenericError,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[must_use]
pub enum RetrieveStoredSearchResponse {
    /// Expected response to a valid request
    ExpectedResponseToAValidRequest {
        body: models::StoredSearchResult,
        cache_control: Option<String>,
        e_tag: Option<String>,
        content_encoding: Option<String>,
    },
    /// Temporary Redirect
    TemporaryRedirect {
        body: models::RedirectResponse,
        location: String,
    },
    /// Permanent Redirect
    PermanentRedirect {
        body: models::RedirectResponse,
        location: String,
    },
}

/// API
#[async_trait]
#[allow(clippy::too_many_arguments, clippy::ptr_arg)]
pub trait Api<C: Send + Sync> {
    fn poll_ready(
        &self,
        _cx: &mut Context,
    ) -> Poll<Result<(), Box<dyn Error + Send + Sync + 'static>>> {
        Poll::Ready(Ok(()))
    }

    async fn retrieve_complete_search(
        &self,
        search_id: String,
        accept_encoding: Option<String>,
        context: &C,
    ) -> Result<RetrieveCompleteSearchResponse, ApiError>;

    /// Deletes a subscription
    async fn scp_domain_routing_info_unsubscribe(
        &self,
        subscription_id: String,
        context: &C,
    ) -> Result<ScpDomainRoutingInfoUnsubscribeResponse, ApiError>;

    /// Search a collection of NF Instances
    async fn search_nf_instances(
        &self,
        target_nf_type: models::NfType,
        requester_nf_type: models::NfType,
        accept_encoding: Option<String>,
        preferred_collocated_nf_types: Option<&Vec<models::CollocatedNfType>>,
        requester_nf_instance_id: Option<uuid::Uuid>,
        service_names: Option<&Vec<models::ServiceName>>,
        requester_nf_instance_fqdn: Option<String>,
        target_plmn_list: Option<&Vec<models::PlmnId>>,
        requester_plmn_list: Option<&Vec<models::PlmnId>>,
        target_nf_instance_id: Option<uuid::Uuid>,
        target_nf_fqdn: Option<String>,
        hnrf_uri: Option<String>,
        snssais: Option<&Vec<models::Snssai>>,
        requester_snssais: Option<&Vec<models::ExtSnssai>>,
        plmn_specific_snssai_list: Option<&Vec<models::PlmnSnssai>>,
        requester_plmn_specific_snssai_list: Option<&Vec<models::PlmnSnssai>>,
        dnn: Option<String>,
        ipv4_index: Option<models::IpIndex>,
        ipv6_index: Option<models::IpIndex>,
        nsi_list: Option<&Vec<String>>,
        smf_serving_area: Option<String>,
        mbsmf_serving_area: Option<String>,
        tai: Option<models::Tai>,
        amf_region_id: Option<String>,
        amf_set_id: Option<String>,
        guami: Option<models::Guami>,
        supi: Option<String>,
        ue_ipv4_address: Option<String>,
        ip_domain: Option<String>,
        ue_ipv6_prefix: Option<models::Ipv6Prefix>,
        pgw_ind: Option<bool>,
        preferred_pgw_ind: Option<bool>,
        pgw: Option<String>,
        pgw_ip: Option<models::IpAddr>,
        gpsi: Option<String>,
        external_group_identity: Option<String>,
        internal_group_identity: Option<String>,
        pfd_data: Option<models::PfdData>,
        data_set: Option<models::DataSetId>,
        routing_indicator: Option<String>,
        group_id_list: Option<&Vec<models::NfGroupId>>,
        dnai_list: Option<&Vec<models::Dnai>>,
        pdu_session_types: Option<&Vec<models::PduSessionType>>,
        event_id_list: Option<&Vec<models::EventId>>,
        nwdaf_event_list: Option<&Vec<models::NwdafEvent>>,
        supported_features: Option<String>,
        upf_iwk_eps_ind: Option<bool>,
        chf_supported_plmn: Option<models::PlmnId>,
        preferred_locality: Option<String>,
        access_type: Option<models::AccessType>,
        limit: Option<i32>,
        required_features: Option<&Vec<models::SupportedFeatures>>,
        complex_query: Option<models::ComplexQuery>,
        max_payload_size: Option<i32>,
        max_payload_size_ext: Option<i32>,
        atsss_capability: Option<models::AtsssCapability>,
        upf_ue_ip_addr_ind: Option<bool>,
        client_type: Option<models::ExternalClientType>,
        lmf_id: Option<String>,
        an_node_type: Option<models::AnNodeType>,
        rat_type: Option<models::RatType>,
        preferred_tai: Option<models::Tai>,
        preferred_nf_instances: Option<&Vec<models::NfInstanceId>>,
        if_none_match: Option<String>,
        target_snpn: Option<models::PlmnIdNid>,
        requester_snpn_list: Option<&Vec<models::PlmnIdNid>>,
        af_ee_data: Option<models::AfEventExposureData>,
        w_agf_info: Option<models::WAgfInfo>,
        tngf_info: Option<models::TngfInfo>,
        twif_info: Option<models::TwifInfo>,
        target_nf_set_id: Option<String>,
        target_nf_service_set_id: Option<String>,
        nef_id: Option<String>,
        notification_type: Option<models::NotificationType>,
        n1_msg_class: Option<models::N1MessageClass>,
        n2_info_class: Option<models::N2InformationClass>,
        serving_scope: Option<&Vec<String>>,
        imsi: Option<String>,
        ims_private_identity: Option<String>,
        ims_public_identity: Option<String>,
        msisdn: Option<String>,
        preferred_api_versions: Option<std::collections::HashMap<String, String>>,
        v2x_support_ind: Option<bool>,
        redundant_gtpu: Option<bool>,
        redundant_transport: Option<bool>,
        ipups: Option<bool>,
        scp_domain_list: Option<&Vec<String>>,
        address_domain: Option<String>,
        ipv4_addr: Option<String>,
        ipv6_prefix: Option<models::Ipv6Prefix>,
        served_nf_set_id: Option<String>,
        remote_plmn_id: Option<models::PlmnId>,
        remote_snpn_id: Option<models::PlmnIdNid>,
        data_forwarding: Option<bool>,
        preferred_full_plmn: Option<bool>,
        requester_features: Option<String>,
        realm_id: Option<String>,
        storage_id: Option<String>,
        vsmf_support_ind: Option<bool>,
        ismf_support_ind: Option<bool>,
        nrf_disc_uri: Option<String>,
        preferred_vendor_specific_features: Option<
            std::collections::HashMap<
                String,
                std::collections::HashMap<String, Vec<models::VendorSpecificFeature>>,
            >,
        >,
        preferred_vendor_specific_nf_features: Option<
            std::collections::HashMap<String, Vec<models::VendorSpecificFeature>>,
        >,
        required_pfcp_features: Option<String>,
        home_pub_key_id: Option<i32>,
        prose_support_ind: Option<bool>,
        analytics_aggregation_ind: Option<bool>,
        serving_nf_set_id: Option<String>,
        serving_nf_type: Option<models::NfType>,
        ml_analytics_info_list: Option<&Vec<models::MlAnalyticsInfo>>,
        analytics_metadata_prov_ind: Option<bool>,
        nsacf_capability: Option<models::NsacfCapability>,
        mbs_session_id_list: Option<&Vec<models::MbsSessionId>>,
        area_session_id: Option<i32>,
        gmlc_number: Option<String>,
        upf_n6_ip: Option<models::IpAddr>,
        tai_list: Option<&Vec<models::Tai>>,
        preferences_precedence: Option<&Vec<String>>,
        support_onboarding_capability: Option<bool>,
        uas_nf_functionality_ind: Option<bool>,
        v2x_capability: Option<models::V2xCapability>,
        prose_capability: Option<models::ProSeCapability>,
        shared_data_id: Option<String>,
        target_hni: Option<String>,
        target_nw_resolution: Option<bool>,
        exclude_nfinst_list: Option<&Vec<models::NfInstanceId>>,
        exclude_nfservinst_list: Option<&Vec<models::NfServiceInstance>>,
        exclude_nfserviceset_list: Option<&Vec<models::NfServiceSetId>>,
        exclude_nfset_list: Option<&Vec<models::NfSetId>>,
        preferred_analytics_delays: Option<std::collections::HashMap<String, models::DurationSec>>,
        context: &C,
    ) -> Result<SearchNfInstancesResponse, ApiError>;

    async fn scp_domain_routing_info_get(
        &self,
        local: Option<bool>,
        accept_encoding: Option<String>,
        context: &C,
    ) -> Result<SCpDomainRoutingInfoGetResponse, ApiError>;

    /// Create a new subscription
    async fn scp_domain_routing_info_subscribe(
        &self,
        scp_domain_routing_info_subscription: models::ScpDomainRoutingInfoSubscription,
        content_encoding: Option<String>,
        accept_encoding: Option<String>,
        context: &C,
    ) -> Result<ScpDomainRoutingInfoSubscribeResponse, ApiError>;

    async fn retrieve_stored_search(
        &self,
        search_id: String,
        accept_encoding: Option<String>,
        context: &C,
    ) -> Result<RetrieveStoredSearchResponse, ApiError>;
}

/// API where `Context` isn't passed on every API call
#[async_trait]
#[allow(clippy::too_many_arguments, clippy::ptr_arg)]
pub trait ApiNoContext<C: Send + Sync> {
    fn poll_ready(
        &self,
        _cx: &mut Context,
    ) -> Poll<Result<(), Box<dyn Error + Send + Sync + 'static>>>;

    fn context(&self) -> &C;

    async fn retrieve_complete_search(
        &self,
        search_id: String,
        accept_encoding: Option<String>,
    ) -> Result<RetrieveCompleteSearchResponse, ApiError>;

    /// Deletes a subscription
    async fn scp_domain_routing_info_unsubscribe(
        &self,
        subscription_id: String,
    ) -> Result<ScpDomainRoutingInfoUnsubscribeResponse, ApiError>;

    /// Search a collection of NF Instances
    async fn search_nf_instances(
        &self,
        target_nf_type: models::NfType,
        requester_nf_type: models::NfType,
        accept_encoding: Option<String>,
        preferred_collocated_nf_types: Option<&Vec<models::CollocatedNfType>>,
        requester_nf_instance_id: Option<uuid::Uuid>,
        service_names: Option<&Vec<models::ServiceName>>,
        requester_nf_instance_fqdn: Option<String>,
        target_plmn_list: Option<&Vec<models::PlmnId>>,
        requester_plmn_list: Option<&Vec<models::PlmnId>>,
        target_nf_instance_id: Option<uuid::Uuid>,
        target_nf_fqdn: Option<String>,
        hnrf_uri: Option<String>,
        snssais: Option<&Vec<models::Snssai>>,
        requester_snssais: Option<&Vec<models::ExtSnssai>>,
        plmn_specific_snssai_list: Option<&Vec<models::PlmnSnssai>>,
        requester_plmn_specific_snssai_list: Option<&Vec<models::PlmnSnssai>>,
        dnn: Option<String>,
        ipv4_index: Option<models::IpIndex>,
        ipv6_index: Option<models::IpIndex>,
        nsi_list: Option<&Vec<String>>,
        smf_serving_area: Option<String>,
        mbsmf_serving_area: Option<String>,
        tai: Option<models::Tai>,
        amf_region_id: Option<String>,
        amf_set_id: Option<String>,
        guami: Option<models::Guami>,
        supi: Option<String>,
        ue_ipv4_address: Option<String>,
        ip_domain: Option<String>,
        ue_ipv6_prefix: Option<models::Ipv6Prefix>,
        pgw_ind: Option<bool>,
        preferred_pgw_ind: Option<bool>,
        pgw: Option<String>,
        pgw_ip: Option<models::IpAddr>,
        gpsi: Option<String>,
        external_group_identity: Option<String>,
        internal_group_identity: Option<String>,
        pfd_data: Option<models::PfdData>,
        data_set: Option<models::DataSetId>,
        routing_indicator: Option<String>,
        group_id_list: Option<&Vec<models::NfGroupId>>,
        dnai_list: Option<&Vec<models::Dnai>>,
        pdu_session_types: Option<&Vec<models::PduSessionType>>,
        event_id_list: Option<&Vec<models::EventId>>,
        nwdaf_event_list: Option<&Vec<models::NwdafEvent>>,
        supported_features: Option<String>,
        upf_iwk_eps_ind: Option<bool>,
        chf_supported_plmn: Option<models::PlmnId>,
        preferred_locality: Option<String>,
        access_type: Option<models::AccessType>,
        limit: Option<i32>,
        required_features: Option<&Vec<models::SupportedFeatures>>,
        complex_query: Option<models::ComplexQuery>,
        max_payload_size: Option<i32>,
        max_payload_size_ext: Option<i32>,
        atsss_capability: Option<models::AtsssCapability>,
        upf_ue_ip_addr_ind: Option<bool>,
        client_type: Option<models::ExternalClientType>,
        lmf_id: Option<String>,
        an_node_type: Option<models::AnNodeType>,
        rat_type: Option<models::RatType>,
        preferred_tai: Option<models::Tai>,
        preferred_nf_instances: Option<&Vec<models::NfInstanceId>>,
        if_none_match: Option<String>,
        target_snpn: Option<models::PlmnIdNid>,
        requester_snpn_list: Option<&Vec<models::PlmnIdNid>>,
        af_ee_data: Option<models::AfEventExposureData>,
        w_agf_info: Option<models::WAgfInfo>,
        tngf_info: Option<models::TngfInfo>,
        twif_info: Option<models::TwifInfo>,
        target_nf_set_id: Option<String>,
        target_nf_service_set_id: Option<String>,
        nef_id: Option<String>,
        notification_type: Option<models::NotificationType>,
        n1_msg_class: Option<models::N1MessageClass>,
        n2_info_class: Option<models::N2InformationClass>,
        serving_scope: Option<&Vec<String>>,
        imsi: Option<String>,
        ims_private_identity: Option<String>,
        ims_public_identity: Option<String>,
        msisdn: Option<String>,
        preferred_api_versions: Option<std::collections::HashMap<String, String>>,
        v2x_support_ind: Option<bool>,
        redundant_gtpu: Option<bool>,
        redundant_transport: Option<bool>,
        ipups: Option<bool>,
        scp_domain_list: Option<&Vec<String>>,
        address_domain: Option<String>,
        ipv4_addr: Option<String>,
        ipv6_prefix: Option<models::Ipv6Prefix>,
        served_nf_set_id: Option<String>,
        remote_plmn_id: Option<models::PlmnId>,
        remote_snpn_id: Option<models::PlmnIdNid>,
        data_forwarding: Option<bool>,
        preferred_full_plmn: Option<bool>,
        requester_features: Option<String>,
        realm_id: Option<String>,
        storage_id: Option<String>,
        vsmf_support_ind: Option<bool>,
        ismf_support_ind: Option<bool>,
        nrf_disc_uri: Option<String>,
        preferred_vendor_specific_features: Option<
            std::collections::HashMap<
                String,
                std::collections::HashMap<String, Vec<models::VendorSpecificFeature>>,
            >,
        >,
        preferred_vendor_specific_nf_features: Option<
            std::collections::HashMap<String, Vec<models::VendorSpecificFeature>>,
        >,
        required_pfcp_features: Option<String>,
        home_pub_key_id: Option<i32>,
        prose_support_ind: Option<bool>,
        analytics_aggregation_ind: Option<bool>,
        serving_nf_set_id: Option<String>,
        serving_nf_type: Option<models::NfType>,
        ml_analytics_info_list: Option<&Vec<models::MlAnalyticsInfo>>,
        analytics_metadata_prov_ind: Option<bool>,
        nsacf_capability: Option<models::NsacfCapability>,
        mbs_session_id_list: Option<&Vec<models::MbsSessionId>>,
        area_session_id: Option<i32>,
        gmlc_number: Option<String>,
        upf_n6_ip: Option<models::IpAddr>,
        tai_list: Option<&Vec<models::Tai>>,
        preferences_precedence: Option<&Vec<String>>,
        support_onboarding_capability: Option<bool>,
        uas_nf_functionality_ind: Option<bool>,
        v2x_capability: Option<models::V2xCapability>,
        prose_capability: Option<models::ProSeCapability>,
        shared_data_id: Option<String>,
        target_hni: Option<String>,
        target_nw_resolution: Option<bool>,
        exclude_nfinst_list: Option<&Vec<models::NfInstanceId>>,
        exclude_nfservinst_list: Option<&Vec<models::NfServiceInstance>>,
        exclude_nfserviceset_list: Option<&Vec<models::NfServiceSetId>>,
        exclude_nfset_list: Option<&Vec<models::NfSetId>>,
        preferred_analytics_delays: Option<std::collections::HashMap<String, models::DurationSec>>,
    ) -> Result<SearchNfInstancesResponse, ApiError>;

    async fn scp_domain_routing_info_get(
        &self,
        local: Option<bool>,
        accept_encoding: Option<String>,
    ) -> Result<SCpDomainRoutingInfoGetResponse, ApiError>;

    /// Create a new subscription
    async fn scp_domain_routing_info_subscribe(
        &self,
        scp_domain_routing_info_subscription: models::ScpDomainRoutingInfoSubscription,
        content_encoding: Option<String>,
        accept_encoding: Option<String>,
    ) -> Result<ScpDomainRoutingInfoSubscribeResponse, ApiError>;

    async fn retrieve_stored_search(
        &self,
        search_id: String,
        accept_encoding: Option<String>,
    ) -> Result<RetrieveStoredSearchResponse, ApiError>;
}

/// Trait to extend an API to make it easy to bind it to a context.
pub trait ContextWrapperExt<C: Send + Sync>
where
    Self: Sized,
{
    /// Binds this API to a context.
    fn with_context(self, context: C) -> ContextWrapper<Self, C>;
}

impl<T: Api<C> + Send + Sync, C: Clone + Send + Sync> ContextWrapperExt<C> for T {
    fn with_context(self: T, context: C) -> ContextWrapper<T, C> {
        ContextWrapper::<T, C>::new(self, context)
    }
}

#[async_trait]
impl<T: Api<C> + Send + Sync, C: Clone + Send + Sync> ApiNoContext<C> for ContextWrapper<T, C> {
    fn poll_ready(&self, cx: &mut Context) -> Poll<Result<(), ServiceError>> {
        self.api().poll_ready(cx)
    }

    fn context(&self) -> &C {
        ContextWrapper::context(self)
    }

    async fn retrieve_complete_search(
        &self,
        search_id: String,
        accept_encoding: Option<String>,
    ) -> Result<RetrieveCompleteSearchResponse, ApiError> {
        let context = self.context().clone();
        self.api()
            .retrieve_complete_search(search_id, accept_encoding, &context)
            .await
    }

    /// Deletes a subscription
    async fn scp_domain_routing_info_unsubscribe(
        &self,
        subscription_id: String,
    ) -> Result<ScpDomainRoutingInfoUnsubscribeResponse, ApiError> {
        let context = self.context().clone();
        self.api()
            .scp_domain_routing_info_unsubscribe(subscription_id, &context)
            .await
    }

    /// Search a collection of NF Instances
    async fn search_nf_instances(
        &self,
        target_nf_type: models::NfType,
        requester_nf_type: models::NfType,
        accept_encoding: Option<String>,
        preferred_collocated_nf_types: Option<&Vec<models::CollocatedNfType>>,
        requester_nf_instance_id: Option<uuid::Uuid>,
        service_names: Option<&Vec<models::ServiceName>>,
        requester_nf_instance_fqdn: Option<String>,
        target_plmn_list: Option<&Vec<models::PlmnId>>,
        requester_plmn_list: Option<&Vec<models::PlmnId>>,
        target_nf_instance_id: Option<uuid::Uuid>,
        target_nf_fqdn: Option<String>,
        hnrf_uri: Option<String>,
        snssais: Option<&Vec<models::Snssai>>,
        requester_snssais: Option<&Vec<models::ExtSnssai>>,
        plmn_specific_snssai_list: Option<&Vec<models::PlmnSnssai>>,
        requester_plmn_specific_snssai_list: Option<&Vec<models::PlmnSnssai>>,
        dnn: Option<String>,
        ipv4_index: Option<models::IpIndex>,
        ipv6_index: Option<models::IpIndex>,
        nsi_list: Option<&Vec<String>>,
        smf_serving_area: Option<String>,
        mbsmf_serving_area: Option<String>,
        tai: Option<models::Tai>,
        amf_region_id: Option<String>,
        amf_set_id: Option<String>,
        guami: Option<models::Guami>,
        supi: Option<String>,
        ue_ipv4_address: Option<String>,
        ip_domain: Option<String>,
        ue_ipv6_prefix: Option<models::Ipv6Prefix>,
        pgw_ind: Option<bool>,
        preferred_pgw_ind: Option<bool>,
        pgw: Option<String>,
        pgw_ip: Option<models::IpAddr>,
        gpsi: Option<String>,
        external_group_identity: Option<String>,
        internal_group_identity: Option<String>,
        pfd_data: Option<models::PfdData>,
        data_set: Option<models::DataSetId>,
        routing_indicator: Option<String>,
        group_id_list: Option<&Vec<models::NfGroupId>>,
        dnai_list: Option<&Vec<models::Dnai>>,
        pdu_session_types: Option<&Vec<models::PduSessionType>>,
        event_id_list: Option<&Vec<models::EventId>>,
        nwdaf_event_list: Option<&Vec<models::NwdafEvent>>,
        supported_features: Option<String>,
        upf_iwk_eps_ind: Option<bool>,
        chf_supported_plmn: Option<models::PlmnId>,
        preferred_locality: Option<String>,
        access_type: Option<models::AccessType>,
        limit: Option<i32>,
        required_features: Option<&Vec<models::SupportedFeatures>>,
        complex_query: Option<models::ComplexQuery>,
        max_payload_size: Option<i32>,
        max_payload_size_ext: Option<i32>,
        atsss_capability: Option<models::AtsssCapability>,
        upf_ue_ip_addr_ind: Option<bool>,
        client_type: Option<models::ExternalClientType>,
        lmf_id: Option<String>,
        an_node_type: Option<models::AnNodeType>,
        rat_type: Option<models::RatType>,
        preferred_tai: Option<models::Tai>,
        preferred_nf_instances: Option<&Vec<models::NfInstanceId>>,
        if_none_match: Option<String>,
        target_snpn: Option<models::PlmnIdNid>,
        requester_snpn_list: Option<&Vec<models::PlmnIdNid>>,
        af_ee_data: Option<models::AfEventExposureData>,
        w_agf_info: Option<models::WAgfInfo>,
        tngf_info: Option<models::TngfInfo>,
        twif_info: Option<models::TwifInfo>,
        target_nf_set_id: Option<String>,
        target_nf_service_set_id: Option<String>,
        nef_id: Option<String>,
        notification_type: Option<models::NotificationType>,
        n1_msg_class: Option<models::N1MessageClass>,
        n2_info_class: Option<models::N2InformationClass>,
        serving_scope: Option<&Vec<String>>,
        imsi: Option<String>,
        ims_private_identity: Option<String>,
        ims_public_identity: Option<String>,
        msisdn: Option<String>,
        preferred_api_versions: Option<std::collections::HashMap<String, String>>,
        v2x_support_ind: Option<bool>,
        redundant_gtpu: Option<bool>,
        redundant_transport: Option<bool>,
        ipups: Option<bool>,
        scp_domain_list: Option<&Vec<String>>,
        address_domain: Option<String>,
        ipv4_addr: Option<String>,
        ipv6_prefix: Option<models::Ipv6Prefix>,
        served_nf_set_id: Option<String>,
        remote_plmn_id: Option<models::PlmnId>,
        remote_snpn_id: Option<models::PlmnIdNid>,
        data_forwarding: Option<bool>,
        preferred_full_plmn: Option<bool>,
        requester_features: Option<String>,
        realm_id: Option<String>,
        storage_id: Option<String>,
        vsmf_support_ind: Option<bool>,
        ismf_support_ind: Option<bool>,
        nrf_disc_uri: Option<String>,
        preferred_vendor_specific_features: Option<
            std::collections::HashMap<
                String,
                std::collections::HashMap<String, Vec<models::VendorSpecificFeature>>,
            >,
        >,
        preferred_vendor_specific_nf_features: Option<
            std::collections::HashMap<String, Vec<models::VendorSpecificFeature>>,
        >,
        required_pfcp_features: Option<String>,
        home_pub_key_id: Option<i32>,
        prose_support_ind: Option<bool>,
        analytics_aggregation_ind: Option<bool>,
        serving_nf_set_id: Option<String>,
        serving_nf_type: Option<models::NfType>,
        ml_analytics_info_list: Option<&Vec<models::MlAnalyticsInfo>>,
        analytics_metadata_prov_ind: Option<bool>,
        nsacf_capability: Option<models::NsacfCapability>,
        mbs_session_id_list: Option<&Vec<models::MbsSessionId>>,
        area_session_id: Option<i32>,
        gmlc_number: Option<String>,
        upf_n6_ip: Option<models::IpAddr>,
        tai_list: Option<&Vec<models::Tai>>,
        preferences_precedence: Option<&Vec<String>>,
        support_onboarding_capability: Option<bool>,
        uas_nf_functionality_ind: Option<bool>,
        v2x_capability: Option<models::V2xCapability>,
        prose_capability: Option<models::ProSeCapability>,
        shared_data_id: Option<String>,
        target_hni: Option<String>,
        target_nw_resolution: Option<bool>,
        exclude_nfinst_list: Option<&Vec<models::NfInstanceId>>,
        exclude_nfservinst_list: Option<&Vec<models::NfServiceInstance>>,
        exclude_nfserviceset_list: Option<&Vec<models::NfServiceSetId>>,
        exclude_nfset_list: Option<&Vec<models::NfSetId>>,
        preferred_analytics_delays: Option<std::collections::HashMap<String, models::DurationSec>>,
    ) -> Result<SearchNfInstancesResponse, ApiError> {
        let context = self.context().clone();
        self.api()
            .search_nf_instances(
                target_nf_type,
                requester_nf_type,
                accept_encoding,
                preferred_collocated_nf_types,
                requester_nf_instance_id,
                service_names,
                requester_nf_instance_fqdn,
                target_plmn_list,
                requester_plmn_list,
                target_nf_instance_id,
                target_nf_fqdn,
                hnrf_uri,
                snssais,
                requester_snssais,
                plmn_specific_snssai_list,
                requester_plmn_specific_snssai_list,
                dnn,
                ipv4_index,
                ipv6_index,
                nsi_list,
                smf_serving_area,
                mbsmf_serving_area,
                tai,
                amf_region_id,
                amf_set_id,
                guami,
                supi,
                ue_ipv4_address,
                ip_domain,
                ue_ipv6_prefix,
                pgw_ind,
                preferred_pgw_ind,
                pgw,
                pgw_ip,
                gpsi,
                external_group_identity,
                internal_group_identity,
                pfd_data,
                data_set,
                routing_indicator,
                group_id_list,
                dnai_list,
                pdu_session_types,
                event_id_list,
                nwdaf_event_list,
                supported_features,
                upf_iwk_eps_ind,
                chf_supported_plmn,
                preferred_locality,
                access_type,
                limit,
                required_features,
                complex_query,
                max_payload_size,
                max_payload_size_ext,
                atsss_capability,
                upf_ue_ip_addr_ind,
                client_type,
                lmf_id,
                an_node_type,
                rat_type,
                preferred_tai,
                preferred_nf_instances,
                if_none_match,
                target_snpn,
                requester_snpn_list,
                af_ee_data,
                w_agf_info,
                tngf_info,
                twif_info,
                target_nf_set_id,
                target_nf_service_set_id,
                nef_id,
                notification_type,
                n1_msg_class,
                n2_info_class,
                serving_scope,
                imsi,
                ims_private_identity,
                ims_public_identity,
                msisdn,
                preferred_api_versions,
                v2x_support_ind,
                redundant_gtpu,
                redundant_transport,
                ipups,
                scp_domain_list,
                address_domain,
                ipv4_addr,
                ipv6_prefix,
                served_nf_set_id,
                remote_plmn_id,
                remote_snpn_id,
                data_forwarding,
                preferred_full_plmn,
                requester_features,
                realm_id,
                storage_id,
                vsmf_support_ind,
                ismf_support_ind,
                nrf_disc_uri,
                preferred_vendor_specific_features,
                preferred_vendor_specific_nf_features,
                required_pfcp_features,
                home_pub_key_id,
                prose_support_ind,
                analytics_aggregation_ind,
                serving_nf_set_id,
                serving_nf_type,
                ml_analytics_info_list,
                analytics_metadata_prov_ind,
                nsacf_capability,
                mbs_session_id_list,
                area_session_id,
                gmlc_number,
                upf_n6_ip,
                tai_list,
                preferences_precedence,
                support_onboarding_capability,
                uas_nf_functionality_ind,
                v2x_capability,
                prose_capability,
                shared_data_id,
                target_hni,
                target_nw_resolution,
                exclude_nfinst_list,
                exclude_nfservinst_list,
                exclude_nfserviceset_list,
                exclude_nfset_list,
                preferred_analytics_delays,
                &context,
            )
            .await
    }

    async fn scp_domain_routing_info_get(
        &self,
        local: Option<bool>,
        accept_encoding: Option<String>,
    ) -> Result<SCpDomainRoutingInfoGetResponse, ApiError> {
        let context = self.context().clone();
        self.api()
            .scp_domain_routing_info_get(local, accept_encoding, &context)
            .await
    }

    /// Create a new subscription
    async fn scp_domain_routing_info_subscribe(
        &self,
        scp_domain_routing_info_subscription: models::ScpDomainRoutingInfoSubscription,
        content_encoding: Option<String>,
        accept_encoding: Option<String>,
    ) -> Result<ScpDomainRoutingInfoSubscribeResponse, ApiError> {
        let context = self.context().clone();
        self.api()
            .scp_domain_routing_info_subscribe(
                scp_domain_routing_info_subscription,
                content_encoding,
                accept_encoding,
                &context,
            )
            .await
    }

    async fn retrieve_stored_search(
        &self,
        search_id: String,
        accept_encoding: Option<String>,
    ) -> Result<RetrieveStoredSearchResponse, ApiError> {
        let context = self.context().clone();
        self.api()
            .retrieve_stored_search(search_id, accept_encoding, &context)
            .await
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
pub enum OnScpDomainRoutingInformationChangePostResponse {
    /// Expected response to a successful callback processing
    ExpectedResponseToASuccessfulCallbackProcessing { accept_encoding: Option<String> },
    /// Bad request
    BadRequest(models::ProblemDetails),
    /// Unauthorized
    Unauthorized(models::ProblemDetails),
    /// Forbidden
    Forbidden(models::ProblemDetails),
    /// Not Found
    NotFound(models::ProblemDetails),
    /// Length Required
    LengthRequired(models::ProblemDetails),
    /// Payload Too Large
    PayloadTooLarge(models::ProblemDetails),
    /// Unsupported Media Type
    UnsupportedMediaType(models::ProblemDetails),
    /// Too Many Requests
    TooManyRequests(models::ProblemDetails),
    /// Internal Server Error
    InternalServerError(models::ProblemDetails),
    /// Not Implemented
    NotImplemented(models::ProblemDetails),
    /// Service Unavailable
    ServiceUnavailable(models::ProblemDetails),
    /// Generic Error
    GenericError,
}

/// Callback API
#[async_trait]
pub trait CallbackApi<C: Send + Sync> {
    fn poll_ready(
        &self,
        _cx: &mut Context,
    ) -> Poll<Result<(), Box<dyn Error + Send + Sync + 'static>>> {
        Poll::Ready(Ok(()))
    }

    async fn on_scp_domain_routing_information_change_post(
        &self,
        callback_request_body_callback_uri: String,
        content_encoding: Option<String>,
        scp_domain_routing_info_notification: Option<models::ScpDomainRoutingInfoNotification>,
        context: &C,
    ) -> Result<OnScpDomainRoutingInformationChangePostResponse, ApiError>;
}

/// Callback API without a `Context`
#[async_trait]
pub trait CallbackApiNoContext<C: Send + Sync> {
    fn poll_ready(
        &self,
        _cx: &mut Context,
    ) -> Poll<Result<(), Box<dyn Error + Send + Sync + 'static>>>;

    fn context(&self) -> &C;

    async fn on_scp_domain_routing_information_change_post(
        &self,
        callback_request_body_callback_uri: String,
        content_encoding: Option<String>,
        scp_domain_routing_info_notification: Option<models::ScpDomainRoutingInfoNotification>,
    ) -> Result<OnScpDomainRoutingInformationChangePostResponse, ApiError>;
}

pub trait CallbackContextWrapperExt<C: Send + Sync>
where
    Self: Sized,
{
    /// Binds this API to a context.
    fn with_context(self, context: C) -> ContextWrapper<Self, C>;
}

impl<T: CallbackApi<C> + Send + Sync, C: Clone + Send + Sync> CallbackContextWrapperExt<C> for T {
    fn with_context(self: T, context: C) -> ContextWrapper<T, C> {
        ContextWrapper::<T, C>::new(self, context)
    }
}

#[async_trait]
impl<T: CallbackApi<C> + Send + Sync, C: Clone + Send + Sync> CallbackApiNoContext<C>
    for ContextWrapper<T, C>
{
    fn poll_ready(&self, cx: &mut Context) -> Poll<Result<(), ServiceError>> {
        self.api().poll_ready(cx)
    }

    fn context(&self) -> &C {
        ContextWrapper::context(self)
    }

    async fn on_scp_domain_routing_information_change_post(
        &self,
        callback_request_body_callback_uri: String,
        content_encoding: Option<String>,
        scp_domain_routing_info_notification: Option<models::ScpDomainRoutingInfoNotification>,
    ) -> Result<OnScpDomainRoutingInformationChangePostResponse, ApiError> {
        let context = self.context().clone();
        self.api()
            .on_scp_domain_routing_information_change_post(
                callback_request_body_callback_uri,
                content_encoding,
                scp_domain_routing_info_notification,
                &context,
            )
            .await
    }
}

#[cfg(feature = "client")]
pub mod client;

// Re-export Client as a top-level name
#[cfg(feature = "client")]
pub use client::Client;

#[cfg(feature = "server")]
pub mod server;

// Re-export router() as a top-level name
#[cfg(feature = "server")]
pub use self::server::Service;

#[cfg(any(feature = "client", feature = "server"))]
pub mod context;

pub mod models;

#[cfg(any(feature = "client", feature = "server"))]
pub(crate) mod header;
