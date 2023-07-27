//! Main library entry point for nudm_sdm implementation.

#![allow(unused_imports)]

use async_trait::async_trait;
use futures::{future, Stream, StreamExt, TryFutureExt, TryStreamExt};
use hyper::server::conn::Http;
use hyper::service::Service;
use log::info;
use std::future::Future;
use std::marker::PhantomData;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll};
use swagger::auth::MakeAllowAllAuthenticator;
use swagger::EmptyContext;
use swagger::{Has, XSpanIdString};
use tokio::net::TcpListener;

#[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "ios")))]
use openssl::ssl::{Ssl, SslAcceptor, SslAcceptorBuilder, SslFiletype, SslMethod};

use nudm_sdm::models::{self, SessionManagementSubscriptionData, SmSubsData};

/// Builds an SSL implementation for Simple HTTPS from some hard-coded file names
pub async fn create(addr: &str, https: bool) {
    let addr = addr.parse().expect("Failed to parse bind address");

    let server = Server::new();

    let service = MakeService::new(server);

    let service = MakeAllowAllAuthenticator::new(service, "cosmo");

    #[allow(unused_mut)]
    let mut service = nudm_sdm::server::context::MakeAddContext::<_, EmptyContext>::new(service);

    if https {
        #[cfg(any(target_os = "macos", target_os = "windows", target_os = "ios"))]
        {
            unimplemented!("SSL is not implemented for the examples on MacOS, Windows or iOS");
        }

        #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "ios")))]
        {
            let mut ssl = SslAcceptor::mozilla_intermediate_v5(SslMethod::tls())
                .expect("Failed to create SSL Acceptor");

            // Server authentication
            ssl.set_private_key_file("examples/server-key.pem", SslFiletype::PEM)
                .expect("Failed to set private key");
            ssl.set_certificate_chain_file("examples/server-chain.pem")
                .expect("Failed to set certificate chain");
            ssl.check_private_key()
                .expect("Failed to check private key");

            let tls_acceptor = ssl.build();
            let tcp_listener = TcpListener::bind(&addr).await.unwrap();

            loop {
                if let Ok((tcp, _)) = tcp_listener.accept().await {
                    let ssl = Ssl::new(tls_acceptor.context()).unwrap();
                    let addr = tcp.peer_addr().expect("Unable to get remote address");
                    let service = service.call(addr);

                    tokio::spawn(async move {
                        let tls = tokio_openssl::SslStream::new(ssl, tcp).map_err(|_| ())?;
                        let service = service.await.map_err(|_| ())?;

                        Http::new()
                            .serve_connection(tls, service)
                            .await
                            .map_err(|_| ())
                    });
                }
            }
        }
    } else {
        // Using HTTP
        hyper::server::Server::bind(&addr)
            .serve(service)
            .await
            .unwrap()
    }
}

#[derive(Copy, Clone)]
pub struct Server<C> {
    marker: PhantomData<C>,
}

impl<C> Server<C> {
    pub fn new() -> Self {
        Server {
            marker: PhantomData,
        }
    }
}

use nudm_sdm::server::MakeService;
use nudm_sdm::{
    Api, CAgAckResponse, GetAmDataResponse, GetDataSetsResponse, GetEcrDataResponse,
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
use std::error::Error;
use swagger::ApiError;

#[async_trait]
impl<C> Api<C> for Server<C>
where
    C: Has<XSpanIdString> + Send + Sync,
{
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
    ) -> Result<GetAmDataResponse, ApiError> {
        let context = context.clone();
        info!(
            "get_am_data(\"{}\", {:?}, {:?}, {:?}, {:?}, {:?}, {:?}) - X-Span-ID: {:?}",
            supi,
            supported_features,
            plmn_id,
            adjacent_plmns,
            disaster_roaming_ind,
            if_none_match,
            if_modified_since,
            context.get().0.clone()
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
        context: &C,
    ) -> Result<GetMbsDataResponse, ApiError> {
        let context = context.clone();
        info!(
            "get_mbs_data(\"{}\", {:?}, {:?}, {:?}) - X-Span-ID: {:?}",
            supi,
            supported_features,
            if_none_match,
            if_modified_since,
            context.get().0.clone()
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
        context: &C,
    ) -> Result<GetEcrDataResponse, ApiError> {
        let context = context.clone();
        info!(
            "get_ecr_data(\"{}\", {:?}, {:?}, {:?}) - X-Span-ID: {:?}",
            supi,
            supported_features,
            if_none_match,
            if_modified_since,
            context.get().0.clone()
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
        context: &C,
    ) -> Result<GetSupiOrGpsiResponse, ApiError> {
        let context = context.clone();
        info!("get_supi_or_gpsi(\"{}\", {:?}, {:?}, {:?}, {:?}, {:?}, {:?}, {:?}, {:?}) - X-Span-ID: {:?}", ue_id, supported_features, af_id, app_port_id, af_service_id, mtc_provider_info, requested_gpsi_type, if_none_match, if_modified_since, context.get().0.clone());
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
        context: &C,
    ) -> Result<GetGroupIdentifiersResponse, ApiError> {
        let context = context.clone();
        info!(
            "get_group_identifiers({:?}, {:?}, {:?}, {:?}, {:?}, {:?}, {:?}) - X-Span-ID: {:?}",
            ext_group_id,
            int_group_id,
            ue_id_ind,
            supported_features,
            af_id,
            if_none_match,
            if_modified_since,
            context.get().0.clone()
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
        context: &C,
    ) -> Result<GetLcsBcaDataResponse, ApiError> {
        let context = context.clone();
        info!(
            "get_lcs_bca_data(\"{}\", {:?}, {:?}, {:?}, {:?}) - X-Span-ID: {:?}",
            supi,
            supported_features,
            plmn_id,
            if_none_match,
            if_modified_since,
            context.get().0.clone()
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
        context: &C,
    ) -> Result<GetLcsMoDataResponse, ApiError> {
        let context = context.clone();
        info!(
            "get_lcs_mo_data(\"{}\", {:?}, {:?}, {:?}) - X-Span-ID: {:?}",
            supi,
            supported_features,
            if_none_match,
            if_modified_since,
            context.get().0.clone()
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
        context: &C,
    ) -> Result<GetLcsPrivacyDataResponse, ApiError> {
        let context = context.clone();
        info!(
            "get_lcs_privacy_data(\"{}\", {:?}, {:?}, {:?}) - X-Span-ID: {:?}",
            ue_id,
            supported_features,
            if_none_match,
            if_modified_since,
            context.get().0.clone()
        );
        Err(ApiError("Generic failure".into()))
    }

    /// Mapping of UE Identifiers
    async fn get_multiple_identifiers(
        &self,
        gpsi_list: &Vec<models::Gpsi>,
        supported_features: Option<String>,
        context: &C,
    ) -> Result<GetMultipleIdentifiersResponse, ApiError> {
        let context = context.clone();
        info!(
            "get_multiple_identifiers({:?}, {:?}) - X-Span-ID: {:?}",
            gpsi_list,
            supported_features,
            context.get().0.clone()
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
        context: &C,
    ) -> Result<GetProseDataResponse, ApiError> {
        let context = context.clone();
        info!(
            "get_prose_data(\"{}\", {:?}, {:?}, {:?}) - X-Span-ID: {:?}",
            supi,
            supported_features,
            if_none_match,
            if_modified_since,
            context.get().0.clone()
        );
        Err(ApiError("Generic failure".into()))
    }

    /// Nudm_Sdm Info operation for CAG acknowledgement
    async fn cag_ack(
        &self,
        supi: String,
        acknowledge_info: Option<models::AcknowledgeInfo>,
        context: &C,
    ) -> Result<CAgAckResponse, ApiError> {
        let context = context.clone();
        info!(
            "cag_ack(\"{}\", {:?}) - X-Span-ID: {:?}",
            supi,
            acknowledge_info,
            context.get().0.clone()
        );
        Err(ApiError("Generic failure".into()))
    }

    /// Nudm_Sdm Info operation for S-NSSAIs acknowledgement
    async fn s_nssais_ack(
        &self,
        supi: String,
        acknowledge_info: Option<models::AcknowledgeInfo>,
        context: &C,
    ) -> Result<SNssaisAckResponse, ApiError> {
        let context = context.clone();
        info!(
            "s_nssais_ack(\"{}\", {:?}) - X-Span-ID: {:?}",
            supi,
            acknowledge_info,
            context.get().0.clone()
        );
        Err(ApiError("Generic failure".into()))
    }

    /// Nudm_Sdm Info service operation
    async fn sor_ack_info(
        &self,
        supi: String,
        acknowledge_info: Option<models::AcknowledgeInfo>,
        context: &C,
    ) -> Result<SorAckInfoResponse, ApiError> {
        let context = context.clone();
        info!(
            "sor_ack_info(\"{}\", {:?}) - X-Span-ID: {:?}",
            supi,
            acknowledge_info,
            context.get().0.clone()
        );
        Err(ApiError("Generic failure".into()))
    }

    /// Nudm_Sdm Info for UPU service operation
    async fn upu_ack(
        &self,
        supi: String,
        acknowledge_info: Option<models::AcknowledgeInfo>,
        context: &C,
    ) -> Result<UpuAckResponse, ApiError> {
        let context = context.clone();
        info!(
            "upu_ack(\"{}\", {:?}) - X-Span-ID: {:?}",
            supi,
            acknowledge_info,
            context.get().0.clone()
        );
        Err(ApiError("Generic failure".into()))
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
        context: &C,
    ) -> Result<GetDataSetsResponse, ApiError> {
        let context = context.clone();
        info!(
            "get_data_sets(\"{}\", {:?}, {:?}, {:?}, {:?}, {:?}, {:?}) - X-Span-ID: {:?}",
            supi,
            dataset_names,
            plmn_id,
            disaster_roaming_ind,
            supported_features,
            if_none_match,
            if_modified_since,
            context.get().0.clone()
        );
        Err(ApiError("Generic failure".into()))
    }

    /// retrieve shared data
    async fn get_shared_data(
        &self,
        shared_data_ids: &Vec<models::SharedDataId>,
        supported_features: Option<String>,
        supported_features2: Option<String>,
        if_none_match: Option<String>,
        if_modified_since: Option<String>,
        context: &C,
    ) -> Result<GetSharedDataResponse, ApiError> {
        let context = context.clone();
        info!(
            "get_shared_data({:?}, {:?}, {:?}, {:?}, {:?}) - X-Span-ID: {:?}",
            shared_data_ids,
            supported_features,
            supported_features2,
            if_none_match,
            if_modified_since,
            context.get().0.clone()
        );
        Err(ApiError("Generic failure".into()))
    }

    /// retrieve the individual shared data
    async fn get_individual_shared_data(
        &self,
        shared_data_id: &Vec<models::SharedDataId>,
        supported_features: Option<String>,
        if_none_match: Option<String>,
        if_modified_since: Option<String>,
        context: &C,
    ) -> Result<GetIndividualSharedDataResponse, ApiError> {
        let context = context.clone();
        info!(
            "get_individual_shared_data({:?}, {:?}, {:?}, {:?}) - X-Span-ID: {:?}",
            shared_data_id,
            supported_features,
            if_none_match,
            if_modified_since,
            context.get().0.clone()
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
        context: &C,
    ) -> Result<GetSmfSelDataResponse, ApiError> {
        let context = context.clone();
        info!(
            "get_smf_sel_data(\"{}\", {:?}, {:?}, {:?}, {:?}, {:?}) - X-Span-ID: {:?}",
            supi,
            supported_features,
            plmn_id,
            disaster_roaming_ind,
            if_none_match,
            if_modified_since,
            context.get().0.clone()
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
        context: &C,
    ) -> Result<GetSmsMngtDataResponse, ApiError> {
        let context = context.clone();
        info!(
            "get_sms_mngt_data(\"{}\", {:?}, {:?}, {:?}, {:?}) - X-Span-ID: {:?}",
            supi,
            supported_features,
            plmn_id,
            if_none_match,
            if_modified_since,
            context.get().0.clone()
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
        context: &C,
    ) -> Result<GetSmsDataResponse, ApiError> {
        let context = context.clone();
        info!(
            "get_sms_data(\"{}\", {:?}, {:?}, {:?}, {:?}) - X-Span-ID: {:?}",
            supi,
            supported_features,
            plmn_id,
            if_none_match,
            if_modified_since,
            context.get().0.clone()
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
        context: &C,
    ) -> Result<GetSmDataResponse, ApiError> {
        let context = context.clone();
        info!(
            "get_sm_data(\"{}\", {:?}, {:?}, {:?}, {:?}, {:?}, {:?}) - X-Span-ID: {:?}",
            supi,
            supported_features,
            single_nssai,
            dnn,
            plmn_id,
            if_none_match,
            if_modified_since,
            context.get().0.clone()
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

        let sm_data = serde_json::from_str::<SessionManagementSubscriptionData>(r#"{"singleNssai":{"sst":1},"dnnConfigurations":{"internet":{"pduSessionTypes":{"defaultSessionType":"IPV4","allowedSessionTypes":["IPV4"]},"sscModes":{"defaultSscMode":"SSC_MODE_1","allowedSscModes":["SSC_MODE_1","SSC_MODE_2","SSC_MODE_3"]},"5gQosProfile":{"5qi":9,"arp":{"priorityLevel":8,"preemptCap":"NOT_PREEMPT","preemptVuln":"NOT_PREEMPTABLE"},"priorityLevel":8},"sessionAmbr":{"uplink":"1048576 Kbps","downlink":"1048576 Kbps"}}}}"#).expect("unable to parse json");
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
        context: &C,
    ) -> Result<GetNssaiResponse, ApiError> {
        let context = context.clone();
        info!(
            "get_nssai(\"{}\", {:?}, {:?}, {:?}, {:?}, {:?}) - X-Span-ID: {:?}",
            supi,
            supported_features,
            plmn_id,
            disaster_roaming_ind,
            if_none_match,
            if_modified_since,
            context.get().0.clone()
        );
        Err(ApiError("Generic failure".into()))
    }

    /// subscribe to notifications
    async fn subscribe(
        &self,
        ue_id: String,
        sdm_subscription: models::SdmSubscription,
        context: &C,
    ) -> Result<SubscribeResponse, ApiError> {
        let context = context.clone();
        info!(
            "subscribe(\"{}\", {:?}) - X-Span-ID: {:?}",
            ue_id,
            sdm_subscription,
            context.get().0.clone()
        );
        Err(ApiError("Generic failure".into()))
    }

    /// subscribe to notifications for shared data
    async fn subscribe_to_shared_data(
        &self,
        sdm_subscription: models::SdmSubscription,
        context: &C,
    ) -> Result<SubscribeToSharedDataResponse, ApiError> {
        let context = context.clone();
        info!(
            "subscribe_to_shared_data({:?}) - X-Span-ID: {:?}",
            sdm_subscription,
            context.get().0.clone()
        );
        Err(ApiError("Generic failure".into()))
    }

    /// unsubscribe from notifications
    async fn unsubscribe(
        &self,
        ue_id: String,
        subscription_id: String,
        context: &C,
    ) -> Result<UnsubscribeResponse, ApiError> {
        let context = context.clone();
        info!(
            "unsubscribe(\"{}\", \"{}\") - X-Span-ID: {:?}",
            ue_id,
            subscription_id,
            context.get().0.clone()
        );
        Err(ApiError("Generic failure".into()))
    }

    /// unsubscribe from notifications for shared data
    async fn unsubscribe_for_shared_data(
        &self,
        subscription_id: String,
        context: &C,
    ) -> Result<UnsubscribeForSharedDataResponse, ApiError> {
        let context = context.clone();
        info!(
            "unsubscribe_for_shared_data(\"{}\") - X-Span-ID: {:?}",
            subscription_id,
            context.get().0.clone()
        );
        Err(ApiError("Generic failure".into()))
    }

    /// modify the subscription
    async fn modify(
        &self,
        ue_id: String,
        subscription_id: String,
        sdm_subs_modification: models::SdmSubsModification,
        supported_features: Option<String>,
        context: &C,
    ) -> Result<ModifyResponse, ApiError> {
        let context = context.clone();
        info!(
            "modify(\"{}\", \"{}\", {:?}, {:?}) - X-Span-ID: {:?}",
            ue_id,
            subscription_id,
            sdm_subs_modification,
            supported_features,
            context.get().0.clone()
        );
        Err(ApiError("Generic failure".into()))
    }

    /// modify the subscription
    async fn modify_shared_data_subs(
        &self,
        subscription_id: String,
        sdm_subs_modification: models::SdmSubsModification,
        supported_features: Option<String>,
        context: &C,
    ) -> Result<ModifySharedDataSubsResponse, ApiError> {
        let context = context.clone();
        info!(
            "modify_shared_data_subs(\"{}\", {:?}, {:?}) - X-Span-ID: {:?}",
            subscription_id,
            sdm_subs_modification,
            supported_features,
            context.get().0.clone()
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
        context: &C,
    ) -> Result<GetTraceConfigDataResponse, ApiError> {
        let context = context.clone();
        info!(
            "get_trace_config_data(\"{}\", {:?}, {:?}, {:?}, {:?}) - X-Span-ID: {:?}",
            supi,
            supported_features,
            plmn_id,
            if_none_match,
            if_modified_since,
            context.get().0.clone()
        );
        Err(ApiError("Generic failure".into()))
    }

    /// Nudm_Sdm custom operation to trigger SOR info update
    async fn update_sor_info(
        &self,
        supi: String,
        sor_update_info: Option<models::SorUpdateInfo>,
        context: &C,
    ) -> Result<UpdateSorInfoResponse, ApiError> {
        let context = context.clone();
        info!(
            "update_sor_info(\"{}\", {:?}) - X-Span-ID: {:?}",
            supi,
            sor_update_info,
            context.get().0.clone()
        );
        Err(ApiError("Generic failure".into()))
    }

    /// retrieve a UE's UE Context In AMF Data
    async fn get_ue_ctx_in_amf_data(
        &self,
        supi: String,
        supported_features: Option<String>,
        context: &C,
    ) -> Result<GetUeCtxInAmfDataResponse, ApiError> {
        let context = context.clone();
        info!(
            "get_ue_ctx_in_amf_data(\"{}\", {:?}) - X-Span-ID: {:?}",
            supi,
            supported_features,
            context.get().0.clone()
        );
        Err(ApiError("Generic failure".into()))
    }

    /// retrieve a UE's UE Context In SMF Data
    async fn get_ue_ctx_in_smf_data(
        &self,
        supi: String,
        supported_features: Option<String>,
        context: &C,
    ) -> Result<GetUeCtxInSmfDataResponse, ApiError> {
        let context = context.clone();
        info!(
            "get_ue_ctx_in_smf_data(\"{}\", {:?}) - X-Span-ID: {:?}",
            supi,
            supported_features,
            context.get().0.clone()
        );
        Err(ApiError("Generic failure".into()))
    }

    /// retrieve a UE's UE Context In SMSF Data
    async fn get_ue_ctx_in_smsf_data(
        &self,
        supi: String,
        supported_features: Option<String>,
        context: &C,
    ) -> Result<GetUeCtxInSmsfDataResponse, ApiError> {
        let context = context.clone();
        info!(
            "get_ue_ctx_in_smsf_data(\"{}\", {:?}) - X-Span-ID: {:?}",
            supi,
            supported_features,
            context.get().0.clone()
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
        context: &C,
    ) -> Result<GetUcDataResponse, ApiError> {
        let context = context.clone();
        info!(
            "get_uc_data(\"{}\", {:?}, {:?}, {:?}, {:?}) - X-Span-ID: {:?}",
            supi,
            supported_features,
            uc_purpose,
            if_none_match,
            if_modified_since,
            context.get().0.clone()
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
        context: &C,
    ) -> Result<GetV2xDataResponse, ApiError> {
        let context = context.clone();
        info!(
            "get_v2x_data(\"{}\", {:?}, {:?}, {:?}) - X-Span-ID: {:?}",
            supi,
            supported_features,
            if_none_match,
            if_modified_since,
            context.get().0.clone()
        );
        Err(ApiError("Generic failure".into()))
    }
}
