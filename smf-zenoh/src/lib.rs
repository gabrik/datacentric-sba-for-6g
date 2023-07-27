use nsfm_pdusession::{
    models, PostPduSessionsResponse, PostSmContextsResponse, ReleasePduSessionResponse,
    ReleaseSmContextResponse, RetrievePduSessionResponse, RetrieveSmContextResponse,
    SendMoDataResponse, TransferMoDataResponse, UpdatePduSessionResponse, UpdateSmContextResponse,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;


#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ApiError(pub String);

