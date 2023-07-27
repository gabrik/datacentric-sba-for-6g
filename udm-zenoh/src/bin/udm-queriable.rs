use clap::Parser;
use prost::Message;
use std::{collections::HashMap, str::FromStr, sync::Arc};
use zenoh::prelude::r#async::*;
use zenoh_config::{EndPoint, ListenConfig};

pub mod nudm_sdm {
    tonic::include_proto!("fiveg_proto"); // The string specified here must match the proto package name
}

use crate::nudm_sdm::{
    Arp, DnnConfiguration, GetSmDataResult, Nssai, PduSessionTypes, QosProfile, SessionAmbr,
    SscModes,
};

#[derive(Parser)]
pub struct Opts {
    // public options
    #[clap(short = 'l', long, default_value = "tcp/127.0.0.1:7071")]
    pub listen: String,
    #[clap(short = 'p', long)]
    pub protobuf: bool,
}

#[async_std::main]
async fn main() {
    env_logger::init();

    let opts = Opts::parse();

    let mut config = zenoh::config::Config::default();
    config
        .set_mode(Some(zenoh::config::whatami::WhatAmI::Peer))
        .unwrap();
    config
        .set_listen(
            ListenConfig::new(vec![EndPoint {
                locator: Locator::from_str(&opts.listen).unwrap(),
                config: None,
            }])
            .unwrap(),
        )
        .unwrap();
    let session = Arc::new(zenoh::open(config).res().await.unwrap());

    let ke = format!("nudm-sdm/v2/*/sm-data");
    let queryable = session.declare_queryable(&ke).res().await.unwrap();

    async_std::task::sleep(std::time::Duration::from_secs(5)).await;

    loop {
        match queryable.recv_async().await {
            Ok(query) => {
                let value = if opts.protobuf {
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

                    reply.encode_to_vec()
                } else {
                    r#"{"singleNssai":{"sst":1},"dnnConfigurations":{"internet":{"pduSessionTypes":{"defaultSessionType":"IPV4","allowedSessionTypes":["IPV4"]},"sscModes":{"defaultSscMode":"SSC_MODE_1","allowedSscModes":["SSC_MODE_1","SSC_MODE_2","SSC_MODE_3"]},"5gQosProfile":{"5qi":9,"arp":{"priorityLevel":8,"preemptCap":"NOT_PREEMPT","preemptVuln":"NOT_PREEMPTABLE"},"priorityLevel":8},"sessionAmbr":{"uplink":"1048576 Kbps","downlink":"1048576 Kbps"}}}}"#.as_bytes().to_vec()
                };

                let ke = query.key_expr();
                // let value = serde_json::from_str::<SessionManagementSubscriptionData>(r#"{"singleNssai":{"sst":1},"dnnConfigurations":{"internet":{"pduSessionTypes":{"defaultSessionType":"IPV4","allowedSessionTypes":["IPV4"]},"sscModes":{"defaultSscMode":"SSC_MODE_1","allowedSscModes":["SSC_MODE_1","SSC_MODE_2","SSC_MODE_3"]},"5gQosProfile":{"5qi":9,"arp":{"priorityLevel":8,"preemptCap":"NOT_PREEMPT","preemptVuln":"NOT_PREEMPTABLE"},"priorityLevel":8},"sessionAmbr":{"uplink":"1048576 Kbps","downlink":"1048576 Kbps"}}}}"#).expect("unable to parse json");

                query
                    .reply(Ok(Sample::new(ke.clone(), value)))
                    .res()
                    .await
                    .unwrap();
            }
            Err(_) => (),
        }
    }
}
