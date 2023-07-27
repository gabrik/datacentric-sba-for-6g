use nudm_sdm::{
    models, CAgAckResponse, GetAmDataResponse, GetDataSetsResponse, GetEcrDataResponse,
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
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use zrpc::zrpcresult::{ZRPCError, ZRPCResult};
use zrpc_macros::zservice;

pub mod server;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ApiError(pub String);

#[zservice(
    timeout_s = 60,
    prefix = "5gsba/nudm-sdm",
    service_uuid = "00000000-0000-0000-0000-00000000000B"
)]
pub trait UDMApi {
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
    ) -> Result<GetAmDataResponse, ApiError>;

    /// retrieve a UE's 5MBS Subscription Data
    async fn get_mbs_data(
        &self,
        supi: String,
        supported_features: Option<String>,
        if_none_match: Option<String>,
        if_modified_since: Option<String>,
    ) -> Result<GetMbsDataResponse, ApiError>;

    /// retrieve a UE's subscribed Enhanced Coverage Restriction Data
    async fn get_ecr_data(
        &self,
        supi: String,
        supported_features: Option<String>,
        if_none_match: Option<String>,
        if_modified_since: Option<String>,
    ) -> Result<GetEcrDataResponse, ApiError>;

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
    ) -> Result<GetSupiOrGpsiResponse, ApiError>;

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
    ) -> Result<GetGroupIdentifiersResponse, ApiError>;

    /// retrieve a UE's LCS Broadcast Assistance Data Types Subscription Data
    async fn get_lcs_bca_data(
        &self,
        supi: String,
        supported_features: Option<String>,
        plmn_id: Option<models::PlmnId>,
        if_none_match: Option<String>,
        if_modified_since: Option<String>,
    ) -> Result<GetLcsBcaDataResponse, ApiError>;

    /// retrieve a UE's LCS Mobile Originated Subscription Data
    async fn get_lcs_mo_data(
        &self,
        supi: String,
        supported_features: Option<String>,
        if_none_match: Option<String>,
        if_modified_since: Option<String>,
    ) -> Result<GetLcsMoDataResponse, ApiError>;

    /// retrieve a UE's LCS Privacy Subscription Data
    async fn get_lcs_privacy_data(
        &self,
        ue_id: String,
        supported_features: Option<String>,
        if_none_match: Option<String>,
        if_modified_since: Option<String>,
    ) -> Result<GetLcsPrivacyDataResponse, ApiError>;

    /// Mapping of UE Identifiers
    async fn get_multiple_identifiers(
        &self,
        gpsi_list: Vec<models::Gpsi>,
        supported_features: Option<String>,
    ) -> Result<GetMultipleIdentifiersResponse, ApiError>;

    /// retrieve a UE's ProSe Subscription Data
    async fn get_prose_data(
        &self,
        supi: String,
        supported_features: Option<String>,
        if_none_match: Option<String>,
        if_modified_since: Option<String>,
    ) -> Result<GetProseDataResponse, ApiError>;

    /// Nudm_Sdm Info operation for CAG acknowledgement
    async fn cag_ack(
        &self,
        supi: String,
        acknowledge_info: Option<models::AcknowledgeInfo>,
    ) -> Result<CAgAckResponse, ApiError>;

    /// Nudm_Sdm Info operation for S-NSSAIs acknowledgement
    async fn s_nssais_ack(
        &self,
        supi: String,
        acknowledge_info: Option<models::AcknowledgeInfo>,
    ) -> Result<SNssaisAckResponse, ApiError>;

    /// Nudm_Sdm Info service operation
    async fn sor_ack_info(
        &self,
        supi: String,
        acknowledge_info: Option<models::AcknowledgeInfo>,
    ) -> Result<SorAckInfoResponse, ApiError>;

    /// Nudm_Sdm Info for UPU service operation
    async fn upu_ack(
        &self,
        supi: String,
        acknowledge_info: Option<models::AcknowledgeInfo>,
    ) -> Result<UpuAckResponse, ApiError>;

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
    ) -> Result<GetDataSetsResponse, ApiError>;

    /// retrieve shared data
    async fn get_shared_data(
        &self,
        shared_data_ids: Vec<models::SharedDataId>,
        supported_features: Option<String>,
        supported_features2: Option<String>,
        if_none_match: Option<String>,
        if_modified_since: Option<String>,
    ) -> Result<GetSharedDataResponse, ApiError>;

    /// retrieve the individual shared data
    async fn get_individual_shared_data(
        &self,
        shared_data_id: Vec<models::SharedDataId>,
        supported_features: Option<String>,
        if_none_match: Option<String>,
        if_modified_since: Option<String>,
    ) -> Result<GetIndividualSharedDataResponse, ApiError>;

    /// retrieve a UE's SMF Selection Subscription Data
    async fn get_smf_sel_data(
        &self,
        supi: String,
        supported_features: Option<String>,
        plmn_id: Option<models::PlmnId>,
        disaster_roaming_ind: Option<bool>,
        if_none_match: Option<String>,
        if_modified_since: Option<String>,
    ) -> Result<GetSmfSelDataResponse, ApiError>;

    /// retrieve a UE's SMS Management Subscription Data
    async fn get_sms_mngt_data(
        &self,
        supi: String,
        supported_features: Option<String>,
        plmn_id: Option<models::PlmnId>,
        if_none_match: Option<String>,
        if_modified_since: Option<String>,
    ) -> Result<GetSmsMngtDataResponse, ApiError>;

    /// retrieve a UE's SMS Subscription Data
    async fn get_sms_data(
        &self,
        supi: String,
        supported_features: Option<String>,
        plmn_id: Option<models::PlmnId>,
        if_none_match: Option<String>,
        if_modified_since: Option<String>,
    ) -> Result<GetSmsDataResponse, ApiError>;

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
    ) -> Result<GetSmDataResponse, ApiError>;

    /// retrieve a UE's subscribed NSSAI
    async fn get_nssai(
        &self,
        supi: String,
        supported_features: Option<String>,
        plmn_id: Option<models::PlmnId>,
        disaster_roaming_ind: Option<bool>,
        if_none_match: Option<String>,
        if_modified_since: Option<String>,
    ) -> Result<GetNssaiResponse, ApiError>;

    /// subscribe to notifications
    async fn subscribe(
        &self,
        ue_id: String,
        sdm_subscription: models::SdmSubscription,
    ) -> Result<SubscribeResponse, ApiError>;

    /// subscribe to notifications for shared data
    async fn subscribe_to_shared_data(
        &self,
        sdm_subscription: models::SdmSubscription,
    ) -> Result<SubscribeToSharedDataResponse, ApiError>;

    /// unsubscribe from notifications
    async fn unsubscribe(
        &self,
        ue_id: String,
        subscription_id: String,
    ) -> Result<UnsubscribeResponse, ApiError>;

    /// unsubscribe from notifications for shared data
    async fn unsubscribe_for_shared_data(
        &self,
        subscription_id: String,
    ) -> Result<UnsubscribeForSharedDataResponse, ApiError>;

    /// modify the subscription
    async fn modify(
        &self,
        ue_id: String,
        subscription_id: String,
        sdm_subs_modification: models::SdmSubsModification,
        supported_features: Option<String>,
    ) -> Result<ModifyResponse, ApiError>;

    /// modify the subscription
    async fn modify_shared_data_subs(
        &self,
        subscription_id: String,
        sdm_subs_modification: models::SdmSubsModification,
        supported_features: Option<String>,
    ) -> Result<ModifySharedDataSubsResponse, ApiError>;

    /// retrieve a UE's Trace Configuration Data
    async fn get_trace_config_data(
        &self,
        supi: String,
        supported_features: Option<String>,
        plmn_id: Option<models::PlmnId>,
        if_none_match: Option<String>,
        if_modified_since: Option<String>,
    ) -> Result<GetTraceConfigDataResponse, ApiError>;

    /// Nudm_Sdm custom operation to trigger SOR info update
    async fn update_sor_info(
        &self,
        supi: String,
        sor_update_info: Option<models::SorUpdateInfo>,
    ) -> Result<UpdateSorInfoResponse, ApiError>;

    /// retrieve a UE's UE Context In AMF Data
    async fn get_ue_ctx_in_amf_data(
        &self,
        supi: String,
        supported_features: Option<String>,
    ) -> Result<GetUeCtxInAmfDataResponse, ApiError>;

    /// retrieve a UE's UE Context In SMF Data
    async fn get_ue_ctx_in_smf_data(
        &self,
        supi: String,
        supported_features: Option<String>,
    ) -> Result<GetUeCtxInSmfDataResponse, ApiError>;

    /// retrieve a UE's UE Context In SMSF Data
    async fn get_ue_ctx_in_smsf_data(
        &self,
        supi: String,
        supported_features: Option<String>,
    ) -> Result<GetUeCtxInSmsfDataResponse, ApiError>;

    /// retrieve a UE's User Consent Subscription Data
    async fn get_uc_data(
        &self,
        supi: String,
        supported_features: Option<String>,
        uc_purpose: Option<models::UcPurpose>,
        if_none_match: Option<String>,
        if_modified_since: Option<String>,
    ) -> Result<GetUcDataResponse, ApiError>;

    /// retrieve a UE's V2X Subscription Data
    async fn get_v2x_data(
        &self,
        supi: String,
        supported_features: Option<String>,
        if_none_match: Option<String>,
        if_modified_since: Option<String>,
    ) -> Result<GetV2xDataResponse, ApiError>;
}
