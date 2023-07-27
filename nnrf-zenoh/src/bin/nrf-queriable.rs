use clap::Parser;
use std::{str::FromStr, sync::Arc};
use zenoh::prelude::r#async::*;
use zenoh_config::{EndPoint, ListenConfig};

#[derive(Parser)]
pub struct Opts {
    // public options
    #[clap(short = 'l', long, default_value = "tcp/127.0.0.1:7070")]
    pub listen: String,
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

                let udm_value = r#"{"validityPeriod":3600,"nfInstances":[{"nfInstanceId":"65a52dac-b832-41ed-ba6e-53c1c0b3ed51","nfType":"UDM","nfStatus":"REGISTERED","heartBeatTimer":10,"ipv4Addresses":["172.22.0.13"],"allowedNfTypes":["AMF","SMF","AUSF","SCP"],"priority":0,"capacity":100,"load":0,"nfServiceList":{"65a54148-b832-41ed-ba6e-53c1c0b3ed51":{"serviceInstanceId":"65a54148-b832-41ed-ba6e-53c1c0b3ed51","serviceName":"nudm-sdm","versions":[{"apiVersionInUri":"v2","apiFullVersion":"2.0.0"}],"scheme":"http","nfServiceStatus":"REGISTERED","ipEndPoints":[{"ipv4Address":"172.22.0.13","port":7777}],"allowedNfTypes":["AMF","SMF"],"priority":0,"capacity":100,"load":0}},"nfProfileChangesSupportInd":true}]}"#;
                let smf_value = r#"{"validityPeriod":3600,"nfInstances":[{"nfInstanceId":"b3a71a80-b8d5-41ed-b2cc-8bbc6f173b7d","nfType":"SMF","nfStatus":"REGISTERED","heartBeatTimer":10,"ipv4Addresses":["172.22.0.7"],"allowedNfTypes":["AMF","SCP"],"priority":0,"capacity":100,"load":0,"nfServiceList":{"b3c40334-b8d5-41ed-b2cc-8bbc6f173b7d":{"serviceInstanceId":"b3c40334-b8d5-41ed-b2cc-8bbc6f173b7d","serviceName":"nsmf-pdusession","versions":[{"apiVersionInUri":"v1","apiFullVersion":"1.0.0"}],"scheme":"http","nfServiceStatus":"REGISTERED","ipEndPoints":[{"ipv4Address":"172.22.0.7","port":7777}],"allowedNfTypes":["AMF"],"priority":0,"capacity":100,"load":0}},"nfProfileChangesSupportInd":true}]}"#;

                let value = if requester_nf_type == "AMF" {
                    smf_value
                } else if requester_nf_type == "SMF" {
                    udm_value
                } else {
                    ""
                };

                let value = value.as_bytes();
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
