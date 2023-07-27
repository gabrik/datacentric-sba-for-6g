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

pub const BASE_PATH: &str = "/nudm-sdm/v2";
pub const API_VERSION: &str = "2.2.2";

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[must_use]
pub enum GetAmDataResponse {
    /// Expected response to a valid request
    ExpectedResponseToAValidRequest {
        body: models::AccessAndMobilitySubscriptionData,
        cache_control: Option<String>,
        e_tag: Option<String>,
        last_modified: Option<String>,
    },
    /// Bad request
    BadRequest(models::ProblemDetails),
    /// Not Found
    NotFound(models::ProblemDetails),
    /// Internal Server Error
    InternalServerError(models::ProblemDetails),
    /// Service Unavailable
    ServiceUnavailable(models::ProblemDetails),
    /// Unexpected error
    UnexpectedError,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[must_use]
pub enum GetMbsDataResponse {
    /// Expected response to a valid request
    ExpectedResponseToAValidRequest {
        body: models::MbsSubscriptionData,
        cache_control: Option<String>,
        e_tag: Option<String>,
        last_modified: Option<String>,
    },
    /// Bad request
    BadRequest(models::ProblemDetails),
    /// Not Found
    NotFound(models::ProblemDetails),
    /// Internal Server Error
    InternalServerError(models::ProblemDetails),
    /// Service Unavailable
    ServiceUnavailable(models::ProblemDetails),
    /// Unexpected error
    UnexpectedError,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[must_use]
pub enum GetEcrDataResponse {
    /// Expected response to a valid request
    ExpectedResponseToAValidRequest {
        body: models::EnhancedCoverageRestrictionData,
        cache_control: Option<String>,
        e_tag: Option<String>,
        last_modified: Option<String>,
    },
    /// Bad request
    BadRequest(models::ProblemDetails),
    /// Not Found
    NotFound(models::ProblemDetails),
    /// Internal Server Error
    InternalServerError(models::ProblemDetails),
    /// Service Unavailable
    ServiceUnavailable(models::ProblemDetails),
    /// Unexpected error
    UnexpectedError,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[must_use]
pub enum GetSupiOrGpsiResponse {
    /// Expected response to a valid request
    ExpectedResponseToAValidRequest {
        body: models::IdTranslationResult,
        cache_control: Option<String>,
        e_tag: Option<String>,
        last_modified: Option<String>,
    },
    /// Bad request
    BadRequest(models::ProblemDetails),
    /// Forbidden
    Forbidden(models::ProblemDetails),
    /// Not Found
    NotFound(models::ProblemDetails),
    /// Internal Server Error
    InternalServerError(models::ProblemDetails),
    /// Service Unavailable
    ServiceUnavailable(models::ProblemDetails),
    /// Unexpected error
    UnexpectedError,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[must_use]
pub enum GetGroupIdentifiersResponse {
    /// Expected response to a valid request
    ExpectedResponseToAValidRequest {
        body: models::GroupIdentifiers,
        cache_control: Option<String>,
        e_tag: Option<String>,
        last_modified: Option<String>,
    },
    /// Bad request
    BadRequest(models::ProblemDetails),
    /// Forbidden
    Forbidden(models::ProblemDetails),
    /// Not Found
    NotFound(models::ProblemDetails),
    /// Internal Server Error
    InternalServerError(models::ProblemDetails),
    /// Service Unavailable
    ServiceUnavailable(models::ProblemDetails),
    /// Unexpected error
    UnexpectedError,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[must_use]
pub enum GetLcsBcaDataResponse {
    /// Expected response to a valid request
    ExpectedResponseToAValidRequest {
        body: models::LcsBroadcastAssistanceTypesData,
        cache_control: Option<String>,
        e_tag: Option<String>,
        last_modified: Option<String>,
    },
    /// Bad request
    BadRequest(models::ProblemDetails),
    /// Not Found
    NotFound(models::ProblemDetails),
    /// Internal Server Error
    InternalServerError(models::ProblemDetails),
    /// Service Unavailable
    ServiceUnavailable(models::ProblemDetails),
    /// Unexpected error
    UnexpectedError,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[must_use]
pub enum GetLcsMoDataResponse {
    /// Expected response to a valid request
    ExpectedResponseToAValidRequest {
        body: models::LcsMoData,
        cache_control: Option<String>,
        e_tag: Option<String>,
        last_modified: Option<String>,
    },
    /// Bad request
    BadRequest(models::ProblemDetails),
    /// Not Found
    NotFound(models::ProblemDetails),
    /// Internal Server Error
    InternalServerError(models::ProblemDetails),
    /// Service Unavailable
    ServiceUnavailable(models::ProblemDetails),
    /// Unexpected error
    UnexpectedError,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[must_use]
pub enum GetLcsPrivacyDataResponse {
    /// Expected response to a valid request
    ExpectedResponseToAValidRequest {
        body: models::LcsPrivacyData,
        cache_control: Option<String>,
        e_tag: Option<String>,
        last_modified: Option<String>,
    },
    /// Bad request
    BadRequest(models::ProblemDetails),
    /// Not Found
    NotFound(models::ProblemDetails),
    /// Internal Server Error
    InternalServerError(models::ProblemDetails),
    /// Service Unavailable
    ServiceUnavailable(models::ProblemDetails),
    /// Unexpected error
    UnexpectedError,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[must_use]
pub enum GetMultipleIdentifiersResponse {
    /// Expected response to a valid request
    ExpectedResponseToAValidRequest {
        body: std::collections::HashMap<String, models::SupiInfo>,
        cache_control: Option<String>,
        e_tag: Option<String>,
        last_modified: Option<String>,
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
    /// Too Many Requests
    TooManyRequests(models::ProblemDetails),
    /// Internal Server Error
    InternalServerError(models::ProblemDetails),
    /// Bad Gateway
    BadGateway(models::ProblemDetails),
    /// Service Unavailable
    ServiceUnavailable(models::ProblemDetails),
    /// Unexpected error
    UnexpectedError,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[must_use]
pub enum GetProseDataResponse {
    /// Expected response to a valid request
    ExpectedResponseToAValidRequest {
        body: models::ProseSubscriptionData,
        cache_control: Option<String>,
        e_tag: Option<String>,
        last_modified: Option<String>,
    },
    /// Bad request
    BadRequest(models::ProblemDetails),
    /// Not Found
    NotFound(models::ProblemDetails),
    /// Internal Server Error
    InternalServerError(models::ProblemDetails),
    /// Service Unavailable
    ServiceUnavailable(models::ProblemDetails),
    /// Unexpected error
    UnexpectedError,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[must_use]
pub enum CAgAckResponse {
    /// Successful acknowledgement
    SuccessfulAcknowledgement,
    /// Bad request
    BadRequest(models::ProblemDetails),
    /// Internal Server Error
    InternalServerError(models::ProblemDetails),
    /// Service Unavailable
    ServiceUnavailable(models::ProblemDetails),
    /// Unexpected error
    UnexpectedError,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[must_use]
pub enum SNssaisAckResponse {
    /// Successful acknowledgement
    SuccessfulAcknowledgement,
    /// Bad request
    BadRequest(models::ProblemDetails),
    /// Internal Server Error
    InternalServerError(models::ProblemDetails),
    /// Service Unavailable
    ServiceUnavailable(models::ProblemDetails),
    /// Unexpected error
    UnexpectedError,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[must_use]
pub enum SorAckInfoResponse {
    /// Successful acknowledgement
    SuccessfulAcknowledgement,
    /// Bad request
    BadRequest(models::ProblemDetails),
    /// Internal Server Error
    InternalServerError(models::ProblemDetails),
    /// Service Unavailable
    ServiceUnavailable(models::ProblemDetails),
    /// Unexpected error
    UnexpectedError,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[must_use]
pub enum UpuAckResponse {
    /// Successful acknowledgement
    SuccessfulAcknowledgement,
    /// Bad request
    BadRequest(models::ProblemDetails),
    /// Internal Server Error
    InternalServerError(models::ProblemDetails),
    /// Service Unavailable
    ServiceUnavailable(models::ProblemDetails),
    /// Unexpected error
    UnexpectedError,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[must_use]
pub enum GetDataSetsResponse {
    /// Expected response to a valid request
    ExpectedResponseToAValidRequest {
        body: models::SubscriptionDataSets,
        cache_control: Option<String>,
        e_tag: Option<String>,
        last_modified: Option<String>,
    },
    /// Bad request
    BadRequest(models::ProblemDetails),
    /// Not Found
    NotFound(models::ProblemDetails),
    /// Internal Server Error
    InternalServerError(models::ProblemDetails),
    /// Service Unavailable
    ServiceUnavailable(models::ProblemDetails),
    /// Unexpected error
    UnexpectedError,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[must_use]
pub enum GetSharedDataResponse {
    /// Expected response to a valid request
    ExpectedResponseToAValidRequest {
        body: Vec<models::SharedData>,
        cache_control: Option<String>,
        e_tag: Option<String>,
        last_modified: Option<String>,
    },
    /// Bad request
    BadRequest(models::ProblemDetails),
    /// Not Found
    NotFound(models::ProblemDetails),
    /// Internal Server Error
    InternalServerError(models::ProblemDetails),
    /// Service Unavailable
    ServiceUnavailable(models::ProblemDetails),
    /// Unexpected error
    UnexpectedError,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[must_use]
pub enum GetIndividualSharedDataResponse {
    /// Expected response to a valid request
    ExpectedResponseToAValidRequest {
        body: models::SharedData,
        cache_control: Option<String>,
        e_tag: Option<String>,
        last_modified: Option<String>,
    },
    /// Bad request
    BadRequest(models::ProblemDetails),
    /// Not Found
    NotFound(models::ProblemDetails),
    /// Internal Server Error
    InternalServerError(models::ProblemDetails),
    /// Service Unavailable
    ServiceUnavailable(models::ProblemDetails),
    /// Unexpected error
    UnexpectedError,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[must_use]
pub enum GetSmfSelDataResponse {
    /// Expected response to a valid request
    ExpectedResponseToAValidRequest {
        body: models::SmfSelectionSubscriptionData,
        cache_control: Option<String>,
        e_tag: Option<String>,
        last_modified: Option<String>,
    },
    /// Bad request
    BadRequest(models::ProblemDetails),
    /// Not Found
    NotFound(models::ProblemDetails),
    /// Internal Server Error
    InternalServerError(models::ProblemDetails),
    /// Service Unavailable
    ServiceUnavailable(models::ProblemDetails),
    /// Unexpected error
    UnexpectedError,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[must_use]
pub enum GetSmsMngtDataResponse {
    /// Expected response to a valid request
    ExpectedResponseToAValidRequest {
        body: models::SmsManagementSubscriptionData,
        cache_control: Option<String>,
        e_tag: Option<String>,
        last_modified: Option<String>,
    },
    /// Bad request
    BadRequest(models::ProblemDetails),
    /// Not Found
    NotFound(models::ProblemDetails),
    /// Internal Server Error
    InternalServerError(models::ProblemDetails),
    /// Service Unavailable
    ServiceUnavailable(models::ProblemDetails),
    /// Unexpected error
    UnexpectedError,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[must_use]
pub enum GetSmsDataResponse {
    /// Expected response to a valid request
    ExpectedResponseToAValidRequest {
        body: models::SmsSubscriptionData,
        cache_control: Option<String>,
        e_tag: Option<String>,
        last_modified: Option<String>,
    },
    /// Bad request
    BadRequest(models::ProblemDetails),
    /// Not Found
    NotFound(models::ProblemDetails),
    /// Internal Server Error
    InternalServerError(models::ProblemDetails),
    /// Service Unavailable
    ServiceUnavailable(models::ProblemDetails),
    /// Unexpected error
    UnexpectedError,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[must_use]
pub enum GetSmDataResponse {
    /// Expected response to a valid request
    ExpectedResponseToAValidRequest {
        body: models::SmSubsData,
        cache_control: Option<String>,
        e_tag: Option<String>,
        last_modified: Option<String>,
    },
    /// Bad request
    BadRequest(models::ProblemDetails),
    /// Not Found
    NotFound(models::ProblemDetails),
    /// Internal Server Error
    InternalServerError(models::ProblemDetails),
    /// Service Unavailable
    ServiceUnavailable(models::ProblemDetails),
    /// Unexpected error
    UnexpectedError,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[must_use]
pub enum GetNssaiResponse {
    /// Expected response to a valid request
    ExpectedResponseToAValidRequest {
        body: models::Nssai,
        cache_control: Option<String>,
        e_tag: Option<String>,
        last_modified: Option<String>,
    },
    /// Bad request
    BadRequest(models::ProblemDetails),
    /// Not Found
    NotFound(models::ProblemDetails),
    /// Internal Server Error
    InternalServerError(models::ProblemDetails),
    /// Service Unavailable
    ServiceUnavailable(models::ProblemDetails),
    /// Unexpected error
    UnexpectedError,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[must_use]
pub enum SubscribeResponse {
    /// Expected response to a valid request
    ExpectedResponseToAValidRequest {
        body: models::SdmSubscription,
        location: String,
    },
    /// Bad request
    BadRequest(models::ProblemDetails),
    /// Not Found
    NotFound(models::ProblemDetails),
    /// Internal Server Error
    InternalServerError(models::ProblemDetails),
    /// Not Implemented
    NotImplemented(models::ProblemDetails),
    /// Service Unavailable
    ServiceUnavailable(models::ProblemDetails),
    /// Unexpected error
    UnexpectedError,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[must_use]
pub enum SubscribeToSharedDataResponse {
    /// Expected response to a valid request
    ExpectedResponseToAValidRequest {
        body: models::SdmSubscription,
        location: String,
    },
    /// Bad request
    BadRequest(models::ProblemDetails),
    /// Not Found
    NotFound(models::ProblemDetails),
    /// Unexpected error
    UnexpectedError,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[must_use]
pub enum UnsubscribeResponse {
    /// Successful response
    SuccessfulResponse,
    /// Bad request
    BadRequest(models::ProblemDetails),
    /// Not Found
    NotFound(models::ProblemDetails),
    /// Internal Server Error
    InternalServerError(models::ProblemDetails),
    /// Service Unavailable
    ServiceUnavailable(models::ProblemDetails),
    /// Unexpected error
    UnexpectedError,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[must_use]
pub enum UnsubscribeForSharedDataResponse {
    /// Successful response
    SuccessfulResponse,
    /// Bad request
    BadRequest(models::ProblemDetails),
    /// Not Found
    NotFound(models::ProblemDetails),
    /// Internal Server Error
    InternalServerError(models::ProblemDetails),
    /// Service Unavailable
    ServiceUnavailable(models::ProblemDetails),
    /// Unexpected error
    UnexpectedError,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[must_use]
pub enum ModifyResponse {
    /// Expected response to a valid request
    ExpectedResponseToAValidRequest(models::Modify200Response),
    /// Bad request
    BadRequest(models::ProblemDetails),
    /// Forbidden
    Forbidden(models::ProblemDetails),
    /// Not Found
    NotFound(models::ProblemDetails),
    /// Internal Server Error
    InternalServerError(models::ProblemDetails),
    /// Service Unavailable
    ServiceUnavailable(models::ProblemDetails),
    /// Unexpected error
    UnexpectedError,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[must_use]
pub enum ModifySharedDataSubsResponse {
    /// Expected response to a valid request
    ExpectedResponseToAValidRequest(models::Modify200Response),
    /// Bad request
    BadRequest(models::ProblemDetails),
    /// Forbidden
    Forbidden(models::ProblemDetails),
    /// Not Found
    NotFound(models::ProblemDetails),
    /// Internal Server Error
    InternalServerError(models::ProblemDetails),
    /// Service Unavailable
    ServiceUnavailable(models::ProblemDetails),
    /// Unexpected error
    UnexpectedError,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[must_use]
pub enum GetTraceConfigDataResponse {
    /// Expected response to a valid request
    ExpectedResponseToAValidRequest {
        body: models::TraceDataResponse,
        cache_control: Option<String>,
        e_tag: Option<String>,
        last_modified: Option<String>,
    },
    /// Bad request
    BadRequest(models::ProblemDetails),
    /// Not Found
    NotFound(models::ProblemDetails),
    /// Internal Server Error
    InternalServerError(models::ProblemDetails),
    /// Service Unavailable
    ServiceUnavailable(models::ProblemDetails),
    /// Unexpected error
    UnexpectedError,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[must_use]
pub enum UpdateSorInfoResponse {
    /// Expected response to a valid request
    ExpectedResponseToAValidRequest(models::SorInfo),
    /// Bad request
    BadRequest(models::ProblemDetails),
    /// Not Found
    NotFound(models::ProblemDetails),
    /// Internal Server Error
    InternalServerError(models::ProblemDetails),
    /// Service Unavailable
    ServiceUnavailable(models::ProblemDetails),
    /// Unexpected error
    UnexpectedError,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[must_use]
pub enum GetUeCtxInAmfDataResponse {
    /// Expected response to a valid request
    ExpectedResponseToAValidRequest(models::UeContextInAmfData),
    /// Bad request
    BadRequest(models::ProblemDetails),
    /// Not Found
    NotFound(models::ProblemDetails),
    /// Internal Server Error
    InternalServerError(models::ProblemDetails),
    /// Service Unavailable
    ServiceUnavailable(models::ProblemDetails),
    /// Unexpected error
    UnexpectedError,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[must_use]
pub enum GetUeCtxInSmfDataResponse {
    /// Expected response to a valid request
    ExpectedResponseToAValidRequest(models::UeContextInSmfData),
    /// Bad request
    BadRequest(models::ProblemDetails),
    /// Not Found
    NotFound(models::ProblemDetails),
    /// Internal Server Error
    InternalServerError(models::ProblemDetails),
    /// Service Unavailable
    ServiceUnavailable(models::ProblemDetails),
    /// Unexpected error
    UnexpectedError,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[must_use]
pub enum GetUeCtxInSmsfDataResponse {
    /// Expected response to a valid request
    ExpectedResponseToAValidRequest(models::UeContextInSmsfData),
    /// Bad request
    BadRequest(models::ProblemDetails),
    /// Not Found
    NotFound(models::ProblemDetails),
    /// Internal Server Error
    InternalServerError(models::ProblemDetails),
    /// Service Unavailable
    ServiceUnavailable(models::ProblemDetails),
    /// Unexpected error
    UnexpectedError,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[must_use]
pub enum GetUcDataResponse {
    /// Expected response to a valid request
    ExpectedResponseToAValidRequest {
        body: models::UcSubscriptionData,
        cache_control: Option<String>,
        e_tag: Option<String>,
        last_modified: Option<String>,
    },
    /// Bad request
    BadRequest(models::ProblemDetails),
    /// Not Found
    NotFound(models::ProblemDetails),
    /// Internal Server Error
    InternalServerError(models::ProblemDetails),
    /// Service Unavailable
    ServiceUnavailable(models::ProblemDetails),
    /// Unexpected error
    UnexpectedError,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[must_use]
pub enum GetV2xDataResponse {
    /// Expected response to a valid request
    ExpectedResponseToAValidRequest {
        body: models::V2xSubscriptionData,
        cache_control: Option<String>,
        e_tag: Option<String>,
        last_modified: Option<String>,
    },
    /// Bad request
    BadRequest(models::ProblemDetails),
    /// Not Found
    NotFound(models::ProblemDetails),
    /// Internal Server Error
    InternalServerError(models::ProblemDetails),
    /// Service Unavailable
    ServiceUnavailable(models::ProblemDetails),
    /// Unexpected error
    UnexpectedError,
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

    /// retrieve a UE's Access and Mobility Subscription Data
    async fn get_am_data(
        &self,
        supi: String,
        supported_features: Option<String>,
        plmn_id: Option<models::PlmnIdNid>,
        adjacent_plmns: Option<&Vec<models::PlmnId>>,
        disaster_roaming_ind: Option<bool>,
        if_none_match: Option<String>,
        if_modified_since: Option<String>,
        context: &C,
    ) -> Result<GetAmDataResponse, ApiError>;

    /// retrieve a UE's 5MBS Subscription Data
    async fn get_mbs_data(
        &self,
        supi: String,
        supported_features: Option<String>,
        if_none_match: Option<String>,
        if_modified_since: Option<String>,
        context: &C,
    ) -> Result<GetMbsDataResponse, ApiError>;

    /// retrieve a UE's subscribed Enhanced Coverage Restriction Data
    async fn get_ecr_data(
        &self,
        supi: String,
        supported_features: Option<String>,
        if_none_match: Option<String>,
        if_modified_since: Option<String>,
        context: &C,
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
        context: &C,
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
        context: &C,
    ) -> Result<GetGroupIdentifiersResponse, ApiError>;

    /// retrieve a UE's LCS Broadcast Assistance Data Types Subscription Data
    async fn get_lcs_bca_data(
        &self,
        supi: String,
        supported_features: Option<String>,
        plmn_id: Option<models::PlmnId>,
        if_none_match: Option<String>,
        if_modified_since: Option<String>,
        context: &C,
    ) -> Result<GetLcsBcaDataResponse, ApiError>;

    /// retrieve a UE's LCS Mobile Originated Subscription Data
    async fn get_lcs_mo_data(
        &self,
        supi: String,
        supported_features: Option<String>,
        if_none_match: Option<String>,
        if_modified_since: Option<String>,
        context: &C,
    ) -> Result<GetLcsMoDataResponse, ApiError>;

    /// retrieve a UE's LCS Privacy Subscription Data
    async fn get_lcs_privacy_data(
        &self,
        ue_id: String,
        supported_features: Option<String>,
        if_none_match: Option<String>,
        if_modified_since: Option<String>,
        context: &C,
    ) -> Result<GetLcsPrivacyDataResponse, ApiError>;

    /// Mapping of UE Identifiers
    async fn get_multiple_identifiers(
        &self,
        gpsi_list: &Vec<models::Gpsi>,
        supported_features: Option<String>,
        context: &C,
    ) -> Result<GetMultipleIdentifiersResponse, ApiError>;

    /// retrieve a UE's ProSe Subscription Data
    async fn get_prose_data(
        &self,
        supi: String,
        supported_features: Option<String>,
        if_none_match: Option<String>,
        if_modified_since: Option<String>,
        context: &C,
    ) -> Result<GetProseDataResponse, ApiError>;

    /// Nudm_Sdm Info operation for CAG acknowledgement
    async fn cag_ack(
        &self,
        supi: String,
        acknowledge_info: Option<models::AcknowledgeInfo>,
        context: &C,
    ) -> Result<CAgAckResponse, ApiError>;

    /// Nudm_Sdm Info operation for S-NSSAIs acknowledgement
    async fn s_nssais_ack(
        &self,
        supi: String,
        acknowledge_info: Option<models::AcknowledgeInfo>,
        context: &C,
    ) -> Result<SNssaisAckResponse, ApiError>;

    /// Nudm_Sdm Info service operation
    async fn sor_ack_info(
        &self,
        supi: String,
        acknowledge_info: Option<models::AcknowledgeInfo>,
        context: &C,
    ) -> Result<SorAckInfoResponse, ApiError>;

    /// Nudm_Sdm Info for UPU service operation
    async fn upu_ack(
        &self,
        supi: String,
        acknowledge_info: Option<models::AcknowledgeInfo>,
        context: &C,
    ) -> Result<UpuAckResponse, ApiError>;

    /// retrieve multiple data sets
    async fn get_data_sets(
        &self,
        supi: String,
        dataset_names: &Vec<models::DataSetName>,
        plmn_id: Option<models::PlmnIdNid>,
        disaster_roaming_ind: Option<bool>,
        supported_features: Option<String>,
        if_none_match: Option<String>,
        if_modified_since: Option<String>,
        context: &C,
    ) -> Result<GetDataSetsResponse, ApiError>;

    /// retrieve shared data
    async fn get_shared_data(
        &self,
        shared_data_ids: &Vec<models::SharedDataId>,
        supported_features: Option<String>,
        supported_features2: Option<String>,
        if_none_match: Option<String>,
        if_modified_since: Option<String>,
        context: &C,
    ) -> Result<GetSharedDataResponse, ApiError>;

    /// retrieve the individual shared data
    async fn get_individual_shared_data(
        &self,
        shared_data_id: &Vec<models::SharedDataId>,
        supported_features: Option<String>,
        if_none_match: Option<String>,
        if_modified_since: Option<String>,
        context: &C,
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
        context: &C,
    ) -> Result<GetSmfSelDataResponse, ApiError>;

    /// retrieve a UE's SMS Management Subscription Data
    async fn get_sms_mngt_data(
        &self,
        supi: String,
        supported_features: Option<String>,
        plmn_id: Option<models::PlmnId>,
        if_none_match: Option<String>,
        if_modified_since: Option<String>,
        context: &C,
    ) -> Result<GetSmsMngtDataResponse, ApiError>;

    /// retrieve a UE's SMS Subscription Data
    async fn get_sms_data(
        &self,
        supi: String,
        supported_features: Option<String>,
        plmn_id: Option<models::PlmnId>,
        if_none_match: Option<String>,
        if_modified_since: Option<String>,
        context: &C,
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
        context: &C,
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
        context: &C,
    ) -> Result<GetNssaiResponse, ApiError>;

    /// subscribe to notifications
    async fn subscribe(
        &self,
        ue_id: String,
        sdm_subscription: models::SdmSubscription,
        context: &C,
    ) -> Result<SubscribeResponse, ApiError>;

    /// subscribe to notifications for shared data
    async fn subscribe_to_shared_data(
        &self,
        sdm_subscription: models::SdmSubscription,
        context: &C,
    ) -> Result<SubscribeToSharedDataResponse, ApiError>;

    /// unsubscribe from notifications
    async fn unsubscribe(
        &self,
        ue_id: String,
        subscription_id: String,
        context: &C,
    ) -> Result<UnsubscribeResponse, ApiError>;

    /// unsubscribe from notifications for shared data
    async fn unsubscribe_for_shared_data(
        &self,
        subscription_id: String,
        context: &C,
    ) -> Result<UnsubscribeForSharedDataResponse, ApiError>;

    /// modify the subscription
    async fn modify(
        &self,
        ue_id: String,
        subscription_id: String,
        sdm_subs_modification: models::SdmSubsModification,
        supported_features: Option<String>,
        context: &C,
    ) -> Result<ModifyResponse, ApiError>;

    /// modify the subscription
    async fn modify_shared_data_subs(
        &self,
        subscription_id: String,
        sdm_subs_modification: models::SdmSubsModification,
        supported_features: Option<String>,
        context: &C,
    ) -> Result<ModifySharedDataSubsResponse, ApiError>;

    /// retrieve a UE's Trace Configuration Data
    async fn get_trace_config_data(
        &self,
        supi: String,
        supported_features: Option<String>,
        plmn_id: Option<models::PlmnId>,
        if_none_match: Option<String>,
        if_modified_since: Option<String>,
        context: &C,
    ) -> Result<GetTraceConfigDataResponse, ApiError>;

    /// Nudm_Sdm custom operation to trigger SOR info update
    async fn update_sor_info(
        &self,
        supi: String,
        sor_update_info: Option<models::SorUpdateInfo>,
        context: &C,
    ) -> Result<UpdateSorInfoResponse, ApiError>;

    /// retrieve a UE's UE Context In AMF Data
    async fn get_ue_ctx_in_amf_data(
        &self,
        supi: String,
        supported_features: Option<String>,
        context: &C,
    ) -> Result<GetUeCtxInAmfDataResponse, ApiError>;

    /// retrieve a UE's UE Context In SMF Data
    async fn get_ue_ctx_in_smf_data(
        &self,
        supi: String,
        supported_features: Option<String>,
        context: &C,
    ) -> Result<GetUeCtxInSmfDataResponse, ApiError>;

    /// retrieve a UE's UE Context In SMSF Data
    async fn get_ue_ctx_in_smsf_data(
        &self,
        supi: String,
        supported_features: Option<String>,
        context: &C,
    ) -> Result<GetUeCtxInSmsfDataResponse, ApiError>;

    /// retrieve a UE's User Consent Subscription Data
    async fn get_uc_data(
        &self,
        supi: String,
        supported_features: Option<String>,
        uc_purpose: Option<models::UcPurpose>,
        if_none_match: Option<String>,
        if_modified_since: Option<String>,
        context: &C,
    ) -> Result<GetUcDataResponse, ApiError>;

    /// retrieve a UE's V2X Subscription Data
    async fn get_v2x_data(
        &self,
        supi: String,
        supported_features: Option<String>,
        if_none_match: Option<String>,
        if_modified_since: Option<String>,
        context: &C,
    ) -> Result<GetV2xDataResponse, ApiError>;
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

    /// retrieve a UE's Access and Mobility Subscription Data
    async fn get_am_data(
        &self,
        supi: String,
        supported_features: Option<String>,
        plmn_id: Option<models::PlmnIdNid>,
        adjacent_plmns: Option<&Vec<models::PlmnId>>,
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
        gpsi_list: &Vec<models::Gpsi>,
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
        dataset_names: &Vec<models::DataSetName>,
        plmn_id: Option<models::PlmnIdNid>,
        disaster_roaming_ind: Option<bool>,
        supported_features: Option<String>,
        if_none_match: Option<String>,
        if_modified_since: Option<String>,
    ) -> Result<GetDataSetsResponse, ApiError>;

    /// retrieve shared data
    async fn get_shared_data(
        &self,
        shared_data_ids: &Vec<models::SharedDataId>,
        supported_features: Option<String>,
        supported_features2: Option<String>,
        if_none_match: Option<String>,
        if_modified_since: Option<String>,
    ) -> Result<GetSharedDataResponse, ApiError>;

    /// retrieve the individual shared data
    async fn get_individual_shared_data(
        &self,
        shared_data_id: &Vec<models::SharedDataId>,
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

    /// retrieve a UE's Access and Mobility Subscription Data
    async fn get_am_data(
        &self,
        supi: String,
        supported_features: Option<String>,
        plmn_id: Option<models::PlmnIdNid>,
        adjacent_plmns: Option<&Vec<models::PlmnId>>,
        disaster_roaming_ind: Option<bool>,
        if_none_match: Option<String>,
        if_modified_since: Option<String>,
    ) -> Result<GetAmDataResponse, ApiError> {
        let context = self.context().clone();
        self.api()
            .get_am_data(
                supi,
                supported_features,
                plmn_id,
                adjacent_plmns,
                disaster_roaming_ind,
                if_none_match,
                if_modified_since,
                &context,
            )
            .await
    }

    /// retrieve a UE's 5MBS Subscription Data
    async fn get_mbs_data(
        &self,
        supi: String,
        supported_features: Option<String>,
        if_none_match: Option<String>,
        if_modified_since: Option<String>,
    ) -> Result<GetMbsDataResponse, ApiError> {
        let context = self.context().clone();
        self.api()
            .get_mbs_data(
                supi,
                supported_features,
                if_none_match,
                if_modified_since,
                &context,
            )
            .await
    }

    /// retrieve a UE's subscribed Enhanced Coverage Restriction Data
    async fn get_ecr_data(
        &self,
        supi: String,
        supported_features: Option<String>,
        if_none_match: Option<String>,
        if_modified_since: Option<String>,
    ) -> Result<GetEcrDataResponse, ApiError> {
        let context = self.context().clone();
        self.api()
            .get_ecr_data(
                supi,
                supported_features,
                if_none_match,
                if_modified_since,
                &context,
            )
            .await
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
        let context = self.context().clone();
        self.api()
            .get_supi_or_gpsi(
                ue_id,
                supported_features,
                af_id,
                app_port_id,
                af_service_id,
                mtc_provider_info,
                requested_gpsi_type,
                if_none_match,
                if_modified_since,
                &context,
            )
            .await
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
        let context = self.context().clone();
        self.api()
            .get_group_identifiers(
                ext_group_id,
                int_group_id,
                ue_id_ind,
                supported_features,
                af_id,
                if_none_match,
                if_modified_since,
                &context,
            )
            .await
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
        let context = self.context().clone();
        self.api()
            .get_lcs_bca_data(
                supi,
                supported_features,
                plmn_id,
                if_none_match,
                if_modified_since,
                &context,
            )
            .await
    }

    /// retrieve a UE's LCS Mobile Originated Subscription Data
    async fn get_lcs_mo_data(
        &self,
        supi: String,
        supported_features: Option<String>,
        if_none_match: Option<String>,
        if_modified_since: Option<String>,
    ) -> Result<GetLcsMoDataResponse, ApiError> {
        let context = self.context().clone();
        self.api()
            .get_lcs_mo_data(
                supi,
                supported_features,
                if_none_match,
                if_modified_since,
                &context,
            )
            .await
    }

    /// retrieve a UE's LCS Privacy Subscription Data
    async fn get_lcs_privacy_data(
        &self,
        ue_id: String,
        supported_features: Option<String>,
        if_none_match: Option<String>,
        if_modified_since: Option<String>,
    ) -> Result<GetLcsPrivacyDataResponse, ApiError> {
        let context = self.context().clone();
        self.api()
            .get_lcs_privacy_data(
                ue_id,
                supported_features,
                if_none_match,
                if_modified_since,
                &context,
            )
            .await
    }

    /// Mapping of UE Identifiers
    async fn get_multiple_identifiers(
        &self,
        gpsi_list: &Vec<models::Gpsi>,
        supported_features: Option<String>,
    ) -> Result<GetMultipleIdentifiersResponse, ApiError> {
        let context = self.context().clone();
        self.api()
            .get_multiple_identifiers(gpsi_list, supported_features, &context)
            .await
    }

    /// retrieve a UE's ProSe Subscription Data
    async fn get_prose_data(
        &self,
        supi: String,
        supported_features: Option<String>,
        if_none_match: Option<String>,
        if_modified_since: Option<String>,
    ) -> Result<GetProseDataResponse, ApiError> {
        let context = self.context().clone();
        self.api()
            .get_prose_data(
                supi,
                supported_features,
                if_none_match,
                if_modified_since,
                &context,
            )
            .await
    }

    /// Nudm_Sdm Info operation for CAG acknowledgement
    async fn cag_ack(
        &self,
        supi: String,
        acknowledge_info: Option<models::AcknowledgeInfo>,
    ) -> Result<CAgAckResponse, ApiError> {
        let context = self.context().clone();
        self.api().cag_ack(supi, acknowledge_info, &context).await
    }

    /// Nudm_Sdm Info operation for S-NSSAIs acknowledgement
    async fn s_nssais_ack(
        &self,
        supi: String,
        acknowledge_info: Option<models::AcknowledgeInfo>,
    ) -> Result<SNssaisAckResponse, ApiError> {
        let context = self.context().clone();
        self.api()
            .s_nssais_ack(supi, acknowledge_info, &context)
            .await
    }

    /// Nudm_Sdm Info service operation
    async fn sor_ack_info(
        &self,
        supi: String,
        acknowledge_info: Option<models::AcknowledgeInfo>,
    ) -> Result<SorAckInfoResponse, ApiError> {
        let context = self.context().clone();
        self.api()
            .sor_ack_info(supi, acknowledge_info, &context)
            .await
    }

    /// Nudm_Sdm Info for UPU service operation
    async fn upu_ack(
        &self,
        supi: String,
        acknowledge_info: Option<models::AcknowledgeInfo>,
    ) -> Result<UpuAckResponse, ApiError> {
        let context = self.context().clone();
        self.api().upu_ack(supi, acknowledge_info, &context).await
    }

    /// retrieve multiple data sets
    async fn get_data_sets(
        &self,
        supi: String,
        dataset_names: &Vec<models::DataSetName>,
        plmn_id: Option<models::PlmnIdNid>,
        disaster_roaming_ind: Option<bool>,
        supported_features: Option<String>,
        if_none_match: Option<String>,
        if_modified_since: Option<String>,
    ) -> Result<GetDataSetsResponse, ApiError> {
        let context = self.context().clone();
        self.api()
            .get_data_sets(
                supi,
                dataset_names,
                plmn_id,
                disaster_roaming_ind,
                supported_features,
                if_none_match,
                if_modified_since,
                &context,
            )
            .await
    }

    /// retrieve shared data
    async fn get_shared_data(
        &self,
        shared_data_ids: &Vec<models::SharedDataId>,
        supported_features: Option<String>,
        supported_features2: Option<String>,
        if_none_match: Option<String>,
        if_modified_since: Option<String>,
    ) -> Result<GetSharedDataResponse, ApiError> {
        let context = self.context().clone();
        self.api()
            .get_shared_data(
                shared_data_ids,
                supported_features,
                supported_features2,
                if_none_match,
                if_modified_since,
                &context,
            )
            .await
    }

    /// retrieve the individual shared data
    async fn get_individual_shared_data(
        &self,
        shared_data_id: &Vec<models::SharedDataId>,
        supported_features: Option<String>,
        if_none_match: Option<String>,
        if_modified_since: Option<String>,
    ) -> Result<GetIndividualSharedDataResponse, ApiError> {
        let context = self.context().clone();
        self.api()
            .get_individual_shared_data(
                shared_data_id,
                supported_features,
                if_none_match,
                if_modified_since,
                &context,
            )
            .await
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
        let context = self.context().clone();
        self.api()
            .get_smf_sel_data(
                supi,
                supported_features,
                plmn_id,
                disaster_roaming_ind,
                if_none_match,
                if_modified_since,
                &context,
            )
            .await
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
        let context = self.context().clone();
        self.api()
            .get_sms_mngt_data(
                supi,
                supported_features,
                plmn_id,
                if_none_match,
                if_modified_since,
                &context,
            )
            .await
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
        let context = self.context().clone();
        self.api()
            .get_sms_data(
                supi,
                supported_features,
                plmn_id,
                if_none_match,
                if_modified_since,
                &context,
            )
            .await
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
        let context = self.context().clone();
        self.api()
            .get_sm_data(
                supi,
                supported_features,
                single_nssai,
                dnn,
                plmn_id,
                if_none_match,
                if_modified_since,
                &context,
            )
            .await
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
        let context = self.context().clone();
        self.api()
            .get_nssai(
                supi,
                supported_features,
                plmn_id,
                disaster_roaming_ind,
                if_none_match,
                if_modified_since,
                &context,
            )
            .await
    }

    /// subscribe to notifications
    async fn subscribe(
        &self,
        ue_id: String,
        sdm_subscription: models::SdmSubscription,
    ) -> Result<SubscribeResponse, ApiError> {
        let context = self.context().clone();
        self.api()
            .subscribe(ue_id, sdm_subscription, &context)
            .await
    }

    /// subscribe to notifications for shared data
    async fn subscribe_to_shared_data(
        &self,
        sdm_subscription: models::SdmSubscription,
    ) -> Result<SubscribeToSharedDataResponse, ApiError> {
        let context = self.context().clone();
        self.api()
            .subscribe_to_shared_data(sdm_subscription, &context)
            .await
    }

    /// unsubscribe from notifications
    async fn unsubscribe(
        &self,
        ue_id: String,
        subscription_id: String,
    ) -> Result<UnsubscribeResponse, ApiError> {
        let context = self.context().clone();
        self.api()
            .unsubscribe(ue_id, subscription_id, &context)
            .await
    }

    /// unsubscribe from notifications for shared data
    async fn unsubscribe_for_shared_data(
        &self,
        subscription_id: String,
    ) -> Result<UnsubscribeForSharedDataResponse, ApiError> {
        let context = self.context().clone();
        self.api()
            .unsubscribe_for_shared_data(subscription_id, &context)
            .await
    }

    /// modify the subscription
    async fn modify(
        &self,
        ue_id: String,
        subscription_id: String,
        sdm_subs_modification: models::SdmSubsModification,
        supported_features: Option<String>,
    ) -> Result<ModifyResponse, ApiError> {
        let context = self.context().clone();
        self.api()
            .modify(
                ue_id,
                subscription_id,
                sdm_subs_modification,
                supported_features,
                &context,
            )
            .await
    }

    /// modify the subscription
    async fn modify_shared_data_subs(
        &self,
        subscription_id: String,
        sdm_subs_modification: models::SdmSubsModification,
        supported_features: Option<String>,
    ) -> Result<ModifySharedDataSubsResponse, ApiError> {
        let context = self.context().clone();
        self.api()
            .modify_shared_data_subs(
                subscription_id,
                sdm_subs_modification,
                supported_features,
                &context,
            )
            .await
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
        let context = self.context().clone();
        self.api()
            .get_trace_config_data(
                supi,
                supported_features,
                plmn_id,
                if_none_match,
                if_modified_since,
                &context,
            )
            .await
    }

    /// Nudm_Sdm custom operation to trigger SOR info update
    async fn update_sor_info(
        &self,
        supi: String,
        sor_update_info: Option<models::SorUpdateInfo>,
    ) -> Result<UpdateSorInfoResponse, ApiError> {
        let context = self.context().clone();
        self.api()
            .update_sor_info(supi, sor_update_info, &context)
            .await
    }

    /// retrieve a UE's UE Context In AMF Data
    async fn get_ue_ctx_in_amf_data(
        &self,
        supi: String,
        supported_features: Option<String>,
    ) -> Result<GetUeCtxInAmfDataResponse, ApiError> {
        let context = self.context().clone();
        self.api()
            .get_ue_ctx_in_amf_data(supi, supported_features, &context)
            .await
    }

    /// retrieve a UE's UE Context In SMF Data
    async fn get_ue_ctx_in_smf_data(
        &self,
        supi: String,
        supported_features: Option<String>,
    ) -> Result<GetUeCtxInSmfDataResponse, ApiError> {
        let context = self.context().clone();
        self.api()
            .get_ue_ctx_in_smf_data(supi, supported_features, &context)
            .await
    }

    /// retrieve a UE's UE Context In SMSF Data
    async fn get_ue_ctx_in_smsf_data(
        &self,
        supi: String,
        supported_features: Option<String>,
    ) -> Result<GetUeCtxInSmsfDataResponse, ApiError> {
        let context = self.context().clone();
        self.api()
            .get_ue_ctx_in_smsf_data(supi, supported_features, &context)
            .await
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
        let context = self.context().clone();
        self.api()
            .get_uc_data(
                supi,
                supported_features,
                uc_purpose,
                if_none_match,
                if_modified_since,
                &context,
            )
            .await
    }

    /// retrieve a UE's V2X Subscription Data
    async fn get_v2x_data(
        &self,
        supi: String,
        supported_features: Option<String>,
        if_none_match: Option<String>,
        if_modified_since: Option<String>,
    ) -> Result<GetV2xDataResponse, ApiError> {
        let context = self.context().clone();
        self.api()
            .get_v2x_data(
                supi,
                supported_features,
                if_none_match,
                if_modified_since,
                &context,
            )
            .await
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[must_use]
pub enum DatachangeNotificationRequestBodyCallbackReferencePostResponse {
    /// Successful Notification response
    SuccessfulNotificationResponse,
    /// Temporary Redirect
    TemporaryRedirect {
        body: models::RedirectResponse,
        location: String,
        param_3gpp_sbi_target_nf_id: Option<String>,
    },
    /// Permanent Redirect
    PermanentRedirect {
        body: models::RedirectResponse,
        location: String,
        param_3gpp_sbi_target_nf_id: Option<String>,
    },
    /// Bad request
    BadRequest(models::ProblemDetails),
    /// Not Found
    NotFound(models::ProblemDetails),
    /// Internal Server Error
    InternalServerError(models::ProblemDetails),
    /// Service Unavailable
    ServiceUnavailable(models::ProblemDetails),
    /// Unexpected error
    UnexpectedError,
}

// #[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
// #[must_use]
// pub enum DatachangeNotificationRequestBodyCallbackReferencePostResponse {
//     /// Successful Notification response
//     SuccessfulNotificationResponse
//     ,
//     /// Temporary Redirect
//     TemporaryRedirect
//     {
//         body: models::RedirectResponse,
//         location:
//         String
//         ,
//         param_3gpp_sbi_target_nf_id:
//         Option<
//         String
//         >
//     }
//     ,
//     /// Permanent Redirect
//     PermanentRedirect
//     {
//         body: models::RedirectResponse,
//         location:
//         String
//         ,
//         param_3gpp_sbi_target_nf_id:
//         Option<
//         String
//         >
//     }
//     ,
//     /// Bad request
//     BadRequest
//     (models::ProblemDetails)
//     ,
//     /// Not Found
//     NotFound
//     (models::ProblemDetails)
//     ,
//     /// Internal Server Error
//     InternalServerError
//     (models::ProblemDetails)
//     ,
//     /// Service Unavailable
//     ServiceUnavailable
//     (models::ProblemDetails)
//     ,
//     /// Unexpected error
//     UnexpectedError
// }

/// Callback API
#[async_trait]
pub trait CallbackApi<C: Send + Sync> {
    fn poll_ready(
        &self,
        _cx: &mut Context,
    ) -> Poll<Result<(), Box<dyn Error + Send + Sync + 'static>>> {
        Poll::Ready(Ok(()))
    }

    async fn datachange_notification_request_body_callback_reference_post(
        &self,
        modification_notification: models::ModificationNotification,
        context: &C,
    ) -> Result<DatachangeNotificationRequestBodyCallbackReferencePostResponse, ApiError>;
}

/// Callback API without a `Context`
#[async_trait]
pub trait CallbackApiNoContext<C: Send + Sync> {
    fn poll_ready(
        &self,
        _cx: &mut Context,
    ) -> Poll<Result<(), Box<dyn Error + Send + Sync + 'static>>>;

    fn context(&self) -> &C;

    async fn datachange_notification_request_body_callback_reference_post(
        &self,
        modification_notification: models::ModificationNotification,
    ) -> Result<DatachangeNotificationRequestBodyCallbackReferencePostResponse, ApiError>;

    // async fn datachange_notification_request_body_callback_reference_post(
    //     &self,
    //     modification_notification: models::ModificationNotification,
    //     ) -> Result<DatachangeNotificationRequestBodyCallbackReferencePostResponse, ApiError>;
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

    async fn datachange_notification_request_body_callback_reference_post(
        &self,
        modification_notification: models::ModificationNotification,
    ) -> Result<DatachangeNotificationRequestBodyCallbackReferencePostResponse, ApiError> {
        let context = self.context().clone();
        self.api()
            .datachange_notification_request_body_callback_reference_post(
                modification_notification,
                &context,
            )
            .await
    }

    // async fn datachange_notification_request_body_callback_reference_post(
    //     &self,
    //     modification_notification: models::ModificationNotification,
    //     ) -> Result<DatachangeNotificationRequestBodyCallbackReferencePostResponse, ApiError>
    // {
    //     let context = self.context().clone();
    //     self.api().datachange_notification_request_body_callback_reference_post(
    //         modification_notification,
    //         &context).await
    // }
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
