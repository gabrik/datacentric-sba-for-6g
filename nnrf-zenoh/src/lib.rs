use nnrf_discovery_server::{
    models, RetrieveCompleteSearchResponse, RetrieveStoredSearchResponse,
    SCpDomainRoutingInfoGetResponse, ScpDomainRoutingInfoSubscribeResponse,
    ScpDomainRoutingInfoUnsubscribeResponse, SearchNfInstancesResponse,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use zrpc::zrpcresult::{ZRPCError, ZRPCResult};
use zrpc_macros::zservice;

pub mod server;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ApiError(pub String);

#[zservice(
    timeout_s = 60,
    prefix = "5gsba/nnrf-disc",
    service_uuid = "00000000-0000-0000-0000-00000000000A"
)]
pub trait NRFApi {
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
        preferred_collocated_nf_types: Option<Vec<models::CollocatedNfType>>,
        requester_nf_instance_id: Option<uuid::Uuid>,
        service_names: Option<Vec<models::ServiceName>>,
        requester_nf_instance_fqdn: Option<String>,
        target_plmn_list: Option<Vec<models::PlmnId>>,
        requester_plmn_list: Option<Vec<models::PlmnId>>,
        target_nf_instance_id: Option<uuid::Uuid>,
        target_nf_fqdn: Option<String>,
        hnrf_uri: Option<String>,
        snssais: Option<Vec<models::Snssai>>,
        requester_snssais: Option<Vec<models::ExtSnssai>>,
        plmn_specific_snssai_list: Option<Vec<models::PlmnSnssai>>,
        requester_plmn_specific_snssai_list: Option<Vec<models::PlmnSnssai>>,
        dnn: Option<String>,
        ipv4_index: Option<models::IpIndex>,
        ipv6_index: Option<models::IpIndex>,
        nsi_list: Option<Vec<String>>,
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
        group_id_list: Option<Vec<models::NfGroupId>>,
        dnai_list: Option<Vec<models::Dnai>>,
        pdu_session_types: Option<Vec<models::PduSessionType>>,
        event_id_list: Option<Vec<models::EventId>>,
        nwdaf_event_list: Option<Vec<models::NwdafEvent>>,
        supported_features: Option<String>,
        upf_iwk_eps_ind: Option<bool>,
        chf_supported_plmn: Option<models::PlmnId>,
        preferred_locality: Option<String>,
        access_type: Option<models::AccessType>,
        limit: Option<i32>,
        required_features: Option<Vec<models::SupportedFeatures>>,
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
        preferred_nf_instances: Option<Vec<models::NfInstanceId>>,
        if_none_match: Option<String>,
        target_snpn: Option<models::PlmnIdNid>,
        requester_snpn_list: Option<Vec<models::PlmnIdNid>>,
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
        serving_scope: Option<Vec<String>>,
        imsi: Option<String>,
        ims_private_identity: Option<String>,
        ims_public_identity: Option<String>,
        msisdn: Option<String>,
        preferred_api_versions: Option<std::collections::HashMap<String, String>>,
        v2x_support_ind: Option<bool>,
        redundant_gtpu: Option<bool>,
        redundant_transport: Option<bool>,
        ipups: Option<bool>,
        scp_domain_list: Option<Vec<String>>,
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
        ml_analytics_info_list: Option<Vec<models::MlAnalyticsInfo>>,
        analytics_metadata_prov_ind: Option<bool>,
        nsacf_capability: Option<models::NsacfCapability>,
        mbs_session_id_list: Option<Vec<models::MbsSessionId>>,
        area_session_id: Option<i32>,
        gmlc_number: Option<String>,
        upf_n6_ip: Option<models::IpAddr>,
        tai_list: Option<Vec<models::Tai>>,
        preferences_precedence: Option<Vec<String>>,
        support_onboarding_capability: Option<bool>,
        uas_nf_functionality_ind: Option<bool>,
        v2x_capability: Option<models::V2xCapability>,
        prose_capability: Option<models::ProSeCapability>,
        shared_data_id: Option<String>,
        target_hni: Option<String>,
        target_nw_resolution: Option<bool>,
        exclude_nfinst_list: Option<Vec<models::NfInstanceId>>,
        exclude_nfservinst_list: Option<Vec<models::NfServiceInstance>>,
        exclude_nfserviceset_list: Option<Vec<models::NfServiceSetId>>,
        exclude_nfset_list: Option<Vec<models::NfSetId>>,
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
