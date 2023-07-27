use clap::Parser;
use std::{str::FromStr, sync::Arc};
use zenoh::prelude::r#async::*;
use zenoh_config::{EndPoint, ListenConfig};
use prost::Message;

pub mod nsfm_pdusession {
    tonic::include_proto!("fiveg_proto"); // The string specified here must match the proto package name
}

use crate::nsfm_pdusession::{CreateSmContextResult};

#[derive(Parser)]
pub struct Opts {
    // public options
    #[clap(short = 'l', long, default_value = "tcp/127.0.0.1:7072")]
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

    let ke = format!("nsmf-pdusession/v1/sm-contexts");
    let queryable = session.declare_queryable(&ke).res().await.unwrap();

    async_std::task::sleep(std::time::Duration::from_secs(5)).await;

    loop {
        match queryable.recv_async().await {
            Ok(query) => {
                let ke = query.key_expr();

                // look for udm
                let _nrf_res = session.get("nnrf-disc/v1/nf-instances?requester-features=20&requester-nf-type=SMF&service-names=nudm-sdm&target-nf-type=UDM").res().await.unwrap();

                // call udm
                let _udm_res = session.get("nudm-sdm/v2/imsi-001011234567895/sm-data?single-nssai=%7B%0A%09%22sst%22%3A%091%0A%7D&dnn=internet").res().await.unwrap();
                // return to AMF

                let value = if opts.protobuf {
                    let reply = CreateSmContextResult {
                        location: "nsmf-pdusession/v1/sm-contexts/4".into(),
                    };
                    reply.encode_to_vec()
                }else {"nsmf-pdusession/v1/sm-contexts/4".as_bytes().to_vec()}
                ;
                query
                    .reply(Ok(Sample::new(ke.clone(), value)))
                    .res()
                    .await
                    .unwrap();

                let data = r#"{"n1MessageContainer":{"n1MessageClass":"SM","n1MessageContent":{"contentId":"5gnas-sm"}},"n2InfoContainer":{"n2InformationClass":"SM","smInfo":{"pduSessionId":1,"n2InfoContent":{"ngapIeType":"PDU_RES_SETUP_REQ","ngapData":{"contentId":"ngap-sm"}}}},"pduSessionId":1}"#.as_bytes();
                let nas = [
                    0x2e, 0x01, 0x01, 0xc2, 0x11, 0x00, 0x09, 0x01, 0x00, 0x06, 0x31, 0x31, 0x01,
                    0x01, 0xff, 0x01, 0x06, 0x0b, 0x00, 0x01, 0x0b, 0x00, 0x01, 0x29, 0x05, 0x01,
                    0xc0, 0xa8, 0x64, 0x05, 0x22, 0x01, 0x01, 0x79, 0x00, 0x06, 0x01, 0x20, 0x41,
                    0x01, 0x01, 0x09, 0x7b, 0x00, 0x0f, 0x80, 0x00, 0x0d, 0x04, 0x08, 0x08, 0x08,
                    0x08, 0x00, 0x0d, 0x04, 0x08, 0x08, 0x04, 0x04, 0x25, 0x09, 0x08, 0x69, 0x6e,
                    0x74, 0x65, 0x72, 0x6e, 0x65, 0x74,
                ];
                let ngap = [
                    0x00, 0x00, 0x04, 0x00, 0x82, 0x00, 0x0a, 0x0c, 0x40, 0x00, 0x00, 0x00, 0x30,
                    0x40, 0x00, 0x00, 0x00, 0x00, 0x8b, 0x00, 0x0a, 0x01, 0xf0, 0xac, 0x16, 0x00,
                    0x08, 0x00, 0x00, 0x00, 0x0e, 0x00, 0x86, 0x00, 0x01, 0x00, 0x00, 0x88, 0x00,
                    0x07, 0x00, 0x01, 0x00, 0x00, 0x09, 0x1c, 0x00,
                ];

                let mut body: Vec<u8> = vec![];
                body.extend_from_slice(data);
                body.extend_from_slice(&nas);
                body.extend_from_slice(&ngap);

                // callback to AMF
                let _amf_res = session
                    .get("namf-comm/v1/ue-contexts/imsi-001011234567895/n1-n2-messages")
                    .with_value(body)
                    .res()
                    .await
                    .unwrap();

                // let value = serde_json::from_str::<SessionManagementSubscriptionData>(r#"{"singleNssai":{"sst":1},"dnnConfigurations":{"internet":{"pduSessionTypes":{"defaultSessionType":"IPV4","allowedSessionTypes":["IPV4"]},"sscModes":{"defaultSscMode":"SSC_MODE_1","allowedSscModes":["SSC_MODE_1","SSC_MODE_2","SSC_MODE_3"]},"5gQosProfile":{"5qi":9,"arp":{"priorityLevel":8,"preemptCap":"NOT_PREEMPT","preemptVuln":"NOT_PREEMPTABLE"},"priorityLevel":8},"sessionAmbr":{"uplink":"1048576 Kbps","downlink":"1048576 Kbps"}}}}"#).expect("unable to parse json");
            }
            Err(_) => (),
        }
    }
}
