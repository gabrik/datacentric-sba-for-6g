//! Main library entry point for nsfm_pdusession implementation.

#![allow(unused_imports)]

use async_trait::async_trait;
use futures::{future, Stream, StreamExt, TryFutureExt, TryStreamExt};
use hyper::client::HttpConnector;
use hyper::server::conn::Http;
use hyper::service::Service;
use log::info;
use nudm_sdm::models::Snssai;
use std::future::Future;
use std::marker::PhantomData;
use std::net::SocketAddr;
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll};
use swagger::auth::MakeAllowAllAuthenticator;
use swagger::{DropContextService, EmptyContext};
use swagger::{Has, XSpanIdString};
use tokio::net::TcpListener;
use uuid::Uuid;

//NRF communication
use nnrf_discovery_server::models::{NfType, ServiceName};
use nnrf_discovery_server::ContextWrapperExt as NFRContextWrapperExt;
use nnrf_discovery_server::{Api as NRFApi, ApiNoContext as NRFApiNoContext, Client as NRFClient};
use swagger::{AuthData, ContextBuilder, Push};

type ClientContext = swagger::make_context_ty!(
    ContextBuilder,
    EmptyContext,
    Option<AuthData>,
    XSpanIdString
);

// UDM communication
use nudm_sdm::ContextWrapperExt as UDMContextWrapperExt;
use nudm_sdm::{Api as UDMApi, ApiNoContext as UDMApiNoContext, Client as UDMClient};

//

#[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "ios")))]
use openssl::ssl::{Ssl, SslAcceptor, SslAcceptorBuilder, SslFiletype, SslMethod};

use nsfm_pdusession::models::{
    self, ExtProblemDetails, SmContextCreateError, SmContextCreatedData,
};

/// Builds an SSL implementation for Simple HTTPS from some hard-coded file names
pub async fn create(
    addr: &str,
    https: bool,
    nrf_url: url::Url,
    udm_url: url::Url,
    amf_url: url::Url,
) {
    let addr = addr.parse().expect("Failed to parse bind address");

    let server = Server::new(nrf_url, udm_url, amf_url);

    let service = MakeService::new(server);

    let service = MakeAllowAllAuthenticator::new(service, "cosmo");

    #[allow(unused_mut)]
    let mut service =
        nsfm_pdusession::server::context::MakeAddContext::<_, EmptyContext>::new(service);

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

#[derive(Clone)]
pub struct Server<C> {
    marker: PhantomData<C>,
    // nrf_client: Arc<NRFClient<DropContextService<hyper::Client<HttpConnector>, C>, C>>,
    client_ctx: ClientContext,
    nfr_url: url::Url,
    udm_url: url::Url,
    amf_url: url::Url,
}

unsafe impl<C> Send for Server<C> {}
unsafe impl<C> Sync for Server<C> {}

// impl<C> Copy for Server<C> { }

impl<C> Server<C> {
    pub fn new(nfr_url: url::Url, udm_url: url::Url, amf_url: url::Url) -> Self {
        let client_ctx: ClientContext = swagger::make_context!(
            ContextBuilder,
            EmptyContext,
            None as Option<AuthData>,
            XSpanIdString::default()
        );

        // let nrf_client: Arc<dyn NRFApiNoContext<NRFClientContext>> = {
        //     if nfr_url.scheme() == "https" {
        //         // Using Simple HTTPS
        //         let client = Arc::new(
        //             NRFClient::try_new_https(&nfr_url.as_str())
        //                 .expect("Failed to create HTTPS client"),
        //         );
        //         Arc::new(NFRContextWrapperExt::with_context(client, nrf_context))
        //     } else {
        //         // Using HTTP
        //         let client = Arc::new(
        //             NRFClient::try_new_http(&nfr_url.as_str())
        //                 .expect("Failed to create HTTP client"),
        //         );
        //         Arc::new(NFRContextWrapperExt::with_context(client, nrf_context))
        //     }
        // };

        // let nrf_client = NRFClient::try_new_http(&nfr_url.as_str()).expect("Failed to create HTTP client");

        Server {
            marker: PhantomData,
            // nrf_client,
            client_ctx,
            nfr_url,
            udm_url,
            amf_url,
        }
    }
}

use nsfm_pdusession::server::MakeService;
use nsfm_pdusession::{
    Api, PostPduSessionsResponse, PostSmContextsResponse, ReleasePduSessionResponse,
    ReleaseSmContextResponse, RetrievePduSessionResponse, RetrieveSmContextResponse,
    SendMoDataResponse, TransferMoDataResponse, UpdatePduSessionResponse, UpdateSmContextResponse,
};
use std::error::Error;
use swagger::ApiError;

#[async_trait]
impl<C> Api<C> for Server<C>
where
    C: Has<XSpanIdString> + Send + Sync,
{
    /// Release
    async fn release_pdu_session(
        &self,
        pdu_session_ref: String,
        release_data: Option<models::ReleaseData>,
        context: &C,
    ) -> Result<ReleasePduSessionResponse, ApiError> {
        let context = context.clone();
        info!(
            "release_pdu_session(\"{}\", {:?}) - X-Span-ID: {:?}",
            pdu_session_ref,
            release_data,
            context.get().0.clone()
        );
        Err(ApiError("Generic failure".into()))
    }

    /// Retrieve
    async fn retrieve_pdu_session(
        &self,
        pdu_session_ref: String,
        retrieve_data: models::RetrieveData,
        context: &C,
    ) -> Result<RetrievePduSessionResponse, ApiError> {
        let context = context.clone();
        info!(
            "retrieve_pdu_session(\"{}\", {:?}) - X-Span-ID: {:?}",
            pdu_session_ref,
            retrieve_data,
            context.get().0.clone()
        );
        Err(ApiError("Generic failure".into()))
    }

    /// Transfer MO Data
    async fn transfer_mo_data(
        &self,
        pdu_session_ref: String,
        json_data: Option<models::TransferMoDataReqData>,
        binary_mo_data: Option<swagger::ByteArray>,
        context: &C,
    ) -> Result<TransferMoDataResponse, ApiError> {
        let context = context.clone();
        info!(
            "transfer_mo_data(\"{}\", {:?}, {:?}) - X-Span-ID: {:?}",
            pdu_session_ref,
            json_data,
            binary_mo_data,
            context.get().0.clone()
        );
        Err(ApiError("Generic failure".into()))
    }

    /// Update (initiated by V-SMF or I-SMF)
    async fn update_pdu_session(
        &self,
        pdu_session_ref: String,
        hsmf_update_data: models::HsmfUpdateData,
        context: &C,
    ) -> Result<UpdatePduSessionResponse, ApiError> {
        let context = context.clone();
        info!(
            "update_pdu_session(\"{}\", {:?}) - X-Span-ID: {:?}",
            pdu_session_ref,
            hsmf_update_data,
            context.get().0.clone()
        );
        Err(ApiError("Generic failure".into()))
    }

    /// Release SM Context
    async fn release_sm_context(
        &self,
        sm_context_ref: String,
        sm_context_release_data: Option<models::SmContextReleaseData>,
        context: &C,
    ) -> Result<ReleaseSmContextResponse, ApiError> {
        let context = context.clone();
        info!(
            "release_sm_context(\"{}\", {:?}) - X-Span-ID: {:?}",
            sm_context_ref,
            sm_context_release_data,
            context.get().0.clone()
        );
        Err(ApiError("Generic failure".into()))
    }

    /// Retrieve SM Context
    async fn retrieve_sm_context(
        &self,
        sm_context_ref: String,
        sm_context_retrieve_data: Option<models::SmContextRetrieveData>,
        context: &C,
    ) -> Result<RetrieveSmContextResponse, ApiError> {
        let context = context.clone();
        info!(
            "retrieve_sm_context(\"{}\", {:?}) - X-Span-ID: {:?}",
            sm_context_ref,
            sm_context_retrieve_data,
            context.get().0.clone()
        );
        Err(ApiError("Generic failure".into()))
    }

    /// Send MO Data
    async fn send_mo_data(
        &self,
        sm_context_ref: String,
        json_data: Option<models::SendMoDataReqData>,
        binary_mo_data: Option<swagger::ByteArray>,
        context: &C,
    ) -> Result<SendMoDataResponse, ApiError> {
        let context = context.clone();
        info!(
            "send_mo_data(\"{}\", {:?}, {:?}) - X-Span-ID: {:?}",
            sm_context_ref,
            json_data,
            binary_mo_data,
            context.get().0.clone()
        );
        Err(ApiError("Generic failure".into()))
    }

    /// Update SM Context
    async fn update_sm_context(
        &self,
        sm_context_ref: String,
        sm_context_update_data: models::SmContextUpdateData,
        context: &C,
    ) -> Result<UpdateSmContextResponse, ApiError> {
        let context = context.clone();
        info!(
            "update_sm_context(\"{}\", {:?}) - X-Span-ID: {:?}",
            sm_context_ref,
            sm_context_update_data,
            context.get().0.clone()
        );
        Err(ApiError("Generic failure".into()))
        // Here we receive

        // Request
        // {
        //     "n2SmInfo":	{
        //         "contentId":	"ngap-sm"
        //     },
        //     "n2SmInfoType":	"PDU_RES_SETUP_RSP"
        // }
        // NGAP
        // 0000   00 03 e0 ac 16 00 17 00 00 00 04 00 01

        // Reply is empty: 204
        // So: UpdateSmContextResponse::SuccessfulUpdateOfAnSMContextWithoutContentInTheResponse
    }

    /// Create
    async fn post_pdu_sessions(
        &self,
        pdu_session_create_data: models::PduSessionCreateData,
        context: &C,
    ) -> Result<PostPduSessionsResponse, ApiError> {
        let context = context.clone();
        info!(
            "post_pdu_sessions({:?}) - X-Span-ID: {:?}",
            pdu_session_create_data,
            context.get().0.clone()
        );
        Err(ApiError("Generic failure".into()))
    }

    /// Create SM Context
    async fn post_sm_contexts(
        &self,
        json_data: Option<models::SmContextCreateData>,
        binary_data_n1_sm_message: Option<swagger::ByteArray>,
        binary_data_n2_sm_information: Option<swagger::ByteArray>,
        binary_data_n2_sm_information_ext1: Option<swagger::ByteArray>,
        context: &C,
    ) -> Result<PostSmContextsResponse, ApiError> {
        let context = context.clone();
        info!(
            "post_sm_contexts({:?}, {:?}, {:?}, {:?}) - X-Span-ID: {:?}",
            json_data,
            binary_data_n1_sm_message,
            binary_data_n2_sm_information,
            binary_data_n2_sm_information_ext1,
            context.get().0.clone()
        );

        // Here we receive
        // Request
        // {
        //     "supi":	"imsi-001011234567895",
        //     "pei":	"imeisv-4370816125816151",
        //     "pduSessionId":	1,
        //     "dnn":	"internet",
        //     "sNssai":	{
        //         "sst":	1
        //     },
        //     "servingNfId":	"66bf4df8-b832-41ed-aa12-4df3ea315a7c",
        //     "guami":	{
        //         "plmnId":	{
        //             "mcc":	"001",
        //             "mnc":	"01"
        //         },
        //         "amfId":	"020040"
        //     },
        //     "servingNetwork":	{
        //         "mcc":	"001",
        //         "mnc":	"01"
        //     },
        //     "n1SmMsg":	{
        //         "contentId":	"5gnas-sm"
        //     },
        //     "anType":	"3GPP_ACCESS",
        //     "ratType":	"NR",
        //     "ueLocation":	{
        //         "nrLocation":	{
        //             "tai":	{
        //                 "plmnId":	{
        //                     "mcc":	"001",
        //                     "mnc":	"01"
        //                 },
        //                 "tac":	"000001"
        //             },
        //             "ncgi":	{
        //                 "plmnId":	{
        //                     "mcc":	"001",
        //                     "mnc":	"01"
        //                 },
        //                 "nrCellId":	"000000010"
        //             },
        //             "ueLocationTimestamp":	"2023-03-01T13:42:11.144288Z"
        //         }
        //     },
        //     "ueTimeZone":	"+00:00",
        //     "smContextStatusUri":	"http://172.22.0.10:7777/namf-callback/v1/imsi-001011234567895/sm-context-status/1",
        //     "pcfId":	"6c05c1d4-b832-41ed-9698-8dec5d3774de"
        // }
        // 5GNAS - binary N1 data
        //
        // 0000   2e 01 01 c1 ff ff 91 a1 28 01 00 7b 00 07 80 00
        // 0010   0a 00 00 0d 00

        let nrf_client =
            NRFClient::try_new_http(&self.nfr_url.as_str()).expect("Failed to create HTTP client");

        //First call the NRF for UDM discovery
        // Open5Gs request is done to uri: /nnrf-disc/v1/nf-instances?requester-features=20&requester-nf-type=SMF&service-names=nudm-sdm&target-nf-type=UDM

        let result = nrf_client
            .search_nf_instances(
                NfType::UDM,
                NfType::SMF,
                Some("applicaiton/json,application/problem+json".to_string()),
                None,
                None,
                Some(&vec![ServiceName::new("nudm-sdm".to_string())]),
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                Some("20".to_string()),
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                &self.client_ctx,
            )
            .await;

        match result {
            Ok(resp) => {
                match resp {
                    nnrf_discovery_server::SearchNfInstancesResponse::ExpectedResponseToAValidRequest { body, cache_control: _, e_tag: _, content_encoding:_ } => {
                        // Check if the NRF found an UDM

                        // Reply should be
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



                        for nf_profile in &body.nf_instances {
                            if nf_profile.nf_type == NfType::UDM {
                                match &nf_profile.ipv4_addresses {
                                    Some(_adresses) => {
                                    // Then we can creaet the client and retrieve the SM context

                                    // let udm_context: UDMClientContext =
                                    // swagger::make_context!(ContextBuilder, EmptyContext, None as Option<AuthData>, XSpanIdString::default());

                                // let mut udm_client : Box<dyn UDMApiNoContext<UDMClientContext>> = if self.udm_url.scheme() == "https" {
                                //     // Using Simple HTTPS
                                //     let client = Box::new(UDMClient::try_new_https(&self.udm_url.as_str())
                                //         .expect("Failed to create HTTPS client"));
                                //     // Box::new(UDMContextWrapperExt::with_context(client,udm_context))
                                // } else {
                                //     // Using HTTP
                                //     let client = Box::new(UDMClient::try_new_http(
                                //         &self.udm_url.as_str())
                                //         .expect("Failed to create HTTP client"));
                                //     // Box::new(UDMContextWrapperExt::with_context(client,udm_context))
                                // };

                                // let mut udm_client = Box::new(UDMClient::try_new_http(
                                //             &self.udm_url.as_str())
                                //             .expect("Failed to create HTTP client"));

                                let udm_client = UDMClient::try_new_http(&self.udm_url.as_str()).expect("Failed to create HTTP client");
                                    // We get the context from the UDM
                                    // Open5gs call is to /nudm-sdm/v2/imsi-001011234567895/sm-data?single-nssai=%7B%0A%09%22sst%22%3A%091%0A%7D&dnn=internet

                                    let resp = UDMApi::get_sm_data(
                                        &udm_client,
                                        json_data.expect("no data!").supi.expect("No supi!"),
                                        None,
                                        Some(Snssai{ sst: 1, sd: None }),
                                        Some("internet".to_string()),
                                        None,
                                        None,
                                        None,
                                        &self.client_ctx,
                                    ).await?;

                                    // Here we should receive
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

                                    match resp {
                                        nudm_sdm::GetSmDataResponse::ExpectedResponseToAValidRequest { body:_, cache_control:_, e_tag:_, last_modified:_ } => {
                                            let response = PostSmContextsResponse::SuccessfulCreationOfAnSMContext{
                                                body: SmContextCreatedData{
                                                    h_smf_uri: None,
                                                    smf_uri:  None,
                                                    pdu_session_id:  None,
                                                    s_nssai: None,
                                                    up_cnx_state:  None,
                                                    n2_sm_info: None,
                                                    n2_sm_info_type: None,
                                                    allocated_ebi_list:  None,
                                                    ho_state:  None,
                                                    gpsi:  None,
                                                    smf_service_instance_id:  None,
                                                    recovery_time:  None,
                                                    supported_features:  None,
                                                    selected_smf_id:  None,
                                                    selected_old_smf_id:  None,
                                                    inter_plmn_api_root: None,
                                                },
                                                location: "nsmf-pdusession/v1/sm-contexts/4".to_string(),
                                            };

                                            let c_amf_url = self.amf_url.clone();
                                            // Async callback to the AMF
                                            tokio::task::spawn(async move {
                                                let data = r#"{"n1MessageContainer":{"n1MessageClass":"SM","n1MessageContent":{"contentId":"5gnas-sm"}},"n2InfoContainer":{"n2InformationClass":"SM","smInfo":{"pduSessionId":1,"n2InfoContent":{"ngapIeType":"PDU_RES_SETUP_REQ","ngapData":{"contentId":"ngap-sm"}}}},"pduSessionId":1}"#.as_bytes();
                                                let nas = [
                                                    0x2e, 0x01, 0x01, 0xc2, 0x11, 0x00, 0x09, 0x01, 0x00, 0x06, 0x31, 0x31, 0x01, 0x01, 0xff, 0x01,
                                                    0x06, 0x0b, 0x00, 0x01, 0x0b, 0x00, 0x01, 0x29, 0x05, 0x01, 0xc0, 0xa8, 0x64, 0x05, 0x22, 0x01,
                                                    0x01, 0x79, 0x00, 0x06, 0x01, 0x20, 0x41, 0x01, 0x01, 0x09, 0x7b, 0x00, 0x0f, 0x80, 0x00, 0x0d,
                                                    0x04, 0x08, 0x08, 0x08, 0x08, 0x00, 0x0d, 0x04, 0x08, 0x08, 0x04, 0x04, 0x25, 0x09, 0x08, 0x69,
                                                    0x6e, 0x74, 0x65, 0x72, 0x6e, 0x65, 0x74,
                                                    ];
                                                let ngap = [
                                                    0x00, 0x00, 0x04, 0x00, 0x82, 0x00, 0x0a, 0x0c, 0x40, 0x00, 0x00, 0x00, 0x30, 0x40, 0x00, 0x00,
                                                    0x00, 0x00, 0x8b, 0x00, 0x0a, 0x01, 0xf0, 0xac, 0x16, 0x00, 0x08, 0x00, 0x00, 0x00, 0x0e, 0x00,
                                                    0x86, 0x00, 0x01, 0x00, 0x00, 0x88, 0x00, 0x07, 0x00, 0x01, 0x00, 0x00, 0x09, 0x1c, 0x00
                                                ];

                                                let mut body : Vec<u8> = vec![];
                                                body.extend_from_slice(data);
                                                body.extend_from_slice(&nas);
                                                body.extend_from_slice(&ngap);


                                                let client = reqwest::Client::new();
                                                let _res = client.post(format!("{}/namf-comm/v1/ue-contexts/imsi-001011234567895/n1-n2-messages", c_amf_url.as_str()))
                                                        .body(body)
                                                        .send()
                                                        .await;
                                            });




                                            // Async sent to AFM via an API call to POST /namf-comm/v1/ue-contexts/imsi-001011234567895/n1-n2-messages
                                            // Creation of SM context
                                            // {
                                            //     "n1MessageContainer":	{
                                            //         "n1MessageClass":	"SM",
                                            //         "n1MessageContent":	{
                                            //             "contentId":	"5gnas-sm"
                                            //         }
                                            //     },
                                            //     "n2InfoContainer":	{
                                            //         "n2InformationClass":	"SM",
                                            //         "smInfo":	{
                                            //             "pduSessionId":	1,
                                            //             "n2InfoContent":	{
                                            //                 "ngapIeType":	"PDU_RES_SETUP_REQ",
                                            //                 "ngapData":	{
                                            //                     "contentId":	"ngap-sm"
                                            //                 }
                                            //             }
                                            //         }
                                            //     },
                                            //     "pduSessionId":	1
                                            // }

                                            // as well as 5GNAS payload and NGAP payloads
                                            // NAS
                                            // 0000   2e 01 01 c2 11 00 09 01 00 06 31 31 01 01 ff 01
                                            // 0010   06 0b 00 01 0b 00 01 29 05 01 c0 a8 64 05 22 01
                                            // 0020   01 79 00 06 01 20 41 01 01 09 7b 00 0f 80 00 0d
                                            // 0030   04 08 08 08 08 00 0d 04 08 08 04 04 25 09 08 69
                                            // 0040   6e 74 65 72 6e 65 74
                                            //
                                            // NGAP
                                            // 0000   00 00 04 00 82 00 0a 0c 40 00 00 00 30 40 00 00
                                            // 0010   00 00 8b 00 0a 01 f0 ac 16 00 08 00 00 00 0e 00
                                            // 0020   86 00 01 00 00 88 00 07 00 01 00 00 09 1c 00

                                            return Ok(response)
                                        },
                                        _ => return Ok(make_error(format!("UDM: invalid response: {resp:?}")))
                                    }

                                    // And then we should send to the AMF
                                    // just the locaiton
                                    // open5g sends http://172.22.0.7:7777/nsmf-pdusession/v1/sm-contexts/4



                                    }
                                    None => return Ok(make_error(format!("NRF: no addresses!"))),
                                }

                            }
                        }
                        ()
                    },
                    _ => return Ok(make_error(format!("NRF: invalid response: {resp:?}")))
                    }
            }
            Err(nfr_error) => {
                log::error!("Got error from NFR {nfr_error}");
                return Ok(make_error(format!("NRF: {nfr_error}")));
            }
        }

        Err(ApiError("Generic failure".into()))
    }
}

fn make_error(title: String) -> PostSmContextsResponse {
    PostSmContextsResponse::UnsuccessfulCreationOfAnSMContext(SmContextCreateError {
        error: ExtProblemDetails {
            r#type: None,
            title: Some(title),
            status: Some(500),
            detail: None,
            instance: Some("uuid".to_string()),
            cause: None,
            invalid_params: None,
            supported_features: None,
            access_token_error: None,
            access_token_request: None,
            nrf_id: None,
            remote_error: None,
        },
        n1_sm_msg: None,       //binary_data_n1_sm_message.into(),
        n2_sm_info: None,      //binary_data_n2_sm_information.into(),
        n2_sm_info_type: None, //binary_data_n2_sm_information_ext1.into(),
        recovery_time: None,
    })
}
