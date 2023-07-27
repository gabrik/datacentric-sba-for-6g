use clap::Parser;
use std::{str::FromStr, sync::Arc};
use zenoh::prelude::r#async::*;
use zenoh_config::{EndPoint, ListenConfig};

#[derive(Parser)]
pub struct Opts {
    // public options
    #[clap(short = 'l', long, default_value = "tcp/127.0.0.1:7071")]
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

    let ke = format!("nudm-sdm/v2/*/sm-data");
    let queryable = session.declare_queryable(&ke).res().await.unwrap();

    async_std::task::sleep(std::time::Duration::from_secs(5)).await;

    loop {
        match queryable.recv_async().await {
            Ok(query) => {
                let ke = query.key_expr();
                // let value = serde_json::from_str::<SessionManagementSubscriptionData>(r#"{"singleNssai":{"sst":1},"dnnConfigurations":{"internet":{"pduSessionTypes":{"defaultSessionType":"IPV4","allowedSessionTypes":["IPV4"]},"sscModes":{"defaultSscMode":"SSC_MODE_1","allowedSscModes":["SSC_MODE_1","SSC_MODE_2","SSC_MODE_3"]},"5gQosProfile":{"5qi":9,"arp":{"priorityLevel":8,"preemptCap":"NOT_PREEMPT","preemptVuln":"NOT_PREEMPTABLE"},"priorityLevel":8},"sessionAmbr":{"uplink":"1048576 Kbps","downlink":"1048576 Kbps"}}}}"#).expect("unable to parse json");
                let value = r#"{"singleNssai":{"sst":1},"dnnConfigurations":{"internet":{"pduSessionTypes":{"defaultSessionType":"IPV4","allowedSessionTypes":["IPV4"]},"sscModes":{"defaultSscMode":"SSC_MODE_1","allowedSscModes":["SSC_MODE_1","SSC_MODE_2","SSC_MODE_3"]},"5gQosProfile":{"5qi":9,"arp":{"priorityLevel":8,"preemptCap":"NOT_PREEMPT","preemptVuln":"NOT_PREEMPTABLE"},"priorityLevel":8},"sessionAmbr":{"uplink":"1048576 Kbps","downlink":"1048576 Kbps"}}}}"#.as_bytes();
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
