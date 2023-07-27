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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ApiError(pub String);

