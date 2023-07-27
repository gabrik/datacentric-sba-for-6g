use nrf_grpc::nnrf_disc::SearchRequest;
use tonic::transport::Channel;
use tonic::{Request, Response, Status};

use nsfm_pdusession::smf_server::Smf;
use nsfm_pdusession::{CreateSmContextRequest, CreateSmContextResult};

use nrf_grpc::nnrf_disc::nrf_discovery_client::NrfDiscoveryClient;
use udm_grpc::nudm_sdm::udm_client::UdmClient;
use udm_grpc::nudm_sdm::{GetSmDataRequest, Nssai};

use std::sync::Arc;
use tokio::sync::Mutex;

pub mod nsfm_pdusession {
    tonic::include_proto!("fiveg_proto"); // The string specified here must match the proto package name
}

#[derive(Debug)]
struct SmfState {
    nrf_client: NrfDiscoveryClient<Channel>,
    udm_client: UdmClient<Channel>,
}

#[derive(Debug, Default)]
pub struct MySmf {
    state: Option<Arc<Mutex<SmfState>>>,
    amf: String,
}

impl MySmf {
    pub async fn new(nrf: String, udm: String, amf: String) -> Self {
        let nrf_client = NrfDiscoveryClient::connect(nrf).await.unwrap();
        let udm_client = UdmClient::connect(udm).await.unwrap();

        let state = SmfState {
            nrf_client,
            udm_client,
        };

        Self {
            state: Some(Arc::new(Mutex::new(state))),
            amf,
        }
    }
}

#[tonic::async_trait]
impl Smf for MySmf {
    async fn sm_context(
        &self,
        request: Request<CreateSmContextRequest>,
    ) -> Result<Response<CreateSmContextResult>, Status> {
        let _req: CreateSmContextRequest = request.into_inner();

        match &self.state {
            Some(state) => {
                let mut guard_state = state.lock().await;
                // First let's look for UDM

                let nrf_req = SearchRequest {
                    requester_nf_type: "SMF".into(),
                    service_names: "nudm-sdm".into(),
                    target_nf_type: "UDM".into(),
                    requester_features: "20".into(),
                };

                let _nrf_reply = guard_state.nrf_client.search(nrf_req).await.unwrap();

                let udm_req = GetSmDataRequest {
                    dnn: "internet".into(),
                    single_nssai: Some(Nssai {
                        sst: 1,
                        sd: "".into(),
                    }),
                };

                let _udm_reply = guard_state.udm_client.get_sm_data(udm_req).await.unwrap();

                let reply = CreateSmContextResult {
                    location: "nsmf-pdusession/v1/sm-contexts/4".into(),
                };

                let c_amf_url = self.amf.clone();
                tokio::task::spawn(async move {
                    let data = r#"{"n1MessageContainer":{"n1MessageClass":"SM","n1MessageContent":{"contentId":"5gnas-sm"}},"n2InfoContainer":{"n2InformationClass":"SM","smInfo":{"pduSessionId":1,"n2InfoContent":{"ngapIeType":"PDU_RES_SETUP_REQ","ngapData":{"contentId":"ngap-sm"}}}},"pduSessionId":1}"#.as_bytes();
                    let nas = [
                        0x2e, 0x01, 0x01, 0xc2, 0x11, 0x00, 0x09, 0x01, 0x00, 0x06, 0x31, 0x31,
                        0x01, 0x01, 0xff, 0x01, 0x06, 0x0b, 0x00, 0x01, 0x0b, 0x00, 0x01, 0x29,
                        0x05, 0x01, 0xc0, 0xa8, 0x64, 0x05, 0x22, 0x01, 0x01, 0x79, 0x00, 0x06,
                        0x01, 0x20, 0x41, 0x01, 0x01, 0x09, 0x7b, 0x00, 0x0f, 0x80, 0x00, 0x0d,
                        0x04, 0x08, 0x08, 0x08, 0x08, 0x00, 0x0d, 0x04, 0x08, 0x08, 0x04, 0x04,
                        0x25, 0x09, 0x08, 0x69, 0x6e, 0x74, 0x65, 0x72, 0x6e, 0x65, 0x74,
                    ];
                    let ngap = [
                        0x00, 0x00, 0x04, 0x00, 0x82, 0x00, 0x0a, 0x0c, 0x40, 0x00, 0x00, 0x00,
                        0x30, 0x40, 0x00, 0x00, 0x00, 0x00, 0x8b, 0x00, 0x0a, 0x01, 0xf0, 0xac,
                        0x16, 0x00, 0x08, 0x00, 0x00, 0x00, 0x0e, 0x00, 0x86, 0x00, 0x01, 0x00,
                        0x00, 0x88, 0x00, 0x07, 0x00, 0x01, 0x00, 0x00, 0x09, 0x1c, 0x00,
                    ];

                    let mut body: Vec<u8> = vec![];
                    body.extend_from_slice(data);
                    body.extend_from_slice(&nas);
                    body.extend_from_slice(&ngap);

                    let client = reqwest::Client::new();
                    let _res = client
                        .post(format!(
                            "{}/namf-comm/v1/ue-contexts/imsi-001011234567895/n1-n2-messages",
                            c_amf_url.as_str()
                        ))
                        .body(body)
                        .send()
                        .await;
                });

                Ok(Response::new(reply))
            }
            None => panic!("No state WTF?"),
        }
    }
}
