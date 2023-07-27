use crate::{ApiError, SmfApi};

use nnrf_discovery_server::models::{NfType, ServiceName};
use nsfm_pdusession::{
    models::{self, ExtProblemDetails, SmContextCreateError, SmContextCreatedData},
    PostPduSessionsResponse, PostSmContextsResponse, ReleasePduSessionResponse,
    ReleaseSmContextResponse, RetrievePduSessionResponse, RetrieveSmContextResponse,
    SendMoDataResponse, TransferMoDataResponse, UpdatePduSessionResponse, UpdateSmContextResponse,
};
use nudm_sdm::models::Snssai;
use std::sync::Arc;
use zrpc_macros::zserver;

use nnrf_zenoh::NRFApiClient;
use udm_zenoh::UDMApiClient;

use zenoh::prelude::r#async::*;
use zenoh::Session;

#[derive(Clone)]
pub struct Server {
    session: Arc<Session>,
    nrf_client: Arc<NRFApiClient>,
    udm_client: Arc<UDMApiClient>,
}

impl Server {
    pub async fn new(session: Arc<Session>) -> Self {
        let nrfs = NRFApiClient::find_servers(session.clone()).await.unwrap();
        let udms = UDMApiClient::find_servers(session.clone()).await.unwrap();

        let nrf_client = Arc::new(NRFApiClient::new(session.clone(), nrfs[0]));
        let udm_client = Arc::new(UDMApiClient::new(session.clone(), udms[0]));

        Self {
            session,
            nrf_client,
            udm_client,
        }
    }
}

#[zserver]
impl SmfApi for Server {
    /// Release
    async fn release_pdu_session(
        &self,
        pdu_session_ref: String,
        release_data: Option<models::ReleaseData>,
    ) -> Result<ReleasePduSessionResponse, ApiError> {
        log::info!(
            "release_pdu_session(\"{}\", {:?}) ",
            pdu_session_ref,
            release_data,
        );
        Err(ApiError("Generic failure".into()))
    }

    /// Retrieve
    async fn retrieve_pdu_session(
        &self,
        pdu_session_ref: String,
        retrieve_data: models::RetrieveData,
    ) -> Result<RetrievePduSessionResponse, ApiError> {
        log::info!(
            "retrieve_pdu_session(\"{}\", {:?}) ",
            pdu_session_ref,
            retrieve_data,
        );
        Err(ApiError("Generic failure".into()))
    }

    /// Transfer MO Data
    async fn transfer_mo_data(
        &self,
        pdu_session_ref: String,
        json_data: Option<models::TransferMoDataReqData>,
        binary_mo_data: Option<Vec<u8>>,
    ) -> Result<TransferMoDataResponse, ApiError> {
        log::info!(
            "transfer_mo_data(\"{}\", {:?}, {:?}) ",
            pdu_session_ref,
            json_data,
            binary_mo_data,
        );
        Err(ApiError("Generic failure".into()))
    }

    /// Update (initiated by V-SMF or I-SMF)
    async fn update_pdu_session(
        &self,
        pdu_session_ref: String,
        hsmf_update_data: models::HsmfUpdateData,
    ) -> Result<UpdatePduSessionResponse, ApiError> {
        log::info!(
            "update_pdu_session(\"{}\", {:?}) ",
            pdu_session_ref,
            hsmf_update_data,
        );
        Err(ApiError("Generic failure".into()))
    }

    /// Release SM Context
    async fn release_sm_context(
        &self,
        sm_context_ref: String,
        sm_context_release_data: Option<models::SmContextReleaseData>,
    ) -> Result<ReleaseSmContextResponse, ApiError> {
        log::info!(
            "release_sm_context(\"{}\", {:?}) ",
            sm_context_ref,
            sm_context_release_data,
        );
        Err(ApiError("Generic failure".into()))
    }

    /// Retrieve SM Context
    async fn retrieve_sm_context(
        &self,
        sm_context_ref: String,
        sm_context_retrieve_data: Option<models::SmContextRetrieveData>,
    ) -> Result<RetrieveSmContextResponse, ApiError> {
        log::info!(
            "retrieve_sm_context(\"{}\", {:?}) ",
            sm_context_ref,
            sm_context_retrieve_data,
        );
        Err(ApiError("Generic failure".into()))
    }

    /// Send MO Data
    async fn send_mo_data(
        &self,
        sm_context_ref: String,
        json_data: Option<models::SendMoDataReqData>,
        binary_mo_data: Option<Vec<u8>>,
    ) -> Result<SendMoDataResponse, ApiError> {
        log::info!(
            "send_mo_data(\"{}\", {:?}, {:?}) ",
            sm_context_ref,
            json_data,
            binary_mo_data,
        );
        Err(ApiError("Generic failure".into()))
    }

    /// Update SM Context
    async fn update_sm_context(
        &self,
        sm_context_ref: String,
        sm_context_update_data: models::SmContextUpdateData,
    ) -> Result<UpdateSmContextResponse, ApiError> {
        log::info!(
            "update_sm_context(\"{}\", {:?}) ",
            sm_context_ref,
            sm_context_update_data,
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
    ) -> Result<PostPduSessionsResponse, ApiError> {
        log::info!("post_pdu_sessions({:?}) ", pdu_session_create_data,);
        Err(ApiError("Generic failure".into()))
    }

    /// Create SM Context
    async fn post_sm_contexts(
        &self,
        json_data: Option<models::SmContextCreateData>,
        binary_data_n1_sm_message: Option<Vec<u8>>,
        binary_data_n2_sm_information: Option<Vec<u8>>,
        binary_data_n2_sm_information_ext1: Option<Vec<u8>>,
    ) -> Result<PostSmContextsResponse, ApiError> {
        log::info!(
            "post_sm_contexts({:?}, {:?}, {:?}, {:?}) ",
            json_data,
            binary_data_n1_sm_message,
            binary_data_n2_sm_information,
            binary_data_n2_sm_information_ext1,
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

        //First call the NRF for UDM discovery
        // Open5Gs request is done to uri: /nnrf-disc/v1/nf-instances?requester-features=20&requester-nf-type=SMF&service-names=nudm-sdm&target-nf-type=UDM

        let result = self
            .nrf_client
            .search_nf_instances(
                NfType::UDM,
                NfType::SMF,
                Some("applicaiton/json,application/problem+json".to_string()),
                None,
                None,
                Some(vec![ServiceName::new("nudm-sdm".to_string())]),
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
            )
            .await
            .map_err(|e| ApiError(format!("ZRPC {e:?}")))?;

        match result {
            Ok(resp) => {
                match resp {
                nnrf_discovery_server::SearchNfInstancesResponse::ExpectedResponseToAValidRequest { body, cache_control, e_tag, content_encoding } => {
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


                                // We get the context from the UDM
                                // Open5gs call is to /nudm-sdm/v2/imsi-001011234567895/sm-data?single-nssai=%7B%0A%09%22sst%22%3A%091%0A%7D&dnn=internet

                                let resp = self.udm_client.get_sm_data(
                                    json_data.expect("no data!").supi.expect("No supi!"),
                                    None,
                                    Some(Snssai{ sst: 1, sd: None }),
                                    Some("internet".to_string()),
                                    None,
                                    None,
                                    None,

                                ).await.map_err(|e| ApiError(format!("ZRPC {e:?}")))?;

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
                                    Ok(nudm_sdm::GetSmDataResponse::ExpectedResponseToAValidRequest { body, cache_control, e_tag, last_modified }) => {
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

                                        let amf_ke = "namf-comm/v1/ue-contexts/imsi-001011234567895/n1-n2-messages";
                                        let c_session  = self.session.clone();
                                        // Async callback to the AMF
                                        async_std::task::spawn(async move {
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


                                            c_session.get(amf_ke).with_value(body).res().await.unwrap();
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
                log::error!("Got error from NFR {nfr_error:?}");
                return Ok(make_error(format!("NRF: {nfr_error:?}")));
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
