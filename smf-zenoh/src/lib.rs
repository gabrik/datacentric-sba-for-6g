use nsfm_pdusession::{
    models, PostPduSessionsResponse, PostSmContextsResponse, ReleasePduSessionResponse,
    ReleaseSmContextResponse, RetrievePduSessionResponse, RetrieveSmContextResponse,
    SendMoDataResponse, TransferMoDataResponse, UpdatePduSessionResponse, UpdateSmContextResponse,
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
    prefix = "5gsba/nsfm-pdusession",
    service_uuid = "00000000-0000-0000-0000-00000000000C"
)]
pub trait SmfApi {
    /// Release
    async fn release_pdu_session(
        &self,
        pdu_session_ref: String,
        release_data: Option<models::ReleaseData>,
    ) -> Result<ReleasePduSessionResponse, ApiError>;

    /// Retrieve
    async fn retrieve_pdu_session(
        &self,
        pdu_session_ref: String,
        retrieve_data: models::RetrieveData,
    ) -> Result<RetrievePduSessionResponse, ApiError>;

    /// Transfer MO Data
    async fn transfer_mo_data(
        &self,
        pdu_session_ref: String,
        json_data: Option<models::TransferMoDataReqData>,
        binary_mo_data: Option<Vec<u8>>,
    ) -> Result<TransferMoDataResponse, ApiError>;

    /// Update (initiated by V-SMF or I-SMF)
    async fn update_pdu_session(
        &self,
        pdu_session_ref: String,
        hsmf_update_data: models::HsmfUpdateData,
    ) -> Result<UpdatePduSessionResponse, ApiError>;

    /// Release SM Context
    async fn release_sm_context(
        &self,
        sm_context_ref: String,
        sm_context_release_data: Option<models::SmContextReleaseData>,
    ) -> Result<ReleaseSmContextResponse, ApiError>;

    /// Retrieve SM Context
    async fn retrieve_sm_context(
        &self,
        sm_context_ref: String,
        sm_context_retrieve_data: Option<models::SmContextRetrieveData>,
    ) -> Result<RetrieveSmContextResponse, ApiError>;

    /// Send MO Data
    async fn send_mo_data(
        &self,
        sm_context_ref: String,
        json_data: Option<models::SendMoDataReqData>,
        binary_mo_data: Option<Vec<u8>>,
    ) -> Result<SendMoDataResponse, ApiError>;

    /// Update SM Context
    async fn update_sm_context(
        &self,
        sm_context_ref: String,
        sm_context_update_data: models::SmContextUpdateData,
    ) -> Result<UpdateSmContextResponse, ApiError>;

    /// Create
    async fn post_pdu_sessions(
        &self,
        pdu_session_create_data: models::PduSessionCreateData,
    ) -> Result<PostPduSessionsResponse, ApiError>;

    /// Create SM Context
    async fn post_sm_contexts(
        &self,
        json_data: Option<models::SmContextCreateData>,
        binary_data_n1_sm_message: Option<Vec<u8>>,
        binary_data_n2_sm_information: Option<Vec<u8>>,
        binary_data_n2_sm_information_ext1: Option<Vec<u8>>,
    ) -> Result<PostSmContextsResponse, ApiError>;
}
