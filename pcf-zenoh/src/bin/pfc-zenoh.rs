use pcf_zenoh::{get_epoch_ns, TerminationNotification};
use std::{str::FromStr, sync::Arc};
use zenoh::prelude::r#async::*;
use zenoh_config::{EndPoint, ListenConfig};

use clap::Parser;

#[derive(Parser)]
pub struct Opts {
    // public options
    #[clap(short = 'l', long, default_value = "tcp/127.0.0.1:7073")]
    pub listen: String,
}

async fn send_policy_notifications(z: Arc<Session>, cb: String) {
    loop {
        let tn = TerminationNotification {
            supi: "imsi-001011234567895".into(),
            pdu_session_id: 1,
            ts: get_epoch_ns(),
        };

        let value = tn.ser();
        z.put(&cb, value).res().await.unwrap();
        async_std::task::sleep(std::time::Duration::from_micros(100)).await;
    }
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

    let ke = format!("npcf-smpolicycontrol/v1/sm-policies");
    let queryable = session.declare_queryable(&ke).res().await.unwrap();

    async_std::task::sleep(std::time::Duration::from_secs(5)).await;

    loop {
        match queryable.recv_async().await {
            Ok(query) => {
                let ke = query.key_expr();
                let value = query.value().unwrap();
                let cb = std::str::from_utf8(&value.payload.contiguous().to_vec())
                    .unwrap()
                    .to_string();

                let z = session.clone();
                async_std::task::spawn(send_policy_notifications(z, cb));

                query
                    .reply(Ok(Sample::new(ke.clone(), vec![])))
                    .res()
                    .await
                    .unwrap();
            }
            Err(_) => (),
        }
    }
}
