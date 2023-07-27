use tonic::{Request, Response, Status};

use nudm_sdm::udm_server::Udm;
use nudm_sdm::{
    Arp, DnnConfiguration, GetSmDataRequest, GetSmDataResult, Nssai, PduSessionTypes, QosProfile,
    SessionAmbr, SscModes,
};

use std::collections::HashMap;

pub mod nudm_sdm {
    tonic::include_proto!("fiveg_proto"); // The string specified here must match the proto package name
}

#[derive(Debug, Default)]
pub struct MyUDM {}

#[tonic::async_trait]
impl Udm for MyUDM {
    async fn get_sm_data(
        &self,
        request: Request<GetSmDataRequest>,
    ) -> Result<Response<GetSmDataResult>, Status> {
        let _req: GetSmDataRequest = request.into_inner();

        let reply = GetSmDataResult {
            single_nssai: Some(Nssai {
                sst: 1,
                sd: "".into(),
            }),
            dnn_configurations: HashMap::from([(
                "internet".into(),
                DnnConfiguration {
                    pdu_session_types: Some(PduSessionTypes {
                        default_session_type: "IPV4".into(),
                        allowed_session_types: vec!["IPV4".into()],
                    }),
                    ssc_modes: Some(SscModes {
                        default_ssc_mode: "SSC_MODE_1".into(),
                        allowed_ssc_modes: vec![
                            "SSC_MODE_1".into(),
                            "SSC_MODE_2".into(),
                            "SSC_MODE_3".into(),
                        ],
                    }),
                    qos_profile: Some(QosProfile {
                        qi: 9,
                        arp: Some(Arp {
                            priority_level: 8,
                            preempt_cap: "NOT_PREEMPT".into(),
                            preempt_vuln: "NOT_PREEMPTABLE".into(),
                        }),
                        priority_level: 8,
                    }),
                    session_ambr: Some(SessionAmbr {
                        uplink: "1048576 Kbps".into(),
                        downlink: "1048576 Kbps".into(),
                    }),
                },
            )]),
        };

        Ok(Response::new(reply))
    }
}
