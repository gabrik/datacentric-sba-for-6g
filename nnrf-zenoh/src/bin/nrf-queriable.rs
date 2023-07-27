use clap::Parser;
use std::{collections::HashMap, str::FromStr, sync::Arc};
use zenoh::prelude::r#async::*;
use zenoh_config::{EndPoint, ListenConfig};

pub mod nnrf_disc {
    tonic::include_proto!("fiveg_proto"); // The string specified here must match the proto package name
}

use crate::nnrf_disc::{IpEndpoints, NfProfile, NfService, NfVersion, SearchResult};
use prost::Message;

#[derive(Parser)]
pub struct Opts {
    // public options
    #[clap(short = 'l', long, default_value = "tcp/127.0.0.1:7070")]
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

    let ke = format!("nnrf-disc/v1/nf-instances");
    let queryable = session.declare_queryable(&ke).res().await.unwrap();

    async_std::task::sleep(std::time::Duration::from_secs(5)).await;

    loop {
        match queryable.recv_async().await {
            Ok(query) => {
                let ke = query.key_expr();
                let selector = query.selector();

                let parsed_selector = selector.parameters_cowmap().unwrap();
                let requester_nf_type = parsed_selector.get("requester-nf-type").unwrap();

                let udm_value = if opts.protobuf {
                    let reply = SearchResult {
                        validity_period: 3600,
                        nf_instances: vec![NfProfile {
                            nf_instance_id: "65a52dac-b832-41ed-ba6e-53c1c0b3ed51".into(),
                            nf_type: "UDM".into(),
                            nf_status: "REGISTERED".into(),
                            heartbeat_timer: 10,
                            ipv4_addresses: vec!["172.22.0.13".into()],
                            allowed_nf_types: vec![
                                "AMF".into(),
                                "SMF".into(),
                                "AUSF".into(),
                                "SCP".into(),
                            ],
                            priority: 0,
                            capacity: 100,
                            load: 0,
                            nf_service_list: HashMap::from([(
                                "65a54148-b832-41ed-ba6e-53c1c0b3ed51".into(),
                                NfService {
                                    service_instance_id: "65a54148-b832-41ed-ba6e-53c1c0b3ed51"
                                        .into(),
                                    service_name: "nudm-sdm".into(),
                                    versions: vec![NfVersion {
                                        api_version_in_url: "v2".into(),
                                        api_full_version: "2.0.0".into(),
                                    }],
                                    scheme: "http".into(),
                                    nf_service_status: "REGISTERED".into(),
                                    ip_endpoints: vec![IpEndpoints {
                                        ipv4_address: "172.22.0.13".into(),
                                        port: 7777,
                                    }],
                                    allowed_nf_types: vec!["AMF".into(), "SMF".into()],
                                    priority: 0,
                                    capacity: 100,
                                    load: 0,
                                },
                            )]),
                            nf_profile_changes_support_ind: true,
                        }],
                    };

                    reply.encode_to_vec()
                } else {
                    r#"{"validityPeriod":3600,"nfInstances":[{"nfInstanceId":"65a52dac-b832-41ed-ba6e-53c1c0b3ed51","nfType":"UDM","nfStatus":"REGISTERED","heartBeatTimer":10,"ipv4Addresses":["172.22.0.13"],"allowedNfTypes":["AMF","SMF","AUSF","SCP"],"priority":0,"capacity":100,"load":0,"nfServiceList":{"65a54148-b832-41ed-ba6e-53c1c0b3ed51":{"serviceInstanceId":"65a54148-b832-41ed-ba6e-53c1c0b3ed51","serviceName":"nudm-sdm","versions":[{"apiVersionInUri":"v2","apiFullVersion":"2.0.0"}],"scheme":"http","nfServiceStatus":"REGISTERED","ipEndPoints":[{"ipv4Address":"172.22.0.13","port":7777}],"allowedNfTypes":["AMF","SMF"],"priority":0,"capacity":100,"load":0}},"nfProfileChangesSupportInd":true}]}"#.as_bytes().to_vec()
                };

                let smf_value = if opts.protobuf {
                    let reply = SearchResult {
                        validity_period: 3600,
                        nf_instances: vec![NfProfile {
                            nf_instance_id: "b3a71a80-b8d5-41ed-b2cc-8bbc6f173b7d".into(),
                            nf_type: "SMF".into(),
                            nf_status: "REGISTERED".into(),
                            heartbeat_timer: 10,
                            ipv4_addresses: vec!["172.22.0.7".into()],
                            allowed_nf_types: vec!["AMF".into(), "SCP".into()],
                            priority: 0,
                            capacity: 100,
                            load: 0,
                            nf_service_list: HashMap::from([(
                                "b3a71a80-b8d5-41ed-b2cc-8bbc6f173b7d".into(),
                                NfService {
                                    service_instance_id: "b3a71a80-b8d5-41ed-b2cc-8bbc6f173b7d"
                                        .into(),
                                    service_name: "nsmf-pdusession".into(),
                                    versions: vec![NfVersion {
                                        api_version_in_url: "v1".into(),
                                        api_full_version: "1.0.0".into(),
                                    }],
                                    scheme: "http".into(),
                                    nf_service_status: "REGISTERED".into(),
                                    ip_endpoints: vec![IpEndpoints {
                                        ipv4_address: "172.22.0.7".into(),
                                        port: 7777,
                                    }],
                                    allowed_nf_types: vec!["AMF".into()],
                                    priority: 0,
                                    capacity: 100,
                                    load: 0,
                                },
                            )]),
                            nf_profile_changes_support_ind: true,
                        }],
                    };
                    reply.encode_to_vec()
                } else {
                    r#"{"validityPeriod":3600,"nfInstances":[{"nfInstanceId":"b3a71a80-b8d5-41ed-b2cc-8bbc6f173b7d","nfType":"SMF","nfStatus":"REGISTERED","heartBeatTimer":10,"ipv4Addresses":["172.22.0.7"],"allowedNfTypes":["AMF","SCP"],"priority":0,"capacity":100,"load":0,"nfServiceList":{"b3c40334-b8d5-41ed-b2cc-8bbc6f173b7d":{"serviceInstanceId":"b3c40334-b8d5-41ed-b2cc-8bbc6f173b7d","serviceName":"nsmf-pdusession","versions":[{"apiVersionInUri":"v1","apiFullVersion":"1.0.0"}],"scheme":"http","nfServiceStatus":"REGISTERED","ipEndPoints":[{"ipv4Address":"172.22.0.7","port":7777}],"allowedNfTypes":["AMF"],"priority":0,"capacity":100,"load":0}},"nfProfileChangesSupportInd":true}]}"#.as_bytes().to_vec()
                };

                let value = if requester_nf_type == "AMF" {
                    smf_value
                } else if requester_nf_type == "SMF" {
                    udm_value
                } else {
                    vec![]
                };

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
