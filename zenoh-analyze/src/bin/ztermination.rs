use clap::Parser;
use pcf_zenoh::{get_epoch_ns, TerminationNotification};
use std::sync::Arc;
use zenoh::prelude::r#async::*;

#[derive(Parser)]
pub struct Opts {
    // public options
    #[clap(short = 'r', long, default_value = "10")]
    pub runs: usize,
}

// rt may be unused if there are no examples

#[async_std::main]
async fn main() {
    env_logger::init();

    let opts = Opts::parse();
    let runs = opts.runs;

    let mut config = zenoh::config::Config::default();
    config
        .set_mode(Some(zenoh::config::whatami::WhatAmI::Peer))
        .unwrap();
    let session = Arc::new(zenoh::open(config).res().await.unwrap());

    let sub = session
        .declare_subscriber("smf-callback/v1/sm-policy-notify/*")
        .res()
        .await
        .unwrap();

    async_std::task::sleep(std::time::Duration::from_secs(5)).await;

    let cb = "smf-callback/v1/sm-policy-notify/1".as_bytes();

    let resp = session
        .get("npcf-smpolicycontrol/v1/sm-policies")
        .with_value(cb)
        .res()
        .await
        .unwrap();
    resp.recv_async().await.unwrap();

    let mut i = 0;

    while i < runs {
        if let Ok(tn) = sub.recv_async().await {
            let tn = TerminationNotification::de(&tn.payload.contiguous());
            let delta = get_epoch_ns() - tn.ts;
            println!("notification,zenoh,{},ns", delta);
            i += 1;
        }
    }
}
