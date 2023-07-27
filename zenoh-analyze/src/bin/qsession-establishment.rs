#![allow(missing_docs, unused_variables, trivial_casts)]

use clap::Parser;
use log::info;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Instant;
use zenoh::prelude::r#async::*;

struct EstErr(String);
#[derive(Parser)]
pub struct Opts {
    // public options
    #[clap(short = 'r', long, default_value = "10")]
    pub runs: usize,
}

async fn establish_session<'a>(
    session: &Arc<Session>,
    flag: Arc<AtomicBool>,
) -> Result<(), EstErr> {
    // Mocking the session establishment.
    // 1. Send a create SM context to the SMF
    // 2. Wait for the SM Context create response
    // Done!

    // println!("Calling SMF");

    let nas_data = [
        0x2E, 0x01, 0x01, 0xC1, 0xFF, 0xFF, 0x91, 0xA1, 0x28, 0x01, 0x00, 0x7B, 0x00, 0x07, 0x80,
        0x00, 0x0A, 0x00, 0x00, 0x0D, 0x00,
    ];
    let json_request = r#"{
             "supi":	"imsi-001011234567895",
             "pei":	"imeisv-4370816125816151",
             "pduSessionId":	1,
             "dnn":	"internet",
             "sNssai":	{
                 "sst":	1
             },
             "servingNfId":	"66bf4df8-b832-41ed-aa12-4df3ea315a7c",
             "guami":	{
                 "plmnId":	{
                     "mcc":	"001",
                     "mnc":	"01"
                 },
                 "amfId":	"020040"
             },
             "servingNetwork":	{
                 "mcc":	"001",
                 "mnc":	"01"
             },
             "n1SmMsg":	{
                 "contentId":	"5gnas-sm"
             },
             "anType":	"3GPP_ACCESS",
             "ratType":	"NR",
             "ueLocation":	{
                 "nrLocation":	{
                     "tai":	{
                         "plmnId":	{
                             "mcc":	"001",
                             "mnc":	"01"
                         },
                         "tac":	"000001"
                     },
                     "ncgi":	{
                         "plmnId":	{
                             "mcc":	"001",
                             "mnc":	"01"
                         },
                         "nrCellId":	"000000010"
                     },
                     "ueLocationTimestamp":	"2023-03-01T13:42:11.144288Z"
                 }
             },
             "ueTimeZone":	"+00:00",
             "smContextStatusUri":	"http://172.22.0.10:7777/namf-callback/v1/imsi-001011234567895/sm-context-status/1",
             "pcfId":	"6c05c1d4-b832-41ed-9698-8dec5d3774de"
         }"#.as_bytes();

    let mut body: Vec<u8> = vec![];
    body.extend_from_slice(json_request);
    body.extend_from_slice(&nas_data);

    let sm_session_creation_result = session
        .get("nsmf-pdusession/v1/sm-contexts")
        .with_value(body)
        .res()
        .await
        .unwrap()
        .recv_async()
        .await
        .unwrap();

    info!("{:?}", sm_session_creation_result,);

    // waits for the callback from AMF
    let _ = flag.compare_exchange(true, false, Ordering::Acquire, Ordering::Relaxed);
    Ok(())
}

// rt may be unused if there are no examples
#[allow(unused_mut)]
#[async_std::main]
async fn main() {
    env_logger::init();

    let opts = Opts::parse();
    let runs = opts.runs;
    let flag = Arc::new(AtomicBool::new(false));

    let mut config = zenoh::config::Config::default();
    config
        .set_mode(Some(zenoh::config::whatami::WhatAmI::Peer))
        .unwrap();
    let session = Arc::new(zenoh::open(config).res().await.unwrap());

    let c_flag = flag.clone();
    let c_session = session.clone();
    async_std::task::spawn(async move {
        let amf_ke = "namf-comm/v1/ue-contexts/imsi-001011234567895/n1-n2-messages";
        let sub = c_session.declare_subscriber(amf_ke).res().await.unwrap();
        loop {
            sub.recv_async().await.unwrap();
            c_flag.store(true, Ordering::Relaxed);
        }
    });

    async_std::task::sleep(std::time::Duration::from_secs(5)).await;

    let mut i = 0;

    while i < runs {
        let now = Instant::now();
        match establish_session(&session, flag.clone()).await {
            Ok(_) => {
                let delta = now.elapsed();
                println!("establishment,zenoh,{},ns", delta.as_nanos());
                i += 1;
            }
            Err(_) => (),
        }
    }
}
