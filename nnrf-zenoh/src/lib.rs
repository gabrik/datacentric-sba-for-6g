use nnrf_discovery_server::{
    models, RetrieveCompleteSearchResponse, RetrieveStoredSearchResponse,
    SCpDomainRoutingInfoGetResponse, ScpDomainRoutingInfoSubscribeResponse,
    ScpDomainRoutingInfoUnsubscribeResponse, SearchNfInstancesResponse,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;


#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ApiError(pub String);
