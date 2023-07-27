use std::{collections::HashMap, str::FromStr};

use crate::{ApiError, NRFApi};

use nnrf_discovery_server::{
    models::{
        self, IpEndPoint, NfProfile, NfService, NfServiceStatus, NfServiceVersion, NfStatus,
        NfType, ProblemDetails, SearchResult, ServiceName, UriScheme,
    },
    RetrieveCompleteSearchResponse, RetrieveStoredSearchResponse, SCpDomainRoutingInfoGetResponse,
    ScpDomainRoutingInfoSubscribeResponse, ScpDomainRoutingInfoUnsubscribeResponse,
    SearchNfInstancesResponse,
};
use uuid::Uuid;
use zrpc_macros::zserver;

#[derive(Clone)]
pub struct Server {}

impl Server {
    fn result_smf_udm() -> SearchResult {
        SearchResult {
            validity_period: Some(3600),
            nf_instances: vec![NfProfile {
                nf_instance_id: Uuid::from_str("65a52dac-b832-41ed-ba6e-53c1c0b3ed51").unwrap(),
                nf_type: NfType::UDM,
                nf_status: NfStatus::new("REGISTERED".into()),
                ipv4_addresses: Some(vec!["172.22.0.13".to_string().into()]),
                priority: Some(0),
                capacity: Some(100),
                load: Some(0),
                nf_service_list: Some(HashMap::from([(
                    "65a54148-b832-41ed-ba6e-53c1c0b3ed51".into(),
                    NfService {
                        service_instance_id: "65a54148-b832-41ed-ba6e-53c1c0b3ed51".into(),
                        service_name: ServiceName::new("nudm-sdm".to_string()),
                        versions: vec![NfServiceVersion {
                            api_version_in_uri: "v2".into(),
                            api_full_version: "2.0.0".into(),
                            expiry: None,
                        }],
                        scheme: UriScheme::new("http".into()),
                        nf_service_status: NfServiceStatus::new("REGISTERED".into()),
                        ip_end_points: Some(vec![IpEndPoint {
                            ipv4_address: Some("172.22.0.13".into()),
                            port: Some(7777),
                            ipv6_address: None,
                            transport: None,
                        }]),
                        priority: Some(0),
                        capacity: Some(100),
                        load: Some(0),
                        fqdn: None,
                        inter_plmn_fqdn: None,
                        api_prefix: None,
                        default_notification_subscriptions: None,
                        load_time_stamp: None,
                        recovery_time: None,
                        supported_features: None,
                        nf_service_set_id_list: None,
                        s_nssais: None,
                        per_plmn_snssai_list: None,
                        vendor_id: None,
                        supported_vendor_specific_features: None,
                        oauth2_required: None,
                        allowed_operations_per_nf_type: None,
                        allowed_operations_per_nf_instance: None,
                    },
                )])),
                nf_instance_name: None,
                collocated_nf_instances: None,
                plmn_list: None,
                s_nssais: None,
                per_plmn_snssai_list: None,
                nsi_list: None,
                fqdn: None,
                inter_plmn_fqdn: None,
                ipv6_addresses: None,
                load_time_stamp: None,
                locality: None,
                udr_info: None,
                udr_info_list: None,
                udm_info: None,
                udm_info_list: None,
                ausf_info: None,
                ausf_info_list: None,
                amf_info: None,
                amf_info_list: None,
                smf_info: None,
                smf_info_list: None,
                upf_info: None,
                upf_info_list: None,
                pcf_info: None,
                pcf_info_list: None,
                bsf_info: None,
                bsf_info_list: None,
                chf_info: None,
                chf_info_list: None,
                udsf_info: None,
                udsf_info_list: None,
                nwdaf_info: None,
                nwdaf_info_list: None,
                nef_info: None,
                pcscf_info_list: None,
                hss_info_list: None,
                custom_info: None,
                recovery_time: None,
                nf_service_persistence: None,
                nf_services: None,
                default_notification_subscriptions: None,
                lmf_info: None,
                gmlc_info: None,
                snpn_list: None,
                nf_set_id_list: None,
                serving_scope: None,
                lc_h_support_ind: None,
                olc_h_support_ind: None,
                nf_set_recovery_time_list: None,
                service_set_recovery_time_list: None,
                scp_domains: None,
                scp_info: None,
                sepp_info: None,
                vendor_id: None,
                supported_vendor_specific_features: None,
                aanf_info_list: None,
                mfaf_info: None,
                easdf_info_list: None,
                dccf_info: None,
                nsacf_info_list: None,
                mb_smf_info_list: None,
                tsctsf_info_list: None,
                mb_upf_info_list: None,
                trust_af_info: None,
                nssaaf_info: None,
                hni_list: None,
                iwmsc_info: None,
                mnpf_info: None,
            }],
            search_id: None,
            num_nf_inst_complete: None,
            preferred_search: None,
            nrf_supported_features: None,
            nf_instance_list: None,
            altered_priority_ind: None,
            no_profile_match_info: None,
        }
    }

    fn result_amf_smf() -> SearchResult {
        SearchResult {
            validity_period: Some(3600),
            nf_instances: vec![NfProfile {
                nf_instance_id: Uuid::from_str("b3a71a80-b8d5-41ed-b2cc-8bbc6f173b7d").unwrap(),
                nf_type: NfType::SMF,
                nf_status: NfStatus::new("REGISTERED".into()),
                ipv4_addresses: Some(vec!["172.22.0.13".to_string().into()]),
                priority: Some(0),
                capacity: Some(100),
                load: Some(0),
                nf_service_list: Some(HashMap::from([(
                    "b3a71a80-b8d5-41ed-b2cc-8bbc6f173b7d".into(),
                    NfService {
                        service_instance_id: "b3a71a80-b8d5-41ed-b2cc-8bbc6f173b7d".into(),
                        service_name: ServiceName::new("nsmf-pdusession".to_string()),
                        versions: vec![NfServiceVersion {
                            api_version_in_uri: "v1".into(),
                            api_full_version: "1.0.0".into(),
                            expiry: None,
                        }],
                        scheme: UriScheme::new("http".into()),
                        nf_service_status: NfServiceStatus::new("REGISTERED".into()),
                        ip_end_points: Some(vec![IpEndPoint {
                            ipv4_address: Some("172.22.0.7".into()),
                            port: Some(7777),
                            ipv6_address: None,
                            transport: None,
                        }]),
                        priority: Some(0),
                        capacity: Some(100),
                        load: Some(0),
                        fqdn: None,
                        inter_plmn_fqdn: None,
                        api_prefix: None,
                        default_notification_subscriptions: None,
                        load_time_stamp: None,
                        recovery_time: None,
                        supported_features: None,
                        nf_service_set_id_list: None,
                        s_nssais: None,
                        per_plmn_snssai_list: None,
                        vendor_id: None,
                        supported_vendor_specific_features: None,
                        oauth2_required: None,
                        allowed_operations_per_nf_type: None,
                        allowed_operations_per_nf_instance: None,
                    },
                )])),
                nf_instance_name: None,
                collocated_nf_instances: None,
                plmn_list: None,
                s_nssais: None,
                per_plmn_snssai_list: None,
                nsi_list: None,
                fqdn: None,
                inter_plmn_fqdn: None,
                ipv6_addresses: None,
                load_time_stamp: None,
                locality: None,
                udr_info: None,
                udr_info_list: None,
                udm_info: None,
                udm_info_list: None,
                ausf_info: None,
                ausf_info_list: None,
                amf_info: None,
                amf_info_list: None,
                smf_info: None,
                smf_info_list: None,
                upf_info: None,
                upf_info_list: None,
                pcf_info: None,
                pcf_info_list: None,
                bsf_info: None,
                bsf_info_list: None,
                chf_info: None,
                chf_info_list: None,
                udsf_info: None,
                udsf_info_list: None,
                nwdaf_info: None,
                nwdaf_info_list: None,
                nef_info: None,
                pcscf_info_list: None,
                hss_info_list: None,
                custom_info: None,
                recovery_time: None,
                nf_service_persistence: None,
                nf_services: None,
                default_notification_subscriptions: None,
                lmf_info: None,
                gmlc_info: None,
                snpn_list: None,
                nf_set_id_list: None,
                serving_scope: None,
                lc_h_support_ind: None,
                olc_h_support_ind: None,
                nf_set_recovery_time_list: None,
                service_set_recovery_time_list: None,
                scp_domains: None,
                scp_info: None,
                sepp_info: None,
                vendor_id: None,
                supported_vendor_specific_features: None,
                aanf_info_list: None,
                mfaf_info: None,
                easdf_info_list: None,
                dccf_info: None,
                nsacf_info_list: None,
                mb_smf_info_list: None,
                tsctsf_info_list: None,
                mb_upf_info_list: None,
                trust_af_info: None,
                nssaaf_info: None,
                hni_list: None,
                iwmsc_info: None,
                mnpf_info: None,
            }],
            search_id: None,
            num_nf_inst_complete: None,
            preferred_search: None,
            nrf_supported_features: None,
            nf_instance_list: None,
            altered_priority_ind: None,
            no_profile_match_info: None,
        }
    }
}

#[zserver]
impl NRFApi for Server {
    async fn retrieve_complete_search(
        &self,
        search_id: String,
        accept_encoding: Option<String>,
    ) -> Result<RetrieveCompleteSearchResponse, ApiError> {
        log::info!(
            "retrieve_complete_search(\"{}\", {:?})",
            search_id,
            accept_encoding,
        );
        Err(ApiError("Generic failure".into()))
    }

    /// Deletes a subscription
    async fn scp_domain_routing_info_unsubscribe(
        &self,
        subscription_id: String,
    ) -> Result<ScpDomainRoutingInfoUnsubscribeResponse, ApiError> {
        log::info!(
            "scp_domain_routing_info_unsubscribe(\"{}\")",
            subscription_id
        );
        Err(ApiError("Generic failure".into()))
    }

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
    ) -> Result<SearchNfInstancesResponse, ApiError> {
        log::info!("search_nf_instances({:?}, {:?}, {:?}, {:?}, {:?}, {:?}, {:?}, {:?}, {:?}, {:?}, {:?}, {:?}, {:?}, {:?}, {:?}, {:?}, {:?}, {:?}, {:?}, {:?}, {:?}, {:?}, {:?}, {:?}, {:?}, {:?}, {:?}, {:?}, {:?}, {:?}, {:?}, {:?}, {:?}, {:?}, {:?}, {:?}, {:?}, {:?}, {:?}, {:?}, {:?}, {:?}, {:?}, {:?}, {:?}, {:?}, {:?}, {:?}, {:?}, {:?}, {:?}, {:?}, {:?}, {:?}, {:?}, {:?}, {:?}, {:?}, {:?}, {:?}, {:?}, {:?}, {:?}, {:?}, {:?}, {:?}, {:?}, {:?}, {:?}, {:?}, {:?}, {:?}, {:?}, {:?}, {:?}, {:?}, {:?}, {:?}, {:?}, {:?}, {:?}, {:?}, {:?}, {:?}, {:?}, {:?}, {:?}, {:?}, {:?}, {:?}, {:?}, {:?}, {:?}, {:?}, {:?}, {:?}, {:?}, {:?}, {:?}, {:?}, {:?}, {:?}, {:?}, {:?}, {:?}, {:?}, {:?}, {:?}, {:?}, {:?}, {:?}, {:?}, {:?}, {:?}, {:?}, {:?}, {:?}, {:?}, {:?}, {:?}, {:?}, {:?}, {:?}, {:?}, {:?}, {:?}, {:?}, {:?}, {:?}, {:?})", target_nf_type, requester_nf_type, accept_encoding, preferred_collocated_nf_types, requester_nf_instance_id, service_names, requester_nf_instance_fqdn, target_plmn_list, requester_plmn_list, target_nf_instance_id, target_nf_fqdn, hnrf_uri, snssais, requester_snssais, plmn_specific_snssai_list, requester_plmn_specific_snssai_list, dnn, ipv4_index, ipv6_index, nsi_list, smf_serving_area, mbsmf_serving_area, tai, amf_region_id, amf_set_id, guami, supi, ue_ipv4_address, ip_domain, ue_ipv6_prefix, pgw_ind, preferred_pgw_ind, pgw, pgw_ip, gpsi, external_group_identity, internal_group_identity, pfd_data, data_set, routing_indicator, group_id_list, dnai_list, pdu_session_types, event_id_list, nwdaf_event_list, supported_features, upf_iwk_eps_ind, chf_supported_plmn, preferred_locality, access_type, limit, required_features, complex_query, max_payload_size, max_payload_size_ext, atsss_capability, upf_ue_ip_addr_ind, client_type, lmf_id, an_node_type, rat_type, preferred_tai, preferred_nf_instances, if_none_match, target_snpn, requester_snpn_list, af_ee_data, w_agf_info, tngf_info, twif_info, target_nf_set_id, target_nf_service_set_id, nef_id, notification_type, n1_msg_class, n2_info_class, serving_scope, imsi, ims_private_identity, ims_public_identity, msisdn, preferred_api_versions, v2x_support_ind, redundant_gtpu, redundant_transport, ipups, scp_domain_list, address_domain, ipv4_addr, ipv6_prefix, served_nf_set_id, remote_plmn_id, remote_snpn_id, data_forwarding, preferred_full_plmn, requester_features, realm_id, storage_id, vsmf_support_ind, ismf_support_ind, nrf_disc_uri, preferred_vendor_specific_features, preferred_vendor_specific_nf_features, required_pfcp_features, home_pub_key_id, prose_support_ind, analytics_aggregation_ind, serving_nf_set_id, serving_nf_type, ml_analytics_info_list, analytics_metadata_prov_ind, nsacf_capability, mbs_session_id_list, area_session_id, gmlc_number, upf_n6_ip, tai_list, preferences_precedence, support_onboarding_capability, uas_nf_functionality_ind, v2x_capability, prose_capability, shared_data_id, target_hni, target_nw_resolution, exclude_nfinst_list, exclude_nfservinst_list, exclude_nfserviceset_list, exclude_nfset_list, preferred_analytics_delays);
        // Err(ApiError("Generic failure".into()))

        // Here we should lookup for the service that was requested

        match requester_nf_type {
            NfType::SMF => match target_nf_type {
                NfType::UDM => {
                    let body = Server::result_smf_udm();
                    let resp = SearchNfInstancesResponse::ExpectedResponseToAValidRequest {
                        body,
                        cache_control: None,
                        e_tag: None,
                        content_encoding: Some("application/json".to_string()),
                    };
                    Ok(resp)
                }
                _ => Ok(SearchNfInstancesResponse::NotImplemented(ProblemDetails {
                    status: Some(501),
                    title: Some("Not implemented".to_string()),
                    r#type: None,
                    detail: Some("Just mock up!".to_string()),
                    instance: None,
                    cause: Some("Not implemented".to_string()),
                    invalid_params: None,
                    supported_features: None,
                    access_token_error: None,
                    access_token_request: None,
                    nrf_id: None,
                })),
            },
            NfType::AMF => match target_nf_type {
                NfType::SMF => {
                    let body = Server::result_amf_smf();
                    let resp = SearchNfInstancesResponse::ExpectedResponseToAValidRequest {
                        body,
                        cache_control: None,
                        e_tag: None,
                        content_encoding: Some("application/json".to_string()),
                    };
                    Ok(resp)
                }
                _ => Ok(SearchNfInstancesResponse::NotImplemented(ProblemDetails {
                    status: Some(501),
                    title: Some("Not implemented".to_string()),
                    r#type: None,
                    detail: Some("Just mock up!".to_string()),
                    instance: None,
                    cause: Some("Not implemented".to_string()),
                    invalid_params: None,
                    supported_features: None,
                    access_token_error: None,
                    access_token_request: None,
                    nrf_id: None,
                })),
            },
            _ => Ok(SearchNfInstancesResponse::NotImplemented(ProblemDetails {
                status: Some(501),
                title: Some("Not implemented".to_string()),
                r#type: None,
                detail: Some("Just mock up!".to_string()),
                instance: None,
                cause: Some("Not implemented".to_string()),
                invalid_params: None,
                supported_features: None,
                access_token_error: None,
                access_token_request: None,
                nrf_id: None,
            })),
        }

        // Here we receive /nnrf-disc/v1/nf-instances?requester-features=20&requester-nf-type=SMF&service-names=nudm-sdm&target-nf-type=UDM
        // if the requester is a SMF asking for UDM we send
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
        //
        // SMF looking for PCF
        // ....
        // AMF looking for SMF
        //
        // Request on: nnrf-disc/v1/nf-instances?requester-nf-type=AMF&service-names=nsmf-pdusession&target-nf-type=SMF&requester-features=20
        // {
        // 	"validityPeriod":	3600,
        // 	"nfInstances":	[{
        // 			"nfInstanceId":	"b3a71a80-b8d5-41ed-b2cc-8bbc6f173b7d",
        // 			"nfType":	"SMF",
        // 			"nfStatus":	"REGISTERED",
        // 			"heartBeatTimer":	10,
        // 			"ipv4Addresses":	["172.22.0.7"],
        // 			"allowedNfTypes":	["AMF", "SCP"],
        // 			"priority":	0,
        // 			"capacity":	100,
        // 			"load":	0,
        // 			"nfServiceList":	{
        // 				"b3c40334-b8d5-41ed-b2cc-8bbc6f173b7d":	{
        // 					"serviceInstanceId":	"b3c40334-b8d5-41ed-b2cc-8bbc6f173b7d",
        // 					"serviceName":	"nsmf-pdusession",
        // 					"versions":	[{
        // 							"apiVersionInUri":	"v1",
        // 							"apiFullVersion":	"1.0.0"
        // 						}],
        // 					"scheme":	"http",
        // 					"nfServiceStatus":	"REGISTERED",
        // 					"ipEndPoints":	[{
        // 							"ipv4Address":	"172.22.0.7",
        // 							"port":	7777
        // 						}],
        // 					"allowedNfTypes":	["AMF"],
        // 					"priority":	0,
        // 					"capacity":	100,
        // 					"load":	0
        // 				}
        // 			},
        // 			"nfProfileChangesSupportInd":	true
        // 		}]
        // }
    }

    async fn scp_domain_routing_info_get(
        &self,
        local: Option<bool>,
        accept_encoding: Option<String>,
    ) -> Result<SCpDomainRoutingInfoGetResponse, ApiError> {
        log::info!(
            "scp_domain_routing_info_get({:?}, {:?})",
            local,
            accept_encoding
        );
        Err(ApiError("Generic failure".into()))
    }

    /// Create a new subscription
    async fn scp_domain_routing_info_subscribe(
        &self,
        scp_domain_routing_info_subscription: models::ScpDomainRoutingInfoSubscription,
        content_encoding: Option<String>,
        accept_encoding: Option<String>,
    ) -> Result<ScpDomainRoutingInfoSubscribeResponse, ApiError> {
        log::info!(
            "scp_domain_routing_info_subscribe({:?}, {:?}, {:?})",
            scp_domain_routing_info_subscription,
            content_encoding,
            accept_encoding,
        );
        Err(ApiError("Generic failure".into()))
    }

    async fn retrieve_stored_search(
        &self,
        search_id: String,
        accept_encoding: Option<String>,
    ) -> Result<RetrieveStoredSearchResponse, ApiError> {
        log::info!(
            "retrieve_stored_search(\"{}\", {:?})",
            search_id,
            accept_encoding,
        );
        Err(ApiError("Generic failure".into()))
    }
}
