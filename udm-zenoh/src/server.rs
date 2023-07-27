use std::collections::HashMap;

use crate::{ApiError, UDMApi};

use nudm_sdm::{
    models::{
        self, Ambr, Arp, DnnConfiguration, PduSessionType, PduSessionTypes, PreemptionCapability,
        PreemptionVulnerability, SessionManagementSubscriptionData, SmSubsData, Snssai, SscMode,
        SscModes, SubscribedDefaultQos,
    },
    CAgAckResponse, GetAmDataResponse, GetDataSetsResponse, GetEcrDataResponse,
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
use zrpc_macros::zserver;

#[derive(Clone)]
pub struct Server {}

#[zserver]
impl UDMApi for Server {
    /// retrieve a UE's Access and Mobility Subscription Data
    async fn get_am_data(
        &self,
        supi: String,
        supported_features: Option<String>,
        plmn_id: Option<models::PlmnIdNid>,
        adjacent_plmns: Option<Vec<models::PlmnId>>,
        disaster_roaming_ind: Option<bool>,
        if_none_match: Option<String>,
        if_modified_since: Option<String>,
    ) -> Result<GetAmDataResponse, ApiError> {
        log::info!(
            "get_am_data(\"{}\", {:?}, {:?}, {:?}, {:?}, {:?}, {:?}) ",
            supi,
            supported_features,
            plmn_id,
            adjacent_plmns,
            disaster_roaming_ind,
            if_none_match,
            if_modified_since,
        );
        Err(ApiError("Generic failure".into()))
    }

    /// retrieve a UE's 5MBS Subscription Data
    async fn get_mbs_data(
        &self,
        supi: String,
        supported_features: Option<String>,
        if_none_match: Option<String>,
        if_modified_since: Option<String>,
    ) -> Result<GetMbsDataResponse, ApiError> {
        log::info!(
            "get_mbs_data(\"{}\", {:?}, {:?}, {:?}) ",
            supi,
            supported_features,
            if_none_match,
            if_modified_since,
        );
        Err(ApiError("Generic failure".into()))
    }

    /// retrieve a UE's subscribed Enhanced Coverage Restriction Data
    async fn get_ecr_data(
        &self,
        supi: String,
        supported_features: Option<String>,
        if_none_match: Option<String>,
        if_modified_since: Option<String>,
    ) -> Result<GetEcrDataResponse, ApiError> {
        log::info!(
            "get_ecr_data(\"{}\", {:?}, {:?}, {:?}) ",
            supi,
            supported_features,
            if_none_match,
            if_modified_since,
        );
        Err(ApiError("Generic failure".into()))
    }

    /// retrieve a UE's SUPI or GPSI
    async fn get_supi_or_gpsi(
        &self,
        ue_id: String,
        supported_features: Option<String>,
        af_id: Option<String>,
        app_port_id: Option<models::AppPortId>,
        af_service_id: Option<String>,
        mtc_provider_info: Option<String>,
        requested_gpsi_type: Option<models::GpsiType>,
        if_none_match: Option<String>,
        if_modified_since: Option<String>,
    ) -> Result<GetSupiOrGpsiResponse, ApiError> {
        log::info!(
            "get_supi_or_gpsi(\"{}\", {:?}, {:?}, {:?}, {:?}, {:?}, {:?}, {:?}, {:?}) ",
            ue_id,
            supported_features,
            af_id,
            app_port_id,
            af_service_id,
            mtc_provider_info,
            requested_gpsi_type,
            if_none_match,
            if_modified_since
        );
        Err(ApiError("Generic failure".into()))
    }

    /// Mapping of Group Identifiers
    async fn get_group_identifiers(
        &self,
        ext_group_id: Option<String>,
        int_group_id: Option<String>,
        ue_id_ind: Option<bool>,
        supported_features: Option<String>,
        af_id: Option<String>,
        if_none_match: Option<String>,
        if_modified_since: Option<String>,
    ) -> Result<GetGroupIdentifiersResponse, ApiError> {
        log::info!(
            "get_group_identifiers({:?}, {:?}, {:?}, {:?}, {:?}, {:?}, {:?}) ",
            ext_group_id,
            int_group_id,
            ue_id_ind,
            supported_features,
            af_id,
            if_none_match,
            if_modified_since,
        );
        Err(ApiError("Generic failure".into()))
    }

    /// retrieve a UE's LCS Broadcast Assistance Data Types Subscription Data
    async fn get_lcs_bca_data(
        &self,
        supi: String,
        supported_features: Option<String>,
        plmn_id: Option<models::PlmnId>,
        if_none_match: Option<String>,
        if_modified_since: Option<String>,
    ) -> Result<GetLcsBcaDataResponse, ApiError> {
        log::info!(
            "get_lcs_bca_data(\"{}\", {:?}, {:?}, {:?}, {:?}) ",
            supi,
            supported_features,
            plmn_id,
            if_none_match,
            if_modified_since,
        );
        Err(ApiError("Generic failure".into()))
    }

    /// retrieve a UE's LCS Mobile Originated Subscription Data
    async fn get_lcs_mo_data(
        &self,
        supi: String,
        supported_features: Option<String>,
        if_none_match: Option<String>,
        if_modified_since: Option<String>,
    ) -> Result<GetLcsMoDataResponse, ApiError> {
        log::info!(
            "get_lcs_mo_data(\"{}\", {:?}, {:?}, {:?}) ",
            supi,
            supported_features,
            if_none_match,
            if_modified_since,
        );
        Err(ApiError("Generic failure".into()))
    }

    /// retrieve a UE's LCS Privacy Subscription Data
    async fn get_lcs_privacy_data(
        &self,
        ue_id: String,
        supported_features: Option<String>,
        if_none_match: Option<String>,
        if_modified_since: Option<String>,
    ) -> Result<GetLcsPrivacyDataResponse, ApiError> {
        log::info!(
            "get_lcs_privacy_data(\"{}\", {:?}, {:?}, {:?}) ",
            ue_id,
            supported_features,
            if_none_match,
            if_modified_since,
        );
        Err(ApiError("Generic failure".into()))
    }

    /// Mapping of UE Identifiers
    async fn get_multiple_identifiers(
        &self,
        gpsi_list: Vec<models::Gpsi>,
        supported_features: Option<String>,
    ) -> Result<GetMultipleIdentifiersResponse, ApiError> {
        log::info!(
            "get_multiple_identifiers({:?}, {:?}) ",
            gpsi_list,
            supported_features,
        );
        Err(ApiError("Generic failure".into()))
    }

    /// retrieve a UE's ProSe Subscription Data
    async fn get_prose_data(
        &self,
        supi: String,
        supported_features: Option<String>,
        if_none_match: Option<String>,
        if_modified_since: Option<String>,
    ) -> Result<GetProseDataResponse, ApiError> {
        log::info!(
            "get_prose_data(\"{}\", {:?}, {:?}, {:?}) ",
            supi,
            supported_features,
            if_none_match,
            if_modified_since,
        );
        Err(ApiError("Generic failure".into()))
    }

    /// Nudm_Sdm Info operation for CAG acknowledgement
    async fn cag_ack(
        &self,
        supi: String,
        acknowledge_info: Option<models::AcknowledgeInfo>,
    ) -> Result<CAgAckResponse, ApiError> {
        log::info!("cag_ack(\"{}\", {:?}) ", supi, acknowledge_info,);
        Err(ApiError("Generic failure".into()))
    }

    /// Nudm_Sdm Info operation for S-NSSAIs acknowledgement
    async fn s_nssais_ack(
        &self,
        supi: String,
        acknowledge_info: Option<models::AcknowledgeInfo>,
    ) -> Result<SNssaisAckResponse, ApiError> {
        log::info!("s_nssais_ack(\"{}\", {:?}) ", supi, acknowledge_info,);
        Err(ApiError("Generic failure".into()))
    }

    /// Nudm_Sdm Info service operation
    async fn sor_ack_info(
        &self,
        supi: String,
        acknowledge_info: Option<models::AcknowledgeInfo>,
    ) -> Result<SorAckInfoResponse, ApiError> {
        log::info!("sor_ack_info(\"{}\", {:?}) ", supi, acknowledge_info,);
        Err(ApiError("Generic failure".into()))
    }

    /// Nudm_Sdm Info for UPU service operation
    async fn upu_ack(
        &self,
        supi: String,
        acknowledge_info: Option<models::AcknowledgeInfo>,
    ) -> Result<UpuAckResponse, ApiError> {
        log::info!("upu_ack(\"{}\", {:?}) ", supi, acknowledge_info,);
        Err(ApiError("Generic failure".into()))
    }

    /// retrieve multiple data sets
    async fn get_data_sets(
        &self,
        supi: String,
        dataset_names: Vec<models::DataSetName>,
        plmn_id: Option<models::PlmnIdNid>,
        disaster_roaming_ind: Option<bool>,
        supported_features: Option<String>,
        if_none_match: Option<String>,
        if_modified_since: Option<String>,
    ) -> Result<GetDataSetsResponse, ApiError> {
        log::info!(
            "get_data_sets(\"{}\", {:?}, {:?}, {:?}, {:?}, {:?}, {:?}) ",
            supi,
            dataset_names,
            plmn_id,
            disaster_roaming_ind,
            supported_features,
            if_none_match,
            if_modified_since,
        );
        Err(ApiError("Generic failure".into()))
    }

    /// retrieve shared data
    async fn get_shared_data(
        &self,
        shared_data_ids: Vec<models::SharedDataId>,
        supported_features: Option<String>,
        supported_features2: Option<String>,
        if_none_match: Option<String>,
        if_modified_since: Option<String>,
    ) -> Result<GetSharedDataResponse, ApiError> {
        log::info!(
            "get_shared_data({:?}, {:?}, {:?}, {:?}, {:?}) ",
            shared_data_ids,
            supported_features,
            supported_features2,
            if_none_match,
            if_modified_since,
        );
        Err(ApiError("Generic failure".into()))
    }

    /// retrieve the individual shared data
    async fn get_individual_shared_data(
        &self,
        shared_data_id: Vec<models::SharedDataId>,
        supported_features: Option<String>,
        if_none_match: Option<String>,
        if_modified_since: Option<String>,
    ) -> Result<GetIndividualSharedDataResponse, ApiError> {
        log::info!(
            "get_individual_shared_data({:?}, {:?}, {:?}, {:?}) ",
            shared_data_id,
            supported_features,
            if_none_match,
            if_modified_since,
        );
        Err(ApiError("Generic failure".into()))
    }

    /// retrieve a UE's SMF Selection Subscription Data
    async fn get_smf_sel_data(
        &self,
        supi: String,
        supported_features: Option<String>,
        plmn_id: Option<models::PlmnId>,
        disaster_roaming_ind: Option<bool>,
        if_none_match: Option<String>,
        if_modified_since: Option<String>,
    ) -> Result<GetSmfSelDataResponse, ApiError> {
        log::info!(
            "get_smf_sel_data(\"{}\", {:?}, {:?}, {:?}, {:?}, {:?}) ",
            supi,
            supported_features,
            plmn_id,
            disaster_roaming_ind,
            if_none_match,
            if_modified_since,
        );
        Err(ApiError("Generic failure".into()))
    }

    /// retrieve a UE's SMS Management Subscription Data
    async fn get_sms_mngt_data(
        &self,
        supi: String,
        supported_features: Option<String>,
        plmn_id: Option<models::PlmnId>,
        if_none_match: Option<String>,
        if_modified_since: Option<String>,
    ) -> Result<GetSmsMngtDataResponse, ApiError> {
        log::info!(
            "get_sms_mngt_data(\"{}\", {:?}, {:?}, {:?}, {:?}) ",
            supi,
            supported_features,
            plmn_id,
            if_none_match,
            if_modified_since,
        );
        Err(ApiError("Generic failure".into()))
    }

    /// retrieve a UE's SMS Subscription Data
    async fn get_sms_data(
        &self,
        supi: String,
        supported_features: Option<String>,
        plmn_id: Option<models::PlmnId>,
        if_none_match: Option<String>,
        if_modified_since: Option<String>,
    ) -> Result<GetSmsDataResponse, ApiError> {
        log::info!(
            "get_sms_data(\"{}\", {:?}, {:?}, {:?}, {:?}) ",
            supi,
            supported_features,
            plmn_id,
            if_none_match,
            if_modified_since,
        );
        Err(ApiError("Generic failure".into()))
    }

    /// retrieve a UE's Session Management Subscription Data
    async fn get_sm_data(
        &self,
        supi: String,
        supported_features: Option<String>,
        single_nssai: Option<models::Snssai>,
        dnn: Option<String>,
        plmn_id: Option<models::PlmnId>,
        if_none_match: Option<String>,
        if_modified_since: Option<String>,
    ) -> Result<GetSmDataResponse, ApiError> {
        log::info!(
            "get_sm_data(\"{}\", {:?}, {:?}, {:?}, {:?}, {:?}, {:?}) ",
            supi,
            supported_features,
            single_nssai,
            dnn,
            plmn_id,
            if_none_match,
            if_modified_since,
        );
        // Err(ApiError("Generic failure".into()))

        // Here we will receive
        // Open5gs call is to /nudm-sdm/v2/imsi-001011234567895/sm-data?single-nssai=%7B%0A%09%22sst%22%3A%091%0A%7D&dnn=internet

        // and we should send
        //
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

        let sm_data = SessionManagementSubscriptionData {
            single_nssai: Snssai { sst: 1, sd: None },
            dnn_configurations: Some(HashMap::from([(
                "internet".into(),
                DnnConfiguration {
                    pdu_session_types: PduSessionTypes {
                        default_session_type: Some(PduSessionType::new("IPV4".into())),
                        allowed_session_types: Some(vec![PduSessionType::new("IPV4".into())]),
                    },
                    ssc_modes: SscModes {
                        default_ssc_mode: SscMode::new("SSC_MODE_1".into()),
                        allowed_ssc_modes: Some(vec![
                            SscMode::new("SSC_MODE_1".into()),
                            SscMode::new("SSC_MODE_2".into()),
                            SscMode::new("SSC_MODE_3".into()),
                        ]),
                    },
                    param_5g_qos_profile: Some(SubscribedDefaultQos {
                        param_5qi: 9,
                        arp: Arp {
                            priority_level: 8.into(),
                            preempt_cap: PreemptionCapability::new("NOT_PREEMPT".into()),
                            preempt_vuln: PreemptionVulnerability::new("NOT_PREEMPTABLE".into()),
                        },
                        priority_level: 8.into(),
                    }),
                    session_ambr: Some(Ambr {
                        uplink: "1048576 Kbps".into(),
                        downlink: "1048576 Kbps".into(),
                    }),
                    iwk_eps_ind: None,
                    param_3gpp_charging_characteristics: None,
                    static_ip_address: None,
                    up_security: None,
                    pdu_session_continuity_ind: None,
                    nidd_nef_id: None,
                    nidd_info: None,
                    redundant_session_allowed: None,
                    acs_info: None,
                    ipv4_frame_route_list: None,
                    ipv6_frame_route_list: None,
                    atsss_allowed: None,
                    secondary_auth: None,
                    uav_secondary_auth: None,
                    dn_aaa_ip_address_allocation: None,
                    dn_aaa_address: None,
                    additional_dn_aaa_addresses: None,
                    dn_aaa_fqdn: None,
                    iptv_acc_ctrl_info: None,
                    ipv4_index: None,
                    ipv6_index: None,
                    ecs_addr_config_info: None,
                    additional_ecs_addr_config_infos: None,
                    shared_ecs_addr_config_info: None,
                    additional_shared_ecs_addr_config_info_ids: None,
                    eas_discovery_authorized: None,
                    onboarding_ind: None,
                    aerial_ue_ind: None,
                    subscribed_max_ipv6_prefix_size: None,
                },
            )])),
            internal_group_ids: None,
            shared_vn_group_data_ids: None,
            shared_dnn_configurations_id: None,
            odb_packet_services: None,
            trace_data: None,
            shared_trace_data_id: None,
            expected_ue_behaviours_list: None,
            suggested_packet_num_dl_list: None,
            param_3gpp_charging_characteristics: None,
            supported_features,
        };

        let body = SmSubsData {
            shared_sm_subs_data_ids: vec![],
            individual_sm_subs_data: Some(vec![sm_data]),
        };
        Ok(GetSmDataResponse::ExpectedResponseToAValidRequest {
            body,
            cache_control: None,
            e_tag: None,
            last_modified: None,
        })
    }

    /// retrieve a UE's subscribed NSSAI
    async fn get_nssai(
        &self,
        supi: String,
        supported_features: Option<String>,
        plmn_id: Option<models::PlmnId>,
        disaster_roaming_ind: Option<bool>,
        if_none_match: Option<String>,
        if_modified_since: Option<String>,
    ) -> Result<GetNssaiResponse, ApiError> {
        log::info!(
            "get_nssai(\"{}\", {:?}, {:?}, {:?}, {:?}, {:?}) ",
            supi,
            supported_features,
            plmn_id,
            disaster_roaming_ind,
            if_none_match,
            if_modified_since,
        );
        Err(ApiError("Generic failure".into()))
    }

    /// subscribe to notifications
    async fn subscribe(
        &self,
        ue_id: String,
        sdm_subscription: models::SdmSubscription,
    ) -> Result<SubscribeResponse, ApiError> {
        log::info!("subscribe(\"{}\", {:?}) ", ue_id, sdm_subscription,);
        Err(ApiError("Generic failure".into()))
    }

    /// subscribe to notifications for shared data
    async fn subscribe_to_shared_data(
        &self,
        sdm_subscription: models::SdmSubscription,
    ) -> Result<SubscribeToSharedDataResponse, ApiError> {
        log::info!("subscribe_to_shared_data({:?}) ", sdm_subscription,);
        Err(ApiError("Generic failure".into()))
    }

    /// unsubscribe from notifications
    async fn unsubscribe(
        &self,
        ue_id: String,
        subscription_id: String,
    ) -> Result<UnsubscribeResponse, ApiError> {
        log::info!("unsubscribe(\"{}\", \"{}\") ", ue_id, subscription_id,);
        Err(ApiError("Generic failure".into()))
    }

    /// unsubscribe from notifications for shared data
    async fn unsubscribe_for_shared_data(
        &self,
        subscription_id: String,
    ) -> Result<UnsubscribeForSharedDataResponse, ApiError> {
        log::info!("unsubscribe_for_shared_data(\"{}\") ", subscription_id,);
        Err(ApiError("Generic failure".into()))
    }

    /// modify the subscription
    async fn modify(
        &self,
        ue_id: String,
        subscription_id: String,
        sdm_subs_modification: models::SdmSubsModification,
        supported_features: Option<String>,
    ) -> Result<ModifyResponse, ApiError> {
        log::info!(
            "modify(\"{}\", \"{}\", {:?}, {:?}) ",
            ue_id,
            subscription_id,
            sdm_subs_modification,
            supported_features,
        );
        Err(ApiError("Generic failure".into()))
    }

    /// modify the subscription
    async fn modify_shared_data_subs(
        &self,
        subscription_id: String,
        sdm_subs_modification: models::SdmSubsModification,
        supported_features: Option<String>,
    ) -> Result<ModifySharedDataSubsResponse, ApiError> {
        log::info!(
            "modify_shared_data_subs(\"{}\", {:?}, {:?}) ",
            subscription_id,
            sdm_subs_modification,
            supported_features,
        );
        Err(ApiError("Generic failure".into()))
    }

    /// retrieve a UE's Trace Configuration Data
    async fn get_trace_config_data(
        &self,
        supi: String,
        supported_features: Option<String>,
        plmn_id: Option<models::PlmnId>,
        if_none_match: Option<String>,
        if_modified_since: Option<String>,
    ) -> Result<GetTraceConfigDataResponse, ApiError> {
        log::info!(
            "get_trace_config_data(\"{}\", {:?}, {:?}, {:?}, {:?}) ",
            supi,
            supported_features,
            plmn_id,
            if_none_match,
            if_modified_since,
        );
        Err(ApiError("Generic failure".into()))
    }

    /// Nudm_Sdm custom operation to trigger SOR info update
    async fn update_sor_info(
        &self,
        supi: String,
        sor_update_info: Option<models::SorUpdateInfo>,
    ) -> Result<UpdateSorInfoResponse, ApiError> {
        log::info!("update_sor_info(\"{}\", {:?}) ", supi, sor_update_info,);
        Err(ApiError("Generic failure".into()))
    }

    /// retrieve a UE's UE Context In AMF Data
    async fn get_ue_ctx_in_amf_data(
        &self,
        supi: String,
        supported_features: Option<String>,
    ) -> Result<GetUeCtxInAmfDataResponse, ApiError> {
        log::info!(
            "get_ue_ctx_in_amf_data(\"{}\", {:?}) ",
            supi,
            supported_features,
        );
        Err(ApiError("Generic failure".into()))
    }

    /// retrieve a UE's UE Context In SMF Data
    async fn get_ue_ctx_in_smf_data(
        &self,
        supi: String,
        supported_features: Option<String>,
    ) -> Result<GetUeCtxInSmfDataResponse, ApiError> {
        log::info!(
            "get_ue_ctx_in_smf_data(\"{}\", {:?}) ",
            supi,
            supported_features,
        );
        Err(ApiError("Generic failure".into()))
    }

    /// retrieve a UE's UE Context In SMSF Data
    async fn get_ue_ctx_in_smsf_data(
        &self,
        supi: String,
        supported_features: Option<String>,
    ) -> Result<GetUeCtxInSmsfDataResponse, ApiError> {
        log::info!(
            "get_ue_ctx_in_smsf_data(\"{}\", {:?}) ",
            supi,
            supported_features,
        );
        Err(ApiError("Generic failure".into()))
    }

    /// retrieve a UE's User Consent Subscription Data
    async fn get_uc_data(
        &self,
        supi: String,
        supported_features: Option<String>,
        uc_purpose: Option<models::UcPurpose>,
        if_none_match: Option<String>,
        if_modified_since: Option<String>,
    ) -> Result<GetUcDataResponse, ApiError> {
        log::info!(
            "get_uc_data(\"{}\", {:?}, {:?}, {:?}, {:?}) ",
            supi,
            supported_features,
            uc_purpose,
            if_none_match,
            if_modified_since,
        );
        Err(ApiError("Generic failure".into()))
    }

    /// retrieve a UE's V2X Subscription Data
    async fn get_v2x_data(
        &self,
        supi: String,
        supported_features: Option<String>,
        if_none_match: Option<String>,
        if_modified_since: Option<String>,
    ) -> Result<GetV2xDataResponse, ApiError> {
        log::info!(
            "get_v2x_data(\"{}\", {:?}, {:?}, {:?}) ",
            supi,
            supported_features,
            if_none_match,
            if_modified_since,
        );
        Err(ApiError("Generic failure".into()))
    }
}
